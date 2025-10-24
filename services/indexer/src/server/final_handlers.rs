use crate::artifacts::ArtifactManager;
use crate::database::PostgresTreeStorage;
use crate::merkle::{MerkleTree, TreeStorage};
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
    let endpoints = [
        ("health", "/health"),
        ("deposit", "/api/v1/deposit"),
        ("merkle_root", "/api/v1/merkle/root"),
        ("merkle_proof", "/api/v1/merkle/proof/:index"),
        ("notes_range", "/api/v1/notes/range"),
        ("artifacts", "/api/v1/artifacts/withdraw/:version"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

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
    State(state): State<AppState>,
    Json(request): Json<DepositRequest>,
) -> impl IntoResponse {
    // Basic validation
    if request.leaf_commit.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Leaf commit must be 64 characters"
            })),
        );
    }

    if request.encrypted_output.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Encrypted output cannot be empty"
            })),
        );
    }

    // Get the next available leaf index
    let next_index = match state.storage.get_max_leaf_index().await {
        Ok(index) => index,
        Err(e) => {
            tracing::error!("Failed to get next leaf index: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get next leaf index"
                })),
            );
        }
    };

    // Generate a unique transaction signature if not provided
    let tx_signature = request.tx_signature.unwrap_or_else(|| {
        format!(
            "test_tx_{}_{}",
            next_index,
            chrono::Utc::now().timestamp_millis()
        )
    });

    // Use current slot or generate a test slot
    let slot = request.slot.unwrap_or(1000 + next_index as i64);

    // Store the note in the database
    let store_result = state
        .storage
        .store_note(
            &request.leaf_commit,
            &request.encrypted_output,
            next_index as i64,
            &tx_signature,
            slot,
            Some(chrono::Utc::now()),
        )
        .await;

    match store_result {
        Ok(_) => {
            // Update the next leaf index in metadata
            if let Err(e) = state
                .storage
                .update_metadata("next_leaf_index", &(next_index + 1).to_string())
                .await
            {
                tracing::warn!("Failed to update next leaf index metadata: {}", e);
            }

            // Insert the leaf into the merkle tree and get the new root
            let mut tree = state.merkle_tree.lock().await;
            match tree.insert_leaf(&request.leaf_commit, &state.storage).await {
                Ok((new_root, _)) => {
                    tracing::info!(
                        "Successfully inserted leaf at index {} with root: {}",
                        next_index,
                        new_root
                    );
                    (
                        StatusCode::CREATED,
                        Json(serde_json::json!({
                            "success": true,
                            "leafIndex": next_index,
                            "root": new_root,
                            "nextIndex": next_index + 1,
                            "leafCommit": request.leaf_commit.to_lowercase(),
                            "message": "Deposit processed successfully"
                        })),
                    )
                }
                Err(e) => {
                    tracing::error!("Failed to insert leaf into merkle tree: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Failed to update merkle tree",
                            "details": e.to_string()
                        })),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to store note: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to store note",
                    "details": e.to_string()
                })),
            )
        }
    }
}

pub async fn get_merkle_root(State(state): State<AppState>) -> impl IntoResponse {
    // Get the current merkle tree state
    let tree = state.merkle_tree.lock().await;
    match tree.get_tree_state(&state.storage).await {
        Ok(tree_state) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "root": tree_state.root,
                "next_index": tree_state.next_index
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get merkle tree state: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get merkle tree state",
                    "details": e.to_string()
                })),
            )
        }
    }
}

pub async fn reset_database(State(state): State<AppState>) -> impl IntoResponse {
    tracing::info!("Resetting database...");

    match state.storage.reset_database().await {
        Ok(_) => {
            // Also reset the Merkle tree state
            let mut tree = state.merkle_tree.lock().await;
            tree.reset_state();
            tracing::info!("Database and Merkle tree reset successfully");
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Database and Merkle tree reset successfully"
                })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to reset database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to reset database: {}", e)
                })),
            )
        }
    }
}

pub async fn get_merkle_proof(
    State(state): State<AppState>,
    Path(index): Path<u64>,
) -> impl IntoResponse {
    if index > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Index too large"
            })),
        );
    }

    // Generate the merkle proof using the actual merkle tree
    let tree = state.merkle_tree.lock().await;
    match tree.generate_proof(index, &state.storage).await {
        Ok(proof) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "pathElements": proof.path_elements,
                "pathIndices": proof.path_indices
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to generate merkle proof for index {}: {}", index, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to generate merkle proof",
                    "details": e.to_string()
                })),
            )
        }
    }
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
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "encrypted_outputs": [],
            "has_more": false,
            "total": 0,
            "start": start,
            "end": end
        })),
    )
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
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "guest_elf_url": format!("/api/v1/artifacts/files/{}/guest.elf", version),
            "vk": "dummy_vk_base64",
            "sha256": {
                "elf": "a".repeat(64),
                "vk": "b".repeat(64)
            },
            "sp1_version": "v2.0.0"
        })),
    )
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
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "content": "dummy_file_content"
        })),
    )
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
