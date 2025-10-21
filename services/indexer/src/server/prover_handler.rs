use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use cloak_proof_extract::extract_groth16_260_sp1;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};
use zk_guest_sp1_host::generate_proof as sp1_generate_proof;

use crate::server::final_handlers::AppState;

#[derive(Debug, Deserialize)]
pub struct ProveRequest {
    pub private_inputs: String, // JSON string
    pub public_inputs: String,  // JSON string
    pub outputs: String,        // JSON string
}

#[derive(Debug, Serialize)]
pub struct ProveResponse {
    pub success: bool,
    pub proof: Option<String>,         // Hex-encoded proof bytes
    pub public_inputs: Option<String>, // Hex-encoded public inputs
    pub generation_time_ms: u64,
    pub total_cycles: Option<u64>,        // Total SP1 cycles consumed
    pub total_syscalls: Option<u64>,      // Total syscalls made
    pub execution_report: Option<String>, // Full SP1 execution report
    pub proof_method: Option<String>,     // "tee" or "local" to indicate which method was used
    pub wallet_address: Option<String>,   // Wallet address used for TEE proving
    pub error: Option<String>,
}

/// POST /api/v1/prove
///
/// Generates an SP1 ZK proof for withdraw transaction
///
/// This endpoint accepts private inputs, public inputs, and outputs,
/// then triggers the SP1 prover to generate a proof. It will attempt to use
/// SP1's TEE Private Proving if configured, otherwise falls back to local proving.
///
/// ⚠️ PRIVACY WARNING: This endpoint receives private inputs on the backend.
/// For production use, implementing client-side proof generation is optimal.
pub async fn generate_proof(
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(request): Json<ProveRequest>,
) -> impl IntoResponse {
    tracing::info!("🔐 Received proof generation request");
    tracing::info!(
        client_ip = client_addr.ip().to_string(),
        private_inputs_len = request.private_inputs.len(),
        public_inputs_len = request.public_inputs.len(),
        outputs_len = request.outputs.len(),
        "Processing proof generation request"
    );

    let start_time = Instant::now();

    // Get client IP for rate limiting (use "unknown" if not available)
    let _client_id = client_addr.ip().to_string();

    // Validate inputs are valid JSON
    tracing::info!("🔍 Validating input JSON");
    if let Err(e) = serde_json::from_str::<serde_json::Value>(&request.private_inputs) {
        tracing::error!("❌ Invalid private_inputs JSON: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            Json(ProveResponse {
                success: false,
                proof: None,
                public_inputs: None,
                generation_time_ms: 0,
                total_cycles: None,
                total_syscalls: None,
                execution_report: None,
                proof_method: None,
                wallet_address: None,
                error: Some(format!("Invalid private_inputs JSON: {}", e)),
            }),
        );
    }

    if let Err(e) = serde_json::from_str::<serde_json::Value>(&request.public_inputs) {
        tracing::error!("❌ Invalid public_inputs JSON: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            Json(ProveResponse {
                success: false,
                proof: None,
                public_inputs: None,
                generation_time_ms: 0,
                total_cycles: None,
                total_syscalls: None,
                execution_report: None,
                proof_method: None,
                wallet_address: None,
                error: Some(format!("Invalid public_inputs JSON: {}", e)),
            }),
        );
    }

    if let Err(e) = serde_json::from_str::<serde_json::Value>(&request.outputs) {
        tracing::error!("❌ Invalid outputs JSON: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            Json(ProveResponse {
                success: false,
                proof: None,
                public_inputs: None,
                generation_time_ms: 0,
                total_cycles: None,
                total_syscalls: None,
                execution_report: None,
                proof_method: None,
                wallet_address: None,
                error: Some(format!("Invalid outputs JSON: {}", e)),
            }),
        );
    }
    tracing::info!("✅ Input validation passed");

    tracing::info!("🚀 Starting SP1 proof generation (this may take 30-180 seconds)...");

    // Try TEE first if available, then fallback to local proving
    let _proof_result: Option<()> = if let Some(tee_client) = &state.tee_client {
        tracing::info!("🔐 TEE client available - attempting TEE proof generation");
        tracing::info!("   Wallet: {}", tee_client.wallet_address());
        tracing::info!("   RPC URL: {}", state.config.sp1_tee.rpc_url);
        tracing::info!(
            "   Timeout: {} seconds",
            state.config.sp1_tee.timeout_seconds
        );

        match tee_client
            .generate_proof(
                &request.private_inputs,
                &request.public_inputs,
                &request.outputs,
            )
            .await
        {
            Ok(tee_result) => {
                tracing::info!("✅ TEE proof generation succeeded");
                tracing::info!(
                    proof_size_bytes = tee_result.proof_bytes.len(),
                    public_inputs_size_bytes = tee_result.public_inputs.len(),
                    generation_time_ms = tee_result.generation_time_ms,
                    total_cycles = tee_result.total_cycles,
                    total_syscalls = tee_result.total_syscalls,
                    wallet_address = tee_client.wallet_address(),
                    "TEE proof generation completed successfully"
                );

                let canonical_proof = match extract_groth16_260_sp1(&tee_result.proof_bytes) {
                    Ok(proof) => proof,
                    Err(err) => {
                        tracing::error!(
                            error = ?err,
                            "Failed to extract canonical Groth16 proof from SP1 bundle"
                        );
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ProveResponse {
                                success: false,
                                proof: None,
                                public_inputs: None,
                                generation_time_ms: tee_result.generation_time_ms,
                                total_cycles: Some(tee_result.total_cycles),
                                total_syscalls: Some(tee_result.total_syscalls),
                                execution_report: Some(tee_result.execution_report),
                                proof_method: Some("tee".to_string()),
                                wallet_address: Some(tee_client.wallet_address().to_string()),
                                error: Some(
                                    "Failed to extract canonical Groth16 proof from SP1 bundle"
                                        .to_string(),
                                ),
                            }),
                        );
                    }
                };

                // Convert to hex for API response
                let proof_hex = hex::encode(canonical_proof);
                let proof_prefix = hex::encode(&canonical_proof[..4]);
                let public_inputs_hex = hex::encode(&tee_result.public_inputs);
                tracing::info!(
                    canonical_proof_bytes = proof_hex.len() / 2,
                    proof_prefix
                );

                tracing::info!("🎉 TEE proof generation request completed successfully");
                return (
                    StatusCode::OK,
                    Json(ProveResponse {
                        success: true,
                        proof: Some(proof_hex),
                        public_inputs: Some(public_inputs_hex),
                        generation_time_ms: tee_result.generation_time_ms,
                        total_cycles: Some(tee_result.total_cycles),
                        total_syscalls: Some(tee_result.total_syscalls),
                        execution_report: Some(tee_result.execution_report),
                        proof_method: Some("tee".to_string()),
                        wallet_address: Some(tee_client.wallet_address().to_string()),
                        error: None,
                    }),
                );
            }
            Err(e) => {
                tracing::warn!("⚠️ TEE proof generation failed: {}", e);
                tracing::info!("🔄 Falling back to local proof generation");
                None
            }
        }
    } else {
        tracing::info!("🏠 TEE client not available, using local proof generation");
        None
    };

    // Fallback to local proof generation
    tracing::info!("🏠 Using local SP1 proof generation");
    match sp1_generate_proof(
        &request.private_inputs,
        &request.public_inputs,
        &request.outputs,
    ) {
        Ok(proof_result) => {
            tracing::info!("✅ SP1 proof generation succeeded");
            tracing::info!(
                proof_size_bytes = proof_result.proof_bytes.len(),
                public_inputs_size_bytes = proof_result.public_inputs.len(),
                generation_time_ms = proof_result.generation_time_ms,
                total_cycles = proof_result.total_cycles,
                total_syscalls = proof_result.total_syscalls,
                "Proof generation completed successfully"
            );

            let canonical_proof = match extract_groth16_260_sp1(&proof_result.proof_bytes) {
                Ok(proof) => proof,
                Err(err) => {
                    tracing::error!(
                        error = ?err,
                        "Failed to extract canonical Groth16 proof from SP1 bundle"
                    );
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ProveResponse {
                            success: false,
                            proof: None,
                            public_inputs: None,
                            generation_time_ms: proof_result.generation_time_ms,
                            total_cycles: Some(proof_result.total_cycles),
                            total_syscalls: Some(proof_result.total_syscalls),
                            execution_report: Some(proof_result.execution_report),
                            proof_method: Some("local".to_string()),
                            wallet_address: None,
                            error: Some(
                                "Failed to extract canonical Groth16 proof from SP1 bundle"
                                    .to_string(),
                            ),
                        }),
                    );
                }
            };

            // Convert to hex for API response
            let proof_hex = hex::encode(canonical_proof);
            let proof_prefix = hex::encode(&canonical_proof[..4]);
            let public_inputs_hex = hex::encode(&proof_result.public_inputs);
            tracing::info!(
                canonical_proof_bytes = proof_hex.len() / 2,
                proof_prefix
            );

            tracing::info!("🎉 Local proof generation request completed successfully");
            (
                StatusCode::OK,
                Json(ProveResponse {
                    success: true,
                    proof: Some(proof_hex),
                    public_inputs: Some(public_inputs_hex),
                    generation_time_ms: proof_result.generation_time_ms,
                    total_cycles: Some(proof_result.total_cycles),
                    total_syscalls: Some(proof_result.total_syscalls),
                    execution_report: Some(proof_result.execution_report),
                    proof_method: Some("local".to_string()),
                    wallet_address: None,
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("❌ SP1 proof generation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProveResponse {
                    success: false,
                    proof: None,
                    public_inputs: None,
                    generation_time_ms: start_time.elapsed().as_millis() as u64,
                    total_cycles: None,
                    total_syscalls: None,
                    execution_report: None,
                    proof_method: Some("local".to_string()),
                    wallet_address: None,
                    error: Some(format!("SP1 proof generation failed: {}", e)),
                }),
            )
        }
    }
}
