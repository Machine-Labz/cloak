use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        details: Vec<ValidationError>,
    },

    #[error("Merkle tree error: {0}")]
    MerkleTree(String),

    #[error("Artifact error: {0}")]
    Artifact(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hex decoding error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<ValidationError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_endpoints: Option<Vec<String>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl IntoResponse for IndexerError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match &self {
            IndexerError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                    Some("Internal database error occurred".to_string()),
                    None,
                )
            }
            IndexerError::Config(e) => {
                tracing::error!("Configuration error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error".to_string(),
                    Some("Service configuration error".to_string()),
                    None,
                )
            }
            IndexerError::Validation { message, details } => (
                StatusCode::BAD_REQUEST,
                "Validation error".to_string(),
                Some(message.clone()),
                Some(details.clone()),
            ),
            IndexerError::MerkleTree(msg) => (
                StatusCode::BAD_REQUEST,
                "Merkle tree error".to_string(),
                Some(msg.clone()),
                None,
            ),
            IndexerError::Artifact(msg) => (
                StatusCode::NOT_FOUND,
                "Artifact error".to_string(),
                Some(msg.clone()),
                None,
            ),
            IndexerError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "Not found".to_string(),
                Some(msg.clone()),
                None,
            ),
            IndexerError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    Some("An internal error occurred".to_string()),
                    None,
                )
            }
            IndexerError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "Bad request".to_string(),
                Some(msg.clone()),
                None,
            ),
            IndexerError::Io(e) => {
                tracing::error!("IO error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    Some("File system error occurred".to_string()),
                    None,
                )
            }
            IndexerError::HexDecode(e) => (
                StatusCode::BAD_REQUEST,
                "Invalid hex string".to_string(),
                Some(format!("Hex decoding failed: {}", e)),
                None,
            ),
            IndexerError::Json(e) => (
                StatusCode::BAD_REQUEST,
                "JSON error".to_string(),
                Some(format!("JSON processing failed: {}", e)),
                None,
            ),
            IndexerError::Base64Decode(e) => (
                StatusCode::BAD_REQUEST,
                "Base64 decode error".to_string(),
                Some(format!("Base64 decoding failed: {}", e)),
                None,
            ),
        };

        let error_response = ErrorResponse {
            error: error_type,
            message,
            details,
            available_endpoints: None,
            timestamp: chrono::Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

impl IndexerError {
    pub fn validation(message: &str, field_errors: Vec<(&str, &str)>) -> Self {
        let details = field_errors
            .into_iter()
            .map(|(field, msg)| ValidationError {
                field: field.to_string(),
                message: msg.to_string(),
            })
            .collect();

        IndexerError::Validation {
            message: message.to_string(),
            details,
        }
    }

    pub fn merkle_tree<T: Into<String>>(msg: T) -> Self {
        IndexerError::MerkleTree(msg.into())
    }

    pub fn artifact<T: Into<String>>(msg: T) -> Self {
        IndexerError::Artifact(msg.into())
    }

    pub fn not_found<T: Into<String>>(msg: T) -> Self {
        IndexerError::NotFound(msg.into())
    }

    pub fn internal<T: Into<String>>(msg: T) -> Self {
        IndexerError::Internal(msg.into())
    }

    pub fn bad_request<T: Into<String>>(msg: T) -> Self {
        IndexerError::BadRequest(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, IndexerError>;

// Helper function to create a 404 response with available endpoints
pub fn not_found_with_endpoints() -> Response {
    let available_endpoints = vec![
        "GET /".to_string(),
        "GET /health".to_string(),
        "POST /api/v1/deposit".to_string(),
        "GET /api/v1/merkle/root".to_string(),
        "GET /api/v1/merkle/proof/:index".to_string(),
        "GET /api/v1/notes/range".to_string(),
        "GET /api/v1/artifacts/withdraw/:version".to_string(),
        "GET /api/v1/artifacts/files/:version/:filename".to_string(),
    ];

    let error_response = ErrorResponse {
        error: "Not found".to_string(),
        message: Some("Route does not exist".to_string()),
        details: None,
        available_endpoints: Some(available_endpoints),
        timestamp: chrono::Utc::now(),
    };

    (StatusCode::NOT_FOUND, Json(error_response)).into_response()
}

// Tests can be added here when needed
