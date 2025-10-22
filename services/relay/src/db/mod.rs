pub mod models;
pub mod repository;

use crate::error::Error;
use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

pub type DatabasePool = PgPool;

/// Connect to PostgreSQL database
pub async fn connect(database_url: &str) -> Result<DatabasePool, Error> {
    info!("Connecting to database: {}", database_url);

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
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    info!("Database connection established");
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<(), Error> {
    info!("Running database migrations");

    let migration_sql = include_str!("../../migrations/001_init.sql");

    // Split by semicolon and execute each statement
    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() && !statement.starts_with("--") {
            sqlx::query(statement)
                .execute(pool)
                .await
                .map_err(|e| Error::DatabaseError(format!("Migration failed: {}", e)))?;
        }
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
