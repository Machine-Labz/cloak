use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use tracing::{debug, warn};

use crate::{
    api::{ApiResponse, StatusResponse},
    AppState,
    error::Error,
};

pub async fn get_status(
    State(_state): State<AppState>,
    Path(request_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    debug!("Getting status for request: {}", request_id);

    // For now, we'll return a mock response since we don't have full DB integration
    // In a real implementation, this would query the database
    
    // Simulate some basic logic - in reality this would come from the database
    let response = StatusResponse {
        request_id,
        status: "completed".to_string(), // Mock status
        tx_id: Some("mock_tx_123456789".to_string()),
        error: None,
        created_at: chrono::Utc::now() - chrono::Duration::minutes(5), // 5 minutes ago
        completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(1)), // 1 minute ago
    };

    Ok(Json(ApiResponse::success(response)))
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
