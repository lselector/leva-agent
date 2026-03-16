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
    pub body: String,       // plain text (for display/reading)
    pub body_html: String,  // raw HTML (for forwarding)
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

    let body_html = extract_body_raw_html(payload).unwrap_or_default();
    let body = if body_html.is_empty() {
        extract_body(payload)
    } else {
        strip_html_tags(&body_html)
    };

    Ok(EmailFull {
        id: msg_id.to_string(),
        from: headers.get("From").cloned().unwrap_or_default(),
        to: headers.get("To").cloned().unwrap_or_default(),
        subject: headers.get("Subject").cloned().unwrap_or_default(),
        date: headers.get("Date").cloned().unwrap_or_default(),
        body,
        body_html,
    })
}

pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<Value> {
    let auth = auth_header().await?;
    let raw_email = format!("To: {to}\r\nSubject: {subject}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{body}");
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_email.as_bytes());
    let resp: Value = gmail_client()
        .post(format!("{BASE}/messages/send"))
        .header("Authorization", &auth)
        .json(&serde_json::json!({"raw": encoded}))
        .send().await?.json().await?;
    Ok(serde_json::json!({"id": resp["id"], "status": "sent"}))
}

pub async fn send_email_html(to: &str, subject: &str, html_body: &str) -> Result<Value> {
    let auth = auth_header().await?;
    let raw_email = format!(
        "To: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{html_body}"
    );
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_email.as_bytes());
    let resp: Value = gmail_client()
        .post(format!("{BASE}/messages/send"))
        .header("Authorization", &auth)
        .json(&serde_json::json!({"raw": encoded}))
        .send().await?.json().await?;
    Ok(serde_json::json!({"id": resp["id"], "status": "sent"}))
}

pub async fn get_raw_payload(msg_id: &str) -> Result<Value> {
    let auth = auth_header().await?;
    let msg: Value = gmail_client()
        .get(format!("{BASE}/messages/{msg_id}?format=full"))
        .header("Authorization", &auth)
        .send().await?.json().await?;
    // Return just payload structure (mimeType + parts tree, no large body data)
    fn summarize(v: &Value) -> Value {
        let mime = v["mimeType"].as_str().unwrap_or("").to_string();
        let has_data = v["body"]["data"].as_str().map(|s| s.len()).unwrap_or(0);
        let parts: Vec<Value> = v["parts"].as_array().unwrap_or(&vec![])
            .iter().map(|p| summarize(p)).collect();
        serde_json::json!({"mimeType": mime, "bodyDataLen": has_data, "parts": parts})
    }
    Ok(summarize(&msg["payload"]))
}

struct Attachment {
    filename: String,
    mime_type: String,
    data_b64: String, // URL-safe base64 as returned by Gmail API
}

/// Collect all attachment parts (parts with a filename) from the payload tree.
fn collect_attachment_infos(payload: &Value, out: &mut Vec<(String, String, String)>) {
    // out entries: (filename, mime_type, attachment_id_or_inline_data)
    // We use attachment_id="" to signal inline data is in body.data
    let filename = payload["filename"].as_str().unwrap_or("");
    if !filename.is_empty() {
        let mime = payload["mimeType"].as_str().unwrap_or("application/octet-stream").to_string();
        let attachment_id = payload["body"]["attachmentId"].as_str().unwrap_or("").to_string();
        let inline_data = payload["body"]["data"].as_str().unwrap_or("").to_string();
        if !attachment_id.is_empty() || !inline_data.is_empty() {
            out.push((filename.to_string(), mime, if attachment_id.is_empty() { format!("inline:{inline_data}") } else { attachment_id }));
        }
    }
    for part in payload["parts"].as_array().unwrap_or(&vec![]) {
        collect_attachment_infos(part, out);
    }
}

async fn fetch_attachment(msg_id: &str, attachment_id: &str, auth: &str) -> Result<String> {
    let resp: Value = gmail_client()
        .get(format!("{BASE}/messages/{msg_id}/attachments/{attachment_id}"))
        .header("Authorization", auth)
        .send().await?.json().await?;
    Ok(resp["data"].as_str().unwrap_or("").to_string())
}

pub async fn forward_email(msg_id: &str, to: &str) -> Result<Value> {
    let auth_str = auth_header().await?;

    // Fetch full message to get payload tree for attachments
    let msg: Value = gmail_client()
        .get(format!("{BASE}/messages/{msg_id}?format=full"))
        .header("Authorization", &auth_str)
        .send().await?.json().await?;

    let payload = &msg["payload"];
    let headers: std::collections::HashMap<String, String> = payload["headers"]
        .as_array().unwrap_or(&vec![])
        .iter()
        .filter_map(|h| Some((h["name"].as_str()?.to_string(), h["value"].as_str()?.to_string())))
        .collect();

    let subject = headers.get("Subject").cloned().unwrap_or_default();
    let from    = headers.get("From").cloned().unwrap_or_default();
    let date    = headers.get("Date").cloned().unwrap_or_default();
    let orig_to = headers.get("To").cloned().unwrap_or_default();

    let fwd_subject = if subject.starts_with("Fwd:") { subject.clone() }
                      else { format!("Fwd: {subject}") };

    // Build HTML body
    let body_html = extract_body_raw_html(payload).unwrap_or_default();
    let fwd_html_body = if !body_html.is_empty() {
        format!(
            "<div style=\"margin-bottom:16px\">---------- Forwarded message ----------<br>\
            From: {from}<br>Date: {date}<br>Subject: {subject}<br>To: {orig_to}</div>{body_html}"
        )
    } else {
        let plain = extract_body(payload);
        format!(
            "<div style=\"margin-bottom:16px\">---------- Forwarded message ----------<br>\
            From: {from}<br>Date: {date}<br>Subject: {subject}<br>To: {orig_to}</div>\
            <pre style=\"white-space:pre-wrap\">{plain}</pre>"
        )
    };

    // Collect attachment metadata
    let mut attachment_infos: Vec<(String, String, String)> = Vec::new();
    collect_attachment_infos(payload, &mut attachment_infos);

    // Fetch attachment data
    let mut attachments: Vec<Attachment> = Vec::new();
    for (filename, mime_type, id_or_data) in &attachment_infos {
        let data_b64 = if let Some(inline) = id_or_data.strip_prefix("inline:") {
            inline.to_string()
        } else {
            match fetch_attachment(msg_id, id_or_data, &auth_str).await {
                Ok(d) => d,
                Err(e) => { eprintln!("[ATTACH] failed to fetch {filename}: {e}"); continue; }
            }
        };
        if !data_b64.is_empty() {
            attachments.push(Attachment { filename: filename.clone(), mime_type: mime_type.clone(), data_b64 });
        }
    }

    // Build MIME message
    let raw_email = if attachments.is_empty() {
        let html_b64 = base64::engine::general_purpose::STANDARD.encode(fwd_html_body.as_bytes());
        let html_b64_wrapped = html_b64.as_bytes().chunks(76)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect::<Vec<_>>().join("\r\n");
        format!(
            "To: {to}\r\nSubject: {fwd_subject}\r\nMIME-Version: 1.0\r\n\
             Content-Type: text/html; charset=UTF-8\r\n\
             Content-Transfer-Encoding: base64\r\n\r\n{html_b64_wrapped}"
        )
    } else {
        // multipart/mixed: HTML body + attachments
        let boundary = format!("fwd_boundary_{}", uuid_hex());
        let html_b64 = base64::engine::general_purpose::STANDARD.encode(fwd_html_body.as_bytes());
        let html_b64_wrapped = html_b64.as_bytes().chunks(76)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect::<Vec<_>>().join("\r\n");
        let mut parts = format!(
            "To: {to}\r\nSubject: {fwd_subject}\r\nMIME-Version: 1.0\r\n\
             Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n\
             --{boundary}\r\n\
             Content-Type: text/html; charset=UTF-8\r\n\
             Content-Transfer-Encoding: base64\r\n\r\n\
             {html_b64_wrapped}\r\n"
        );
        for att in &attachments {
            // Convert URL-safe base64 to standard base64 with line wrapping for MIME
            let std_b64 = to_standard_b64_wrapped(&att.data_b64);
            parts.push_str(&format!(
                "--{boundary}\r\n\
                 Content-Type: {}; name=\"{}\"\r\n\
                 Content-Transfer-Encoding: base64\r\n\
                 Content-Disposition: attachment; filename=\"{}\"\r\n\r\n\
                 {}\r\n",
                att.mime_type, att.filename, att.filename, std_b64
            ));
        }
        parts.push_str(&format!("--{boundary}--"));
        parts
    };

    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_email.as_bytes());
    let auth_str2 = auth_header().await?;
    let resp: Value = gmail_client()
        .post(format!("{BASE}/messages/send"))
        .header("Authorization", &auth_str2)
        .json(&serde_json::json!({"raw": encoded}))
        .send().await?.json().await?;
    Ok(serde_json::json!({"id": resp["id"], "status": "sent"}))
}

fn uuid_hex() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}{:x}", t.as_secs(), t.subsec_nanos())
}

/// Convert Gmail's URL-safe base64 to standard base64 with 76-char line wrapping (MIME).
fn to_standard_b64_wrapped(url_safe: &str) -> String {
    // Decode from URL-safe, re-encode as standard base64
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(url_safe)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(url_safe))
        .unwrap_or_default();
    let std = base64::engine::general_purpose::STANDARD.encode(&bytes);
    // Wrap at 76 chars
    std.as_bytes().chunks(76)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\r\n")
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
    extract_body_prefer_plain(payload)
        .or_else(|| extract_body_html(payload))
        .unwrap_or_default()
}

fn decode_part_data(payload: &Value) -> Option<String> {
    let data = payload["body"]["data"].as_str()?;
    if data.is_empty() { return None; }
    // Gmail API uses URL-safe base64, sometimes with or without padding
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(data))
        .or_else(|_| {
            // Strip any whitespace/newlines and retry
            let clean: String = data.chars().filter(|c| !c.is_whitespace()).collect();
            base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&clean)
        })
        .ok()?;
    Some(String::from_utf8_lossy(&bytes).to_string())
}

fn extract_body_prefer_plain(payload: &Value) -> Option<String> {
    let mime = payload["mimeType"].as_str().unwrap_or("");
    if mime == "text/plain" {
        if let Some(text) = decode_part_data(payload) {
            if !text.is_empty() { return Some(text); }
        }
    }
    for part in payload["parts"].as_array().unwrap_or(&vec![]) {
        if let Some(text) = extract_body_prefer_plain(part) {
            return Some(text);
        }
    }
    None
}

fn extract_body_html(payload: &Value) -> Option<String> {
    let mime = payload["mimeType"].as_str().unwrap_or("");
    if mime == "text/html" {
        if let Some(html) = decode_part_data(payload) {
            if !html.is_empty() { return Some(strip_html_tags(&html)); }
        }
    }
    for part in payload["parts"].as_array().unwrap_or(&vec![]) {
        if let Some(text) = extract_body_html(part) {
            return Some(text);
        }
    }
    None
}

/// Return the raw HTML body without stripping, for use in HTML forwards.
fn extract_body_raw_html(payload: &Value) -> Option<String> {
    let mime = payload["mimeType"].as_str().unwrap_or("");
    if mime == "text/html" {
        if let Some(html) = decode_part_data(payload) {
            if !html.is_empty() { return Some(html); }
        }
    }
    for part in payload["parts"].as_array().unwrap_or(&vec![]) {
        if let Some(html) = extract_body_raw_html(part) {
            return Some(html);
        }
    }
    None
}

fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse excessive blank lines
    let mut result = String::new();
    let mut blank_count = 0u32;
    for line in out.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 { result.push('\n'); }
        } else {
            blank_count = 0;
            result.push_str(line.trim());
            result.push('\n');
        }
    }
    result
}
