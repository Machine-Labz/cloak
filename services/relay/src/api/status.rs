use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::{ApiResponse, StatusResponse},
    config::Config,
    error::Error,
};

#[derive(Debug, Serialize)]
pub struct StatusRequest {
    pub request_id: Uuid,
}

pub async fn get_status(
    State(_config): State<Arc<Config>>,
    Path(request_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    // TODO: Look up status from database
    // For now, return a mock response
    let status = if request_id.as_u128() % 2 == 0 {
        "completed"
    } else {
        "pending"
    };

    let response = StatusResponse {
        request_id,
        status: status.to_string(),
        tx_id: if status == "completed" {
            Some("5F1LhTohUMJzVFs3pFw58j5G4d5uGYHddiTb5C2L5E5X".to_string())
        } else {
            None
        },
        error: None,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: crate::config::ServerConfig {
                port: 3001,
                host: "0.0.0.0".to_string(),
                request_timeout_seconds: 30,
            },
            solana: crate::config::SolanaConfig {
                rpc_url: "http://localhost:8899".to_string(),
                ws_url: "ws://localhost:8900".to_string(),
                commitment: "confirmed".to_string(),
                program_id: "11111111111111111111111111111111".to_string(),
                withdraw_authority: None,
                max_retries: 3,
                retry_delay_ms: 1000,
            },
            database: crate::config::DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/relay".to_string(),
                max_connections: 5,
            },
            metrics: crate::config::MetricsConfig {
                enabled: true,
                port: 9090,
                route: "/metrics".to_string(),
            },
        })
    }

    #[tokio::test]
    async fn test_get_status() {
        let config = create_test_config();
        
        // Test with even request ID (mocked as completed)
        let request_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let response = get_status(
            State(config.clone()),
            Path(request_id),
        ).await;
        
        assert!(response.is_ok());
        
        // Test with odd request ID (mocked as pending)
        let request_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let response = get_status(
            State(config),
            Path(request_id),
        ).await;
        
        assert!(response.is_ok());
    }
}
