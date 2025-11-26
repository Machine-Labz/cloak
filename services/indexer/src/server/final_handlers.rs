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
use base64::Engine;
use serde::Deserialize;
use solana_sdk::transaction::VersionedTransaction;
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
pub struct DepositPrepareRequest {
    pub tx_bytes_base64: String,
    pub leaf_commit: String,
    pub encrypted_output: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositConfirmRequest {
    pub prepared_deposit_id: String, // commitment hash
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
    // Returns (leaf_index, is_existing) where is_existing is true if commitment already existed
    tracing::info!("💾 Atomically allocating index and storing note");
    let (allocated_index, is_existing) = match state
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
        Ok((index, was_existing)) => {
            if was_existing {
                tracing::info!(
                    leaf_index = index,
                    "✅ Note already exists, returning existing leaf_index (idempotent)"
                );
            } else {
                tracing::info!(
                    leaf_index = index,
                    "✅ Note stored successfully with atomically allocated index"
                );
            }
            (index, was_existing)
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

    let mut tree = state.merkle_tree.lock().await;

    if is_existing {
        // Note already exists - just get the current root and return it
        tracing::info!("🌳 Note already in Merkle tree, retrieving current root");
        match tree.get_tree_state(&state.storage).await {
            Ok(tree_state) => {
                tracing::info!(
                    leaf_index = allocated_index,
                    root = tree_state.root,
                    "✅ Returning existing note data (idempotent)"
                );
                (
                    StatusCode::OK, // 200 OK for existing resource
                    Json(serde_json::json!({
                        "success": true,
                        "leafIndex": allocated_index,
                        "root": tree_state.root,
                        "nextIndex": tree_state.next_index,
                        "leafCommit": request.leaf_commit.to_lowercase(),
                        "message": "Deposit already exists (idempotent)"
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
    } else {
        // New note - insert into merkle tree
        tracing::info!("🌳 Inserting leaf into Merkle tree");
        match tree
            .insert_leaf(allocated_index as u64, &request.leaf_commit, &state.storage)
            .await
        {
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
}

/// Prepare a deposit: verify transaction, allocate index, process, push root
/// This ensures the root is on-chain BEFORE the user sends the transaction
pub async fn deposit_prepare(
    State(state): State<AppState>,
    Json(request): Json<DepositPrepareRequest>,
) -> impl IntoResponse {
    tracing::info!("📥 Received deposit prepare request");
    tracing::info!(
        leaf_commit = request.leaf_commit,
        encrypted_output_len = request.encrypted_output.len(),
        tx_bytes_len = request.tx_bytes_base64.len(),
        "Processing deposit prepare request"
    );

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

    // Verify transaction structure (optional but recommended)
    // Decode and parse the transaction to verify it's a valid deposit
    let tx_bytes = match base64::engine::general_purpose::STANDARD.decode(&request.tx_bytes_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!("Invalid base64 transaction: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid base64 transaction encoding"
                })),
            );
        }
    };

    // Try to deserialize as VersionedTransaction to verify structure
    let _tx: VersionedTransaction = match bincode::deserialize(&tx_bytes) {
        Ok(tx) => tx,
        Err(e) => {
            tracing::warn!("Invalid transaction structure: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid transaction structure"
                })),
            );
        }
    };

    // TODO: Optionally verify the transaction contains a deposit instruction
    // and extract commitment from instruction data to verify it matches request.leaf_commit
    // For now, we trust the client to provide the correct commitment

    // Atomically allocate next index and store the note as "pending"
    // Use "pending" as placeholder signature until transaction is confirmed
    tracing::info!("💾 Atomically allocating index and storing pending note");
    let (allocated_index, is_existing) = match state
        .storage
        .allocate_and_store_note(
            &request.leaf_commit,
            &request.encrypted_output,
            "pending", // Placeholder signature
            0,         // Placeholder slot
            Some(chrono::Utc::now()),
        )
        .await
    {
        Ok((index, was_existing)) => {
            if was_existing {
                tracing::info!(
                    leaf_index = index,
                    "✅ Note already exists, returning existing leaf_index (idempotent)"
                );
            } else {
                tracing::info!(
                    leaf_index = index,
                    "✅ Pending note stored successfully with atomically allocated index"
                );
            }
            (index, was_existing)
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

    let mut tree = state.merkle_tree.lock().await;

    if is_existing {
        // Note already exists - check if it's still pending
        if let Ok(Some(note)) = state.storage.get_note_by_commitment(&request.leaf_commit).await {
            if note.tx_signature == "pending" || note.tx_signature.is_empty() {
                // Still pending, return prepare response
                match tree.get_tree_state(&state.storage).await {
                    Ok(tree_state) => {
                        return (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "success": true,
                                "prepared_deposit_id": request.leaf_commit.to_lowercase(),
                                "leafIndex": allocated_index,
                                "root": tree_state.root,
                                "message": "Deposit already prepared (idempotent)"
                            })),
                        );
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to get merkle tree state: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({
                                "error": "Failed to get merkle tree state",
                                "details": e.to_string()
                            })),
                        );
                    }
                }
            } else {
                // Already confirmed
                match tree.get_tree_state(&state.storage).await {
                    Ok(tree_state) => {
                        return (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "success": true,
                                "prepared_deposit_id": request.leaf_commit.to_lowercase(),
                                "leafIndex": allocated_index,
                                "root": tree_state.root,
                                "message": "Deposit already confirmed"
                            })),
                        );
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to get merkle tree state: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({
                                "error": "Failed to get merkle tree state",
                                "details": e.to_string()
                            })),
                        );
                    }
                }
            }
        }
    }

    // New note - insert into merkle tree
    tracing::info!("🌳 Inserting leaf into Merkle tree");
    match tree
        .insert_leaf(allocated_index as u64, &request.leaf_commit, &state.storage)
        .await
    {
        Ok((new_root, _)) => {
            tracing::info!(
                leaf_index = allocated_index,
                new_root = new_root,
                "✅ Successfully inserted leaf into Merkle tree"
            );

            // Push new root to on-chain roots ring synchronously
            // This is critical - the root MUST be on-chain before the deposit transaction is sent
            tracing::info!("🔗 Pushing root to on-chain roots ring (CRITICAL: must succeed before deposit)");
            if let Err(e) = push_root_to_chain(&new_root, &state.config.solana).await {
                tracing::error!("❌ Failed to push root to on-chain roots ring: {}", e);
                // This is a critical failure - we cannot proceed if root push fails
                // The deposit transaction should not be sent if root is not on-chain
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to push root to on-chain roots ring",
                        "details": e.to_string(),
                        "message": "Deposit cannot proceed - root must be on-chain first"
                    })),
                );
            } else {
                tracing::info!("✅ Root successfully pushed to on-chain roots ring");
            }

            tracing::info!("🎉 Deposit prepare completed successfully");
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "success": true,
                    "prepared_deposit_id": request.leaf_commit.to_lowercase(),
                    "leafIndex": allocated_index,
                    "root": new_root,
                    "nextIndex": allocated_index + 1,
                    "message": "Deposit prepared successfully - root is on-chain, safe to send transaction"
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

/// Confirm a prepared deposit: update with transaction signature and slot
pub async fn deposit_confirm(
    State(state): State<AppState>,
    Json(request): Json<DepositConfirmRequest>,
) -> impl IntoResponse {
    tracing::info!("📥 Received deposit confirm request");
    tracing::info!(
        prepared_deposit_id = request.prepared_deposit_id,
        tx_signature = request.tx_signature,
        slot = request.slot,
        "Processing deposit confirm request"
    );

    // Validate inputs
    if request.prepared_deposit_id.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Prepared deposit ID (commitment) must be 64 characters"
            })),
        );
    }

    if request.tx_signature.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Transaction signature is required"
            })),
        );
    }

    // Update the pending deposit with the actual transaction signature
    tracing::info!("💾 Updating pending deposit with transaction signature");
    match state
        .storage
        .update_note_signature(
            &request.prepared_deposit_id,
            &request.tx_signature,
            request.slot,
        )
        .await
    {
        Ok(_) => {
            tracing::info!(
                prepared_deposit_id = request.prepared_deposit_id,
                tx_signature = request.tx_signature,
                "✅ Deposit confirmed successfully"
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Deposit confirmed successfully"
                })),
            )
        }
        Err(e) => {
            tracing::error!("❌ Failed to confirm deposit: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to confirm deposit",
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
