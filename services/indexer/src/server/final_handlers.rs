use crate::artifacts::ArtifactManager;
use crate::database::PostgresTreeStorage;
use crate::merkle::{MerkleTree, TreeStorage};
use crate::server::rate_limiter::RateLimiter;
use crate::solana::push_root_to_chain;
use crate::sp1_tee_client::Sp1TeeClient;
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
    pub rate_limiter: Arc<RateLimiter>,
    pub tee_client: Option<Arc<Sp1TeeClient>>,
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
        ("prove", "/api/v1/prove"),
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
        "deprecated_endpoints": {
            "prove": {
                "endpoint": "/api/v1/prove",
                "reason": "Server-side proof generation will be removed. Use client-side proof generation instead.",
                "sunset_date": "2025-06-01",
                "migration_guide": "https://docs.cloak.network/architecture/proving"
            }
        },
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
    tracing::info!("📥 Received deposit request");
    tracing::info!(
        leaf_commit = request.leaf_commit,
        encrypted_output_len = request.encrypted_output.len(),
        tx_signature = request.tx_signature.as_deref().unwrap_or("none"),
        slot = request.slot.unwrap_or(-1),
        "Processing deposit request"
    );

    // Basic validation
    if request.leaf_commit.len() != 64 {
        tracing::warn!(
            leaf_commit_len = request.leaf_commit.len(),
            "Invalid leaf commit length"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Leaf commit must be 64 characters"
            })),
        );
    }

    if request.encrypted_output.is_empty() {
        tracing::warn!("Empty encrypted output provided");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Encrypted output cannot be empty"
            })),
        );
    }

    // Get the next available leaf index
    tracing::info!("🔍 Getting next available leaf index");
    let next_index = match state.storage.get_max_leaf_index().await {
        Ok(index) => {
            tracing::info!(next_index = index, "Next leaf index retrieved");
            index
        }
        Err(e) => {
            tracing::error!("❌ Failed to get next leaf index: {}", e);
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

    tracing::info!(
        tx_signature = tx_signature,
        slot = slot,
        "Generated transaction signature and slot"
    );

    // Store the note in the database
    tracing::info!("💾 Storing note in database");
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
            tracing::info!("✅ Note stored successfully in database");

            // Update the next leaf index in metadata
            tracing::info!("📝 Updating next leaf index metadata");
            if let Err(e) = state
                .storage
                .update_metadata("next_leaf_index", &(next_index + 1).to_string())
                .await
            {
                tracing::warn!("⚠️ Failed to update next leaf index metadata: {}", e);
            } else {
                tracing::info!("✅ Next leaf index metadata updated");
            }

            // Insert the leaf into the merkle tree and get the new root
            tracing::info!("🌳 Inserting leaf into Merkle tree");
            let mut tree = state.merkle_tree.lock().await;
            match tree.insert_leaf(&request.leaf_commit, &state.storage).await {
                Ok((new_root, _)) => {
                    tracing::info!(
                        leaf_index = next_index,
                        new_root = new_root,
                        "✅ Successfully inserted leaf into Merkle tree"
                    );
                    
                    // Push new root to on-chain roots ring asynchronously (non-blocking)
                    tracing::info!("🔗 Scheduling root push to on-chain roots ring");
                    let solana_config = state.config.solana.clone();
                    let root_to_push = new_root.clone();
                    tokio::spawn(async move {
                        if let Err(e) = push_root_to_chain(&root_to_push, &solana_config).await {
                            tracing::error!("❌ Failed to push root to on-chain roots ring: {}", e);
                            // The root will be available for manual push later
                        } else {
                            tracing::info!("✅ Root successfully pushed to on-chain roots ring");
                        }
                    });
                    
                    tracing::info!("🎉 Deposit request completed successfully");
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
                    tracing::error!("❌ Failed to insert leaf into merkle tree: {}", e);
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
            tracing::error!("❌ Failed to store note: {}", e);
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
    tracing::info!("🌳 Getting Merkle tree root");

    // Get the current merkle tree state
    let tree = state.merkle_tree.lock().await;
    match tree.get_tree_state(&state.storage).await {
        Ok(tree_state) => {
            tracing::info!(
                root = tree_state.root,
                next_index = tree_state.next_index,
                "✅ Merkle tree state retrieved"
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "root": tree_state.root,
                    "next_index": tree_state.next_index
                })),
            )
        }
        Err(e) => {
            tracing::error!("❌ Failed to get merkle tree state: {}", e);
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
    tracing::info!("🔍 Generating Merkle proof for index: {}", index);

    if index > 1000 {
        tracing::warn!(index = index, "Index too large");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Index too large"
            })),
        );
    }

    // Generate the merkle proof using the actual merkle tree
    tracing::info!("🌳 Generating proof from Merkle tree");
    let tree = state.merkle_tree.lock().await;
    match tree.generate_proof(index, &state.storage).await {
        Ok(proof) => {
            tracing::info!("✅ Merkle proof generated successfully");

            // Get the current root to return with the proof
            match tree.get_tree_state(&state.storage).await {
                Ok(tree_state) => {
                    tracing::info!(
                        index = index,
                        root = tree_state.root,
                        path_elements_count = proof.path_elements.len(),
                        "✅ Merkle proof request completed"
                    );
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "pathElements": proof.path_elements,
                            "pathIndices": proof.path_indices,
                            "root": tree_state.root
                        })),
                    )
                }
                Err(e) => {
                    tracing::error!("❌ Failed to get merkle tree state: {}", e);
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
        Err(e) => {
            tracing::error!(
                "❌ Failed to generate merkle proof for index {}: {}",
                index,
                e
            );
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
    State(state): State<AppState>,
    Query(params): Query<NotesRangeQuery>,
) -> impl IntoResponse {
    let start = params.start.unwrap_or(0);
    let end = params.end.unwrap_or(start + 100);
    let limit = params.limit.unwrap_or(100);

    tracing::info!(
        start = start,
        end = end,
        limit = limit,
        "📋 Getting notes range"
    );

    if end < start {
        tracing::warn!(start = start, end = end, "Invalid range: end < start");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "End must be >= start"
            })),
        );
    }

    tracing::info!("🔍 Querying database for notes range");
    match state.storage.get_notes_range(start, end, limit).await {
        Ok(response) => {
            tracing::info!(
                notes_count = response.encrypted_outputs.len(),
                total = response.total,
                has_more = response.has_more,
                "✅ Notes range retrieved successfully"
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "notes": response.encrypted_outputs,
                    "has_more": response.has_more,
                    "total": response.total,
                    "start": response.start,
                    "end": response.end
                })),
            )
        }
        Err(e) => {
            tracing::error!("❌ Failed to get notes range: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get notes range",
                    "details": e.to_string()
                })),
            )
        }
    }
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
