use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::{ApiResponse, WithdrawResponse},
    config::Config,
    error::Error,
};

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub proof: String,          // Base58 encoded proof
    pub public_inputs: String,  // Base58 encoded public inputs
    pub outputs: Vec<Output>,   // List of output recipients and amounts
    pub fee_bps: u16,          // Fee in basis points (1/10000)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Output {
    pub recipient: String,      // Base58 encoded public key
    pub amount: u64,           // Amount in lamports
}

pub async fn handle_withdraw(
    State(_config): State<Arc<Config>>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<impl IntoResponse, Error> {
    // Generate a unique request ID
    let request_id = Uuid::new_v4();
    
    // TODO: Validate the request
    validate_withdraw_request(&payload)?;
    
    // TODO: Add to job queue
    
    // TODO: Return immediate response with request ID
    let response = WithdrawResponse {
        request_id,
        status: "queued".to_string(),
    };
    
    Ok((StatusCode::ACCEPTED, Json(ApiResponse::success(response))))
}

fn validate_withdraw_request(request: &WithdrawRequest) -> Result<(), Error> {
    // Validate proof length (256 bytes)
    let proof_bytes = bs58::decode(&request.proof)
        .into_vec()
        .map_err(|_| Error::BadRequest("Invalid proof format".to_string()))?;
    
    if proof_bytes.len() != 256 {
        return Err(Error::BadRequest("Invalid proof length".to_string()));
    }
    
    // Validate public inputs length (64 bytes)
    let public_inputs_bytes = bs58::decode(&request.public_inputs)
        .into_vec()
        .map_err(|_| Error::BadRequest("Invalid public inputs format".to_string()))?;
    
    if public_inputs_bytes.len() != 64 {
        return Err(Error::BadRequest("Invalid public inputs length".to_string()));
    }
    
    // Validate outputs
    if request.outputs.is_empty() {
        return Err(Error::BadRequest("At least one output is required".to_string()));
    }
    
    // Validate each output
    for output in &request.outputs {
        if output.amount == 0 {
            return Err(Error::BadRequest("Output amount must be greater than zero".to_string()));
        }
        
        // Validate recipient address format
        if bs58::decode(&output.recipient).into_vec().is_err() {
            return Err(Error::BadRequest("Invalid recipient address format".to_string()));
        }
    }
    
    // Validate fee is reasonable (0-1000 bps = 0-10%)
    if request.fee_bps > 1000 {
        return Err(Error::BadRequest("Fee must be 1000 bps (10%) or less".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use std::sync::Arc;

    fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: crate::config::ServerConfig {
                port: 3000,
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
    async fn test_handle_withdraw_validation() {
        let config = create_test_config();
        
        // Test valid request
        let valid_request = WithdrawRequest {
            proof: bs58::encode(vec![0u8; 256]).into_string(),
            public_inputs: bs58::encode(vec![0u8; 64]).into_string(),
            outputs: vec![Output {
                recipient: bs58::encode(vec![0u8; 32]).into_string(),
                amount: 1000,
            }],
            fee_bps: 10,
        };
        
        let response = handle_withdraw(
            State(config.clone()),
            Json(valid_request),
        ).await;
        
        assert!(response.is_ok());
        
        // Test invalid proof length
        let invalid_proof = WithdrawRequest {
            proof: bs58::encode(vec![0u8; 255]).into_string(), // 255 bytes instead of 256
            public_inputs: bs58::encode(vec![0u8; 64]).into_string(),
            outputs: vec![Output {
                recipient: bs58::encode(vec![0u8; 32]).into_string(),
                amount: 1000,
            }],
            fee_bps: 10,
        };
        
        let response = handle_withdraw(
            State(config.clone()),
            Json(invalid_proof),
        ).await;
        
        assert!(matches!(
            response.err().unwrap(),
            Error::BadRequest(msg) if msg == "Invalid proof length"
        ));
    }
}
