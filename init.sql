-- Database initialization script for Docker
-- Creates the single shared database for indexer and relay services
-- with the complete schema
--
-- This file is automatically executed by the postgres container
-- when the container starts for the first time

-- Create the unified database
CREATE DATABASE cloak;

-- Connect to the cloak database
\c cloak;

-- Set timezone to UTC
ALTER DATABASE cloak SET timezone TO 'UTC';

-- Create extension for UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- Create job_status enum type (for relay service)
DO $$ BEGIN
    CREATE TYPE public.job_status AS ENUM (
        'queued',
        'processing',
        'completed',
        'failed',
        'cancelled'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

-- ========================================
-- TABLES FOR INDEXER SERVICE
-- ========================================

-- Merkle tree nodes storage
CREATE SEQUENCE IF NOT EXISTS public.merkle_tree_nodes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE IF NOT EXISTS public.merkle_tree_nodes (
    id bigint NOT NULL DEFAULT nextval('public.merkle_tree_nodes_id_seq'::regclass),
    level integer NOT NULL,
    index_at_level bigint NOT NULL,
    value character(64) NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id),
    UNIQUE (level, index_at_level)
);

CREATE INDEX IF NOT EXISTS idx_merkle_tree_level_index ON public.merkle_tree_nodes USING btree (level, index_at_level);
CREATE INDEX IF NOT EXISTS idx_merkle_tree_level ON public.merkle_tree_nodes USING btree (level);

-- Notes table
CREATE SEQUENCE IF NOT EXISTS public.notes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE IF NOT EXISTS public.notes (
    id bigint NOT NULL DEFAULT nextval('public.notes_id_seq'::regclass),
    leaf_commit character(64) NOT NULL,
    encrypted_output text NOT NULL,
    leaf_index bigint NOT NULL,
    tx_signature character varying(88) NOT NULL,
    slot bigint NOT NULL,
    block_time timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id),
    UNIQUE (leaf_commit),
    UNIQUE (leaf_index),
    UNIQUE (tx_signature)
);

CREATE INDEX IF NOT EXISTS idx_notes_leaf_index ON public.notes USING btree (leaf_index);
CREATE INDEX IF NOT EXISTS idx_notes_tx_signature ON public.notes USING btree (tx_signature);
CREATE INDEX IF NOT EXISTS idx_notes_slot ON public.notes USING btree (slot);
CREATE INDEX IF NOT EXISTS idx_notes_created_at ON public.notes USING btree (created_at);

-- Metadata table
CREATE TABLE IF NOT EXISTS public.indexer_metadata (
    key character varying(64) NOT NULL,
    value text NOT NULL,
    updated_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (key)
);

-- Initialize metadata (only if not exists)
INSERT INTO public.indexer_metadata (key, value)
VALUES
    ('next_leaf_index', '0'),
    ('tree_height', '32'),
    ('last_processed_slot', '0'),
    ('last_processed_signature', ''),
    ('schema_version', '1')
ON CONFLICT (key) DO NOTHING;

-- Artifacts table
CREATE TABLE IF NOT EXISTS public.artifacts (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    artifact_type character varying(32) NOT NULL,
    version character varying(32) NOT NULL,
    file_path text NOT NULL,
    sha256_hash character(64) NOT NULL,
    file_size bigint NOT NULL,
    sp1_version character varying(32),
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id),
    UNIQUE (artifact_type, version)
);

CREATE INDEX IF NOT EXISTS idx_artifacts_type_version ON public.artifacts USING btree (artifact_type, version);
CREATE INDEX IF NOT EXISTS idx_artifacts_sp1_version ON public.artifacts USING btree (sp1_version);

-- Event processing log table
CREATE SEQUENCE IF NOT EXISTS public.event_processing_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE IF NOT EXISTS public.event_processing_log (
    id bigint NOT NULL DEFAULT nextval('public.event_processing_log_id_seq'::regclass),
    tx_signature character varying(88) NOT NULL,
    slot bigint NOT NULL,
    event_type character varying(32) NOT NULL,
    processed_at timestamp with time zone DEFAULT now(),
    processing_status character varying(16) DEFAULT 'success'::character varying,
    error_message text,
    PRIMARY KEY (id),
    UNIQUE (tx_signature, event_type)
);

CREATE INDEX IF NOT EXISTS idx_event_log_slot ON public.event_processing_log USING btree (slot);
CREATE INDEX IF NOT EXISTS idx_event_log_status ON public.event_processing_log USING btree (processing_status);
CREATE INDEX IF NOT EXISTS idx_event_log_processed_at ON public.event_processing_log USING btree (processed_at);

-- Schema migrations table
CREATE TABLE IF NOT EXISTS public.schema_migrations (
    id character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    applied_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

-- Mark existing migrations as applied (since init.sql creates the schema)
-- This prevents the indexer from trying to run migrations when tables already exist
INSERT INTO public.schema_migrations (id, name)
VALUES
    ('001_initial_schema', 'Initial schema for Cloak Indexer')
ON CONFLICT (id) DO NOTHING;

-- ========================================
-- TABLES FOR RELAY SERVICE
-- ========================================

-- Jobs table for withdraw requests
CREATE TABLE IF NOT EXISTS public.jobs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    request_id uuid NOT NULL,
    status public.job_status DEFAULT 'queued'::public.job_status NOT NULL,
    proof_bytes bytea NOT NULL,
    public_inputs bytea NOT NULL,
    outputs_json jsonb NOT NULL,
    fee_bps smallint NOT NULL,
    root_hash bytea NOT NULL,
    nullifier bytea NOT NULL,
    amount bigint NOT NULL,
    outputs_hash bytea NOT NULL,
    tx_id text,
    solana_signature text,
    error_message text,
    retry_count integer DEFAULT 0 NOT NULL,
    max_retries integer DEFAULT 3 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    PRIMARY KEY (id),
    UNIQUE (request_id)
);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON public.jobs USING btree (status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON public.jobs USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_jobs_request_id ON public.jobs USING btree (request_id);

-- Nullifiers table for double-spend prevention
CREATE TABLE IF NOT EXISTS public.nullifiers (
    nullifier bytea NOT NULL,
    job_id uuid NOT NULL,
    block_height bigint,
    tx_signature text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (nullifier),
    FOREIGN KEY (job_id) REFERENCES public.jobs(id)
);

CREATE INDEX IF NOT EXISTS idx_nullifiers_created_at ON public.nullifiers USING btree (created_at);

-- ========================================
-- TRIGGERS
-- ========================================

-- Create triggers for updating updated_at columns
DROP TRIGGER IF EXISTS update_merkle_tree_nodes_updated_at ON public.merkle_tree_nodes;
CREATE TRIGGER update_merkle_tree_nodes_updated_at
    BEFORE UPDATE ON public.merkle_tree_nodes
    FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

DROP TRIGGER IF EXISTS update_indexer_metadata_updated_at ON public.indexer_metadata;
CREATE TRIGGER update_indexer_metadata_updated_at
    BEFORE UPDATE ON public.indexer_metadata
    FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

-- Grant all privileges on schema public to cloak user
GRANT ALL ON SCHEMA public TO cloak;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO cloak;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO cloak;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO cloak;
