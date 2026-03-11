/// Dispatch tool calls — local tools handled here, remote tools forwarded to server_auto.
use std::collections::HashMap;
use serde_json::Value;
use common::{config, tools::registry};

/// Execute a tool call by name with parsed args.
/// Local tools are run in-process; remote tools are forwarded to server_auto via HTTP.
pub async fn execute(name: &str, args: &HashMap<String, Value>) -> String {
    if registry::local_tool_names().contains(&name) {
        return registry::execute(name, args);
    }
    // Remote tool — forward to server_auto
    let auto_port = config::get().auto_port;
    let url = format!("http://127.0.0.1:{auto_port}/tools/{name}");
    let client = reqwest::Client::new();
    match client.post(&url).json(args).send().await {
        Ok(resp) => resp.text().await.unwrap_or_else(|e| format!("Error reading response: {e}")),
        Err(e) => format!("Error calling server_auto tool '{name}': {e}"),
    }
}
