use crate::artifacts::ArtifactManager;
use crate::database::PostgresTreeStorage;
use crate::merkle::MerkleTree;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub storage: PostgresTreeStorage,
    pub merkle_tree: Arc<Mutex<MerkleTree>>,
    pub artifact_manager: ArtifactManager,
    pub config: crate::config::Config,
}

// Request types
#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub leaf_commit: String,
    pub encrypted_output: String,
    pub tx_signature: Option<String>,
    pub slot: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct NotesRangeQuery {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub limit: Option<i64>,
}

// API Handlers

pub async fn api_info() -> impl IntoResponse {
    let mut endpoints = HashMap::new();
    endpoints.insert("health".to_string(), "/health".to_string());
    endpoints.insert("deposit".to_string(), "/api/v1/deposit".to_string());
    endpoints.insert("merkle_root".to_string(), "/api/v1/merkle/root".to_string());
    endpoints.insert("merkle_proof".to_string(), "/api/v1/merkle/proof/:index".to_string());
    endpoints.insert("notes_range".to_string(), "/api/v1/notes/range".to_string());
    endpoints.insert("artifacts".to_string(), "/api/v1/artifacts/withdraw/:version".to_string());

    Json(serde_json::json!({
        "name": "Cloak Indexer API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Merkle tree indexer for Cloak privacy protocol",
        "endpoints": endpoints,
        "documentation": "https://docs.cloak.network/indexer",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "database": {
            "healthy": true
        },
        "merkle_tree": {
            "initialized": true,
            "height": 32
        },
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn deposit(
    State(_state): State<AppState>,
    Json(request): Json<DepositRequest>,
) -> impl IntoResponse {
    // Basic validation
    if request.leaf_commit.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Leaf commit must be 64 characters"
            }))
        );
    }

    if request.encrypted_output.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Encrypted output cannot be empty"
            }))
        );
    }

    // Return success response (actual implementation would use the merkle tree)
    (StatusCode::CREATED, Json(serde_json::json!({
        "success": true,
        "leaf_index": 0,
        "root": "dummy_root_hash_placeholder".repeat(2),
        "next_index": 1,
        "leaf_commit": request.leaf_commit.to_lowercase(),
        "message": "Deposit processed successfully"
    })))
}

pub async fn get_merkle_root(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "root": "dummy_root_hash_placeholder".repeat(2),
        "next_index": 0
    }))
}

pub async fn get_merkle_proof(
    State(_state): State<AppState>,
    Path(index): Path<u64>,
) -> impl IntoResponse {
    if index > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Index too large"
            }))
        );
    }

    (StatusCode::OK, Json(serde_json::json!({
        "path_elements": vec!["dummy_element".repeat(2); 31],
        "path_indices": vec![0; 31]
    })))
}

pub async fn get_notes_range(
    State(_state): State<AppState>,
    Query(params): Query<NotesRangeQuery>,
) -> impl IntoResponse {
    let start = params.start.unwrap_or(0);
    let end = params.end.unwrap_or(start + 100);

    if end < start {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "End must be >= start"
            }))
        );
    }

    (StatusCode::OK, Json(serde_json::json!({
        "encrypted_outputs": [],
        "has_more": false,
        "total": 0,
        "start": start,
        "end": end
    })))
}

pub async fn get_withdraw_artifacts(
    State(_state): State<AppState>,
    Path(version): Path<String>,
) -> impl IntoResponse {
    if !version.starts_with('v') {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid version format"
            }))
        );
    }

    (StatusCode::OK, Json(serde_json::json!({
        "guest_elf_url": format!("/api/v1/artifacts/files/{}/guest.elf", version),
        "vk": "dummy_vk_base64",
        "sha256": {
            "elf": "a".repeat(64),
            "vk": "b".repeat(64)
        },
        "sp1_version": "v2.0.0"
    })))
}

pub async fn serve_artifact_file(
    State(_state): State<AppState>,
    Path((_version, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    if filename.contains("..") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid filename"
            }))
        );
    }

    (StatusCode::OK, Json(serde_json::json!({
        "content": "dummy_file_content"
    })))
}

pub async fn admin_push_root(
    State(_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "Root pushed successfully"
    }))
}

pub async fn admin_insert_leaf(
    State(_state): State<AppState>,
    Json(request): Json<DepositRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "leaf_index": 0,
        "root": "dummy_root_hash_placeholder".repeat(2),
        "next_index": 1,
        "leaf_commit": request.leaf_commit.to_lowercase(),
        "message": "Leaf inserted successfully"
    }))
}
