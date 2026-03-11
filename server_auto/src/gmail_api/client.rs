/// Gmail REST API client operations.
use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::auth::get_access_token;

const BASE: &str = "https://www.googleapis.com/gmail/v1/users/me";

fn gmail_client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn auth_header() -> Result<String> {
    let token = get_access_token().await?;
    Ok(format!("Bearer {token}"))
}

// ---------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSummary {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub snippet: String,
}

pub async fn get_inbox(max_results: u32) -> Result<Vec<EmailSummary>> {
    let auth = auth_header().await?;
    let list: Value = gmail_client()
        .get(format!("{BASE}/messages?labelIds=INBOX&maxResults={max_results}"))
        .header("Authorization", &auth)
        .send().await?.json().await?;

    let mut results = Vec::new();
    for msg in list["messages"].as_array().unwrap_or(&vec![]) {
        let id = msg["id"].as_str().unwrap_or("");
        let meta: Value = gmail_client()
            .get(format!("{BASE}/messages/{id}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date"))
            .header("Authorization", &auth)
            .send().await?.json().await?;
        let headers: std::collections::HashMap<String, String> = meta["payload"]["headers"]
            .as_array().unwrap_or(&vec![])
            .iter()
            .filter_map(|h| {
                let name = h["name"].as_str()?.to_string();
                let value = h["value"].as_str()?.to_string();
                Some((name, value))
            })
            .collect();
        results.push(EmailSummary {
            id: id.to_string(),
            from: headers.get("From").cloned().unwrap_or_default(),
            subject: headers.get("Subject").cloned().unwrap_or_default(),
            date: headers.get("Date").cloned().unwrap_or_default(),
            snippet: meta["snippet"].as_str().unwrap_or("").to_string(),
        });
    }
    Ok(results)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailFull {
    pub id: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub date: String,
    pub body: String,
}

pub async fn get_email(msg_id: &str) -> Result<EmailFull> {
    let auth = auth_header().await?;
    let msg: Value = gmail_client()
        .get(format!("{BASE}/messages/{msg_id}?format=full"))
        .header("Authorization", &auth)
        .send().await?.json().await?;

    let payload = &msg["payload"];
    let headers: std::collections::HashMap<String, String> = payload["headers"]
        .as_array().unwrap_or(&vec![])
        .iter()
        .filter_map(|h| Some((h["name"].as_str()?.to_string(), h["value"].as_str()?.to_string())))
        .collect();

    let body = extract_body(payload);

    Ok(EmailFull {
        id: msg_id.to_string(),
        from: headers.get("From").cloned().unwrap_or_default(),
        to: headers.get("To").cloned().unwrap_or_default(),
        subject: headers.get("Subject").cloned().unwrap_or_default(),
        date: headers.get("Date").cloned().unwrap_or_default(),
        body,
    })
}

pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<Value> {
    let auth = auth_header().await?;
    let raw_email = format!("To: {to}\r\nSubject: {subject}\r\n\r\n{body}");
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_email.as_bytes());
    let resp: Value = gmail_client()
        .post(format!("{BASE}/messages/send"))
        .header("Authorization", &auth)
        .json(&serde_json::json!({"raw": encoded}))
        .send().await?.json().await?;
    Ok(serde_json::json!({"id": resp["id"], "status": "sent"}))
}

pub async fn search_emails(query: &str, max_results: u32) -> Result<Vec<EmailSummary>> {
    let auth = auth_header().await?;
    let encoded = urlencoding::encode(query);
    let list: Value = gmail_client()
        .get(format!("{BASE}/messages?q={encoded}&maxResults={max_results}"))
        .header("Authorization", &auth)
        .send().await?.json().await?;

    let mut results = Vec::new();
    for msg in list["messages"].as_array().unwrap_or(&vec![]) {
        let id = msg["id"].as_str().unwrap_or("");
        let meta: Value = gmail_client()
            .get(format!("{BASE}/messages/{id}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date"))
            .header("Authorization", &auth)
            .send().await?.json().await?;
        let headers: std::collections::HashMap<String, String> = meta["payload"]["headers"]
            .as_array().unwrap_or(&vec![])
            .iter()
            .filter_map(|h| Some((h["name"].as_str()?.to_string(), h["value"].as_str()?.to_string())))
            .collect();
        results.push(EmailSummary {
            id: id.to_string(),
            from: headers.get("From").cloned().unwrap_or_default(),
            subject: headers.get("Subject").cloned().unwrap_or_default(),
            date: headers.get("Date").cloned().unwrap_or_default(),
            snippet: meta["snippet"].as_str().unwrap_or("").to_string(),
        });
    }
    Ok(results)
}

pub async fn get_labels() -> Result<Vec<String>> {
    let auth = auth_header().await?;
    let resp: Value = gmail_client()
        .get(format!("{BASE}/labels"))
        .header("Authorization", &auth)
        .send().await?.json().await?;
    Ok(resp["labels"].as_array().unwrap_or(&vec![])
        .iter()
        .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
        .collect())
}

// ---------------------------------------------------------------

fn extract_body(payload: &Value) -> String {
    let mime = payload["mimeType"].as_str().unwrap_or("");
    if mime == "text/plain" {
        let data = payload["body"]["data"].as_str().unwrap_or("");
        if !data.is_empty() {
            if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data) {
                return String::from_utf8_lossy(&bytes).to_string();
            }
        }
    }
    for part in payload["parts"].as_array().unwrap_or(&vec![]) {
        let body = extract_body(part);
        if !body.is_empty() { return body; }
    }
    String::new()
}
