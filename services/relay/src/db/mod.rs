pub mod models;
pub mod repository;

use sqlx::{
    migrate::MigrateDatabase,
    postgres::{PgPool, PgPoolOptions},
};
use tracing::info;

use crate::error::Error;

pub type DatabasePool = PgPool;

/// Connect to PostgreSQL database
pub async fn connect(database_url: &str, max_connections: u32) -> Result<DatabasePool, Error> {
    info!("Connecting to database");

    // Create database if it doesn't exist
    if !sqlx::Postgres::database_exists(database_url)
        .await
        .unwrap_or(false)
    {
        info!("Database doesn't exist, creating it...");
        sqlx::Postgres::create_database(database_url)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to create database: {}", e)))?;
    }

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    info!("Database connection established");
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<(), Error> {
    info!("Running database migrations");

    // Split migration into statements that can be executed individually
    // This approach works around sqlx's limitation with multiple statements

    // First statement: Create enum type
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE job_status AS ENUM ('queued', 'processing', 'completed', 'failed', 'cancelled');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    ).execute(pool).await
    .map_err(|e| Error::DatabaseError(format!("Failed to create enum: {}", e)))?;

    // Second statement: Create jobs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            request_id UUID NOT NULL UNIQUE,
            status job_status NOT NULL DEFAULT 'queued',
            proof_bytes BYTEA NOT NULL,
            public_inputs BYTEA NOT NULL,
            outputs_json JSONB NOT NULL,
            fee_bps SMALLINT NOT NULL,
            root_hash BYTEA NOT NULL,
            nullifier BYTEA NOT NULL,
            amount BIGINT NOT NULL,
            outputs_hash BYTEA NOT NULL,
            tx_id TEXT,
            solana_signature TEXT,
            error_message TEXT,
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_retries INTEGER NOT NULL DEFAULT 3,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            started_at TIMESTAMPTZ,
            completed_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::DatabaseError(format!("Failed to create jobs table: {}", e)))?;

    // Third statement: Create nullifiers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS nullifiers (
            nullifier BYTEA PRIMARY KEY,
            job_id UUID NOT NULL REFERENCES jobs(id),
            block_height BIGINT,
            tx_signature TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::DatabaseError(format!("Failed to create nullifiers table: {}", e)))?;

    // Create indexes
    for index_sql in &[
        "CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status)",
        "CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_jobs_request_id ON jobs(request_id)",
        "CREATE INDEX IF NOT EXISTS idx_nullifiers_created_at ON nullifiers(created_at)",
    ] {
        sqlx::query(index_sql)
            .execute(pool)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to create index: {}", e)))?;
    }

    info!("Database migrations completed");
    Ok(())
}

/// Check database health
pub async fn health_check(pool: &DatabasePool) -> Result<(), Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Database health check failed: {}", e)))?;
    Ok(())
}
