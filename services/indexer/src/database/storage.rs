use crate::error::{IndexerError, Result};
use crate::merkle::TreeStorage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredNote {
    pub id: i64,
    pub leaf_commit: String,
    pub encrypted_output: String,
    pub leaf_index: i64,
    pub tx_signature: String,
    pub slot: i64,
    pub block_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotesRangeResponse {
    pub encrypted_outputs: Vec<String>,
    pub has_more: bool,
    pub total: i64,
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MerkleTreeRow {
    pub level: i32,
    pub index_at_level: i64,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PostgresTreeStorage {
    pool: Pool<Postgres>,
}

impl PostgresTreeStorage {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Atomically allocate the next leaf index and store the note
    /// This prevents race conditions when multiple deposits arrive concurrently
    pub async fn allocate_and_store_note(
        &self,
        leaf_commit: &str,
        encrypted_output: &str,
        tx_signature: &str,
        slot: i64,
        block_time: Option<DateTime<Utc>>,
    ) -> Result<i64> {
        let clean_commit = leaf_commit
            .strip_prefix("0x")
            .unwrap_or(leaf_commit)
            .to_lowercase();

        let start = std::time::Instant::now();

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            IndexerError::Database(e)
        })?;

        // Atomically allocate next index using PostgreSQL sequence
        // This is guaranteed to be unique across all concurrent transactions
        let next_index: i64 = sqlx::query_scalar("SELECT nextval('leaf_index_seq')")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to allocate leaf index from sequence: {}", e);
                IndexerError::Database(e)
            })?;

        tracing::info!(
            next_index = next_index,
            "Allocated leaf index from sequence (atomic, no race conditions)"
        );

        // Reserve the index in merkle_tree_nodes to maintain consistency
        // This prevents the merkle tree from being out of sync with allocated indices
        // The actual leaf hash will be computed and updated by insert_leaf()
        sqlx::query(
            r#"
            INSERT INTO merkle_tree_nodes (level, index_at_level, value)
            VALUES (0, $1, '0000000000000000000000000000000000000000000000000000000000000000')
            ON CONFLICT (level, index_at_level)
            DO UPDATE SET value = EXCLUDED.value
            "#
        )
        .bind(next_index)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(
                index = next_index,
                error = %e,
                "Failed to reserve merkle tree node"
            );
            IndexerError::Database(e)
        })?;

        tracing::info!(
            next_index = next_index,
            "Reserved merkle tree node (will be updated with actual leaf hash)"
        );

        // Insert the note with the allocated index
        sqlx::query(
            r#"
            INSERT INTO notes (leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time) 
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(&clean_commit)
        .bind(encrypted_output)
        .bind(next_index)
        .bind(tx_signature)
        .bind(slot)
        .bind(block_time.unwrap_or_else(Utc::now))
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(
                leaf_commit = %clean_commit,
                leaf_index = next_index,
                tx_signature = tx_signature,
                slot = slot,
                error = %e,
                "Failed to store note"
            );
            IndexerError::Database(e)
        })?;

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            IndexerError::Database(e)
        })?;

        let duration = start.elapsed();
        crate::log_database_operation!("INSERT", "notes", duration.as_millis() as u64);

        tracing::info!(
            leaf_commit = %clean_commit,
            leaf_index = next_index,
            tx_signature = tx_signature,
            slot = slot,
            encrypted_output_length = encrypted_output.len(),
            "Stored note with atomically allocated index"
        );

        Ok(next_index)
    }

    /// Reset the database by clearing all data
    pub async fn reset_database(&self) -> Result<()> {
        tracing::info!("Resetting database - clearing all data...");

        let start = std::time::Instant::now();

        // Clear all tables in the correct order (respecting foreign key constraints)
        sqlx::query("DELETE FROM notes")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to clear notes table: {}", e);
                IndexerError::Database(e)
            })?;

        sqlx::query("DELETE FROM merkle_tree_nodes")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to clear merkle_tree_nodes table: {}", e);
                IndexerError::Database(e)
            })?;

        sqlx::query("DELETE FROM indexer_metadata")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to clear indexer_metadata table: {}", e);
                IndexerError::Database(e)
            })?;

        let duration = start.elapsed();
        crate::log_database_operation!("RESET", "all_tables", duration.as_millis() as u64);

        tracing::info!("Database reset completed successfully");
        Ok(())
    }

    /// Get notes in a range with pagination
    pub async fn get_notes_range(
        &self,
        start: i64,
        end: i64,
        limit: i64,
    ) -> Result<NotesRangeResponse> {
        if start < 0 || end < start {
            return Err(IndexerError::bad_request(
                "Invalid range: start must be >= 0 and end must be >= start",
            ));
        }

        let limit = limit.min(1000); // Cap limit to prevent excessive memory usage

        let query_start = std::time::Instant::now();

        // Get total count in range
        let count_row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notes WHERE leaf_index >= $1 AND leaf_index <= $2",
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let total = count_row.0;

        // Get the actual notes
        let notes: Vec<StoredNote> = sqlx::query_as(
            r#"
            SELECT id, leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time, created_at 
            FROM notes 
            WHERE leaf_index >= $1 AND leaf_index <= $2
            ORDER BY leaf_index ASC 
            LIMIT $3
            "#
        )
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let encrypted_outputs: Vec<String> = notes
            .into_iter()
            .map(|note| note.encrypted_output)
            .collect();
        let has_more = total > limit;

        let duration = query_start.elapsed();
        crate::log_database_operation!("SELECT", "notes", duration.as_millis() as u64);

        tracing::debug!(
            start = start,
            end = end,
            limit = limit,
            total = total,
            returned = encrypted_outputs.len(),
            has_more = has_more,
            "Retrieved notes range"
        );

        Ok(NotesRangeResponse {
            encrypted_outputs,
            has_more,
            total,
            start,
            end,
        })
    }

    /// Get a specific note by leaf index
    pub async fn get_note_by_index(&self, leaf_index: i64) -> Result<Option<StoredNote>> {
        let start = std::time::Instant::now();

        let note = sqlx::query_as::<_, StoredNote>(
            r#"
            SELECT id, leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time, created_at 
            FROM notes 
            WHERE leaf_index = $1
            "#
        )
        .bind(leaf_index)
        .fetch_optional(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!("SELECT", "notes", duration.as_millis() as u64);

        Ok(note)
    }

    /// Get a specific note by leaf commit
    pub async fn get_note_by_commit(&self, leaf_commit: &str) -> Result<Option<StoredNote>> {
        let clean_commit = leaf_commit
            .strip_prefix("0x")
            .unwrap_or(leaf_commit)
            .to_lowercase();

        let start = std::time::Instant::now();

        let note = sqlx::query_as::<_, StoredNote>(
            r#"
            SELECT id, leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time, created_at 
            FROM notes 
            WHERE leaf_commit = $1
            "#
        )
        .bind(&clean_commit)
        .fetch_optional(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!("SELECT", "notes", duration.as_millis() as u64);

        Ok(note)
    }

    /// Update indexer metadata
    pub async fn update_metadata(&self, key: &str, value: &str) -> Result<()> {
        let start = std::time::Instant::now();

        sqlx::query(
            r#"
            INSERT INTO indexer_metadata (key, value) 
            VALUES ($1, $2)
            ON CONFLICT (key) 
            DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!("UPSERT", "indexer_metadata", duration.as_millis() as u64);

        tracing::debug!(key = key, value = value, "Updated metadata");
        Ok(())
    }

    /// Get indexer metadata value
    pub async fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let start = std::time::Instant::now();

        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM indexer_metadata WHERE key = $1")
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!("SELECT", "indexer_metadata", duration.as_millis() as u64);

        Ok(row.map(|(value,)| value))
    }

    /// Log event processing
    pub async fn log_event_processing(
        &self,
        tx_signature: &str,
        slot: i64,
        event_type: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        let start = std::time::Instant::now();

        sqlx::query(
            r#"
            INSERT INTO event_processing_log (tx_signature, slot, event_type, processing_status, error_message) 
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tx_signature, event_type) 
            DO UPDATE SET 
                processing_status = EXCLUDED.processing_status,
                error_message = EXCLUDED.error_message,
                processed_at = NOW()
            "#
        )
        .bind(tx_signature)
        .bind(slot)
        .bind(event_type)
        .bind(status)
        .bind(error_message)
        .execute(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!(
            "UPSERT",
            "event_processing_log",
            duration.as_millis() as u64
        );

        tracing::debug!(
            tx_signature = tx_signature,
            slot = slot,
            event_type = event_type,
            status = status,
            "Logged event processing"
        );

        Ok(())
    }

    /// Get all tree nodes for a specific level
    pub async fn get_nodes_at_level(&self, level: u32) -> Result<Vec<MerkleTreeRow>> {
        let start = std::time::Instant::now();

        let nodes = sqlx::query_as::<_, MerkleTreeRow>(
            r#"
            SELECT level, index_at_level, value, created_at, updated_at 
            FROM merkle_tree_nodes 
            WHERE level = $1 
            ORDER BY index_at_level
            "#,
        )
        .bind(level as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!("SELECT", "merkle_tree_nodes", duration.as_millis() as u64);

        Ok(nodes)
    }

    /// Health check - verify database connectivity and basic operations
    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();

        // Test basic connectivity
        let time_row: (DateTime<Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(&self.pool)
            .await
            .map_err(IndexerError::Database)?;

        // Test table accessibility
        let tables: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name IN ('merkle_tree_nodes', 'notes', 'indexer_metadata')
            ORDER BY table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let tables_found: Vec<String> = tables.into_iter().map(|(name,)| name).collect();

        // Get basic stats
        let stats_row: Option<(i64, i64, Option<String>)> = sqlx::query_as(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM merkle_tree_nodes) as tree_nodes,
                (SELECT COUNT(*) FROM notes) as notes_count,
                (SELECT value FROM indexer_metadata WHERE key = 'next_leaf_index') as next_index
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();

        let details = serde_json::json!({
            "healthy": true,
            "current_time": time_row.0,
            "tables_found": tables_found,
            "stats": stats_row.map(|(tree_nodes, notes_count, next_index)| {
                serde_json::json!({
                    "tree_nodes": tree_nodes,
                    "notes_count": notes_count,
                    "next_index": next_index.unwrap_or_else(|| "0".to_string())
                })
            }),
            "response_time_ms": duration.as_millis()
        });

        Ok(details)
    }
}

#[async_trait::async_trait]
impl TreeStorage for PostgresTreeStorage {
    async fn store_node(&self, level: u32, index: u64, value: &str) -> Result<()> {
        let clean_value = value.strip_prefix("0x").unwrap_or(value).to_lowercase();

        if clean_value.len() != 64 {
            return Err(IndexerError::bad_request(format!(
                "Invalid node value length: {} (expected 64 hex chars)",
                clean_value.len()
            )));
        }

        let start = std::time::Instant::now();

        sqlx::query(
            r#"
            INSERT INTO merkle_tree_nodes (level, index_at_level, value) 
            VALUES ($1, $2, $3)
            ON CONFLICT (level, index_at_level) 
            DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()
            "#,
        )
        .bind(level as i32)
        .bind(index as i64)
        .bind(&clean_value)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(
                level = level,
                index = index,
                value = %clean_value,
                error = %e,
                "Failed to store tree node"
            );
            IndexerError::Database(e)
        })?;

        let duration = start.elapsed();
        crate::log_database_operation!("UPSERT", "merkle_tree_nodes", duration.as_millis() as u64);

        tracing::debug!(
            level = level,
            index = index,
            value = %clean_value,
            "Stored tree node"
        );

        Ok(())
    }

    async fn get_node(&self, level: u32, index: u64) -> Result<Option<String>> {
        let start = std::time::Instant::now();

        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM merkle_tree_nodes WHERE level = $1 AND index_at_level = $2",
        )
        .bind(level as i32)
        .bind(index as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();
        crate::log_database_operation!("SELECT", "merkle_tree_nodes", duration.as_millis() as u64);

        if let Some((value,)) = row {
            tracing::debug!(
                level = level,
                index = index,
                value = %value,
                "Retrieved tree node"
            );
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn get_max_leaf_index(&self) -> Result<u64> {
        let start = std::time::Instant::now();

        // Get the current value of the SEQUENCE
        // This is the source of truth for next index allocation
        let sequence_value: Option<i64> = sqlx::query_scalar(
            "SELECT last_value FROM leaf_index_seq",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        // If sequence doesn't exist yet or is at initial value, check notes table as fallback
        let next_index = if let Some(seq_val) = sequence_value {
            // Sequence exists, next index is current value + 1
            // (last_value is the last allocated value, so next is +1)
            seq_val + 1
        } else {
            // Fallback: query notes table for max leaf_index
            let max_index: Option<i64> = sqlx::query_scalar(
                "SELECT COALESCE(MAX(leaf_index), -1) FROM notes",
            )
            .fetch_one(&self.pool)
            .await
            .map_err(IndexerError::Database)?;

            max_index.unwrap_or(-1) + 1
        };

        let duration = start.elapsed();
        crate::log_database_operation!("SELECT", "leaf_index_seq", duration.as_millis() as u64);

        tracing::info!(
            sequence_value = sequence_value,
            next_index = next_index,
            "Retrieved max leaf index from SEQUENCE"
        );

        Ok(next_index as u64)
    }
}

// Tests can be added here when needed
