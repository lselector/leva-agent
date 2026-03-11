/// Axum routes for CDP browser and LinkedIn operations.
use axum::{routing::{get, post}, Router, Json};
use serde::Deserialize;
use serde_json::Value;
use super::{actions, client, perplexity};

pub fn router() -> Router {
    Router::new()
        .route("/api/web/tabs", get(web_tabs))
        .route("/api/web/search", post(web_search))
        .route("/api/web/fetch", post(web_fetch))
        .route("/api/web/screenshot", post(web_screenshot))
        .route("/api/linkedin/feed", post(linkedin_feed))
        .route("/api/linkedin/like", post(linkedin_like))
        // Tool forwarding endpoint used by server_llm
        .route("/tools/web_search", post(tool_web_search))
        .route("/tools/web_fetch", post(tool_web_fetch))
        .route("/tools/linkedin_feed", post(tool_linkedin_feed))
        .route("/tools/linkedin_like", post(tool_linkedin_like))
        .route("/tools/web_research", post(tool_web_research))
}

#[derive(Deserialize)] struct SearchReq { query: String }
#[derive(Deserialize)] struct FetchReq { url: String }
#[derive(Deserialize)] struct LikeReq { post_index: Option<usize>, dry_run: Option<bool> }

async fn web_tabs() -> axum::response::Response {
    match client::list_tabs().await {
        Ok(tabs) => json_ok(serde_json::to_value(tabs).unwrap_or_default()),
        Err(e) => json_err(503, &format!("Chrome CDP not available: {e}")),
    }
}

async fn web_search(Json(req): Json<SearchReq>) -> axum::response::Response {
    match actions::web_search(&req.query).await {
        Ok(r) => json_ok(serde_json::to_value(r).unwrap_or_default()),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn web_fetch(Json(req): Json<FetchReq>) -> axum::response::Response {
    match actions::web_fetch(&req.url, 10_000).await {
        Ok(content) => json_ok(serde_json::json!({"url": req.url, "content": content})),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn web_screenshot(Json(req): Json<FetchReq>) -> axum::response::Response {
    match actions::web_screenshot(&req.url).await {
        Ok(png) => axum::response::Response::builder()
            .header("Content-Type", "image/png")
            .body(axum::body::Body::from(png))
            .unwrap(),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn linkedin_feed() -> axum::response::Response {
    match actions::linkedin_feed().await {
        Ok(posts) => json_ok(serde_json::to_value(posts).unwrap_or_default()),
        Err(e) => json_err(500, &e.to_string()),
    }
}

async fn linkedin_like(Json(req): Json<LikeReq>) -> axum::response::Response {
    let idx = req.post_index.unwrap_or(0);
    let dry = req.dry_run.unwrap_or(true);
    match actions::linkedin_like(idx, dry).await {
        Ok(v) => json_ok(v),
        Err(e) => json_err(500, &e.to_string()),
    }
}

// Tool proxy endpoints (called by server_llm tool_dispatch)
async fn tool_web_search(Json(args): Json<Value>) -> String {
    let query = args["query"].as_str().unwrap_or("");
    match actions::web_search(query).await {
        Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}

async fn tool_web_fetch(Json(args): Json<Value>) -> String {
    let url = args["url"].as_str().unwrap_or("");
    match actions::web_fetch(url, 10_000).await {
        Ok(c) => c,
        Err(e) => format!("Error: {e}"),
    }
}

async fn tool_linkedin_feed(Json(_args): Json<Value>) -> String {
    match actions::linkedin_feed().await {
        Ok(posts) => serde_json::to_string(&posts).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}

async fn tool_linkedin_like(Json(args): Json<Value>) -> String {
    let idx = args["post_index"].as_u64().unwrap_or(0) as usize;
    let dry = args["dry_run"].as_bool().unwrap_or(true);
    match actions::linkedin_like(idx, dry).await {
        Ok(v) => serde_json::to_string(&v).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}

async fn tool_web_research(Json(args): Json<Value>) -> String {
    let query = args["query"].as_str().unwrap_or("");
    match perplexity::web_research(query).await {
        Ok(r) => r,
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
