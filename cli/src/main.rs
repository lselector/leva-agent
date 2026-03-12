/// Jarvis CLI — interactive REPL with tool-call loop.
use std::collections::HashMap;
use common::{config, tools::{memory::soul_read, registry}};
use rustyline::{DefaultEditor, error::ReadlineError};

const MAX_TOOL_ROUNDS: usize = 10;

#[tokio::main]
async fn main() {
    let cfg = config::get();
    let model = cfg.model_name.read().unwrap().clone();

    println!("Jarvis CLI (model: {model}). Type 'exit' or Ctrl+D to quit.");

    let system_prompt = load_system_prompt();
    let mut history: Vec<serde_json::Value> = Vec::new();

    let history_file = dirs_next::home_dir()
        .map(|h| h.join(".jarvis_history"))
        .unwrap_or_else(|| std::path::PathBuf::from(".jarvis_history"));

    let mut rl = DefaultEditor::new().expect("failed to init readline");
    let _ = rl.load_history(&history_file);

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let input = line.trim().to_string();
                if input.is_empty() { continue; }
                if input == "exit" || input == "quit" {
                    println!("bye");
                    break;
                }
                rl.add_history_entry(&input).ok();
                history.push(serde_json::json!({"role": "user", "content": input}));
                let reply = run_agent(&system_prompt, &mut history, cfg).await;
                println!("{reply}");
            }
            Err(ReadlineError::Interrupted) => { println!("^C"); continue; }
            Err(ReadlineError::Eof) => { println!("bye"); break; }
            Err(e) => { eprintln!("readline error: {e}"); break; }
        }
    }

    let _ = rl.save_history(&history_file);
}

fn load_system_prompt() -> String {
    let cfg = config::get();
    let base = std::fs::read_to_string(cfg.prompts_dir.join("SYSTEM.md")).unwrap_or_default();
    let soul = soul_read().unwrap_or_default();
    format!("{base}\n\n# Core Identity (Soul)\n\n{soul}")
}

async fn run_agent(
    system_prompt: &str,
    history: &mut Vec<serde_json::Value>,
    cfg: &'static common::config::Config,
) -> String {
    let tool_schemas = registry::get_tools_schema();
    let model = cfg.model_name.read().unwrap().clone();

    for _round in 0..MAX_TOOL_ROUNDS {
        let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];
        messages.extend_from_slice(history);

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "tools": tool_schemas,
        });

        let resp = reqwest::Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&cfg.openai_api_key)
            .json(&body)
            .send()
            .await;

        let json: serde_json::Value = match resp {
            Ok(r) => r.json().await.unwrap_or_default(),
            Err(e) => return format!("Error calling OpenAI: {e}"),
        };

        let choice = &json["choices"][0];
        let finish = choice["finish_reason"].as_str().unwrap_or("");
        let message = &choice["message"];

        if finish == "tool_calls" {
            // Append assistant message
            history.push(message.clone());

            for tc in message["tool_calls"].as_array().unwrap_or(&vec![]) {
                let name = tc["function"]["name"].as_str().unwrap_or("");
                let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let tc_id = tc["id"].as_str().unwrap_or("");

                let args: HashMap<String, serde_json::Value> =
                    serde_json::from_str(args_str).unwrap_or_default();
                let result = execute_tool(name, &args, cfg).await;

                history.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tc_id,
                    "content": result,
                }));
            }
            continue;
        }

        let answer = message["content"].as_str().unwrap_or("").to_string();
        history.push(serde_json::json!({"role": "assistant", "content": answer}));
        return answer;
    }
    "Error: tool call loop exceeded max rounds".to_string()
}

async fn execute_tool(
    name: &str,
    args: &HashMap<String, serde_json::Value>,
    cfg: &'static common::config::Config,
) -> String {
    if registry::local_tool_names().contains(&name) {
        return registry::execute(name, args);
    }
    // Remote tool — forward to server_auto
    let url = format!("http://127.0.0.1:{}/tools/{name}", cfg.auto_port);
    match reqwest::Client::new().post(&url).json(args).send().await {
        Ok(r) => r.text().await.unwrap_or_else(|e| format!("Error: {e}")),
        Err(e) => format!("Error calling server_auto: {e}"),
    }
}
