use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found")]
    NotFound,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_message(&self) -> String {
        match self {
            Self::BadRequest(msg) => msg.clone(),
            Self::ValidationError(msg) => msg.clone(),
            Self::InternalServerError(msg) => msg.clone(),
            Self::ConfigError(msg) => msg.clone(),
            _ => self.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            success: false,
            error: self.error_message(),
            details: None,
        };

        (status, Json(error_response)).into_response()
    }
}

// Convert configuration errors
impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error::ConfigError(err.to_string())
    }
}

// For anyhow errors
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::InternalServerError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            Error::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(Error::NotFound.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            Error::InternalServerError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(
            Error::BadRequest("test".to_string()).error_message(),
            "test"
        );
        assert_eq!(
            Error::ValidationError("test".to_string()).error_message(),
            "test"
        );
        assert_eq!(
            Error::InternalServerError("test".to_string()).error_message(),
            "test"
        );
    }
}
