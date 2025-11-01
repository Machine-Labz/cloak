use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use blake3::Hasher;

use crate::error::Error;
use crate::{
    db::repository::{JobRepository, NullifierRepository},
    AppState,
};
use base64::Engine;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

#[derive(Debug, Deserialize)]
pub struct WithdrawJobRequest {
    pub public_bin_hex: String, // 208 hex chars
    pub outputs: Vec<OutputReq>,
    pub deadline_iso: String,
    pub payer_hints: Option<PayerHints>,
    pub fee_caps: Option<FeeCaps>,
}

#[derive(Debug, Deserialize)]
pub struct OutputReq {
    pub address_hex32: String,
    pub amount_u64: String, // decimal string
}

#[derive(Debug, Deserialize)]
pub struct PayerHints {
    pub use_jito: Option<bool>,
    pub bundle_tip_lamports: Option<String>,
    pub cu_limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FeeCaps {
    pub max_priority_fee_lamports: Option<String>,
    pub max_total_fee_lamports: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub artifacts: Option<JobArtifacts>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JobArtifacts {
    pub proof_hex: Option<String>,
    pub public_bin_hex_104: Option<String>,
    pub tx_bytes_base64: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    pub tx_bytes_base64: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub signature: String,
    pub slot: Option<u64>,
}

#[inline]
fn parse_hex32(s: &str) -> Result<[u8; 32], Error> {
    if s.len() != 64 {
        return Err(Error::BadRequest(
            "address_hex32 must be 64 hex chars".into(),
        ));
    }
    let bytes = hex::decode(s).map_err(|e| Error::BadRequest(format!("invalid hex: {}", e)))?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[inline]
fn calculate_fee(amount: u64) -> u64 {
    let fixed = 2_500_000u64;
    let variable = (amount.saturating_mul(5)) / 1_000; // 0.5%
    fixed.saturating_add(variable)
}

pub async fn create_withdraw_job(
    State(state): State<AppState>,
    Json(req): Json<WithdrawJobRequest>,
) -> Result<impl IntoResponse, Error> {
    // Validate public inputs length and decode
    if req.public_bin_hex.len() != 208 {
        return Err(Error::ValidationError(
            "public_bin_hex must be 208 hex chars".into(),
        ));
    }
    let public = hex::decode(&req.public_bin_hex)
        .map_err(|e| Error::ValidationError(format!("invalid public_bin_hex: {}", e)))?;
    if public.len() != 104 {
        return Err(Error::ValidationError(
            "decoded public inputs must be 104 bytes".into(),
        ));
    }

    // Parse public inputs
    let root = &public[0..32];
    let nf = &public[32..64];
    let outputs_hash_pub = &public[64..96];
    let amount = u64::from_le_bytes(public[96..104].try_into().unwrap());

    // Validate outputs (MVP: single output)
    if req.outputs.is_empty() {
        return Err(Error::ValidationError("outputs must be non-empty".into()));
    }
    if req.outputs.len() != 1 {
        return Err(Error::ValidationError("MVP only supports 1 output".into()));
    }
    let out0 = &req.outputs[0];
    let addr32 = parse_hex32(&out0.address_hex32)?;
    let recipient_amount: u64 = out0
        .amount_u64
        .parse()
        .map_err(|_| Error::ValidationError("amount_u64 must be a decimal u64".into()))?;

    // Compute outputs_hash = BLAKE3(address:32 || amount:u64_le)
    let mut hasher = Hasher::new();
    hasher.update(&addr32);
    hasher.update(&recipient_amount.to_le_bytes());
    let out_hash = hasher.finalize();
    let out_hash_bytes = out_hash.as_bytes();

    if out_hash_bytes != outputs_hash_pub {
        return Err(Error::ValidationError(
            "outputs_hash mismatch with public inputs".into(),
        ));
    }

    // Conservation: sum(outputs) + fee == amount
    let fee = calculate_fee(amount);
    let outputs_sum = recipient_amount;
    if outputs_sum + fee != amount {
        return Err(Error::ValidationError(
            "conservation failed: outputs + fee != amount".into(),
        ));
    }

    // Build DB job record
    let request_id = Uuid::new_v4();
    let outputs_json = serde_json::json!([
        {
            // store base58 for compatibility with downstream builder
            "recipient": bs58::encode(addr32).into_string(),
            "amount": recipient_amount
        }
    ]);

    let job = state
        .job_repo
        .create_job(crate::db::models::CreateJob {
            request_id,
            proof_bytes: Vec::new(),       // proof to be generated
            public_inputs: public.clone(), // 104 bytes
            outputs_json,
            fee_bps: 0, // fixed fee mode
            root_hash: root.to_vec(),
            nullifier: nf.to_vec(),
            amount: amount as i64,
            outputs_hash: outputs_hash_pub.to_vec(),
        })
        .await?;

    // Record nullifier
    state
        .nullifier_repo
        .create_nullifier(nf.to_vec(), job.id)
        .await?;

    let resp = JobResponse {
        job_id: job.id,
        status: "queued".to_string(),
    };
    Ok((StatusCode::ACCEPTED, Json(resp)))
}

pub async fn get_job(
    State(state): State<AppState>,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let Some(job) = state.job_repo.get_job_by_id(job_id).await? else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "code": "not_found",
                "message": "Job not found"
            })),
        ));
    };

    // Build artifacts if available
    let mut artifacts = JobArtifacts {
        proof_hex: None,
        public_bin_hex_104: None,
        tx_bytes_base64: None,
    };
    if job.public_inputs.len() == 104 {
        artifacts.public_bin_hex_104 = Some(hex::encode(&job.public_inputs));
    }
    if !job.proof_bytes.is_empty() {
        artifacts.proof_hex = Some(hex::encode(&job.proof_bytes));
    }

    let resp = JobStatusResponse {
        job_id: job.id,
        status: job.status.to_string(),
        artifacts: Some(artifacts),
        error: job.error_message,
    };
    let value =
        serde_json::to_value(resp).map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok((StatusCode::OK, Json(value)))
}

pub async fn submit_tx(Json(req): Json<SubmitRequest>) -> Result<impl IntoResponse, Error> {
    // 1) Decode base64 → bytes
    let raw = base64::engine::general_purpose::STANDARD
        .decode(&req.tx_bytes_base64)
        .map_err(|e| Error::ValidationError(format!("invalid base64 tx: {}", e)))?;

    // 2) Deserialize VersionedTransaction
    let vtx: VersionedTransaction = bincode::deserialize(&raw)
        .map_err(|e| Error::ValidationError(format!("invalid transaction payload: {}", e)))?;

    // 3) Choose RPC URL from env
    let rpc_url = std::env::var("RELAY_SOLANA__RPC_URL")
        .or_else(|_| std::env::var("SOLANA_RPC_URL"))
        .unwrap_or_else(|_| "http://localhost:8899".to_string());

    // 4) Send via RPC with retry backoff (simple loop)
    let cfg = RpcSendTransactionConfig {
        skip_preflight: false,
        preflight_commitment: Some(CommitmentLevel::Processed),
        max_retries: Some(5),
        ..Default::default()
    };
    let sig: Signature = {
        let rpc = RpcClient::new(rpc_url.clone());
        let mut out_sig: Option<Signature> = None;
        for attempt in 0..=5 {
            match rpc.send_transaction_with_config(&vtx, cfg) {
                Ok(s) => {
                    out_sig = Some(s);
                    break;
                }
                Err(e) => {
                    if attempt == 5 {
                        return Err(Error::InternalServerError(format!(
                            "rpc send failed: {}",
                            e
                        )));
                    }
                    let backoff_ms = (200u64).saturating_mul(1u64 << attempt.min(4));
                    let jitter = fastrand::u64(0..=backoff_ms.min(2000));
                    std::thread::sleep(std::time::Duration::from_millis(jitter));
                }
            }
        }
        out_sig.expect("signature set or returned error")
    };

    // 5) Confirm (best-effort)
    let rpc = RpcClient::new(rpc_url.clone());
    // best-effort confirm: poll statuses briefly
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(10) {
        if let Ok(sts) = rpc.get_signature_statuses(&[sig]) {
            if let Some(Some(st)) = sts.value.get(0) {
                if st.err.is_none() {
                    break;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // 6) Fetch slot (best-effort)
    let mut slot = None;
    if let Ok(statuses) = rpc.get_signature_statuses(&[sig]) {
        if let Some(Some(st)) = statuses.value.get(0) {
            slot = Some(st.slot);
        }
    }

    let resp = SubmitResponse {
        signature: sig.to_string(),
        slot,
    };
    Ok((StatusCode::OK, Json(serde_json::to_value(resp).unwrap())))
}
