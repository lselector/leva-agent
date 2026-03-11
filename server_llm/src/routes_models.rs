/// Model selection endpoints — GET /api/models, PUT /api/models/current
use axum::{routing::{get, put}, Router, Json};
use serde::{Deserialize, Serialize};
use common::config;

const AVAILABLE_MODELS: &[&str] = &[
    "gpt-4.1-mini",
    "gpt-4.1",
    "gpt-4o",
    "gpt-4o-mini",
    "o3-mini",
];

#[derive(Deserialize)]
pub struct ModelSwitch {
    pub model: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/models", get(get_models))
        .route("/api/models/current", put(set_model))
}

async fn get_models() -> Json<serde_json::Value> {
    let current = config::get().model_name.read().unwrap().clone();
    Json(serde_json::json!({
        "current": current,
        "available": AVAILABLE_MODELS,
    }))
}

async fn set_model(Json(req): Json<ModelSwitch>) -> Json<serde_json::Value> {
    if !AVAILABLE_MODELS.contains(&req.model.as_str()) {
        return Json(serde_json::json!({
            "error": format!("Unknown model: {}", req.model),
            "available": AVAILABLE_MODELS,
        }));
    }
    *config::get().model_name.write().unwrap() = req.model.clone();
    Json(serde_json::json!({"status": "ok", "model": req.model}))
}
