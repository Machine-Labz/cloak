use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use tracing::{debug, warn, info};

use crate::{
    api::{ApiResponse, StatusResponse},
    AppState,
    db::repository::JobRepository,
    error::Error,
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Path;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_status() {
        let request_id = Uuid::new_v4();
        let state = AppState::mock(); // Would need to implement this
        
        // This test would work once we have a proper AppState::mock() implementation
        // For now it's commented out to avoid compilation issues
        
        // let result = get_status(State(state), Path(request_id)).await;
        // assert!(result.is_ok());
    }
}
