/// In-memory background job runner using tokio tasks.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub job_id: String,
    pub job_type: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: f64,
    pub completed_at: Option<f64>,
}

impl Job {
    fn new(job_type: &str) -> Self {
        let id = &Uuid::new_v4().to_string()[..8];
        Self {
            job_id: id.to_string(),
            job_type: job_type.to_string(),
            status: "pending".to_string(),
            result: None,
            error: None,
            created_at: unix_now(),
            completed_at: None,
        }
    }
}

fn unix_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[derive(Clone)]
pub struct JobState {
    pub jobs: Arc<Mutex<HashMap<String, Job>>>,
}

impl JobState {
    pub fn new() -> Self {
        Self { jobs: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn start_job(&self, job_type: &str, _params: Value) -> Job {
        let mut job = Job::new(job_type);
        job.status = "running".to_string();
        let job_id = job.job_id.clone();
        let jtype = job.job_type.clone();
        self.jobs.lock().unwrap().insert(job_id.clone(), job.clone());

        let state = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let mut jobs = state.jobs.lock().unwrap();
            if let Some(j) = jobs.get_mut(&job_id) {
                if jtype == "test" {
                    j.result = Some("test completed".to_string());
                } else {
                    j.result = Some(format!("unknown type: {jtype}"));
                }
                j.status = "completed".to_string();
                j.completed_at = Some(unix_now());
            }
        });
        job
    }

    pub fn get_job(&self, job_id: &str) -> Option<Job> {
        self.jobs.lock().unwrap().get(job_id).cloned()
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().values().cloned().collect()
    }
}
