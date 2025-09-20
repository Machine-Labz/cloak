use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use base64::Engine;
use serde::Deserialize;
use uuid::Uuid;
use tracing::info;

use crate::{
    api::{ApiResponse, WithdrawResponse},
    error::Error,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub outputs: Vec<Output>,
    pub policy: Policy,
    pub public_inputs: PublicInputs,
    pub proof_bytes: String, // base64 encoded
}

#[derive(Debug, Deserialize)]
pub struct Output {
    pub recipient: String, // Solana address as string
    pub amount: u64,
}

#[derive(Debug, Deserialize)]
pub struct Policy {
    pub fee_bps: u16,
}

#[derive(Debug, Deserialize)]
pub struct PublicInputs {
    pub root: String,
    pub nf: String, // nullifier
    pub amount: u64,
    pub fee_bps: u16,
    pub outputs_hash: String,
}

pub async fn handle_withdraw(
    State(_state): State<AppState>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<impl IntoResponse, Error> {
    info!("Received withdraw request");

    // Validate the request
    validate_request(&payload)?;

    // For now, we'll create a mock response since we don't have full DB integration
    let request_id = Uuid::new_v4();
    
    info!("Processing withdraw request with ID: {}", request_id);

    // Simulate basic validation
    let fee_amount = (payload.public_inputs.amount * payload.public_inputs.fee_bps as u64) / 10000;
    let total_output_amount: u64 = payload.outputs.iter().map(|o| o.amount).sum();
    
    // Check conservation
    if total_output_amount + fee_amount != payload.public_inputs.amount {
        return Err(Error::ValidationError(
            "Conservation check failed: outputs + fee != amount".to_string()
        ));
    }

    // Check policy consistency
    if payload.policy.fee_bps != payload.public_inputs.fee_bps {
        return Err(Error::ValidationError(
            "Fee BPS mismatch between policy and public inputs".to_string()
        ));
    }

    // Decode proof bytes to validate format
    let _proof_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.proof_bytes)
        .map_err(|e| Error::ValidationError(format!("Invalid proof base64: {}", e)))?;

    info!("Withdraw request validated successfully");

    // Create successful response
    let response = WithdrawResponse {
        request_id,
        status: "queued".to_string(),
        message: "Withdraw request received and queued for processing".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

fn validate_request(request: &WithdrawRequest) -> Result<(), Error> {
    // Validate outputs
    if request.outputs.is_empty() {
        return Err(Error::ValidationError("No outputs specified".to_string()));
    }

    if request.outputs.len() > 10 {
        return Err(Error::ValidationError("Too many outputs (max 10)".to_string()));
    }

    // Validate amounts
    for output in &request.outputs {
        if output.amount == 0 {
            return Err(Error::ValidationError("Output amount cannot be zero".to_string()));
        }
        
        if output.recipient.len() < 32 {
            return Err(Error::ValidationError("Invalid recipient address".to_string()));
        }
    }

    // Validate public inputs
    if request.public_inputs.amount == 0 {
        return Err(Error::ValidationError("Amount cannot be zero".to_string()));
    }

    if request.public_inputs.fee_bps > 10000 {
        return Err(Error::ValidationError("Fee BPS cannot exceed 10000".to_string()));
    }

    // Validate hex strings
    if request.public_inputs.root.len() != 64 {
        return Err(Error::ValidationError("Root must be 64 hex characters".to_string()));
    }

    if request.public_inputs.nf.len() != 64 {
        return Err(Error::ValidationError("Nullifier must be 64 hex characters".to_string()));
    }

    if request.public_inputs.outputs_hash.len() != 64 {
        return Err(Error::ValidationError("Outputs hash must be 64 hex characters".to_string()));
    }

    // Validate hex format
    hex::decode(&request.public_inputs.root)
        .map_err(|_| Error::ValidationError("Root must be valid hex".to_string()))?;
    
    hex::decode(&request.public_inputs.nf)
        .map_err(|_| Error::ValidationError("Nullifier must be valid hex".to_string()))?;
    
    hex::decode(&request.public_inputs.outputs_hash)
        .map_err(|_| Error::ValidationError("Outputs hash must be valid hex".to_string()))?;

    // Validate proof bytes are valid base64
    if request.proof_bytes.is_empty() {
        return Err(Error::ValidationError("Proof bytes cannot be empty".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_request() {
        let valid_request = WithdrawRequest {
            outputs: vec![Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 1000000,
            }],
            policy: Policy { fee_bps: 100 },
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 1000000,
                fee_bps: 100,
                outputs_hash: "2".repeat(64),
            },
            proof_bytes: base64::encode(vec![0u8; 256]),
        };
        
        assert!(validate_request(&valid_request).is_ok());
    }

    #[test]
    fn test_validate_request_empty_outputs() {
        let invalid_request = WithdrawRequest {
            outputs: vec![],
            policy: Policy { fee_bps: 100 },
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 1000000,
                fee_bps: 100,
                outputs_hash: "2".repeat(64),
            },
            proof_bytes: base64::encode(vec![0u8; 256]),
        };
        
        assert!(validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validate_request_invalid_fee() {
        let invalid_request = WithdrawRequest {
            outputs: vec![Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 1000000,
            }],
            policy: Policy { fee_bps: 10001 }, // Too high
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 1000000,
                fee_bps: 10001,
                outputs_hash: "2".repeat(64),
            },
            proof_bytes: base64::encode(vec![0u8; 256]),
        };
        
        assert!(validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validate_request_invalid_hex() {
        let invalid_request = WithdrawRequest {
            outputs: vec![Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 1000000,
            }],
            policy: Policy { fee_bps: 100 },
            public_inputs: PublicInputs {
                root: "G".repeat(64), // Invalid hex
                nf: "1".repeat(64),
                amount: 1000000,
                fee_bps: 100,
                outputs_hash: "2".repeat(64),
            },
            proof_bytes: base64::encode(vec![0u8; 256]),
        };
        
        assert!(validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validate_request_empty_proof() {
        let invalid_request = WithdrawRequest {
            outputs: vec![Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 1000000,
            }],
            policy: Policy { fee_bps: 100 },
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 1000000,
                fee_bps: 100,
                outputs_hash: "2".repeat(64),
            },
            proof_bytes: "".to_string(), // Empty base64
        };
        
        assert!(validate_request(&invalid_request).is_err());
    }
}
