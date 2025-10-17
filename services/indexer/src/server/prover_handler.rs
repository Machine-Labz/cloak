use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
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
    pub error: Option<String>,
}

/// POST /api/v1/prove
///
/// Generates an SP1 ZK proof for withdraw transaction
///
/// This endpoint accepts private inputs, public inputs, and outputs,
/// then triggers the SP1 prover (cloak-zk) to generate a Groth16 proof.
///
/// ⚠️ PRIVACY WARNING: This endpoint receives private inputs on the backend.
/// For production use, implementing client-side proof generation is optimal.
pub async fn generate_proof(
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(request): Json<ProveRequest>,
) -> impl IntoResponse {
    tracing::info!("Received proof generation request");

    let start_time = Instant::now();

    // Get client IP for rate limiting (use "unknown" if not available)
    let client_id = client_addr.ip().to_string();

    // Check rate limit
    if !state.rate_limiter.is_allowed(&client_id).await {
        tracing::warn!("Rate limit exceeded for client: {}", client_id);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ProveResponse {
                success: false,
                proof: None,
                public_inputs: None,
                generation_time_ms: 0,
                total_cycles: None,
                total_syscalls: None,
                execution_report: None,
                error: Some("Rate limit exceeded. Please try again later.".to_string()),
            }),
        );
    }

    // Validate inputs are valid JSON
    if let Err(e) = serde_json::from_str::<serde_json::Value>(&request.private_inputs) {
        tracing::error!("Invalid private_inputs JSON: {}", e);
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
                error: Some(format!("Invalid private_inputs JSON: {}", e)),
            }),
        );
    }

    if let Err(e) = serde_json::from_str::<serde_json::Value>(&request.public_inputs) {
        tracing::error!("Invalid public_inputs JSON: {}", e);
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
                error: Some(format!("Invalid public_inputs JSON: {}", e)),
            }),
        );
    }

    if let Err(e) = serde_json::from_str::<serde_json::Value>(&request.outputs) {
        tracing::error!("Invalid outputs JSON: {}", e);
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
                error: Some(format!("Invalid outputs JSON: {}", e)),
            }),
        );
    }

    tracing::info!("Starting SP1 proof generation (this may take 30-180 seconds)...");

    // Generate proof using the library function
    match sp1_generate_proof(
        &request.private_inputs,
        &request.public_inputs,
        &request.outputs,
    ) {
        Ok(proof_result) => {
            tracing::info!("SP1 proof generation succeeded");
            tracing::info!("Proof size: {} bytes", proof_result.proof_bytes.len());
            tracing::info!(
                "Public inputs size: {} bytes",
                proof_result.public_inputs.len()
            );
            tracing::info!("Generation time: {}ms", proof_result.generation_time_ms);
            tracing::info!("Total cycles consumed: {}", proof_result.total_cycles);
            tracing::info!("Total syscalls: {}", proof_result.total_syscalls);

            // Convert to hex for API response
            let proof_hex = hex::encode(&proof_result.proof_bytes);
            let public_inputs_hex = hex::encode(&proof_result.public_inputs);

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
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("SP1 proof generation failed: {}", e);
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
                    error: Some(format!("SP1 proof generation failed: {}", e)),
                }),
            )
        }
    }
}
