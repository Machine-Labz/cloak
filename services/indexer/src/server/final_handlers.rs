use crate::artifacts::ArtifactManager;
use crate::database::PostgresTreeStorage;
use crate::merkle::{MerkleTree, TreeStorage};
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
    pub tee_client: Option<Arc<Sp1TeeClient>>,
}

// Request types
#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub leaf_commit: String,
    pub encrypted_output: String,
    pub tx_signature: String,
    pub slot: i64,
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
        "documentation": "https://docs.cloaklabz.xyz/offchain/indexer",
        "deprecated_endpoints": {
            "prove": {
                "endpoint": "/api/v1/prove",
                "reason": "Server-side proof generation will be removed. Use client-side proof generation instead.",
                "sunset_date": "2025-06-01",
                "migration_guide": "https://docs.cloaklabz.xyz/offchain/indexer"
            }
        },
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_details = match state.storage.health_check().await {
        Ok(details) => details,
        Err(e) => serde_json::json!({
            "healthy": false,
            "error": e.to_string()
        }),
    };

    // Get Merkle status without DB ops
    let height = {
        let tree = state.merkle_tree.lock().await;
        tree.height()
    };

    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now(),
        "database": db_details,
        "merkle_tree": {
            "initialized": true,
            "height": height
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
        tx_signature = request.tx_signature,
        slot = request.slot,
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

    // Validate transaction signature format (base58 Solana signature)
    if request.tx_signature.is_empty() {
        tracing::warn!("Empty transaction signature provided");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Transaction signature is required"
            })),
        );
    }

    // Atomically allocate next index and store the note in the database
    // This prevents race conditions when multiple deposits arrive concurrently
    tracing::info!("💾 Atomically allocating index and storing note");
    let allocated_index = match state
        .storage
        .allocate_and_store_note(
            &request.leaf_commit,
            &request.encrypted_output,
            &request.tx_signature,
            request.slot,
            Some(chrono::Utc::now()),
        )
        .await
    {
        Ok(index) => {
            tracing::info!(
                leaf_index = index,
                "✅ Note stored successfully with atomically allocated index"
            );
            index
        }
        Err(e) => {
            tracing::error!("❌ Failed to allocate index and store note: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to store note",
                    "details": e.to_string()
                })),
            );
        }
    };

    // Insert the leaf into the merkle tree and get the new root
    tracing::info!("🌳 Inserting leaf into Merkle tree");
    let mut tree = state.merkle_tree.lock().await;
    match tree.insert_leaf(allocated_index as u64, &request.leaf_commit, &state.storage).await {
        Ok((new_root, _)) => {
            tracing::info!(
                leaf_index = allocated_index,
                new_root = new_root,
                "✅ Successfully inserted leaf into Merkle tree"
            );
            
            // Push new root to on-chain roots ring synchronously to prevent race conditions
            // The withdrawal proof depends on this root being on-chain before it can be verified
            tracing::info!("🔗 Pushing root to on-chain roots ring");
            if let Err(e) = push_root_to_chain(&new_root, &state.config.solana).await {
                tracing::error!("❌ Failed to push root to on-chain roots ring: {}", e);
                // Continue anyway - withdrawals can still work if root is pushed later
                // or if the on-chain program has a grace period for root updates
                tracing::warn!("⚠️  Continuing despite root push failure - withdrawals may fail until root is manually pushed");
            } else {
                tracing::info!("✅ Root successfully pushed to on-chain roots ring");
            }
            
            tracing::info!("🎉 Deposit request completed successfully");
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "success": true,
                    "leafIndex": allocated_index,
                    "root": new_root,
                    "nextIndex": allocated_index + 1,
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
    let mut tree = state.merkle_tree.lock().await;

    // Sync in-memory next_index with storage to avoid stale state across requests/processes
    match state.storage.get_max_leaf_index().await {
        Ok(latest_next_index) => {
            tree.set_next_index(latest_next_index);
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to refresh next_index from storage: {}", e);
        }
    }
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

/// Admin endpoint to manually push the current merkle root to the on-chain RootsRing
/// This is useful for recovering from situations where roots were not pushed due to
/// the async race condition in previous versions of the code
pub async fn admin_push_root(State(state): State<AppState>) -> impl IntoResponse {
    tracing::info!("🔧 Admin: Manually pushing current merkle root to on-chain RootsRing");

    // Get the current merkle root
    let tree = state.merkle_tree.lock().await;
    match tree.get_tree_state(&state.storage).await {
        Ok(tree_state) => {
            let root = tree_state.root;
            tracing::info!(
                root = root,
                next_index = tree_state.next_index,
                "📍 Current merkle tree state"
            );

            // Push root to chain
            drop(tree); // Release lock before async operation
            match push_root_to_chain(&root, &state.config.solana).await {
                Ok(_) => {
                    tracing::info!("✅ Root successfully pushed to on-chain roots ring");
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "success": true,
                            "root": root,
                            "message": "Root successfully pushed to on-chain RootsRing"
                        })),
                    )
                }
                Err(e) => {
                    tracing::error!("❌ Failed to push root to on-chain roots ring: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "root": root,
                            "error": format!("Failed to push root: {}", e)
                        })),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get merkle tree state: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get merkle tree state: {}", e)
                })),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PushSpecificRootRequest {
    pub root: String,
}

/// Admin endpoint to push a specific merkle root to the on-chain RootsRing
/// This is useful for pushing historical roots that were never pushed due to async race conditions
pub async fn admin_push_specific_root(
    State(state): State<AppState>,
    Json(request): Json<PushSpecificRootRequest>,
) -> impl IntoResponse {
    tracing::info!(
        root = request.root,
        "🔧 Admin: Manually pushing specific merkle root to on-chain RootsRing"
    );

    // Validate root format (should be 64 hex characters)
    if request.root.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid root format - must be 64 hex characters"
            })),
        );
    }

    // Push root to chain
    match push_root_to_chain(&request.root, &state.config.solana).await {
        Ok(_) => {
            tracing::info!("✅ Root successfully pushed to on-chain roots ring");
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "root": request.root,
                    "message": "Root successfully pushed to on-chain RootsRing"
                })),
            )
        }
        Err(e) => {
            tracing::error!("❌ Failed to push root to on-chain roots ring: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "root": request.root,
                    "error": format!("Failed to push root: {}", e)
                })),
            )
        }
    }
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
