use axum::{extract::State, http::StatusCode, Json};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::{ApiResponse, WithdrawResponse};
use crate::db::models::CreateJob;
use crate::db::repository::JobRepository;
use crate::stake::UnstakeConfig;
use crate::AppState;

/// Unstake request payload
#[derive(Debug, Deserialize)]
pub struct UnstakeRequest {
    /// ZK proof (base64 encoded)
    pub proof: String,
    /// Public inputs for the proof
    pub public_inputs: UnstakePublicInputs,
    /// Unstake configuration
    pub unstake: UnstakeConfig,
    /// Optional partially signed transaction (base64 encoded)
    /// If provided, relay will add its signature and submit
    /// If not provided, relay will create and sign the transaction (requires stake_authority to be relay)
    #[serde(default)]
    #[allow(dead_code)] // TODO: Will be used when 2-phase signing is implemented
    pub partially_signed_tx: Option<String>,
}

/// Public inputs for unstake proof
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnstakePublicInputs {
    /// The new commitment (will be added to merkle tree)
    pub commitment: String,
    /// Hash of stake account (proves ownership)
    pub stake_account_hash: String,
    /// Outputs hash (binds commitment to stake account)
    pub outputs_hash: String,
    /// Amount being unstaked (in lamports)
    pub amount: u64,
}

/// Handle unstake request
/// 
/// This endpoint allows users to unstake from a deactivated stake account
/// directly into the shield pool with a new commitment.
/// 
/// The flow:
/// 1. User deactivates stake account (public transaction)
/// 2. After cooldown, user calls this endpoint with ZK proof
/// 3. Relay submits UnstakeToPool transaction
/// 4. Funds go to shield pool with new commitment
pub async fn handle_unstake(
    State(state): State<AppState>,
    Json(payload): Json<UnstakeRequest>,
) -> Result<Json<ApiResponse<WithdrawResponse>>, (StatusCode, Json<ApiResponse<String>>)> {
    info!("Received unstake request");

    // Validate unstake config
    if let Err(e) = payload.unstake.validate() {
        warn!("Invalid unstake config: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Validation error: {}", e))),
        ));
    }

    // Validate public inputs
    if let Err(e) = validate_unstake_public_inputs(&payload.public_inputs) {
        warn!("Invalid public inputs: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Validation error: {}", e))),
        ));
    }

    // Decode proof
    let proof_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.proof)
        .map_err(|e| {
            error!("Failed to decode proof: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(format!("Invalid proof encoding: {}", e))),
            )
        })?;

    // Validate proof size (260 bytes for Groth16)
    if proof_bytes.len() != 260 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!(
                "Invalid proof size: expected 260 bytes, got {}",
                proof_bytes.len()
            ))),
        ));
    }

    // Decode commitment
    let commitment = hex::decode(&payload.public_inputs.commitment).map_err(|e| {
        error!("Failed to decode commitment: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid commitment: {}", e))),
        )
    })?;

    if commitment.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Commitment must be 32 bytes".to_string())),
        ));
    }

    // Decode stake account hash
    let stake_account_hash = hex::decode(&payload.public_inputs.stake_account_hash).map_err(|e| {
        error!("Failed to decode stake_account_hash: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid stake_account_hash: {}", e))),
        )
    })?;

    // Decode outputs hash
    let outputs_hash = hex::decode(&payload.public_inputs.outputs_hash).map_err(|e| {
        error!("Failed to decode outputs_hash: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid outputs_hash: {}", e))),
        )
    })?;

    // Build public inputs blob (104 bytes)
    // Format: commitment(32) || stake_account_hash(32) || outputs_hash(32) || amount(8)
    let mut public_inputs_bytes = Vec::with_capacity(104);
    public_inputs_bytes.extend_from_slice(&commitment);
    public_inputs_bytes.extend_from_slice(&stake_account_hash);
    public_inputs_bytes.extend_from_slice(&outputs_hash);
    public_inputs_bytes.extend_from_slice(&payload.public_inputs.amount.to_le_bytes());

    // Generate request ID
    let request_id = Uuid::new_v4();

    // Create metadata for the job
    let mut metadata = serde_json::json!({
        "type": "unstake",
        "unstake": payload.unstake,
    });
    
    // If frontend provided a partially signed transaction, include it
    if let Some(ref partially_signed_tx) = payload.partially_signed_tx {
        metadata["partially_signed_tx"] = serde_json::Value::String(partially_signed_tx.clone());
        info!("✅ Partially signed transaction included in job metadata");
    } else {
        warn!("⚠️  No partially signed transaction provided - job may fail");
    }

    // Create job in database
    let create_job = CreateJob {
        request_id,
        proof_bytes: proof_bytes.to_vec(),
        public_inputs: public_inputs_bytes,
        outputs_json: metadata,
        fee_bps: 50, // 0.5% variable fee for unstake
        root_hash: commitment, // Vec<u8>
        nullifier: vec![0u8; 32], // No nullifier for unstake (it's a deposit)
        amount: payload.public_inputs.amount as i64,
        outputs_hash, // Vec<u8>
    };

    state.job_repo.create_job(create_job).await.map_err(|e| {
        error!("Failed to create job: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to queue request: {}", e))),
        )
    })?;

    info!("Unstake request queued successfully: {}", request_id);

    let response = WithdrawResponse {
        request_id,
        status: "queued".to_string(),
        message: "Unstake request received and queued for processing".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

fn validate_unstake_public_inputs(inputs: &UnstakePublicInputs) -> Result<(), String> {
    // Validate commitment (32 bytes hex = 64 chars)
    if inputs.commitment.len() != 64 {
        return Err("Commitment must be 64 hex characters".to_string());
    }
    hex::decode(&inputs.commitment).map_err(|_| "Invalid commitment hex")?;

    // Validate stake_account_hash (32 bytes hex = 64 chars)
    if inputs.stake_account_hash.len() != 64 {
        return Err("Stake account hash must be 64 hex characters".to_string());
    }
    hex::decode(&inputs.stake_account_hash).map_err(|_| "Invalid stake_account_hash hex")?;

    // Validate outputs_hash (32 bytes hex = 64 chars)
    if inputs.outputs_hash.len() != 64 {
        return Err("Outputs hash must be 64 hex characters".to_string());
    }
    hex::decode(&inputs.outputs_hash).map_err(|_| "Invalid outputs_hash hex")?;

    // Validate amount
    if inputs.amount == 0 {
        return Err("Amount cannot be zero".to_string());
    }

    Ok(())
}
