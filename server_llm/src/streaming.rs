/// SSE streaming with tool-call loop — async-openai based.
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use common::{config, tools::registry};
use crate::tool_dispatch;

const MAX_TOOL_ROUNDS: usize = 10;

/// Yield SSE lines for a chat request (token/status/full_text/done).
pub async fn stream_chat(
    messages: Vec<Value>,
    tx: tokio::sync::mpsc::Sender<String>,
) {
    let cfg = config::get();
    let model = cfg.model_name.read().unwrap().clone();

    // Build tool list for OpenAI
    let tool_schemas = registry::get_tools_schema();

    let mut msgs: Vec<Value> = messages;
    let mut full_text = String::new();

    for _round in 0..MAX_TOOL_ROUNDS {
        // Serialize request manually via reqwest so we can use streaming JSON
        let request_body = serde_json::json!({
            "model": model,
            "messages": msgs,
            "tools": tool_schemas,
            "stream": true,
        });

        let response = reqwest::Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&cfg.openai_api_key)
            .json(&request_body)
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(sse_json(&serde_json::json!({"error": e.to_string()}))).await;
                return;
            }
        };

        let mut content_buf = String::new();
        // tool_calls_buf: index -> {id, name, args}
        let mut tool_calls_buf: HashMap<usize, (String, String, String)> = HashMap::new();
        let mut finish_reason = String::new();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(_) => break,
            };
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if !line.starts_with("data: ") { continue; }
                let data = &line["data: ".len()..];
                if data == "[DONE]" { break; }
                let Ok(obj) = serde_json::from_str::<Value>(data) else { continue };
                let choice = &obj["choices"][0];
                let delta = &choice["delta"];

                if let Some(fr) = choice["finish_reason"].as_str() {
                    if !fr.is_empty() { finish_reason = fr.to_string(); }
                }

                if let Some(token) = delta["content"].as_str() {
                    if !token.is_empty() {
                        content_buf.push_str(token);
                        full_text.push_str(token);
                        let _ = tx.send(sse_json(&serde_json::json!({"token": token}))).await;
                    }
                }

                if let Some(tcs) = delta["tool_calls"].as_array() {
                    for tc in tcs {
                        let idx = tc["index"].as_u64().unwrap_or(0) as usize;
                        let entry = tool_calls_buf.entry(idx).or_insert_with(|| (String::new(), String::new(), String::new()));
                        if let Some(id) = tc["id"].as_str() { if !id.is_empty() { entry.0 = id.to_string(); } }
                        if let Some(name) = tc["function"]["name"].as_str() { entry.1.push_str(name); }
                        if let Some(args) = tc["function"]["arguments"].as_str() { entry.2.push_str(args); }
                    }
                }
            }
        }

        if finish_reason != "tool_calls" {
            break;
        }

        let _ = tx.send(sse_json(&serde_json::json!({"status": "Using tools..."}))).await;

        // Build sorted tool call list
        let mut sorted_keys: Vec<usize> = tool_calls_buf.keys().cloned().collect();
        sorted_keys.sort();

        let tool_calls_json: Vec<Value> = sorted_keys.iter().map(|idx| {
            let (id, name, args) = &tool_calls_buf[idx];
            serde_json::json!({
                "id": id,
                "type": "function",
                "function": {"name": name, "arguments": args}
            })
        }).collect();

        // Append assistant message with tool_calls
        msgs.push(serde_json::json!({
            "role": "assistant",
            "content": if content_buf.is_empty() { Value::Null } else { Value::String(content_buf.clone()) },
            "tool_calls": tool_calls_json,
        }));

        // Execute each tool and append result
        for idx in &sorted_keys {
            let (id, name, args_str) = &tool_calls_buf[idx];
            let args: HashMap<String, Value> = serde_json::from_str(args_str).unwrap_or_default();
            let result = tool_dispatch::execute(name, &args).await;
            msgs.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": id,
                "content": result,
            }));
        }
    }

    let _ = tx.send("data: [DONE]\n\n".to_string()).await;
    let _ = tx.send(sse_json(&serde_json::json!({"full_text": full_text}))).await;
}

fn sse_json(val: &Value) -> String {
    format!("data: {}\n\n", serde_json::to_string(val).unwrap_or_default())
}

/// Non-streaming chat with tool-call loop. Returns the final reply text.
pub async fn non_stream_chat(messages: Vec<Value>) -> Result<String, String> {
    let cfg = config::get();
    let model = cfg.model_name.read().unwrap().clone();
    let tool_schemas = registry::get_tools_schema();
    let mut msgs = messages;

    for _round in 0..MAX_TOOL_ROUNDS {
        let request_body = serde_json::json!({
            "model": model,
            "messages": msgs,
            "tools": tool_schemas,
            "tool_choice": "auto",
        });

        let resp = reqwest::Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&cfg.openai_api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        let choice = &json["choices"][0];
        let finish_reason = choice["finish_reason"].as_str().unwrap_or("");
        let message = &choice["message"];

        if finish_reason != "tool_calls" {
            return Ok(message["content"].as_str().unwrap_or("").to_string());
        }

        // Append assistant message with tool_calls
        msgs.push(message.clone());

        // Execute each tool
        if let Some(tool_calls) = message["tool_calls"].as_array() {
            for tc in tool_calls {
                let id = tc["id"].as_str().unwrap_or("").to_string();
                let name = tc["function"]["name"].as_str().unwrap_or("");
                let args: HashMap<String, Value> = serde_json::from_str(
                    tc["function"]["arguments"].as_str().unwrap_or("{}")
                ).unwrap_or_default();
                let result = tool_dispatch::execute(name, &args).await;
                msgs.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": id,
                    "content": result,
                }));
            }
        }
    }

    Ok(String::new())
}
