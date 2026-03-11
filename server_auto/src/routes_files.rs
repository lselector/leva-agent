/// File operation endpoints — /files/list, /files/read, /files/write
use axum::{routing::{get, post}, Router, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use common::config;

pub fn router() -> Router {
    Router::new()
        .route("/files/list", get(file_list))
        .route("/files/read", post(file_read))
        .route("/files/write", post(file_write))
}

fn safe_path(rel: &str) -> Option<PathBuf> {
    let base = &config::get().base_dir;
    let p = base.join(rel);
    let resolved = p.canonicalize().unwrap_or_else(|_| p.clone());
    if resolved.starts_with(base) { Some(resolved) } else { None }
}

#[derive(Deserialize)]
struct ReadReq { path: String }

#[derive(Deserialize)]
struct WriteReq { path: String, content: String }

async fn file_list(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let rel = params.get("path").map(|s| s.as_str()).unwrap_or(".");
    let recursive = params.get("recursive").map(|s| s == "true").unwrap_or(false);
    let Some(target) = safe_path(rel) else {
        return Json(serde_json::json!({"error": "path outside workspace"}));
    };
    if !target.exists() {
        return Json(serde_json::json!({"error": format!("not found: {rel}")}));
    }
    if !target.is_dir() {
        return Json(serde_json::json!({"error": "not a directory"}));
    }
    let base = &config::get().base_dir;
    let mut files = Vec::new();
    if recursive {
        collect_recursive(&target, base, &mut files);
    } else {
        if let Ok(rd) = std::fs::read_dir(&target) {
            let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let p = entry.path();
                let rel_p = p.strip_prefix(base).unwrap_or(&p);
                let suffix = if p.is_dir() { "/" } else { "" };
                files.push(format!("{}{suffix}", rel_p.display()));
            }
        }
    }
    Json(serde_json::json!({"path": rel, "files": files}))
}

fn collect_recursive(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let p = entry.path();
            if p.is_dir() {
                collect_recursive(&p, base, out);
            } else {
                let rel = p.strip_prefix(base).unwrap_or(&p);
                out.push(rel.display().to_string());
            }
        }
    }
}

async fn file_read(Json(req): Json<ReadReq>) -> Json<Value> {
    let Some(p) = safe_path(&req.path) else {
        return Json(serde_json::json!({"error": "path outside workspace"}));
    };
    match std::fs::read_to_string(&p) {
        Ok(content) => Json(serde_json::json!({
            "path": req.path,
            "content": content,
            "chars": content.len(),
        })),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn file_write(Json(req): Json<WriteReq>) -> Json<Value> {
    let Some(p) = safe_path(&req.path) else {
        return Json(serde_json::json!({"error": "path outside workspace"}));
    };
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&p, &req.content) {
        Ok(_) => Json(serde_json::json!({
            "status": "ok",
            "path": req.path,
            "chars": req.content.len(),
        })),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}
