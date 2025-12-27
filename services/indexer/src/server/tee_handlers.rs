//! TEE Artifact-Based Proof Generation Handlers
//!
//! This module implements the artifact-based proof generation flow where
//! private inputs are uploaded directly to the TEE, never passing through
//! the backend in plaintext.
//!
//! Flow:
//! 1. POST /api/v1/tee/artifact - Create artifact and return upload URL
//! 2. POST /api/v1/tee/artifact/:artifact_id/upload - Upload stdin (private inputs)
//! 3. POST /api/v1/tee/request-proof - Request proof generation using artifact_id
//! 4. GET /api/v1/tee/proof-status - Poll for proof status

#![allow(dead_code)]

use crate::server::final_handlers::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cloak_proof_extract::extract_groth16_260_sp1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory storage for artifacts and proof requests
/// For production, consider using Redis or database
#[derive(Debug, Clone, Default)]
pub struct TeeArtifactStore {
    /// Artifact ID -> Stdin JSON (private inputs)
    artifacts: Arc<RwLock<HashMap<String, ArtifactData>>>,
    /// Request ID -> Proof request status
    requests: Arc<RwLock<HashMap<String, ProofRequestStatus>>>,
}

#[derive(Debug, Clone)]
pub struct ArtifactData {
    pub stdin: Option<String>,  // JSON string of stdin
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofRequestStatus {
    pub request_id: String,
    pub artifact_id: String,
    pub status: ProofStatus,
    pub proof: Option<String>,
    pub public_inputs: Option<String>,
    pub generation_time_ms: Option<u64>,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProofStatus {
    Pending,
    Processing,
    Ready,
    Failed,
}

impl TeeArtifactStore {
    pub fn new() -> Self {
        Self {
            artifacts: Arc::new(RwLock::new(HashMap::new())),
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_artifact(&self) -> (String, chrono::DateTime<chrono::Utc>) {
        let artifact_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        let expires_at = created_at + chrono::Duration::minutes(30);
        
        let mut artifacts = self.artifacts.write().await;
        artifacts.insert(artifact_id.clone(), ArtifactData {
            stdin: None,
            created_at,
            expires_at,
        });
        
        (artifact_id, expires_at)
    }

    pub async fn upload_stdin(&self, artifact_id: &str, stdin: String) -> Result<(), &'static str> {
        let mut artifacts = self.artifacts.write().await;
        
        if let Some(artifact) = artifacts.get_mut(artifact_id) {
            if chrono::Utc::now() > artifact.expires_at {
                return Err("Artifact has expired");
            }
            artifact.stdin = Some(stdin);
            Ok(())
        } else {
            Err("Artifact not found")
        }
    }

    pub async fn get_stdin(&self, artifact_id: &str) -> Option<String> {
        let artifacts = self.artifacts.read().await;
        artifacts.get(artifact_id)
            .filter(|a| chrono::Utc::now() <= a.expires_at)
            .and_then(|a| a.stdin.clone())
    }

    pub async fn create_request(&self, artifact_id: &str) -> String {
        let request_id = Uuid::new_v4().to_string();
        
        let mut requests = self.requests.write().await;
        requests.insert(request_id.clone(), ProofRequestStatus {
            request_id: request_id.clone(),
            artifact_id: artifact_id.to_string(),
            status: ProofStatus::Pending,
            proof: None,
            public_inputs: None,
            generation_time_ms: None,
            error: None,
            created_at: chrono::Utc::now(),
        });
        
        request_id
    }

    pub async fn update_request_status(&self, request_id: &str, status: ProofStatus) {
        let mut requests = self.requests.write().await;
        if let Some(req) = requests.get_mut(request_id) {
            req.status = status;
        }
    }

    pub async fn complete_request(
        &self,
        request_id: &str,
        proof: String,
        public_inputs: String,
        generation_time_ms: u64,
    ) {
        let mut requests = self.requests.write().await;
        if let Some(req) = requests.get_mut(request_id) {
            req.status = ProofStatus::Ready;
            req.proof = Some(proof);
            req.public_inputs = Some(public_inputs);
            req.generation_time_ms = Some(generation_time_ms);
        }
    }

    pub async fn fail_request(&self, request_id: &str, error: String) {
        let mut requests = self.requests.write().await;
        if let Some(req) = requests.get_mut(request_id) {
            req.status = ProofStatus::Failed;
            req.error = Some(error);
        }
    }

    pub async fn get_request_status(&self, request_id: &str) -> Option<ProofRequestStatus> {
        let requests = self.requests.read().await;
        requests.get(request_id).cloned()
    }

    /// Clean up expired artifacts and old requests (call periodically)
    pub async fn cleanup(&self) {
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::hours(1);
        
        // Clean expired artifacts
        {
            let mut artifacts = self.artifacts.write().await;
            artifacts.retain(|_, a| a.expires_at > now);
        }
        
        // Clean old completed/failed requests
        {
            let mut requests = self.requests.write().await;
            requests.retain(|_, r| {
                r.created_at > cutoff || 
                (r.status != ProofStatus::Ready && r.status != ProofStatus::Failed)
            });
        }
    }
}

// Global artifact store - initialized lazily
lazy_static::lazy_static! {
    pub static ref ARTIFACT_STORE: TeeArtifactStore = TeeArtifactStore::new();
}

// ============== Request/Response Types ==============

#[derive(Debug, Deserialize)]
pub struct CreateArtifactRequest {
    #[serde(default)]
    pub program_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateArtifactResponse {
    pub artifact_id: String,
    pub upload_url: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadStdinRequest {
    pub private: serde_json::Value,
    pub public: serde_json::Value,
    pub outputs: serde_json::Value,
    #[serde(default)]
    pub swap_params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UploadStdinResponse {
    pub success: bool,
    pub artifact_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestProofRequest {
    pub artifact_id: String,
    #[serde(default)]
    pub program_id: Option<String>,
    pub public_inputs: String,  // JSON string
    #[serde(default)]
    pub swap_params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RequestProofResponse {
    pub request_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ProofStatusQuery {
    pub request_id: String,
}

#[derive(Debug, Serialize)]
pub struct ProofStatusResponse {
    pub request_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_inputs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============== Handlers ==============

/// POST /api/v1/tee/artifact
/// 
/// Create a new artifact and return the upload URL
pub async fn create_artifact(
    Json(_request): Json<CreateArtifactRequest>,
) -> impl IntoResponse {
    tracing::info!("📦 Creating new TEE artifact");
    
    let (artifact_id, expires_at) = ARTIFACT_STORE.create_artifact().await;
    let upload_url = format!("/api/v1/tee/artifact/{}/upload", artifact_id);
    
    tracing::info!(
        artifact_id = %artifact_id,
        upload_url = %upload_url,
        expires_at = %expires_at,
        "✅ Artifact created"
    );
    
    (
        StatusCode::OK,
        Json(CreateArtifactResponse {
            artifact_id,
            upload_url,
            expires_at: expires_at.to_rfc3339(),
        }),
    )
}

/// POST /api/v1/tee/artifact/:artifact_id/upload
/// 
/// Upload stdin (private inputs) for an artifact
pub async fn upload_stdin(
    Path(artifact_id): Path<String>,
    Json(request): Json<UploadStdinRequest>,
) -> impl IntoResponse {
    tracing::info!(artifact_id = %artifact_id, "📤 Receiving stdin upload");
    
    // Debug: log what we received
    tracing::info!(
        has_swap_params = request.swap_params.is_some(),
        "📋 Upload request params"
    );
    
    // Convert the request to a combined JSON string (format expected by TEE)
    let combined = serde_json::json!({
        "private": request.private,
        "public": request.public,
        "outputs": request.outputs,
        "swap_params": request.swap_params,
    });
    
    let stdin_json = match serde_json::to_string(&combined) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("❌ Failed to serialize stdin: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to serialize stdin: {}", e)
                })),
            );
        }
    };
    
    
    match ARTIFACT_STORE.upload_stdin(&artifact_id, stdin_json).await {
        Ok(_) => {
            tracing::info!(artifact_id = %artifact_id, "✅ Stdin uploaded successfully");
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "artifact_id": artifact_id
                })),
            )
        }
        Err(e) => {
            tracing::error!(artifact_id = %artifact_id, error = e, "❌ Failed to upload stdin");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": e
                })),
            )
        }
    }
}

/// POST /api/v1/tee/request-proof
/// 
/// Request proof generation using an artifact
pub async fn request_proof(
    State(state): State<AppState>,
    Json(request): Json<RequestProofRequest>,
) -> impl IntoResponse {
    tracing::info!(
        artifact_id = %request.artifact_id,
        "🔐 Requesting proof generation"
    );
    
    // Verify artifact exists and has stdin
    let stdin = match ARTIFACT_STORE.get_stdin(&request.artifact_id).await {
        Some(s) => s,
        None => {
            tracing::error!(
                artifact_id = %request.artifact_id,
                "❌ Artifact not found or expired or stdin not uploaded"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Artifact not found, expired, or stdin not uploaded"
                })),
            );
        }
    };
    
    // Check TEE client is available
    let tee_client = match &state.tee_client {
        Some(client) => client.clone(),
        None => {
            tracing::error!("❌ TEE client not configured");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "success": false,
                    "error": "TEE client not configured"
                })),
            );
        }
    };
    
    // Create request and start proof generation in background
    let request_id = ARTIFACT_STORE.create_request(&request.artifact_id).await;
    
    // Parse stdin to extract components
    let stdin_parsed: serde_json::Value = match serde_json::from_str(&stdin) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("❌ Failed to parse stored stdin: {}", e);
            ARTIFACT_STORE.fail_request(&request_id, format!("Invalid stdin: {}", e)).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid stored stdin: {}", e)
                })),
            );
        }
    };
    
    // Debug: log the root from public inputs (both from stdin and request)
    if let Some(stdin_public) = stdin_parsed.get("public") {
        if let Some(stdin_root) = stdin_public.get("root") {
            tracing::info!("📋 Root from stdin.public: {:?}", stdin_root);
        }
        // Log all public input fields
        tracing::info!("📋 stdin.public full: {}", serde_json::to_string_pretty(stdin_public).unwrap_or_default());
    }
    
    // Also log private inputs summary
    if let Some(stdin_private) = stdin_parsed.get("private") {
        if let Some(leaf_index) = stdin_private.get("leaf_index") {
            tracing::info!("📋 stdin.private.leaf_index: {:?}", leaf_index);
        }
        if let Some(amount) = stdin_private.get("amount") {
            tracing::info!("📋 stdin.private.amount: {:?}", amount);
        }
        // Log merkle_path summary
        if let Some(merkle_path) = stdin_private.get("merkle_path") {
            if let Some(elements) = merkle_path.get("path_elements") {
                if let Some(arr) = elements.as_array() {
                    tracing::info!("📋 stdin.private.merkle_path.path_elements count: {}", arr.len());
                    if !arr.is_empty() {
                        tracing::info!("📋 First path element: {:?}", arr[0]);
                    }
                }
            }
            if let Some(indices) = merkle_path.get("path_indices") {
                if let Some(arr) = indices.as_array() {
                    tracing::info!("📋 stdin.private.merkle_path.path_indices count: {}", arr.len());
                    // Log first few indices
                    let indices_preview: Vec<_> = arr.iter().take(5).collect();
                    tracing::info!("📋 First 5 path indices: {:?}", indices_preview);
                }
            }
        }
    }
    
    // Also parse the public_inputs from the request for comparison
    if let Ok(req_public) = serde_json::from_str::<serde_json::Value>(&request.public_inputs) {
        if let Some(req_root) = req_public.get("root") {
            tracing::info!("📋 Root from request.public_inputs: {:?}", req_root);
        }
    }
    
    // Extract components from stdin
    let private_inputs = match serde_json::to_string(&stdin_parsed["private"]) {
        Ok(s) => s,
        Err(e) => {
            ARTIFACT_STORE.fail_request(&request_id, format!("Invalid private inputs: {}", e)).await;
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid private inputs: {}", e)
                })),
            );
        }
    };
    
    let public_inputs = request.public_inputs.clone();
    
    let outputs = match serde_json::to_string(&stdin_parsed["outputs"]) {
        Ok(s) => s,
        Err(e) => {
            ARTIFACT_STORE.fail_request(&request_id, format!("Invalid outputs: {}", e)).await;
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid outputs: {}", e)
                })),
            );
        }
    };
    
    // Extract optional params from stdin (these were uploaded with private inputs)
    let swap_params = stdin_parsed.get("swap_params")
        .filter(|v| !v.is_null())
        .and_then(|v| serde_json::to_string(v).ok());
    
    // Debug: log what we extracted
    tracing::info!(
        has_swap_params = swap_params.is_some(),
        "📋 Extracted params from stdin"
    );
    
    // Spawn background task for proof generation
    let request_id_clone = request_id.clone();
    tokio::spawn(async move {
        tracing::info!(request_id = %request_id_clone, "🚀 Starting background proof generation");
        
        ARTIFACT_STORE.update_request_status(&request_id_clone, ProofStatus::Processing).await;
        
        match tee_client.generate_proof(
            &private_inputs,
            &public_inputs,
            &outputs,
            swap_params.as_deref(),
        ).await {
            Ok(result) => {
                // Extract canonical proof
                match extract_groth16_260_sp1(&result.proof_bytes) {
                    Ok(canonical_proof) => {
                        let proof_hex = hex::encode(canonical_proof);
                        let public_inputs_hex = hex::encode(&result.public_inputs);
                        
                        ARTIFACT_STORE.complete_request(
                            &request_id_clone,
                            proof_hex,
                            public_inputs_hex,
                            result.generation_time_ms,
                        ).await;
                        
                        tracing::info!(
                            request_id = %request_id_clone,
                            generation_time_ms = result.generation_time_ms,
                            "✅ Proof generation completed successfully"
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            request_id = %request_id_clone,
                            error = ?e,
                            "❌ Failed to extract canonical proof"
                        );
                        ARTIFACT_STORE.fail_request(
                            &request_id_clone,
                            format!("Failed to extract canonical proof: {:?}", e),
                        ).await;
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    request_id = %request_id_clone,
                    error = %e,
                    "❌ TEE proof generation failed"
                );
                ARTIFACT_STORE.fail_request(&request_id_clone, format!("TEE proof generation failed: {}", e)).await;
            }
        }
    });
    
    tracing::info!(
        request_id = %request_id,
        "✅ Proof request created, processing in background"
    );
    
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "request_id": request_id,
            "status": "pending"
        })),
    )
}

/// GET /api/v1/tee/proof-status
/// 
/// Get the status of a proof generation request
pub async fn get_proof_status(
    Query(params): Query<ProofStatusQuery>,
) -> impl IntoResponse {
    let request_id = &params.request_id;
    
    tracing::debug!(request_id = %request_id, "🔍 Checking proof status");
    
    match ARTIFACT_STORE.get_request_status(request_id).await {
        Some(status) => {
            let status_str = match &status.status {
                ProofStatus::Pending => "pending",
                ProofStatus::Processing => "processing",
                ProofStatus::Ready => "ready",
                ProofStatus::Failed => "failed",
            };
            
            (
                StatusCode::OK,
                Json(ProofStatusResponse {
                    request_id: status.request_id,
                    status: status_str.to_string(),
                    proof: status.proof,
                    public_inputs: status.public_inputs,
                    generation_time_ms: status.generation_time_ms,
                    error: status.error,
                }),
            )
        }
        None => {
            (
                StatusCode::NOT_FOUND,
                Json(ProofStatusResponse {
                    request_id: request_id.clone(),
                    status: "not_found".to_string(),
                    proof: None,
                    public_inputs: None,
                    generation_time_ms: None,
                    error: Some("Request not found".to_string()),
                }),
            )
        }
    }
}

