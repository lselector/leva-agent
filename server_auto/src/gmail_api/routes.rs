/// Axum routes for Gmail API operations.
use axum::{extract::Path, routing::{get, post}, Json, Router};
use serde::Deserialize;
use serde_json::Value;
use super::client;

pub fn router() -> Router {
    Router::new()
        .route("/api/gmail/inbox", post(gmail_inbox))
        .route("/api/gmail/email/{id}", get(gmail_get_email))
        .route("/api/gmail/send", post(gmail_send))
        .route("/api/gmail/search", post(gmail_search))
        .route("/api/gmail/labels", get(gmail_labels))
        // Tool proxy endpoints
        .route("/tools/gmail_inbox", post(tool_inbox))
        .route("/tools/gmail_send", post(tool_send))
        .route("/tools/gmail_search", post(tool_search))
}

#[derive(Deserialize)] struct InboxReq { max_results: Option<u32> }
#[derive(Deserialize)] struct SendReq { to: String, subject: String, body: String }
#[derive(Deserialize)] struct SearchReq { query: String, max_results: Option<u32> }

async fn gmail_inbox(Json(req): Json<InboxReq>) -> axum::response::Response {
    match client::get_inbox(req.max_results.unwrap_or(15)).await {
        Ok(msgs) => json_ok(serde_json::to_value(msgs).unwrap_or_default()),
        Err(e) => json_err(503, &e.to_string()),
    }
}

async fn gmail_get_email(Path(id): Path<String>) -> axum::response::Response {
    match client::get_email(&id).await {
        Ok(msg) => json_ok(serde_json::to_value(msg).unwrap_or_default()),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn gmail_send(Json(req): Json<SendReq>) -> axum::response::Response {
    match client::send_email(&req.to, &req.subject, &req.body).await {
        Ok(v) => json_ok(v),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn gmail_search(Json(req): Json<SearchReq>) -> axum::response::Response {
    match client::search_emails(&req.query, req.max_results.unwrap_or(10)).await {
        Ok(msgs) => json_ok(serde_json::to_value(msgs).unwrap_or_default()),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn gmail_labels() -> axum::response::Response {
    match client::get_labels().await {
        Ok(labels) => json_ok(Value::Array(labels.into_iter().map(Value::String).collect())),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn tool_inbox(Json(args): Json<Value>) -> String {
    let max = args["max_results"].as_u64().unwrap_or(15) as u32;
    match client::get_inbox(max).await {
        Ok(msgs) => serde_json::to_string(&msgs).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}

async fn tool_send(Json(args): Json<Value>) -> String {
    let to = args["to"].as_str().unwrap_or("");
    let subject = args["subject"].as_str().unwrap_or("");
    let body = args["body"].as_str().unwrap_or("");
    match client::send_email(to, subject, body).await {
        Ok(v) => serde_json::to_string(&v).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}

async fn tool_search(Json(args): Json<Value>) -> String {
    let query = args["query"].as_str().unwrap_or("");
    let max = args["max_results"].as_u64().unwrap_or(10) as u32;
    match client::search_emails(query, max).await {
        Ok(msgs) => serde_json::to_string(&msgs).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}

fn json_ok(v: Value) -> axum::response::Response {
    axum::response::Response::builder()
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&v).unwrap()))
        .unwrap()
}

fn json_err(status: u16, msg: &str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&serde_json::json!({"detail": msg})).unwrap(),
        ))
        .unwrap()
}
