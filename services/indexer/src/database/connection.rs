use crate::config::Config;
use crate::error::{IndexerError, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(config: &Config) -> Result<Self> {
        let database_url = config.database_url();
        
        tracing::info!(
            host = config.database.host,
            port = config.database.port,
            database = config.database.name,
            max_connections = config.database.max_connections,
            "Connecting to database"
        );

        let pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&database_url)
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to database: {}", e);
                IndexerError::Database(e)
            })?;

        // Test the connection
        let row: (chrono::DateTime<chrono::Utc>, String) = sqlx::query_as(
            "SELECT NOW() as current_time, version() as version"
        )
        .fetch_one(&pool)
        .await
        .map_err(IndexerError::Database)?;

        tracing::info!(
            current_time = %row.0,
            postgres_version = %row.1.split(' ').take(2).collect::<Vec<&str>>().join(" "),
            "Database connection established"
        );

        Ok(Database { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Test database connectivity
    pub async fn test_connection(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                tracing::debug!("Database connection test successful");
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Database connection test failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get connection pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            max_size: self.pool.size(),
        }
    }

    /// Execute a health check query
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        let start = std::time::Instant::now();
        
        // Test basic connectivity
        let time_result: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(&self.pool)
            .await
            .map_err(IndexerError::Database)?;

        // Check if required tables exist
        let table_check: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name IN ('merkle_tree_nodes', 'notes', 'indexer_metadata', 'artifacts')
            ORDER BY table_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let tables_found: Vec<String> = table_check.into_iter().map(|(name,)| name).collect();

        // Get basic statistics
        let stats: Option<(i64, i64, Option<String>)> = sqlx::query_as(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM merkle_tree_nodes) as tree_nodes,
                (SELECT COUNT(*) FROM notes) as notes_count,
                (SELECT value FROM indexer_metadata WHERE key = 'next_leaf_index') as next_index
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(IndexerError::Database)?;

        let duration = start.elapsed();

        Ok(HealthCheckResult {
            healthy: true,
            current_time: time_result.0,
            tables_found,
            stats: stats.map(|(tree_nodes, notes_count, next_index)| DatabaseStats {
                tree_nodes,
                notes_count,
                next_index: next_index.unwrap_or_else(|| "0".to_string()),
            }),
            pool_stats: self.pool_stats(),
            response_time_ms: duration.as_millis() as u64,
        })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        use crate::database::migrations::run_migrations;
        run_migrations(&self.pool).await
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        tracing::info!("Closing database connection pool");
        self.pool.close().await;
    }
}

#[derive(Debug, serde::Serialize)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub max_size: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct DatabaseStats {
    pub tree_nodes: i64,
    pub notes_count: i64,
    pub next_index: String,
}

#[derive(Debug, serde::Serialize)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub current_time: chrono::DateTime<chrono::Utc>,
    pub tables_found: Vec<String>,
    pub stats: Option<DatabaseStats>,
    pub pool_stats: PoolStats,
    pub response_time_ms: u64,
}

// Tests can be added here when needed
