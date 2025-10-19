use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    api::{ApiResponse, StatusResponse},
    db::repository::JobRepository,
    error::Error,
    AppState,
};

pub async fn get_status(
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    info!("Status endpoint called for request: {}", request_id);
    debug!("Getting status for request: {}", request_id);

    // Look up job by request ID
    info!("Querying database for job with request_id: {}", request_id);
    match state.job_repo.get_job_by_request_id(request_id).await {
        Ok(Some(job)) => {
            info!("Job found in database: {:?}", job.status);
            let response = StatusResponse {
                request_id,
                status: job.status.to_string(),
                tx_id: job.tx_id.clone(),
                error: job.error_message.clone(),
                created_at: job.created_at,
                completed_at: job.completed_at,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Ok(None) => {
            warn!("Job not found in database for request ID: {}", request_id);
            Err(Error::NotFound)
        }
        Err(e) => {
            warn!("Database error while looking up job {}: {}", request_id, e);
            Err(e)
        }
    }
}
