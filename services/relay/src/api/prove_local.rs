use axum::{extract::{Path, State}, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::{AppState, error::Error, db::repository::JobRepository, queue::{JobMessage, JobQueue}};

#[derive(Debug, Deserialize)]
pub struct ProveLocalRequest {
    pub private_inputs: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub struct ProveLocalResponse {
    pub job_id: Uuid,
    pub proof_size: usize,
    pub public_len: usize,
    pub status: String,
}

pub async fn prove_local(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(req): Json<ProveLocalRequest>,
) -> Result<impl IntoResponse, Error> {
    // Load job
    let job = state
        .job_repo
        .get_job_by_id(job_id)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    if job.public_inputs.len() != 104 {
        return Err(Error::ValidationError("job public inputs must be 104 bytes".into()));
    }
    // Build public JSON from columns
    let root_hex = hex::encode(&job.root_hash);
    let nf_hex = hex::encode(&job.nullifier);
    let out_hash_hex = hex::encode(&job.outputs_hash);
    let amount = job.amount as u64;

    let public_json = serde_json::json!({
        "root": root_hex,
        "nf": nf_hex,
        "outputs_hash": out_hash_hex,
        "amount": amount
    });
    let outputs_json = match job.outputs_json {
        v @ serde_json::Value::Array(_) => {
            // Transform to guest outputs: [{"address": recipient, "amount": amount}, ...]
            let arr = v.as_array().unwrap();
            let mapped: Vec<serde_json::Value> = arr
                .iter()
                .map(|o| {
                    let addr = o.get("recipient").and_then(|x| x.as_str()).unwrap_or("");
                    let amt = o.get("amount").and_then(|x| x.as_u64()).unwrap_or(0);
                    serde_json::json!({"address": addr, "amount": amt})
                })
                .collect();
            serde_json::Value::Array(mapped)
        }
        v @ serde_json::Value::Object(_) => v, // assume already in correct shape
        _ => serde_json::json!([]),
    };

    let private_str = serde_json::to_string(&req.private_inputs)
        .map_err(|e| Error::ValidationError(format!("invalid private_inputs: {}", e)))?;
    let public_str = serde_json::to_string(&public_json).unwrap();
    let outputs_str = serde_json::to_string(&outputs_json).unwrap();

    // Generate SP1 proof locally
    let proof = zk_guest_sp1_host::generate_proof(&private_str, &public_str, &outputs_str)
        .map_err(|e| Error::InternalServerError(format!("proof generation failed: {}", e)))?;

    // Persist proof and public inputs from the artifact (public inputs may be normalized by SP1)
    state
        .job_repo
        .update_job_proof(job_id, proof.proof_bytes, proof.public_inputs)
        .await?;

    // Enqueue for submission immediately
    state.queue.enqueue(JobMessage::new(job_id, job.request_id)).await?;

    info!("Local proof generated for job {}", job_id);
    Ok(Json(ProveLocalResponse {
        job_id,
        proof_size: job.public_inputs.len(),
        public_len: 104,
        status: "proof_ready".into(),
    }))
}

