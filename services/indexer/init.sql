-- Database initialization script for Docker
-- Creates the databases for indexer and relay services

-- This file is automatically executed by the postgres container
-- when the container starts for the first time

-- Create databases
CREATE DATABASE cloak_indexer;
CREATE DATABASE cloak_relay;

-- Set up database configuration
\c cloak_indexer;
ALTER DATABASE cloak_indexer SET timezone TO 'UTC';

\c cloak_relay;
ALTER DATABASE cloak_relay SET timezone TO 'UTC';

-- Grant permissions to cloak user (created by POSTGRES_USER env var)
\c postgres;
GRANT ALL PRIVILEGES ON DATABASE cloak_indexer TO cloak;
GRANT ALL PRIVILEGES ON DATABASE cloak_relay TO cloak;