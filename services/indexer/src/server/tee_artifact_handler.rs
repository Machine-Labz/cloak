use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::final_handlers::AppState;

/// Request to create a stdin artifact
#[derive(Debug, Deserialize)]
pub struct CreateArtifactRequest {
    pub program_id: Option<String>, // Optional program ID for validation
}

/// Response with artifact ID and upload URL
#[derive(Debug, Serialize)]
pub struct CreateArtifactResponse {
    pub artifact_id: String,
    pub upload_url: String,
    pub expires_at: Option<String>, // ISO 8601 timestamp
}

/// Request to generate proof using artifact
#[derive(Debug, Deserialize)]
pub struct RequestProofRequest {
    pub artifact_id: String,
    pub program_id: Option<String>,
    pub public_inputs: String, // JSON string
}

/// Response with proof request ID
#[derive(Debug, Serialize)]
pub struct RequestProofResponse {
    pub request_id: String,
    pub status: String, // "pending", "processing", "ready", "failed"
}

/// Query parameters for proof status
#[derive(Debug, Deserialize)]
pub struct ProofStatusQuery {
    pub request_id: String,
}

/// Response with proof status and result
#[derive(Debug, Serialize)]
pub struct ProofStatusResponse {
    pub request_id: String,
    pub status: String,        // "pending", "processing", "ready", "failed"
    pub proof: Option<String>, // Hex-encoded proof bytes (when ready)
    pub public_inputs: Option<String>, // Hex-encoded public inputs (when ready)
    pub generation_time_ms: Option<u64>,
    pub error: Option<String>,
}

/// In-memory store for artifact uploads and proof requests
/// In production, this should be replaced with a proper database or Redis
#[derive(Clone)]
pub struct ArtifactStore {
    // artifact_id -> (upload_url, expires_at, stdin_data)
    artifacts: Arc<
        tokio::sync::RwLock<
            HashMap<
                String,
                (
                    String,
                    Option<chrono::DateTime<chrono::Utc>>,
                    Option<String>,
                ),
            >,
        >,
    >,
    // request_id -> proof_status
    proof_requests: Arc<tokio::sync::RwLock<HashMap<String, ProofRequestStatus>>>,
}

#[derive(Debug, Clone)]
struct ProofRequestStatus {
    artifact_id: String,
    status: String,
    proof: Option<String>,
    public_inputs: Option<String>,
    generation_time_ms: Option<u64>,
    error: Option<String>,
}

impl Default for ArtifactStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactStore {
    pub fn new() -> Self {
        Self {
            artifacts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            proof_requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_artifact(&self, artifact_id: String, upload_url: String) {
        let mut artifacts = self.artifacts.write().await;
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        artifacts.insert(artifact_id, (upload_url, Some(expires_at), None));
    }

    pub async fn get_artifact(
        &self,
        artifact_id: &str,
    ) -> Option<(String, Option<chrono::DateTime<chrono::Utc>>)> {
        let artifacts = self.artifacts.read().await;
        artifacts
            .get(artifact_id)
            .map(|(url, expires, _)| (url.clone(), *expires))
    }

    pub async fn store_stdin(&self, artifact_id: &str, stdin_data: String) -> Result<(), String> {
        let mut artifacts = self.artifacts.write().await;
        if let Some((url, expires, _)) = artifacts.get_mut(artifact_id) {
            *artifacts.get_mut(artifact_id).unwrap() = (url.clone(), *expires, Some(stdin_data));
            Ok(())
        } else {
            Err("Artifact not found".to_string())
        }
    }

    pub async fn get_stdin(&self, artifact_id: &str) -> Option<String> {
        let artifacts = self.artifacts.read().await;
        artifacts
            .get(artifact_id)
            .and_then(|(_, _, stdin)| stdin.clone())
    }

    pub async fn create_proof_request(&self, request_id: String, artifact_id: String) {
        let mut requests = self.proof_requests.write().await;
        requests.insert(
            request_id,
            ProofRequestStatus {
                artifact_id,
                status: "pending".to_string(),
                proof: None,
                public_inputs: None,
                generation_time_ms: None,
                error: None,
            },
        );
    }

    pub async fn update_proof_status(
        &self,
        request_id: &str,
        status: String,
        proof: Option<String>,
        public_inputs: Option<String>,
        generation_time_ms: Option<u64>,
        error: Option<String>,
    ) {
        let mut requests = self.proof_requests.write().await;
        if let Some(request) = requests.get_mut(request_id) {
            request.status = status;
            request.proof = proof;
            request.public_inputs = public_inputs;
            request.generation_time_ms = generation_time_ms;
            request.error = error;
        }
    }

    pub async fn get_proof_status(&self, request_id: &str) -> Option<ProofRequestStatus> {
        let requests = self.proof_requests.read().await;
        requests.get(request_id).cloned()
    }
}

// Global artifact store (in production, use a database)
use once_cell::sync::Lazy;

static ARTIFACT_STORE: Lazy<ArtifactStore> = Lazy::new(ArtifactStore::new);

/// POST /tee/artifact
/// Create a stdin artifact and get upload URL
pub async fn create_artifact(
    State(_state): State<AppState>,
    Json(_request): Json<CreateArtifactRequest>,
) -> impl IntoResponse {
    let artifact_id = Uuid::new_v4().to_string();
    let upload_url = format!("/api/v1/tee/artifact/{}/upload", artifact_id);
    ARTIFACT_STORE
        .create_artifact(artifact_id.clone(), upload_url.clone())
        .await;
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    (
        StatusCode::CREATED,
        Json(CreateArtifactResponse {
            artifact_id,
            upload_url,
            expires_at: Some(expires_at.to_rfc3339()),
        }),
    )
}

/// POST /tee/artifact/:artifact_id/upload
/// Upload stdin data to artifact (called by frontend directly)
pub async fn upload_stdin(Path(artifact_id): Path<String>, body: Body) -> impl IntoResponse {
    // Convert body to string
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read body: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Failed to read request body"
                })),
            );
        }
    };

    let stdin_data = match String::from_utf8(body_bytes.to_vec()) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse stdin as UTF-8: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid UTF-8 in stdin data"
                })),
            );
        }
    };

    match ARTIFACT_STORE.store_stdin(&artifact_id, stdin_data).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "artifact_id": artifact_id
            })),
        ),
        Err(e) => {
            tracing::error!(artifact_id = %artifact_id, error = %e, "Failed to store stdin");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Artifact not found: {}", e)
                })),
            )
        }
    }
}

/// POST /tee/request-proof
/// Request proof generation using artifact
pub async fn request_proof(
    State(state): State<AppState>,
    Json(request): Json<RequestProofRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Validate artifact exists
    let artifact = ARTIFACT_STORE.get_artifact(&request.artifact_id).await;
    if artifact.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Artifact not found"
            })),
        ));
    }

    // Get stdin data
    let stdin_data = match ARTIFACT_STORE.get_stdin(&request.artifact_id).await {
        Some(data) => data,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Artifact stdin data not uploaded yet"
                })),
            ));
        }
    };

    // Generate request ID
    let request_id = Uuid::new_v4().to_string();

    // Create proof request
    ARTIFACT_STORE
        .create_proof_request(request_id.clone(), request.artifact_id.clone())
        .await;

    // Get TEE client
    let Some(tee_client) = &state.tee_client else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "TEE client not configured"
            })),
        ));
    };

    // Spawn async task to generate proof
    let tee_client_clone = tee_client.clone();
    let request_id_clone = request_id.clone();
    let public_inputs = request.public_inputs.clone();

    tokio::spawn(async move {
        // Update status to processing
        ARTIFACT_STORE
            .update_proof_status(
                &request_id_clone,
                "processing".to_string(),
                None,
                None,
                None,
                None,
            )
            .await;

        // Parse stdin to extract private/public/outputs
        // The stdin_data is a JSON string with the combined input
        // We need to parse it and extract the parts
        let stdin_json: serde_json::Value = match serde_json::from_str(&stdin_data) {
            Ok(json) => json,
            Err(e) => {
                ARTIFACT_STORE
                    .update_proof_status(
                        &request_id_clone,
                        "failed".to_string(),
                        None,
                        None,
                        None,
                        Some(format!("Failed to parse stdin JSON: {}", e)),
                    )
                    .await;
                return;
            }
        };

        // Extract private, public, and outputs from stdin
        let private_inputs = match stdin_json.get("private") {
            Some(p) => serde_json::to_string(p).unwrap_or_default(),
            None => {
                ARTIFACT_STORE
                    .update_proof_status(
                        &request_id_clone,
                        "failed".to_string(),
                        None,
                        None,
                        None,
                        Some("Missing 'private' field in stdin".to_string()),
                    )
                    .await;
                return;
            }
        };

        let public_inputs_from_stdin = match stdin_json.get("public") {
            Some(p) => serde_json::to_string(p).unwrap_or_default(),
            None => public_inputs.clone(),
        };

        let outputs = match stdin_json.get("outputs") {
            Some(o) => serde_json::to_string(o).unwrap_or_default(),
            None => {
                ARTIFACT_STORE
                    .update_proof_status(
                        &request_id_clone,
                        "failed".to_string(),
                        None,
                        None,
                        None,
                        Some("Missing 'outputs' field in stdin".to_string()),
                    )
                    .await;
                return;
            }
        };

        // Use provided public_inputs if available, otherwise use from stdin
        let public_inputs_final = if !public_inputs.is_empty() {
            public_inputs
        } else {
            public_inputs_from_stdin
        };

        // Extract swap_params if present (optional for swap transactions)
        let swap_params = stdin_json
            .get("swap_params")
            .and_then(|sp| serde_json::to_string(sp).ok());

        // Generate proof
        let start_time = std::time::Instant::now();
        match tee_client_clone
            .generate_proof(
                &private_inputs,
                &public_inputs_final,
                &outputs,
                swap_params.as_deref(),
            )
            .await
        {
            Ok(result) => {
                let _generation_time = start_time.elapsed().as_millis() as u64;
                let proof_hex = hex::encode(&result.proof_bytes);
                let public_inputs_hex = hex::encode(&result.public_inputs);

                ARTIFACT_STORE
                    .update_proof_status(
                        &request_id_clone,
                        "ready".to_string(),
                        Some(proof_hex),
                        Some(public_inputs_hex),
                        Some(result.generation_time_ms),
                        None,
                    )
                    .await;
            }
            Err(e) => {
                ARTIFACT_STORE
                    .update_proof_status(
                        &request_id_clone,
                        "failed".to_string(),
                        None,
                        None,
                        None,
                        Some(format!("Proof generation failed: {}", e)),
                    )
                    .await;

                tracing::error!(
                    request_id = %request_id_clone,
                    error = %e,
                    "Proof generation failed"
                );
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(RequestProofResponse {
            request_id,
            status: "pending".to_string(),
        }),
    ))
}

/// GET /tee/proof-status
/// Get proof generation status
pub async fn get_proof_status(
    Query(query): Query<ProofStatusQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match ARTIFACT_STORE.get_proof_status(&query.request_id).await {
        Some(status) => Ok((
            StatusCode::OK,
            Json(ProofStatusResponse {
                request_id: query.request_id,
                status: status.status,
                proof: status.proof,
                public_inputs: status.public_inputs,
                generation_time_ms: status.generation_time_ms,
                error: status.error,
            }),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Proof request not found"
            })),
        )),
    }
}
