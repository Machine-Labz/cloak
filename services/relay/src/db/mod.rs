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

    // Parse SQL statements, handling dollar-quoted strings
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_tag = String::new();
    
    for line in migration_sql.lines() {
        let trimmed = line.trim();
        
        // Skip comment-only lines when not in dollar quote
        if !in_dollar_quote && (trimmed.starts_with("--") || trimmed.is_empty()) {
            continue;
        }
        
        current_statement.push_str(line);
        current_statement.push('\n');
        
        // Check for dollar-quoted string delimiters
        if trimmed.contains("$$") {
            if !in_dollar_quote {
                // Starting a dollar-quoted block
                in_dollar_quote = true;
                dollar_tag = "$$".to_string();
            } else if trimmed.contains(&format!("END {}", dollar_tag)) {
                // Ending a dollar-quoted block
                in_dollar_quote = false;
                dollar_tag.clear();
            }
        }
        
        // If line ends with semicolon and we're not in a dollar-quoted block, statement is complete
        if !in_dollar_quote && trimmed.ends_with(';') {
            statements.push(current_statement.trim().to_string());
            current_statement.clear();
        }
    }
    
    // Execute each statement
    for statement in statements {
        if !statement.is_empty() {
            sqlx::query(&statement)
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
