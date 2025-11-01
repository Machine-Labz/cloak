use axum::{extract::State, response::IntoResponse, Json};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::planner::{build_public_inputs_104, calculate_fee};
use crate::{
    db::models::CreateJob,
    db::repository::{JobRepository, NullifierRepository},
    error::Error,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct OrchestrateRequest {
    pub amount: u64,
    pub nf_hex: String,
    pub recipient: String, // base58
    pub root_hex: String,
}

#[derive(Debug, Serialize)]
pub struct OrchestrateResponse {
    pub job_id: Uuid,
    pub status: String,
    pub root_used: String,
    pub recipient_amount: u64,
}

pub async fn orchestrate_withdraw(
    State(state): State<AppState>,
    Json(req): Json<OrchestrateRequest>,
) -> Result<impl IntoResponse, Error> {
    // Root is provided explicitly
    let root_hex = req.root_hex.clone();
    if root_hex.len() != 64 {
        return Err(Error::ValidationError("root must be 64 hex chars".into()));
    }
    let root_bytes = hex::decode(&root_hex).map_err(|e| Error::ValidationError(e.to_string()))?;
    let mut root_arr = [0u8; 32];
    root_arr.copy_from_slice(&root_bytes);

    // Parse nullifier
    if req.nf_hex.len() != 64 {
        return Err(Error::ValidationError("nf_hex must be 64 hex chars".into()));
    }
    let nf_bytes = hex::decode(&req.nf_hex).map_err(|e| Error::ValidationError(e.to_string()))?;
    let mut nf_arr = [0u8; 32];
    nf_arr.copy_from_slice(&nf_bytes);

    // Compute recipient amount and outputs_hash
    let fee = calculate_fee(req.amount);
    let recipient_amount = req.amount.saturating_sub(fee);
    if recipient_amount == 0 {
        return Err(Error::ValidationError(
            "recipient amount would be zero".into(),
        ));
    }
    let recipient_bytes = bs58::decode(&req.recipient)
        .into_vec()
        .map_err(|e| Error::ValidationError(format!("invalid recipient base58: {}", e)))?;
    if recipient_bytes.len() != 32 {
        return Err(Error::ValidationError(
            "recipient must decode to 32 bytes".into(),
        ));
    }
    let mut outputs_hasher = Hasher::new();
    outputs_hasher.update(&recipient_bytes);
    outputs_hasher.update(&recipient_amount.to_le_bytes());
    let outputs_hash = *outputs_hasher.finalize().as_bytes();

    // Build canonical public inputs (104 bytes)
    let public_104 = build_public_inputs_104(&root_arr, &nf_arr, &outputs_hash, req.amount);

    // Build outputs_json
    let outputs_json = serde_json::json!([
        {"recipient": req.recipient, "amount": recipient_amount}
    ]);

    // Create job with empty proof for now; worker will requeue until proof available
    let request_id = Uuid::new_v4();
    let job = state
        .job_repo
        .create_job(CreateJob {
            request_id,
            proof_bytes: Vec::new(),
            public_inputs: public_104.to_vec(),
            outputs_json,
            fee_bps: 0,
            root_hash: root_arr.to_vec(),
            nullifier: nf_arr.to_vec(),
            amount: req.amount as i64,
            outputs_hash: outputs_hash.to_vec(),
        })
        .await?;

    // store nullifier
    let _ = state
        .nullifier_repo
        .create_nullifier(nf_arr.to_vec(), job.id)
        .await;

    info!("Orchestrated withdraw job {} queued", job.id);
    Ok(Json(OrchestrateResponse {
        job_id: job.id,
        status: "queued".to_string(),
        root_used: root_hex,
        recipient_amount,
    }))
}
