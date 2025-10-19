use axum::{extract::State, response::IntoResponse, Json};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    api::{ApiResponse, WithdrawResponse},
    db::models::CreateJob,
    db::repository::{JobRepository, NullifierRepository},
    error::Error,
    queue::JobMessage,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub outputs: Vec<Output>,
    pub policy: Policy,
    pub public_inputs: PublicInputs,
    pub proof_bytes: String, // base64 encoded
}

#[derive(Debug, Deserialize, Serialize)]
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
    State(state): State<AppState>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<impl IntoResponse, Error> {
    info!("Received withdraw request");

    // Validate the request
    validate_request(&payload)?;

    let request_id = Uuid::new_v4();
    info!("Processing withdraw request with ID: {}", request_id);

    // Decode and validate proof bytes
    let proof_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.proof_bytes)
        .map_err(|e| Error::ValidationError(format!("Invalid proof base64: {}", e)))?;

    // Parse public inputs
    let root_hash = hex::decode(&payload.public_inputs.root)
        .map_err(|e| Error::ValidationError(format!("Invalid root hex: {}", e)))?;
    let nullifier = hex::decode(&payload.public_inputs.nf)
        .map_err(|e| Error::ValidationError(format!("Invalid nullifier hex: {}", e)))?;
    let outputs_hash = hex::decode(&payload.public_inputs.outputs_hash)
        .map_err(|e| Error::ValidationError(format!("Invalid outputs hash hex: {}", e)))?;

    // Check for nullifier double-spend
    if state.nullifier_repo.exists_nullifier(&nullifier).await? {
        return Err(Error::ValidationError(
            "Nullifier already exists (double spend)".to_string(),
        ));
    }

    // Encode public inputs for storage (canonical 104-byte format)
    // Format: root(32) || nf(32) || outputs_hash(32) || amount(8) = 104 bytes
    let mut public_inputs_bytes = Vec::new();
    public_inputs_bytes.extend_from_slice(&root_hash);
    public_inputs_bytes.extend_from_slice(&nullifier);
    public_inputs_bytes.extend_from_slice(&outputs_hash);
    public_inputs_bytes.extend_from_slice(&payload.public_inputs.amount.to_le_bytes());

    // Create job in database
    let create_job = CreateJob {
        request_id,
        proof_bytes,
        public_inputs: public_inputs_bytes,
        outputs_json: serde_json::to_value(&payload.outputs).map_err(|e| {
            Error::InternalServerError(format!("Failed to serialize outputs: {}", e))
        })?,
        fee_bps: payload.policy.fee_bps as i16,
        root_hash,
        nullifier: nullifier.clone(),
        amount: payload.public_inputs.amount as i64,
        outputs_hash,
    };

    let job = state.job_repo.create_job(create_job).await?;

    // Create nullifier record
    state
        .nullifier_repo
        .create_nullifier(nullifier, job.id)
        .await?;

    // Add to processing queue
    let job_message = JobMessage::new(job.id, request_id);
    state.queue.enqueue(job_message).await?;

    info!("Withdraw request queued successfully: {}", request_id);

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
        return Err(Error::ValidationError(
            "Too many outputs (max 10)".to_string(),
        ));
    }

    // Validate amounts
    for output in &request.outputs {
        if output.amount == 0 {
            return Err(Error::ValidationError(
                "Output amount cannot be zero".to_string(),
            ));
        }

        if output.recipient.len() < 32 {
            return Err(Error::ValidationError(
                "Invalid recipient address".to_string(),
            ));
        }
    }

    // Validate public inputs
    if request.public_inputs.amount == 0 {
        return Err(Error::ValidationError("Amount cannot be zero".to_string()));
    }

    if request.public_inputs.fee_bps > 10000 {
        return Err(Error::ValidationError(
            "Fee BPS cannot exceed 10000".to_string(),
        ));
    }

    // Check conservation
    let fee_amount = (request.public_inputs.amount * request.public_inputs.fee_bps as u64) / 10000;
    let total_output_amount: u64 = request.outputs.iter().map(|o| o.amount).sum();

    if total_output_amount + fee_amount != request.public_inputs.amount {
        return Err(Error::ValidationError(
            "Conservation check failed: outputs + fee != amount".to_string(),
        ));
    }

    // Check policy consistency
    if request.policy.fee_bps != request.public_inputs.fee_bps {
        return Err(Error::ValidationError(
            "Fee BPS mismatch between policy and public inputs".to_string(),
        ));
    }

    // Validate hex strings
    if request.public_inputs.root.len() != 64 {
        return Err(Error::ValidationError(
            "Root must be 64 hex characters".to_string(),
        ));
    }

    if request.public_inputs.nf.len() != 64 {
        return Err(Error::ValidationError(
            "Nullifier must be 64 hex characters".to_string(),
        ));
    }

    if request.public_inputs.outputs_hash.len() != 64 {
        return Err(Error::ValidationError(
            "Outputs hash must be 64 hex characters".to_string(),
        ));
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
        return Err(Error::ValidationError(
            "Proof bytes cannot be empty".to_string(),
        ));
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
                amount: 990000,
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
