/// SSE streaming with tool-call loop — Anthropic Claude API.
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;
use common::{config, tools::registry};
use crate::tool_dispatch;

const MAX_TOOL_ROUNDS: usize = 10;

fn is_model_unavailable(body: &str) -> bool {
    body.contains("overloaded") || body.contains("not_found") || body.contains("unavailable")
}

/// Yield SSE lines for a chat request using Anthropic Claude.
pub async fn stream_chat_anthropic(
    messages: Vec<Value>,
    tx: tokio::sync::mpsc::Sender<String>,
) {
    let cfg = config::get();
    let primary = cfg.model_name.read().unwrap().clone();
    let mut model = primary.clone();

    // Extract system message; convert rest to Anthropic user/assistant format
    let mut system = String::new();
    let mut anthropic_msgs: Vec<Value> = Vec::new();
    for msg in messages {
        match msg["role"].as_str().unwrap_or("") {
            "system" => system = msg["content"].as_str().unwrap_or("").to_string(),
            role @ ("user" | "assistant") => {
                let content = msg["content"].clone();
                anthropic_msgs.push(json!({"role": role, "content": content}));
            }
            _ => {}
        }
    }

    let tools = registry::get_anthropic_tools_schema();
    let mut full_text = String::new();

    for _round in 0..MAX_TOOL_ROUNDS {
        let request_body = json!({
            "model": model,
            "max_tokens": 8096,
            "system": system,
            "messages": anthropic_msgs,
            "tools": tools,
            "stream": true,
        });

        let response = reqwest::Client::new()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &cfg.anthropic_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(sse_json(&json!({"error": e.to_string()}))).await;
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eprintln!("[ANTHROPIC] error {} : {}", status, body);
            if is_model_unavailable(&body) && model == primary {
                eprintln!("[ANTHROPIC] {} unavailable, falling back to {}", model, cfg.fallback_model);
                model = cfg.fallback_model.clone();
                continue;
            }
            let _ = tx.send(sse_json(&json!({"error": format!("Anthropic API error {}: {}", status, body)}))).await;
            return;
        }

        // index -> (id, name, accumulated_args)
        let mut tool_blocks: HashMap<usize, (String, String, String)> = HashMap::new();
        let mut text_buf = String::new();
        let mut stop_reason = String::new();

        let mut stream = response.bytes_stream();
        let mut line_buf = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk { Ok(c) => c, Err(_) => break };
            line_buf.push_str(&String::from_utf8_lossy(&chunk));
            loop {
                if let Some(pos) = line_buf.find('\n') {
                    let line = line_buf[..pos].trim_end_matches('\r').to_string();
                    line_buf = line_buf[pos + 1..].to_string();

                    if !line.starts_with("data: ") { continue; }
                    let data = &line["data: ".len()..];
                    let Ok(obj) = serde_json::from_str::<Value>(data) else { continue };

                    match obj["type"].as_str().unwrap_or("") {
                        "content_block_start" => {
                            let idx = obj["index"].as_u64().unwrap_or(0) as usize;
                            let block = &obj["content_block"];
                            if block["type"] == "tool_use" {
                                let id = block["id"].as_str().unwrap_or("").to_string();
                                let name = block["name"].as_str().unwrap_or("").to_string();
                                eprintln!("[TOOL] delta name idx={} name={}", idx, name);
                                tool_blocks.insert(idx, (id, name, String::new()));
                            }
                        }
                        "content_block_delta" => {
                            let idx = obj["index"].as_u64().unwrap_or(0) as usize;
                            let delta = &obj["delta"];
                            match delta["type"].as_str().unwrap_or("") {
                                "text_delta" => {
                                    if let Some(text) = delta["text"].as_str() {
                                        if !text.is_empty() {
                                            text_buf.push_str(text);
                                            full_text.push_str(text);
                                            let _ = tx.send(sse_json(&json!({"token": text}))).await;
                                        }
                                    }
                                }
                                "input_json_delta" => {
                                    if let Some(partial) = delta["partial_json"].as_str() {
                                        if let Some(entry) = tool_blocks.get_mut(&idx) {
                                            entry.2.push_str(partial);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        "message_delta" => {
                            if let Some(reason) = obj["delta"]["stop_reason"].as_str() {
                                stop_reason = reason.to_string();
                            }
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }

        if stop_reason != "tool_use" {
            break;
        }

        let mut sorted_keys: Vec<usize> = tool_blocks.keys().cloned().collect();
        sorted_keys.sort();

        let _ = tx.send(sse_json(&json!({"status": "Using tools..."}))).await;

        // Build assistant message with tool_use content blocks
        let mut assistant_content: Vec<Value> = Vec::new();
        if !text_buf.is_empty() {
            assistant_content.push(json!({"type": "text", "text": text_buf}));
        }
        for idx in &sorted_keys {
            let (id, name, args_str) = &tool_blocks[idx];
            let input: Value = serde_json::from_str(args_str).unwrap_or(json!({}));
            assistant_content.push(json!({"type": "tool_use", "id": id, "name": name, "input": input}));
        }
        anthropic_msgs.push(json!({"role": "assistant", "content": assistant_content}));

        // Execute tools and build user message with tool_result blocks
        let mut result_blocks: Vec<Value> = Vec::new();
        for idx in &sorted_keys {
            let (id, name, args_str) = &tool_blocks[idx];
            let args: HashMap<String, Value> = serde_json::from_str(args_str).unwrap_or_default();
            eprintln!("[TOOL] call  : {} args={}", name, args_str);
            let result = tool_dispatch::execute(name, &args).await;
            eprintln!("[TOOL] result: {} => {}", name, &result[..result.len().min(200)]);
            result_blocks.push(json!({"type": "tool_result", "tool_use_id": id, "content": result}));
        }
        anthropic_msgs.push(json!({"role": "user", "content": result_blocks}));

        text_buf = String::new();
        tool_blocks = HashMap::new();
    }

    eprintln!("[CHAT] full_text len={}", full_text.len());
    let _ = tx.send("data: [DONE]\n\n".to_string()).await;
    let _ = tx.send(sse_json(&json!({"full_text": full_text}))).await;
}

fn sse_json(val: &Value) -> String {
    format!("data: {}\n\n", serde_json::to_string(val).unwrap_or_default())
}

/// Non-streaming Anthropic chat with tool-call loop. Returns the final reply text.
pub async fn non_stream_chat_anthropic(messages: Vec<Value>) -> Result<String, String> {
    let cfg = config::get();
    let primary = cfg.model_name.read().unwrap().clone();
    let mut model = primary.clone();
    let tools = registry::get_anthropic_tools_schema();

    let mut system = String::new();
    let mut msgs: Vec<Value> = Vec::new();
    for msg in messages {
        match msg["role"].as_str().unwrap_or("") {
            "system" => system = msg["content"].as_str().unwrap_or("").to_string(),
            role @ ("user" | "assistant") => msgs.push(json!({"role": role, "content": msg["content"].clone()})),
            _ => {}
        }
    }

    for _round in 0..MAX_TOOL_ROUNDS {
        let body = json!({
            "model": model,
            "max_tokens": 8096,
            "system": system,
            "messages": msgs,
            "tools": tools,
        });

        let resp = reqwest::Client::new()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &cfg.anthropic_api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send().await.map_err(|e| e.to_string())?;

        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(err) = json["error"]["message"].as_str() {
            if is_model_unavailable(err) && model == primary {
                eprintln!("[ANTHROPIC] {} unavailable, falling back to {}", model, cfg.fallback_model);
                model = cfg.fallback_model.clone();
                continue;
            }
            return Err(err.to_string());
        }

        let stop_reason = json["stop_reason"].as_str().unwrap_or("");
        let content = json["content"].as_array().cloned().unwrap_or_default();

        if stop_reason != "tool_use" {
            return Ok(content.iter()
                .filter_map(|b| if b["type"] == "text" { b["text"].as_str() } else { None })
                .collect::<Vec<_>>().join(""));
        }

        msgs.push(json!({"role": "assistant", "content": content.clone()}));

        let mut result_blocks: Vec<Value> = Vec::new();
        for block in &content {
            if block["type"] != "tool_use" { continue; }
            let id   = block["id"].as_str().unwrap_or("");
            let name = block["name"].as_str().unwrap_or("");
            let args: HashMap<String, Value> = serde_json::from_value(block["input"].clone()).unwrap_or_default();
            let result = tool_dispatch::execute(name, &args).await;
            result_blocks.push(json!({"type": "tool_result", "tool_use_id": id, "content": result}));
        }
        msgs.push(json!({"role": "user", "content": result_blocks}));
    }
    Ok(String::new())
}
