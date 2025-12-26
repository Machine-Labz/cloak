#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found")]
    NotFound,

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::{http::StatusCode, Json};
        use serde_json::json;
        use tracing::{error, warn};

        let (status, message) = match &self {
            Error::BadRequest(msg) => {
                warn!("❌ Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            Error::NotFound => {
                warn!("❌ Not found");
                (StatusCode::NOT_FOUND, "Not found".to_string())
            }
            Error::ValidationError(msg) => {
                warn!("❌ Validation error: {}", msg);
                (
                    StatusCode::BAD_REQUEST,
                    format!("Validation error: {}", msg),
                )
            }
            Error::DatabaseError(msg) => {
                error!("❌ Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", msg),
                )
            }
            _ => {
                // Log the actual error for debugging
                error!("❌ Internal server error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", self),
                )
            }
        };

        let body = Json(json!({
            "error": true,
            "message": message
        }));

        (status, body).into_response()
    }
}
