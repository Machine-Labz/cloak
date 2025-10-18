pub mod status;
pub mod withdraw;
pub mod validator_agent;
pub mod prove_local;

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(data: T) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(data),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub request_id: Uuid,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub request_id: Uuid,
    pub status: String,
    pub tx_id: Option<String>,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::json;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test");
        assert!(response.success);
        assert_eq!(response.data, Some("test"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}
