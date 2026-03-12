/// Chat endpoints — POST /api/chat (SSE), GET/DELETE /api/chat/{id}
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use common::{config, tools::memory::soul_read};
use crate::session_store::AppState;
use crate::streaming::{stream_chat, non_stream_chat};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub content: Option<Value>,
    pub session_id: Option<String>,
    pub stream: Option<bool>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/chat", post(chat_endpoint))
        .route("/api/chat/history", get(list_sessions))
        .route("/api/chat/{session_id}", get(get_session))
        .route("/api/chat/{session_id}", delete(delete_session))
        .with_state(state)
}

async fn load_system_prompt() -> String {
    let cfg = config::get();
    let base = std::fs::read_to_string(cfg.prompts_dir.join("SYSTEM.md")).unwrap_or_default();
    let soul = soul_read().unwrap_or_default();
    format!("{base}\n\n# Core Identity (Soul)\n\n{soul}")
}

async fn chat_endpoint(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> axum::response::Response {
    let sid = req.session_id.clone().unwrap_or_else(|| "default".to_string());
    let system_prompt = load_system_prompt().await;

    state.store.add_message(&sid, "user", &req.message);

    let history = state.store.get_messages(&sid);
    let mut messages: Vec<Value> = vec![serde_json::json!({
        "role": "system",
        "content": system_prompt,
    })];
    for msg in &history {
        messages.push(serde_json::to_value(msg).unwrap_or_default());
    }
    if let Some(content) = &req.content {
        if let Some(last) = messages.last_mut() {
            if last["role"] == "user" {
                last["content"] = content.clone();
            }
        }
    }

    if req.stream.unwrap_or(true) {
        let (tx, rx) = mpsc::channel::<String>(256);

        tokio::spawn(async move {
            stream_chat(messages, tx.clone()).await;
        });

        // Wrap the channel into a byte stream, intercepting full_text to save to session
        let store2 = state.store.clone();
        let sid2 = sid.clone();
        let (byte_tx, byte_rx) = mpsc::channel::<Result<bytes::Bytes, std::convert::Infallible>>(256);

        tokio::spawn(async move {
            let mut recv_stream = ReceiverStream::new(rx);
            while let Some(line) = recv_stream.next().await {
                if line.contains("\"full_text\"") {
                    let data = line.trim_start_matches("data: ").trim();
                    if let Ok(obj) = serde_json::from_str::<Value>(data) {
                        if let Some(ft) = obj["full_text"].as_str() {
                            store2.add_message(&sid2, "assistant", ft);
                        }
                    }
                }
                let _ = byte_tx.send(Ok(bytes::Bytes::from(line))).await;
            }
        });

        let body = axum::body::Body::from_stream(ReceiverStream::new(byte_rx));
        axum::response::Response::builder()
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("X-Accel-Buffering", "no")
            .body(body)
            .unwrap()
    } else {
        match non_stream_chat(messages).await {
            Ok(answer) => {
                state.store.add_message(&sid, "assistant", &answer);
                axum::response::Response::builder()
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "reply": answer,
                            "session_id": sid,
                        }))
                        .unwrap(),
                    ))
                    .unwrap()
            }
            Err(e) => axum::response::Response::builder()
                .status(500)
                .body(axum::body::Body::from(e.to_string()))
                .unwrap(),
        }
    }
}

async fn list_sessions(State(state): State<AppState>) -> Json<Value> {
    Json(Value::Array(
        state.store.list_sessions()
            .into_iter()
            .map(Value::String)
            .collect(),
    ))
}

async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    match state.store.get_session(&session_id) {
        Some(msgs) => axum::response::Response::builder()
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&msgs).unwrap()))
            .unwrap(),
        None => axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from(r#"{"detail":"Session not found"}"#))
            .unwrap(),
    }
}

async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    if state.store.delete_session(&session_id) {
        axum::response::Response::builder()
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"status":"deleted"}"#))
            .unwrap()
    } else {
        axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from(r#"{"detail":"Session not found"}"#))
            .unwrap()
    }
}
