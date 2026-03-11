/// Job management endpoints — /jobs/start, /jobs/list, /jobs/status/{id}
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use crate::job_runner::JobState;

pub fn router(state: JobState) -> Router {
    Router::new()
        .route("/jobs/start", post(start_job))
        .route("/jobs/list", get(list_jobs))
        .route("/jobs/status/{job_id}", get(job_status))
        .with_state(state)
}

#[derive(Deserialize)]
struct StartReq {
    #[serde(rename = "type")]
    job_type: String,
    params: Option<Value>,
}

async fn start_job(State(state): State<JobState>, Json(req): Json<StartReq>) -> Json<Value> {
    let job = state.start_job(&req.job_type, req.params.unwrap_or_default());
    Json(serde_json::json!({"job_id": job.job_id, "status": job.status}))
}

async fn list_jobs(State(state): State<JobState>) -> Json<Value> {
    let jobs = state.list_jobs();
    Json(serde_json::to_value(jobs).unwrap_or_default())
}

async fn job_status(
    State(state): State<JobState>,
    Path(job_id): Path<String>,
) -> axum::response::Response {
    match state.get_job(&job_id) {
        Some(job) => axum::response::Response::builder()
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&job).unwrap()))
            .unwrap(),
        None => axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from(r#"{"detail":"Job not found"}"#))
            .unwrap(),
    }
}
