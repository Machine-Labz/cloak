#!/usr/bin/env node

/**
 * Database migration script for Cloak Indexer
 * 
 * Runs SQL migrations to set up the database schema.
 * Can be run manually or as part of the Docker container initialization.
 */

import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { db } from '../db/connection.js';
import { logger } from '../lib/logger.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

interface Migration {
  id: string;
  filename: string;
  content: string;
}

async function loadMigrations(): Promise<Migration[]> {
  const migrationsDir = join(__dirname, '../db/migrations');
  
  try {
    // For now, we'll manually list migrations
    // In production, you might want to read the directory dynamically
    const migrationFiles = [
      '001_initial_schema.sql'
    ];

    const migrations: Migration[] = [];

    for (const filename of migrationFiles) {
      const filePath = join(migrationsDir, filename);
      const content = readFileSync(filePath, 'utf-8');
      
      migrations.push({
        id: filename.replace('.sql', ''),
        filename,
        content
      });
    }

    return migrations;
  } catch (error) {
    logger.error('Failed to load migrations', { error });
    throw error;
  }
}

async function createMigrationsTable(): Promise<void> {
  const sql = `
    CREATE TABLE IF NOT EXISTS schema_migrations (
      id VARCHAR(255) PRIMARY KEY,
      filename VARCHAR(255) NOT NULL,
      applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
  `;

  try {
    await db.query(sql);
    logger.info('Migrations table created or already exists');
  } catch (error) {
    logger.error('Failed to create migrations table', { error });
    throw error;
  }
}

async function getMigratedIds(): Promise<Set<string>> {
  try {
    const result = await db.query<{ id: string }>('SELECT id FROM schema_migrations ORDER BY applied_at');
    return new Set(result.rows.map(row => row.id));
  } catch (error) {
    logger.error('Failed to get migrated IDs', { error });
    throw error;
  }
}

async function applyMigration(migration: Migration): Promise<void> {
  try {
    logger.info('Applying migration', { id: migration.id, filename: migration.filename });
    
    await db.transaction(async (client) => {
      // Execute the migration SQL
      await client.query(migration.content);
      
      // Record that this migration was applied
      await client.query(
        'INSERT INTO schema_migrations (id, filename) VALUES ($1, $2)',
        [migration.id, migration.filename]
      );
    });
    
    logger.info('Migration applied successfully', { id: migration.id });
  } catch (error) {
    logger.error('Failed to apply migration', { 
      id: migration.id, 
      filename: migration.filename, 
      error 
    });
    throw error;
  }
}

async function runMigrations(): Promise<void> {
  try {
    logger.info('Starting database migrations');
    
    // Test database connection
    const isConnected = await db.testConnection();
    if (!isConnected) {
      throw new Error('Cannot connect to database');
    }

    // Create migrations tracking table
    await createMigrationsTable();

    // Load all available migrations
    const migrations = await loadMigrations();
    logger.info('Loaded migrations', { count: migrations.length });

    // Get already applied migrations
    const appliedMigrations = await getMigratedIds();
    logger.info('Found applied migrations', { count: appliedMigrations.size });

    // Apply pending migrations
    let appliedCount = 0;
    for (const migration of migrations) {
      if (!appliedMigrations.has(migration.id)) {
        await applyMigration(migration);
        appliedCount++;
      } else {
        logger.debug('Migration already applied', { id: migration.id });
      }
    }

    if (appliedCount === 0) {
      logger.info('No new migrations to apply');
    } else {
      logger.info('Database migrations completed', { applied: appliedCount });
    }

  } catch (error) {
    logger.error('Migration failed', { error });
    throw error;
  } finally {
    // Close database connection
    await db.close();
  }
}

// Run migrations if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runMigrations()
    .then(() => {
      logger.info('Migration script completed successfully');
      process.exit(0);
    })
    .catch((error) => {
      logger.error('Migration script failed', { error });
      process.exit(1);
    });
}
