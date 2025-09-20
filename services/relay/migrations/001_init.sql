-- Initial schema for relay service

-- Job status enum
CREATE TYPE job_status AS ENUM ('queued', 'processing', 'completed', 'failed', 'cancelled');

-- Jobs table for withdraw requests
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL UNIQUE,
    status job_status NOT NULL DEFAULT 'queued',
    
    -- Request data
    proof_bytes BYTEA NOT NULL,
    public_inputs BYTEA NOT NULL,
    outputs_json JSONB NOT NULL,
    fee_bps SMALLINT NOT NULL,
    
    -- Extracted public inputs for indexing
    root_hash BYTEA NOT NULL,
    nullifier BYTEA NOT NULL,
    amount BIGINT NOT NULL,
    outputs_hash BYTEA NOT NULL,
    
    -- Processing results
    tx_id TEXT,
    solana_signature TEXT,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Nullifiers table for double-spend prevention
CREATE TABLE nullifiers (
    nullifier BYTEA PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id),
    block_height BIGINT,
    tx_signature TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at);
CREATE INDEX idx_jobs_request_id ON jobs(request_id);
CREATE INDEX idx_jobs_nullifier ON jobs(nullifier);
CREATE INDEX idx_nullifiers_created_at ON nullifiers(created_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to automatically update updated_at
CREATE TRIGGER update_jobs_updated_at BEFORE UPDATE ON jobs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column(); 