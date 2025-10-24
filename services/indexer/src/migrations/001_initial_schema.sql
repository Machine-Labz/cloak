-- Initial schema for Cloak Indexer
-- Creates tables for Merkle tree storage, notes, and metadata

-- Extension for better UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Merkle tree nodes storage
-- Stores all nodes of the Merkle tree at different levels
CREATE TABLE merkle_tree_nodes (
    id BIGSERIAL PRIMARY KEY,
    level INTEGER NOT NULL, -- 0 = leaves, height-1 = root
    index_at_level BIGINT NOT NULL, -- Index within that level
    value CHAR(64) NOT NULL, -- 32-byte hash as hex string (64 chars)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    -- Ensure uniqueness of (level, index)
    UNIQUE(level, index_at_level)
);

-- Index for fast lookups
CREATE INDEX idx_merkle_tree_level_index ON merkle_tree_nodes (level, index_at_level);

CREATE INDEX idx_merkle_tree_level ON merkle_tree_nodes (level);

-- Notes table - stores encrypted outputs and metadata
CREATE TABLE notes (
    id BIGSERIAL PRIMARY KEY,
    leaf_commit CHAR(64) NOT NULL, -- 32-byte commitment as hex string
    encrypted_output TEXT NOT NULL, -- Base64 or hex encoded encrypted data
    leaf_index BIGINT NOT NULL, -- Position in Merkle tree
    tx_signature VARCHAR(88) NOT NULL, -- Solana transaction signature
    slot BIGINT NOT NULL, -- Solana slot number
    block_time TIMESTAMP WITH TIME ZONE, -- When the transaction was confirmed
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    -- Ensure uniqueness
    UNIQUE(leaf_commit),
    UNIQUE(leaf_index),
    UNIQUE(tx_signature)
);

-- Indexes for efficient queries
CREATE INDEX idx_notes_leaf_index ON notes (leaf_index);

CREATE INDEX idx_notes_tx_signature ON notes (tx_signature);

CREATE INDEX idx_notes_slot ON notes (slot);

CREATE INDEX idx_notes_created_at ON notes (created_at);

-- Metadata table for storing tree state and configuration
CREATE TABLE indexer_metadata (
    key VARCHAR(64) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Initialize metadata
INSERT INTO
    indexer_metadata (key, value)
VALUES ('next_leaf_index', '0'),
    ('tree_height', '32'),
    ('last_processed_slot', '0'),
    (
        'last_processed_signature',
        ''
    ),
    ('schema_version', '1');

-- Artifacts table for storing SP1 guest ELF and verification keys
CREATE TABLE artifacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    artifact_type VARCHAR(32) NOT NULL, -- 'guest_elf' or 'verification_key'
    version VARCHAR(32) NOT NULL, -- e.g., 'v2.0.0'
    file_path TEXT NOT NULL, -- Path to the artifact file
    sha256_hash CHAR(64) NOT NULL, -- SHA-256 hash of the file
    file_size BIGINT NOT NULL, -- File size in bytes
    sp1_version VARCHAR(32), -- SP1 version if applicable
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    -- Ensure uniqueness per type and version
    UNIQUE(artifact_type, version)
);

-- Index for artifact lookups
CREATE INDEX idx_artifacts_type_version ON artifacts (artifact_type, version);

CREATE INDEX idx_artifacts_sp1_version ON artifacts (sp1_version);

-- Event processing log to track Solana event ingestion
CREATE TABLE event_processing_log (
    id BIGSERIAL PRIMARY KEY,
    tx_signature VARCHAR(88) NOT NULL,
    slot BIGINT NOT NULL,
    event_type VARCHAR(32) NOT NULL, -- 'deposit', 'admin_push_root'
    processed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processing_status VARCHAR(16) DEFAULT 'success', -- 'success', 'failed', 'skipped'
    error_message TEXT,
    UNIQUE (tx_signature, event_type)
);

-- Index for event processing queries
CREATE INDEX idx_event_log_slot ON event_processing_log (slot);

CREATE INDEX idx_event_log_status ON event_processing_log (processing_status);

CREATE INDEX idx_event_log_processed_at ON event_processing_log (processed_at);

-- Function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers to automatically update updated_at
CREATE TRIGGER update_merkle_tree_nodes_updated_at 
    BEFORE UPDATE ON merkle_tree_nodes 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_indexer_metadata_updated_at 
    BEFORE UPDATE ON indexer_metadata 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE merkle_tree_nodes IS 'Stores all nodes of the append-only Merkle tree';

COMMENT ON TABLE notes IS 'Stores encrypted note outputs with metadata from deposit transactions';

COMMENT ON TABLE indexer_metadata IS 'Key-value store for indexer state and configuration';

COMMENT ON TABLE artifacts IS 'Stores SP1 guest ELF files and verification keys with hashes';

COMMENT ON TABLE event_processing_log IS 'Tracks processing of Solana blockchain events';

COMMENT ON COLUMN merkle_tree_nodes.level IS '0 = leaves, height-1 = root level';

COMMENT ON COLUMN merkle_tree_nodes.index_at_level IS 'Index within the specified level';

COMMENT ON COLUMN merkle_tree_nodes.value IS '32-byte hash as 64-character hex string';

COMMENT ON COLUMN notes.leaf_commit IS '32-byte commitment hash as 64-character hex string';

COMMENT ON COLUMN notes.encrypted_output IS 'Encrypted note output (base64 or hex encoded)';

COMMENT ON COLUMN artifacts.sha256_hash IS 'SHA-256 hash of the artifact file (64 hex chars)';

-- Grant permissions (adjust as needed for your setup)
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO cloak;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO cloak;