use crate::error::{IndexerError, Result};
use sqlx::{Acquire, Executor, Pool, Postgres, Row};

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
        sql: include_str!("../migrations/001_initial_schema_fixed.sql"),
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

    // Execute migration SQL using raw connection to handle multiple statements
    tracing::info!(
        id = migration.id,
        "Executing migration SQL using raw connection"
    );

    // Use raw connection to execute multiple statements
    let conn = tx.acquire().await.map_err(IndexerError::Database)?;
    conn.execute(migration.sql).await.map_err(|e| {
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

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_string = false;
    let mut string_char = '\0';
    let mut in_function = false;
    let mut dollar_quote = String::new();
    let mut i = 0;
    let chars: Vec<char> = sql.chars().collect();

    while i < chars.len() {
        let ch = chars[i];

        if !in_string && !in_function {
            // Check for dollar-quoted strings (used in PostgreSQL functions)
            if ch == '$' {
                let mut j = i + 1;
                let mut tag = String::new();
                while j < chars.len() && chars[j] != '$' {
                    tag.push(chars[j]);
                    j += 1;
                }
                if j < chars.len() {
                    dollar_quote = format!("${}$", tag);
                    in_function = true;
                    current_statement.push_str(&dollar_quote);
                    i = j;
                    continue;
                }
            } else if ch == '\'' || ch == '"' {
                in_string = true;
                string_char = ch;
            } else if ch == ';' {
                // Found statement terminator
                let trimmed = current_statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    statements.push(trimmed.to_string());
                }
                current_statement.clear();
                i += 1;
                continue;
            }
        } else if in_function {
            // Check for end of dollar-quoted string
            let dollar_chars: Vec<char> = dollar_quote.chars().collect();
            if chars[i..].starts_with(&dollar_chars) {
                current_statement.push_str(&dollar_quote);
                i += dollar_quote.len() - 1;
                in_function = false;
                dollar_quote.clear();
            }
        } else if in_string {
            if ch == string_char {
                // Check for escaped quotes
                if i + 1 < chars.len() && chars[i + 1] == string_char {
                    current_statement.push(ch);
                    current_statement.push(ch);
                    i += 2;
                    continue;
                } else {
                    in_string = false;
                }
            }
        }

        current_statement.push(ch);
        i += 1;
    }

    // Add the last statement if it's not empty
    let trimmed = current_statement.trim();
    if !trimmed.is_empty() && !trimmed.starts_with("--") {
        statements.push(trimmed.to_string());
    }

    statements
}

// Tests can be added here when needed
