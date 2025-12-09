use axum::{extract::State, response::IntoResponse, Json};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    api::{ApiResponse, WithdrawResponse},
    db::models::CreateJob,
    db::repository::JobRepository,
    error::Error,
    planner::calculate_fee,
    stake::StakeConfig,
    swap::SwapConfig,
    AppState,
};
use cloak_proof_extract::extract_groth16_260_sp1;

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub outputs: Vec<Output>,
    pub policy: Policy,
    pub public_inputs: PublicInputs,
    pub proof_bytes: String, // base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<SwapConfig>, // optional swap configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stake: Option<StakeConfig>, // optional staking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partially_signed_tx: Option<String>, // optional base64 encoded partially signed transaction (for stake delegate)
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

    // Determine decimals based on MINT_ADDRESS environment variable
    // If set and non-empty, assume SPL token (6 decimals for USDC)
    // Otherwise assume native SOL (9 decimals)
    let decimals = match std::env::var("MINT_ADDRESS") {
        Ok(mint_str) if !mint_str.is_empty() => 6, // SPL tokens
        _ => 9,                                    // Native SOL
    };

    // Validate the request
    validate_request(&payload, decimals)?;

    // Validate swap config if present
    if let Some(ref swap_config) = payload.swap {
        swap_config.validate().map_err(Error::ValidationError)?;
        info!(
            "Swap requested: {} → {}, slippage {} bps",
            std::env::var("MINT_ADDRESS").unwrap_or_else(|_| "SOL".to_string()),
            swap_config.output_mint,
            swap_config.slippage_bps
        );
    }

    // Validate stake config if present
    if let Some(ref stake_config) = payload.stake {
        stake_config.validate().map_err(Error::ValidationError)?;
        info!(
            "Staking requested: stake_account={}, validator={}",
            stake_config.stake_account,
            stake_config.validator_vote_account
        );
    }

    // Ensure only one of swap or stake is specified
    if payload.swap.is_some() && payload.stake.is_some() {
        return Err(Error::ValidationError(
            "Cannot specify both swap and stake configurations".to_string(),
        ));
    }

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
        let expected_fee = calculate_fee(payload.public_inputs.amount, decimals);
        ((expected_fee.saturating_mul(10_000)) + payload.public_inputs.amount - 1)
            / payload.public_inputs.amount
    };

    // Create metadata object with outputs and optional swap/stake config
    let mut metadata = serde_json::json!({
        "outputs": payload.outputs
    });

    if let Some(swap_config) = &payload.swap {
        metadata["swap"] = serde_json::to_value(swap_config).map_err(|e| {
            Error::InternalServerError(format!("Failed to serialize swap config: {}", e))
        })?;
    }

    if let Some(stake_config) = &payload.stake {
        metadata["stake"] = serde_json::to_value(stake_config).map_err(|e| {
            Error::InternalServerError(format!("Failed to serialize stake config: {}", e))
        })?;
        
        // Store partially signed transaction if provided
        if let Some(partially_signed_tx) = &payload.partially_signed_tx {
            metadata["partially_signed_tx"] = serde_json::Value::String(partially_signed_tx.clone());
        }
    }

    let create_job = CreateJob {
        request_id,
        proof_bytes: proof_bytes.to_vec(),
        public_inputs: public_inputs_bytes,
        outputs_json: metadata,
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

fn validate_request(request: &WithdrawRequest, decimals: u8) -> Result<(), Error> {
    // For swap and stake modes, outputs can be empty
    // For regular mode, outputs are required
    let is_special_mode = request.swap.is_some() || request.stake.is_some();

    if !is_special_mode {
        // Regular mode: validate outputs
        if request.outputs.is_empty() {
            return Err(Error::ValidationError("No outputs specified".to_string()));
        }
    }

    if request.outputs.len() > 10 {
        return Err(Error::ValidationError(
            "Too many outputs (max 10)".to_string(),
        ));
    }

    // Validate amounts for non-empty outputs
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

    // Calculate expected fee based on mode:
    // - For swap requests, use variable-only fee (matches SPL swap economics)
    // - For stake requests, use full fee (fixed + variable) - same as regular withdrawals
    // - For regular SOL withdrawals (no swap/stake), use full fee (fixed + variable)
    //   to stay consistent with the SP1 circuit and validator_agent API.
    let expected_fee = if request.swap.is_some() {
        // Swap mode: same semantics as planner::calculate_fee (variable 0.5%)
        calculate_fee(request.public_inputs.amount, decimals)
    } else {
        // Stake mode and regular withdrawals: fixed 0.0025 SOL + variable 0.5%
        // This matches zk-guest-sp1/guest/src/encoding.rs::calculate_fee
        // and services/relay/src/api/validator_agent.rs::calculate_fee.
        let fixed_fee: u64 = 2_500_000; // 0.0025 SOL in lamports
        let variable_fee = calculate_fee(request.public_inputs.amount, decimals);
        fixed_fee.saturating_add(variable_fee)
    };
    if expected_fee == 0 {
        return Err(Error::ValidationError(
            "Fee calculation resulted in zero; amount may be too small".to_string(),
        ));
    }

    // For swap mode, skip conservation check since outputs will be in swapped tokens, not SOL
    // For stake mode, outputs should be empty (all goes to stake)
    // For regular mode, verify outputs + fee = amount
    if request.swap.is_some() {
        // Skip conservation check for swap mode
    } else if request.stake.is_some() {
        // Stake mode: outputs should be empty
        if !request.outputs.is_empty() {
            return Err(Error::ValidationError(
                "Stake mode requires zero outputs".to_string(),
            ));
        }
        // Verify stake_amount + fee = amount (stake_amount is implicit)
        if expected_fee >= request.public_inputs.amount {
            return Err(Error::ValidationError(
                "Fee exceeds total amount in stake mode".to_string(),
            ));
        }
    } else {
        // Regular mode: verify outputs + fee = amount
        let total_output_amount: u64 = request.outputs.iter().map(|o| o.amount).sum();
        if total_output_amount + expected_fee != request.public_inputs.amount {
            return Err(Error::ValidationError(
                "Conservation check failed: outputs + fee != amount".to_string(),
            ));
        }
    }

    let expected_fee_bps = if request.public_inputs.amount == 0 {
        0
    } else {
        ((expected_fee.saturating_mul(10_000)) + request.public_inputs.amount - 1)
            / request.public_inputs.amount
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
    use super::*;
    use serde_json::json;

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
            proof_bytes: base64::engine::general_purpose::STANDARD.encode(vec![0u8; 256]),
            swap: None,
            stake: None,
            partially_signed_tx: None,
        };

        assert!(validate_request(&valid_request, 9).is_ok());
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
            proof_bytes: base64::engine::general_purpose::STANDARD.encode(vec![0u8; 256]),
            swap: None,
            stake: None,
            partially_signed_tx: None,
        };

        assert!(validate_request(&invalid_request, 9).is_err());
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
            proof_bytes: base64::engine::general_purpose::STANDARD.encode(vec![0u8; 256]),
            swap: None,
            stake: None,
            partially_signed_tx: None,
        };

        assert!(validate_request(&invalid_request, 9).is_err());
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
            proof_bytes: base64::engine::general_purpose::STANDARD.encode(vec![0u8; 256]),
            swap: None,
            stake: None,
            partially_signed_tx: None,
        };

        assert!(validate_request(&invalid_request, 9).is_err());
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
            swap: None,
            stake: None,
            partially_signed_tx: None,
        };

        assert!(validate_request(&invalid_request, 9).is_err());
    }
}
