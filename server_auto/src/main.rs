mod routes_files;
mod routes_jobs;
mod job_runner;
mod cdp_browser;
mod gmail_api;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = common::config::get();
    let port = cfg.auto_port;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let job_state = job_runner::JobState::new();

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes_files::router())
        .merge(routes_jobs::router(job_state))
        .merge(cdp_browser::routes::router())
        .merge(gmail_api::routes::router())
        .layer(cors);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Server Auto listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok", "server": "auto"}))
}
