mod routes_chat;
mod routes_models;
mod session_store;
mod streaming;
mod tool_dispatch;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = common::config::get();
    let port = cfg.llm_port;
    let frontend_dir = cfg.base_dir.join("frontend");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = session_store::AppState::new();

    let app = Router::new()
        .route("/api/health", get(health))
        .merge(routes_chat::router(state.clone()))
        .merge(routes_models::router())
        .fallback_service(ServeDir::new(&frontend_dir))
        .layer(cors);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Server LLM listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok", "server": "llm"}))
}
