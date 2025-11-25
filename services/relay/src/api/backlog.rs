use axum::{extract::State, response::Json};
use serde::Serialize;

use crate::db::repository::JobRepository;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct BacklogStatus {
    pub pending_count: usize,
    pub queued_jobs: Vec<String>,
}

pub async fn get_backlog_status(
    State(state): State<AppState>,
) -> Result<Json<BacklogStatus>, String> {
    // Get queued jobs
    let jobs = state
        .job_repo
        .get_queued_jobs(100)
        .await
        .map_err(|e| format!("Failed to get backlog: {}", e))?;

    // Extract job IDs
    let job_ids: Vec<String> = jobs.iter().map(|j| j.id.to_string()).collect();

    Ok(Json(BacklogStatus {
        pending_count: jobs.len(),
        queued_jobs: job_ids,
    }))
}
