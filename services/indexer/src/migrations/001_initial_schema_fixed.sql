-- Extension for better UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Merkle tree nodes storage
CREATE TABLE merkle_tree_nodes (
    id BIGSERIAL PRIMARY KEY,
    level INTEGER NOT NULL,
    index_at_level BIGINT NOT NULL,
    value CHAR(64) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(level, index_at_level)
);

-- Indexes for merkle tree nodes
CREATE INDEX idx_merkle_tree_level_index ON merkle_tree_nodes (level, index_at_level);
CREATE INDEX idx_merkle_tree_level ON merkle_tree_nodes (level);

-- Notes table
CREATE TABLE notes (
    id BIGSERIAL PRIMARY KEY,
    leaf_commit CHAR(64) NOT NULL,
    encrypted_output TEXT NOT NULL,
    leaf_index BIGINT NOT NULL,
    tx_signature VARCHAR(88) NOT NULL,
    slot BIGINT NOT NULL,
    block_time TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(leaf_commit),
    UNIQUE(leaf_index),
    UNIQUE(tx_signature)
);

-- Indexes for notes
CREATE INDEX idx_notes_leaf_index ON notes (leaf_index);
CREATE INDEX idx_notes_tx_signature ON notes (tx_signature);
CREATE INDEX idx_notes_slot ON notes (slot);
CREATE INDEX idx_notes_created_at ON notes (created_at);

-- Metadata table
CREATE TABLE indexer_metadata (
    key VARCHAR(64) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Initialize metadata
INSERT INTO indexer_metadata (key, value) VALUES 
    ('next_leaf_index', '0'),
    ('tree_height', '32'),
    ('last_processed_slot', '0'),
    ('last_processed_signature', ''),
    ('schema_version', '1');

-- Artifacts table
CREATE TABLE artifacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    artifact_type VARCHAR(32) NOT NULL,
    version VARCHAR(32) NOT NULL,
    file_path TEXT NOT NULL,
    sha256_hash CHAR(64) NOT NULL,
    file_size BIGINT NOT NULL,
    sp1_version VARCHAR(32),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(artifact_type, version)
);

-- Indexes for artifacts
CREATE INDEX idx_artifacts_type_version ON artifacts (artifact_type, version);
CREATE INDEX idx_artifacts_sp1_version ON artifacts (sp1_version);

-- Event processing log table
CREATE TABLE event_processing_log (
    id BIGSERIAL PRIMARY KEY,
    tx_signature VARCHAR(88) NOT NULL,
    slot BIGINT NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processing_status VARCHAR(16) DEFAULT 'success',
    error_message TEXT,
    UNIQUE (tx_signature, event_type)
);

-- Indexes for event processing log
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

-- Triggers
CREATE TRIGGER update_merkle_tree_nodes_updated_at 
    BEFORE UPDATE ON merkle_tree_nodes 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_indexer_metadata_updated_at 
    BEFORE UPDATE ON indexer_metadata 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();


