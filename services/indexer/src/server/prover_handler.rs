use crate::server::final_handlers::AppState;
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use blake3::Hasher;
use cloak_proof_extract::extract_groth16_260_sp1;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};

/// Helper function to create deprecation headers
fn create_deprecation_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Deprecation", HeaderValue::from_static("true"));
    headers.insert(
        "Link",
        HeaderValue::from_static("<https://docs.cloaklabz.xyz/zk>; rel=\"deprecation\""),
    );
    headers
}

#[derive(Debug, Deserialize)]
pub struct ProveRequest {
    pub private_inputs: String, // JSON string
    pub public_inputs: String,  // JSON string
    pub outputs: String,        // JSON string
    #[serde(default)]
    pub swap_params: Option<serde_json::Value>, // Optional swap params JSON object
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
/// **⚠️ DEPRECATED ENDPOINT** - This endpoint is deprecated and will be removed in a future version.
///
/// Generates an SP1 ZK proof for withdraw transaction using TEE (Trusted Execution Environment)
///
/// This endpoint accepts private inputs, public inputs, and outputs,
/// then triggers the SP1 TEE prover to generate a proof. This endpoint requires
/// TEE to be configured - no local fallback is available.
///
/// ⚠️ PRIVACY WARNING: This endpoint receives private inputs on the backend.
/// For production use, implementing client-side proof generation is optimal.
///
/// **Migration Path**: Generate SP1 proofs in the client or wallet. Upload the SP1Stdin
/// to the TEE proving service and submit the resulting proof to the relay.
pub async fn generate_proof(
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(request): Json<ProveRequest>,
) -> Response {
    // Log deprecation warning
    tracing::warn!("⚠️ DEPRECATED: /api/v1/prove endpoint called. This endpoint is deprecated and will be removed in a future version.");
    tracing::warn!("📋 Migration: Generate SP1 proofs client-side and submit to relay. See: https://docs.cloaklabz.xyz/zk");

    tracing::info!("🔐 Received proof generation request");
    tracing::info!(
        client_ip = client_addr.ip().to_string(),
        private_inputs_len = request.private_inputs.len(),
        public_inputs_len = request.public_inputs.len(),
        outputs_len = request.outputs.len(),
        "Processing proof generation request"
    );

    // Log swap_params presence
    if let Some(ref sp) = request.swap_params {
        tracing::info!(
            "✅ swap_params is present: {}",
            serde_json::to_string(sp).unwrap_or_else(|_| "error".to_string())
        );
    } else {
        tracing::info!("⚠️  swap_params is MISSING/None");
    }

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
        )
            .into_response();
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
        )
            .into_response();
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
        )
            .into_response();
    }
    if let Some(ref sp) = request.swap_params {
        if let Err(e) = serde_json::to_string(sp) {
            tracing::error!("❌ Invalid swap_params JSON: {}", e);
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
                    error: Some(format!("Invalid swap_params JSON: {}", e)),
                }),
            )
                .into_response();
        }
    }
    tracing::info!("✅ Input validation passed");

    // Optional: pre-compute and log expected outputs_hash in swap mode for easier debugging
    if let Some(ref sp) = request.swap_params {
        // Try to parse public inputs to extract amount and outputs_hash
        if let (Ok(public_inputs_val), Ok(sp_val)) = (
            serde_json::from_str::<serde_json::Value>(&request.public_inputs),
            serde_json::from_value::<serde_json::Value>(sp.clone()),
        ) {
            let amount_opt = public_inputs_val.get("amount").and_then(|v| v.as_u64());
            let outputs_hash_hex_opt = public_inputs_val
                .get("outputs_hash")
                .and_then(|v| v.as_str());

            let out_mint_opt = sp_val.get("output_mint").and_then(|v| v.as_str());
            let recip_ata_opt = sp_val.get("recipient_ata").and_then(|v| v.as_str());
            let min_out_opt = sp_val.get("min_output_amount").and_then(|v| v.as_u64());

            if let (
                Some(amount),
                Some(outputs_hash_hex),
                Some(out_mint),
                Some(recip_ata),
                Some(min_out),
            ) = (
                amount_opt,
                outputs_hash_hex_opt,
                out_mint_opt,
                recip_ata_opt,
                min_out_opt,
            ) {
                // Helper to parse base58 or 0x-hex into 32 bytes
                let parse_addr = |s: &str| -> Option<[u8; 32]> {
                    let raw = if let Some(hex) = s.strip_prefix("0x") {
                        hex
                    } else {
                        s
                    };
                    if let Ok(bytes) = hex::decode(raw) {
                        if bytes.len() == 32 {
                            let mut arr = [0u8; 32];
                            arr.copy_from_slice(&bytes);
                            return Some(arr);
                        }
                    }
                    if let Ok(bytes) = bs58::decode(s).into_vec() {
                        if bytes.len() == 32 {
                            let mut arr = [0u8; 32];
                            arr.copy_from_slice(&bytes);
                            return Some(arr);
                        }
                    }
                    None
                };

                if let (Some(mint32), Some(ata32)) = (parse_addr(out_mint), parse_addr(recip_ata)) {
                    let mut hasher = Hasher::new();
                    hasher.update(&mint32);
                    hasher.update(&ata32);
                    hasher.update(&min_out.to_le_bytes());
                    hasher.update(&amount.to_le_bytes());
                    let expected = hasher.finalize();
                    let expected_hex = hex::encode(expected.as_bytes());
                    tracing::info!(
                        expected_outputs_hash = expected_hex.as_str(),
                        public_outputs_hash = outputs_hash_hex,
                        amount,
                        min_out,
                        output_mint = out_mint,
                        recipient_ata = recip_ata,
                        "Computed swap outputs_hash debug"
                    );
                }
            }
        }
    }

    tracing::info!("🚀 Starting TEE proof generation (this may take 30-180 seconds)...");

    // TEE-only proof generation - no fallback
    let Some(tee_client) = &state.tee_client else {
        tracing::error!("❌ TEE client not configured");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ProveResponse {
                success: false,
                proof: None,
                public_inputs: None,
                generation_time_ms: start_time.elapsed().as_millis() as u64,
                total_cycles: None,
                total_syscalls: None,
                execution_report: None,
                proof_method: None,
                wallet_address: None,
                error: Some(
                    "TEE client not configured. Proof generation is only available via TEE."
                        .to_string(),
                ),
            }),
        )
            .into_response();
    };

    tracing::info!("🔐 Using TEE proof generation");
    tracing::info!("   Wallet: {}", tee_client.wallet_address());
    tracing::info!(
        "   Timeout: {} seconds",
        state.config.sp1_tee.timeout_seconds
    );

    // Convert swap_params JSON Value to string using serde_json::to_string
    // This ensures valid JSON formatting (to_string() on Value doesn't produce valid JSON)
    let swap_params_str = request.swap_params.as_ref().and_then(|v| {
        serde_json::to_string(v).ok()
    });

    match tee_client
        .generate_proof(
            &request.private_inputs,
            &request.public_inputs,
            &request.outputs,
            swap_params_str.as_deref(),
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
                    )
                        .into_response();
                }
            };

            // Convert to hex for API response
            let proof_hex = hex::encode(canonical_proof);
            let proof_prefix = hex::encode(&canonical_proof[..4]);
            let public_inputs_hex = hex::encode(&tee_result.public_inputs);

            tracing::info!(canonical_proof_bytes = proof_hex.len() / 2, proof_prefix);
            tracing::info!("🎉 TEE proof generation request completed successfully");

            let mut response = (
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
            )
                .into_response();

            // Add deprecation headers
            let headers = response.headers_mut();
            headers.extend(create_deprecation_headers());

            response
        }
        Err(e) => {
            tracing::error!("❌ TEE proof generation failed: {}", e);
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
                    proof_method: Some("tee".to_string()),
                    wallet_address: Some(tee_client.wallet_address().to_string()),
                    error: Some(format!("TEE proof generation failed: {}", e)),
                }),
            )
                .into_response()
        }
    }
}
