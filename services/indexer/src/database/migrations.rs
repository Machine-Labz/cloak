use crate::error::{IndexerError, Result};
use sqlx::{Pool, Postgres, Row};

pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<()> {
    tracing::info!("Starting database migrations");

    // Create migrations tracking table
    create_migrations_table(pool).await?;

    // Get already applied migrations
    let applied_migrations = get_applied_migrations(pool).await?;
    tracing::info!(
        applied_count = applied_migrations.len(),
        "Found applied migrations"
    );

    // Define migrations
    let migrations = get_migrations();
    tracing::info!(total_migrations = migrations.len(), "Loaded migrations");

    // Apply pending migrations
    let mut applied_count = 0;
    for migration in migrations {
        if !applied_migrations.contains(&migration.id.to_string()) {
            apply_migration(pool, &migration).await?;
            applied_count += 1;
        } else {
            tracing::debug!(id = migration.id, "Migration already applied");
        }
    }

    if applied_count == 0 {
        tracing::info!("No new migrations to apply");
    } else {
        tracing::info!(applied = applied_count, "Database migrations completed");
    }

    Ok(())
}

struct Migration {
    id: &'static str,
    name: &'static str,
    sql: &'static str,
}

fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        id: "001_initial_schema",
        name: "Initial schema for Cloak Indexer",
        sql: include_str!("../migrations/001_initial_schema.sql"),
    }]
}

async fn create_migrations_table(pool: &Pool<Postgres>) -> Result<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            id VARCHAR(255) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
    "#;

    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(IndexerError::Database)?;

    tracing::debug!("Migrations table created or already exists");
    Ok(())
}

async fn get_applied_migrations(pool: &Pool<Postgres>) -> Result<Vec<String>> {
    let rows = sqlx::query("SELECT id FROM schema_migrations ORDER BY applied_at")
        .fetch_all(pool)
        .await
        .map_err(IndexerError::Database)?;

    let applied: Vec<String> = rows.into_iter().map(|row| row.get("id")).collect();
    Ok(applied)
}

async fn apply_migration(pool: &Pool<Postgres>, migration: &Migration) -> Result<()> {
    tracing::info!(
        id = migration.id,
        name = migration.name,
        "Applying migration"
    );

    // Start transaction
    let mut tx = pool.begin().await.map_err(IndexerError::Database)?;

    // Execute migration SQL
    sqlx::query(migration.sql)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(
                id = migration.id,
                error = %e,
                "Migration execution failed"
            );
            IndexerError::Database(e)
        })?;

    // Record migration as applied
    sqlx::query("INSERT INTO schema_migrations (id, name) VALUES ($1, $2)")
        .bind(migration.id)
        .bind(migration.name)
        .execute(&mut *tx)
        .await
        .map_err(IndexerError::Database)?;

    // Commit transaction
    tx.commit().await.map_err(IndexerError::Database)?;

    tracing::info!(id = migration.id, "Migration applied successfully");
    Ok(())
}

// Tests can be added here when needed
