use axum::{extract::State, response::IntoResponse, Json};
use base64::Engine;
use cloak_proof_extract::extract_groth16_260_sp1;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    api::{ApiResponse, WithdrawResponse},
    db::{models::CreateJob, repository::JobRepository},
    error::Error,
    planner::calculate_fee,
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
    let proof_bundle = base64::engine::general_purpose::STANDARD
        .decode(&payload.proof_bytes)
        .map_err(|e| Error::ValidationError(format!("Invalid proof base64: {}", e)))?;

    let proof_bytes = match extract_groth16_260_sp1(&proof_bundle) {
        Ok(bytes) => bytes,
        Err(_) if proof_bundle.len() == 260 => {
            let mut arr = [0u8; 260];
            arr.copy_from_slice(&proof_bundle);
            arr
        }
        Err(_) => {
            return Err(Error::ValidationError(
                "Invalid SP1 proof bundle".to_string(),
            ))
        }
    };

    // Parse public inputs
    // Strip "0x" prefix if present
    let root_str = payload
        .public_inputs
        .root
        .strip_prefix("0x")
        .unwrap_or(&payload.public_inputs.root);
    let nf_str = payload
        .public_inputs
        .nf
        .strip_prefix("0x")
        .unwrap_or(&payload.public_inputs.nf);
    let outputs_hash_str = payload
        .public_inputs
        .outputs_hash
        .strip_prefix("0x")
        .unwrap_or(&payload.public_inputs.outputs_hash);

    let root_hash = hex::decode(root_str)
        .map_err(|e| Error::ValidationError(format!("Invalid root hex: {}", e)))?;
    let nullifier = hex::decode(nf_str)
        .map_err(|e| Error::ValidationError(format!("Invalid nullifier hex: {}", e)))?;
    let outputs_hash = hex::decode(outputs_hash_str)
        .map_err(|e| Error::ValidationError(format!("Invalid outputs hash hex: {}", e)))?;

    // Validate lengths
    if root_hash.len() != 32 {
        return Err(Error::ValidationError(format!(
            "Root must be 32 bytes, got {}",
            root_hash.len()
        )));
    }
    if nullifier.len() != 32 {
        return Err(Error::ValidationError(format!(
            "Nullifier must be 32 bytes, got {}",
            nullifier.len()
        )));
    }
    if outputs_hash.len() != 32 {
        return Err(Error::ValidationError(format!(
            "Outputs hash must be 32 bytes, got {}",
            outputs_hash.len()
        )));
    }

    // Note: We no longer check for nullifier double-spend at queueing time.
    // The on-chain program will enforce uniqueness when the transaction executes.
    // The worker stores the nullifier AFTER successful on-chain execution
    // (see services/relay/src/worker/processor.rs:113-120)

    // Encode public inputs for storage (canonical 104-byte format)
    // Format: root(32) || nf(32) || outputs_hash(32) || amount(8) = 104 bytes
    let mut public_inputs_bytes = Vec::new();
    public_inputs_bytes.extend_from_slice(&root_hash);
    public_inputs_bytes.extend_from_slice(&nullifier);
    public_inputs_bytes.extend_from_slice(&outputs_hash);
    public_inputs_bytes.extend_from_slice(&payload.public_inputs.amount.to_le_bytes());

    // Create job in database
    let effective_fee_bps = if payload.public_inputs.amount == 0 {
        0
    } else {
        let expected_fee = calculate_fee(payload.public_inputs.amount);
        (expected_fee.saturating_mul(10_000)).div_ceil(payload.public_inputs.amount)
    };

    let create_job = CreateJob {
        request_id,
        proof_bytes: proof_bytes.to_vec(),
        public_inputs: public_inputs_bytes,
        outputs_json: serde_json::to_value(&payload.outputs).map_err(|e| {
            Error::InternalServerError(format!("Failed to serialize outputs: {}", e))
        })?,
        fee_bps: effective_fee_bps as i16,
        root_hash,
        nullifier: nullifier.clone(),
        amount: payload.public_inputs.amount as i64,
        outputs_hash,
    };

    state.job_repo.create_job(create_job).await?;

    // Note: Nullifier is NOT created here!
    // It will be created by the worker AFTER the transaction succeeds on-chain
    // (see services/relay/src/worker/processor.rs:113-120)

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

    let expected_fee = calculate_fee(request.public_inputs.amount);
    if expected_fee == 0 {
        return Err(Error::ValidationError(
            "Fee calculation resulted in zero; amount may be too small".to_string(),
        ));
    }

    let total_output_amount: u64 = request.outputs.iter().map(|o| o.amount).sum();
    if total_output_amount + expected_fee != request.public_inputs.amount {
        return Err(Error::ValidationError(
            "Conservation check failed: outputs + fee != amount".to_string(),
        ));
    }

    let expected_fee_bps = if request.public_inputs.amount == 0 {
        0
    } else {
        (expected_fee.saturating_mul(10_000)).div_ceil(request.public_inputs.amount)
    };

    if expected_fee_bps > 10_000 {
        return Err(Error::ValidationError(
            "Effective fee exceeds protocol cap of 10000 bps".to_string(),
        ));
    }

    if request.public_inputs.fee_bps != expected_fee_bps as u16 {
        return Err(Error::ValidationError(format!(
            "Fee BPS mismatch: expected {} bps, got {} bps",
            expected_fee_bps, request.public_inputs.fee_bps
        )));
    }

    if request.policy.fee_bps != expected_fee_bps as u16 {
        return Err(Error::ValidationError(format!(
            "Fee BPS mismatch between policy and public inputs (expected {} bps, got {} bps)",
            expected_fee_bps, request.policy.fee_bps
        )));
    }

    // Validate hex strings (strip 0x prefix if present for validation)
    let root_str = request
        .public_inputs
        .root
        .strip_prefix("0x")
        .unwrap_or(&request.public_inputs.root);
    let nf_str = request
        .public_inputs
        .nf
        .strip_prefix("0x")
        .unwrap_or(&request.public_inputs.nf);
    let outputs_hash_str = request
        .public_inputs
        .outputs_hash
        .strip_prefix("0x")
        .unwrap_or(&request.public_inputs.outputs_hash);

    if root_str.len() != 64 {
        return Err(Error::ValidationError(
            "Root must be 64 hex characters (or 66 with 0x prefix)".to_string(),
        ));
    }

    if nf_str.len() != 64 {
        return Err(Error::ValidationError(
            "Nullifier must be 64 hex characters (or 66 with 0x prefix)".to_string(),
        ));
    }

    if outputs_hash_str.len() != 64 {
        return Err(Error::ValidationError(
            "Outputs hash must be 64 hex characters (or 66 with 0x prefix)".to_string(),
        ));
    }

    // Validate hex format
    hex::decode(root_str)
        .map_err(|_| Error::ValidationError("Root must be valid hex".to_string()))?;

    hex::decode(nf_str)
        .map_err(|_| Error::ValidationError("Nullifier must be valid hex".to_string()))?;

    hex::decode(outputs_hash_str)
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
    use serde_json::json;

    use super::*;

    #[test]
    fn test_validate_request() {
        let valid_request = WithdrawRequest {
            outputs: vec![Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 97_000_000,
            }],
            policy: Policy { fee_bps: 300 },
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 100_000_000,
                fee_bps: 300,
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
            policy: Policy { fee_bps: 300 },
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 100_000_000,
                fee_bps: 300,
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
                amount: 97_000_000,
            }],
            policy: Policy { fee_bps: 10001 }, // Too high
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 100_000_000,
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
                amount: 97_000_000,
            }],
            policy: Policy { fee_bps: 300 },
            public_inputs: PublicInputs {
                root: "G".repeat(64), // Invalid hex
                nf: "1".repeat(64),
                amount: 100_000_000,
                fee_bps: 300,
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
                amount: 97_000_000,
            }],
            policy: Policy { fee_bps: 300 },
            public_inputs: PublicInputs {
                root: "0".repeat(64),
                nf: "1".repeat(64),
                amount: 100_000_000,
                fee_bps: 300,
                outputs_hash: "2".repeat(64),
            },
            proof_bytes: "".to_string(), // Empty base64
        };

        assert!(validate_request(&invalid_request).is_err());
    }
}
