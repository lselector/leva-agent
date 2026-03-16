/// Leva CLI — interactive REPL with tool-call loop (Anthropic Claude).
use std::collections::HashMap;
use std::borrow::Cow;
use common::{config, tools::{memory::soul_read, registry}};
use rustyline::{Editor, Config, KeyEvent, KeyCode, Modifiers, Cmd, error::ReadlineError,
                history::DefaultHistory, hint::HistoryHinter, highlight::Highlighter};
use rustyline_derive::{Completer, Helper, Validator};

#[derive(Helper, Completer, Validator)]
struct LevaHelper { hinter: HistoryHinter }

impl rustyline::hint::Hinter for LevaHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}
impl Highlighter for LevaHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[2m{hint}\x1b[0m"))
    }
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> { Cow::Borrowed(line) }
    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool { false }
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }
}

const MAX_TOOL_ROUNDS: usize = 10;

#[tokio::main]
async fn main() {
    let cfg = config::get();
    let model = cfg.model_name.read().unwrap().clone();
    println!("Leva CLI (model: {model}). Type 'exit' or Ctrl+D to quit.");

    let system_prompt = load_system_prompt();
    let mut history: Vec<serde_json::Value> = Vec::new();

    let history_file = dirs_next::home_dir()
        .map(|h| h.join(".leva_history"))
        .unwrap_or_else(|| std::path::PathBuf::from(".leva_history"));

    let rl_config = Config::builder().history_ignore_space(true).build();
    let mut rl = Editor::<LevaHelper, DefaultHistory>::with_config(rl_config)
        .expect("failed to init readline");
    rl.set_helper(Some(LevaHelper { hinter: HistoryHinter::new() }));
    rl.bind_sequence(KeyEvent(KeyCode::Up, Modifiers::NONE), Cmd::HistorySearchBackward);
    rl.bind_sequence(KeyEvent(KeyCode::Down, Modifiers::NONE), Cmd::HistorySearchForward);
    let _ = rl.load_history(&history_file);

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let input = line.trim().to_string();
                if input.is_empty() { continue; }
                if input == "exit" || input == "quit" { println!("bye"); break; }
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
    let tools = registry::get_anthropic_tools_schema();
    let model = cfg.model_name.read().unwrap().clone();
    let mut msgs: Vec<serde_json::Value> = history.clone();

    for _round in 0..MAX_TOOL_ROUNDS {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 8096,
            "system": system_prompt,
            "messages": msgs,
            "tools": tools,
        });

        let resp = reqwest::Client::new()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &cfg.anthropic_api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send().await;

        let json: serde_json::Value = match resp {
            Ok(r) => r.json().await.unwrap_or_default(),
            Err(e) => return format!("Error: {e}"),
        };

        if let Some(err) = json["error"]["message"].as_str() {
            return format!("API error: {err}");
        }

        let stop_reason = json["stop_reason"].as_str().unwrap_or("");
        let content = json["content"].as_array().cloned().unwrap_or_default();

        if stop_reason != "tool_use" {
            let answer = content.iter()
                .filter_map(|b| if b["type"] == "text" { b["text"].as_str() } else { None })
                .collect::<Vec<_>>().join("");
            history.push(serde_json::json!({"role": "assistant", "content": answer}));
            return answer;
        }

        msgs.push(serde_json::json!({"role": "assistant", "content": content.clone()}));

        let mut result_blocks: Vec<serde_json::Value> = Vec::new();
        for block in &content {
            if block["type"] != "tool_use" { continue; }
            let id   = block["id"].as_str().unwrap_or("");
            let name = block["name"].as_str().unwrap_or("");
            let args: HashMap<String, serde_json::Value> =
                serde_json::from_value(block["input"].clone()).unwrap_or_default();
            let result = execute_tool(name, &args, cfg).await;
            result_blocks.push(serde_json::json!({
                "type": "tool_result",
                "tool_use_id": id,
                "content": result,
            }));
        }
        msgs.push(serde_json::json!({"role": "user", "content": result_blocks}));
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
    let url = format!("http://127.0.0.1:{}/tools/{name}", cfg.auto_port);
    match reqwest::Client::new().post(&url).json(args).send().await {
        Ok(r) => r.text().await.unwrap_or_else(|e| format!("Error: {e}")),
        Err(e) => format!("Error calling server_auto: {e}"),
    }
}
