/// SSE streaming with tool-call loop — Anthropic Claude.
use serde_json::Value;

/// Yield SSE lines for a chat request (token/status/full_text/done).
pub async fn stream_chat(
    messages: Vec<Value>,
    tx: tokio::sync::mpsc::Sender<String>,
) {
    crate::anthropic_stream::stream_chat_anthropic(messages, tx).await;
}
