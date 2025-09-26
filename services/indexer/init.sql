-- Database initialization script for Docker
-- Creates the database and user if they don't exist

-- This file is automatically executed by the postgres container
-- when the container starts for the first time

-- The database and user are already created by the POSTGRES_DB and POSTGRES_USER
-- environment variables, so this file can contain additional setup if needed

-- Example: Create additional users or grant specific permissions
-- CREATE USER cloak_readonly WITH PASSWORD 'readonly_password';
-- GRANT SELECT ON ALL TABLES IN SCHEMA public TO cloak_readonly;

-- Set up any additional database configuration
ALTER DATABASE cloak_indexer SET timezone TO 'UTC';