/// Tool registry — maps names to functions and builds OpenAI JSON schemas.
use serde_json::{json, Value};
use std::collections::HashMap;

pub type ToolFn = fn(&HashMap<String, Value>) -> String;

/// Execute a tool by name with JSON args. Returns result string.
pub fn execute(name: &str, args: &HashMap<String, Value>) -> String {
    use super::{files, memory};

    let str_arg = |key: &str| -> &str {
        args.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    };
    match name {
        "file_read" => files::file_read(str_arg("path"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "file_write" => files::file_write(str_arg("path"), str_arg("content"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "memory_append" => memory::memory_append(str_arg("text"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "memory_search" => memory::memory_search(str_arg("query"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "soul_read" => memory::soul_read()
            .unwrap_or_else(|e| format!("Error: {e}")),
        "memory_topic_write" => memory::memory_topic_write(str_arg("topic"), str_arg("content"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "memory_topic_read" => memory::memory_topic_read(str_arg("topic"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "memory_topic_list" => memory::memory_topic_list()
            .unwrap_or_else(|e| format!("Error: {e}")),
        "reference_read" => memory::reference_read(str_arg("name"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "reference_write" => memory::reference_write(str_arg("name"), str_arg("content"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        "reference_list" => memory::reference_list()
            .unwrap_or_else(|e| format!("Error: {e}")),
        "reference_search" => memory::reference_search(str_arg("query"))
            .unwrap_or_else(|e| format!("Error: {e}")),
        _ => format!("Error: unknown local tool '{name}'"),
    }
}

/// Tool names that are handled locally (not forwarded to server_auto).
pub fn local_tool_names() -> &'static [&'static str] {
    &[
        "file_read",
        "file_write",
        "memory_append",
        "memory_search",
        "soul_read",
        "memory_topic_write",
        "memory_topic_read",
        "memory_topic_list",
        "reference_read",
        "reference_write",
        "reference_list",
        "reference_search",
    ]
}

/// Return all tool schemas in OpenAI format.
pub fn get_tools_schema() -> Vec<Value> {
    vec![
        tool("file_read", "Read a small text file from the workspace.",
            json!({"type":"object","properties":{"path":{"type":"string","description":"Relative path to the file."}},"required":["path"]})),
        tool("file_write", "Overwrite or create a text file.",
            json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]})),
        tool("memory_append", "Append a note to today's daily memory file.",
            json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"]})),
        tool("memory_search", "Search daily and topic memory files for a query.",
            json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]})),
        tool("soul_read", "Read all Layer 1 soul files (identity, agents, user profile).",
            json!({"type":"object","properties":{}})),
        tool("memory_topic_write", "Write or overwrite a topic summary in Layer 2 memory.",
            json!({"type":"object","properties":{"topic":{"type":"string"},"content":{"type":"string"}},"required":["topic","content"]})),
        tool("memory_topic_read", "Read a specific topic file from Layer 2 memory.",
            json!({"type":"object","properties":{"topic":{"type":"string"}},"required":["topic"]})),
        tool("memory_topic_list", "List all topic files in Layer 2 memory.",
            json!({"type":"object","properties":{}})),
        tool("reference_read", "Read a document from the Layer 3 reference library.",
            json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]})),
        tool("reference_write", "Write or overwrite a document in the Layer 3 reference library.",
            json!({"type":"object","properties":{"name":{"type":"string"},"content":{"type":"string"}},"required":["name","content"]})),
        tool("reference_list", "List all documents in the Layer 3 reference library.",
            json!({"type":"object","properties":{}})),
        tool("reference_search", "Search lines in Layer 3 reference documents.",
            json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]})),
        tool("gmail_inbox", "Get recent emails from Gmail inbox.",
            json!({"type":"object","properties":{"max_results":{"type":"integer","description":"Max emails to return (default 15)."}}})),
        tool("gmail_send", "Send an email via Gmail.",
            json!({"type":"object","properties":{"to":{"type":"string"},"subject":{"type":"string"},"body":{"type":"string"}},"required":["to","subject","body"]})),
        tool("gmail_search", "Search Gmail using query syntax (e.g. 'is:unread', 'from:bob').",
            json!({"type":"object","properties":{"query":{"type":"string"},"max_results":{"type":"integer"}},"required":["query"]})),
        tool("web_search", "Search the web via Google and return top results.",
            json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]})),
        tool("web_fetch", "Fetch a URL and return its main text content.",
            json!({"type":"object","properties":{"url":{"type":"string"}},"required":["url"]})),
        tool("linkedin_feed", "Get recent LinkedIn feed posts.",
            json!({"type":"object","properties":{}})),
        tool("linkedin_like", "Like a LinkedIn feed post by index. Use dry_run=true to preview.",
            json!({"type":"object","properties":{"post_index":{"type":"integer"},"dry_run":{"type":"boolean"}},"required":["post_index"]})),
        tool("web_research", "Research a topic using Perplexity AI (web-grounded, no browser needed).",
            json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]})),
    ]
}

fn tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters,
        }
    })
}
