---
title: Indexer Service
description: Rust Axum service that maintains the append-only Merkle tree, stores encrypted notes, and serves artifacts.
---

# Indexer Service

The indexer is a Rust/Axum microservice that ingests on-chain deposit events, maintains the Merkle tree, and exposes HTTP APIs for clients and relayers. It serves as the source of truth for the Merkle tree state and provides all cryptographic proofs needed for private withdrawals.

Source: `services/indexer/`

## Overview

The indexer bridges on-chain deposit events with off-chain proof generation:

1. **Monitors** shield-pool program for deposit events
2. **Appends** commitments to an append-only Merkle tree
3. **Stores** encrypted note outputs for client discovery
4. **Serves** Merkle roots and inclusion proofs via HTTP API
5. **Hosts** SP1 verification artifacts and proving infrastructure

## Architecture

### Module Structure

```
services/indexer/src/
├── lib.rs                      - Library exports
├── main.rs                     - Server entry point
├── config.rs                   - Environment configuration
├── error.rs                    - Error types and handling
├── logging.rs                  - Structured logging setup
├── cloudwatch.rs               - AWS CloudWatch integration
├── artifacts.rs                - SP1 artifact file management
├── merkle.rs                   - BLAKE3 Merkle tree implementation
├── sp1_tee_client.rs          - TEE proof generation client
├── database/
│   ├── mod.rs                 - Database module exports
│   ├── connection.rs          - PostgreSQL connection pooling
│   ├── storage.rs             - TreeStorage implementation
│   └── migrations.rs          - Schema migration runner
├── server/
│   ├── mod.rs                 - Server module exports
│   ├── routes.rs              - HTTP route definitions
│   ├── middleware.rs          - CORS, timeouts, logging
│   ├── rate_limiter.rs        - Request rate limiting
│   ├── final_handlers.rs      - Route handler implementations
│   └── prover_handler.rs      - **DEPRECATED** proof endpoint
└── migrations/
    └── 001_initial_schema.sql - Database schema
```

### Core Components

#### 1. Merkle Tree (`merkle.rs`)

Implements an append-only BLAKE3-based Merkle tree:

- **Height**: Configurable (default: 32 levels)
- **Hash Function**: BLAKE3-256
- **Storage**: PostgreSQL-backed via `TreeStorage` trait
- **Operations**: Append leaf, compute root, generate inclusion proof

**Key Functions:**
```rust
pub fn append(&mut self, leaf: [u8; 32]) -> Result<u32, MerkleError>
pub fn root(&self) -> [u8; 32]
pub fn proof(&self, index: u32) -> Result<MerklePath, MerkleError>
```

**Reference:** `services/indexer/src/merkle.rs`

#### 2. Database Layer (`database/`)

PostgreSQL storage implementation:

**Tables:**
- `merkle_nodes` - Tree structure (level, index, hash)
- `encrypted_outputs` - User note discovery data

**Schema:** `services/indexer/migrations/001_initial_schema.sql`

**Storage Trait:**
```rust
#[async_trait]
pub trait TreeStorage {
    async fn get_node(&self, level: u32, index: u32) -> Result<Option<[u8; 32]>, Error>;
    async fn set_node(&self, level: u32, index: u32, hash: &[u8; 32]) -> Result<(), Error>;
    async fn get_max_leaf_index(&self) -> Result<u32, Error>;
}
```

**Implementation:** `PostgresTreeStorage` in `services/indexer/src/database/storage.rs`

#### 3. Artifact Manager (`artifacts.rs`)

Manages SP1 verification artifacts:

- **Location**: Configurable directory (default: `artifacts/`)
- **Versions**: Multiple withdraw circuit versions supported
- **Files**: Proof parameters, verification keys, circuit metadata

**Endpoints:**
- `GET /api/v1/artifacts/withdraw/:version` - List available files
- `GET /api/v1/artifacts/files/:version/:filename` - Download artifact

**Reference:** `services/indexer/src/artifacts.rs`

#### 4. SP1 TEE Client (`sp1_tee_client.rs`)

Integration with SP1 network for remote proof generation:

- **Mode**: TEE (Trusted Execution Environment) proving
- **Fallback**: Local proving if TEE unavailable
- **Status**: Optional integration for future use

**Note:** The `/api/v1/prove` endpoint has been deprecated in favor of client-side proving.

**Reference:** `services/indexer/src/sp1_tee_client.rs`

## Technology Stack

- **Rust** - Systems programming language
- **Axum** - Web framework with Tower HTTP middleware
- **SQLx** - Async PostgreSQL access with connection pooling
- **Tokio** - Async runtime
- **Tracing** - Structured logging
- **BLAKE3** - Fast cryptographic hash for Merkle tree
- **Tower HTTP** - Middleware layers (CORS, compression, tracing)
- **Serde** - JSON serialization/deserialization

## Configuration

Copy `env.example` to `.env` and adjust the following:

```env
DB_HOST=localhost
DB_PORT=5432
DB_NAME=cloak
DB_USER=cloak
DB_PASSWORD=secret
PORT=3001
TREE_HEIGHT=32
SOLANA_RPC_URL=http://127.0.0.1:8899
SHIELD_POOL_PROGRAM_ID=<program-id>
```

Set `RUST_LOG=info` to view structured logs.

## HTTP API

### Public Endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/` | Service info and available endpoints |
| `GET` | `/health` | Health check (returns "OK") |

### API v1 (`/api/v1`)

#### Merkle Tree Operations

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/merkle/root` | Current Merkle root and next leaf index |
| `GET` | `/merkle/proof/:index` | Merkle inclusion proof for given leaf index |

**Example Response (`/merkle/root`):**
```json
{
  "root": "0x1234...abcd",
  "nextIndex": 42
}
```

**Example Response (`/merkle/proof/5`):**
```json
{
  "path_elements": ["0x...", "0x..."],
  "path_indices": [0, 1, 0, 1, ...]
}
```

#### Note Discovery

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/notes/range?start=0&end=100&limit=50` | Paginated encrypted note outputs |
| `POST` | `/deposit` | Ingest commitment + encrypted output |

**Note:** Deposit events are primarily ingested from on-chain logs, not via this HTTP endpoint.

#### Artifact Serving

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/artifacts/withdraw/:version` | List SP1 artifact files for version |
| `GET` | `/artifacts/files/:version/:filename` | Download specific artifact file |

**Example:** `GET /api/v1/artifacts/files/v1/vkey.bin`

#### Deprecated Endpoints

| Method | Path | Status |
| --- | --- | --- |
| `POST` | `/prove` | **DEPRECATED** - Returns 410 GONE with deprecation notice |

**Migration:** Proof generation now happens client-side using SP1 host CLI or TEE proving service.

### Admin Endpoints (Development Only)

These endpoints are available for testing and should be disabled in production:

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/admin/push-root` | Manually push a Merkle root |
| `POST` | `/admin/insert-leaf` | Directly insert a leaf node |
| `POST` | `/admin/reset` | Reset database (dangerous!) |

**Warning:** Admin endpoints should never be exposed in production.

## Event Ingestion

The indexer monitors the `shield-pool` program for deposit events:

```rust
// Shield-pool emits this log on deposit
Log: "deposit_commit:{commitment_hex}"
```

**Ingestion Flow:**
1. Subscribe to shield-pool program logs via Solana RPC
2. Parse commitment from log message
3. Append commitment to Merkle tree
4. Update PostgreSQL with new leaf
5. Serve updated root via `/merkle/root`

**Implementation:** Event subscription logic would be in the main service loop (not yet implemented in current codebase - deposits are currently added via `/deposit` POST endpoint or admin endpoints).

## Middleware Stack

The server applies middleware in the following order (outermost to innermost):

1. **Compression** - Gzip response compression
2. **CORS** - Cross-origin resource sharing (configurable origins)
3. **Timeout** - Request timeout enforcement
4. **Size Limit** - Maximum request body size
5. **Logging** - Custom logging middleware
6. **Tracing** - Tower HTTP trace layer

**Reference:** `services/indexer/src/server/routes.rs:74-89`

## Running Locally

### Prerequisites

- PostgreSQL 16+ running locally or via Docker
- Rust toolchain installed
- Solana localnet (optional, for testing deposits)

### Setup Steps

```bash
# Navigate to indexer directory
cd services/indexer

# Start PostgreSQL (via Docker Compose)
docker compose up -d postgres

# Copy environment template
cp env.example .env

# Edit .env with your configuration
# DB_HOST=localhost
# DB_PORT=5434
# DB_NAME=cloak
# ...

# Run the service
cargo run
```

### Verify It's Running

```bash
# Health check
curl http://localhost:3001/health
# Expected: "OK"

# Get current root
curl http://localhost:3001/api/v1/merkle/root
# Expected: { "root": "0x...", "nextIndex": 0 }

# Get service info
curl http://localhost:3001/
```

## Database Management

### Running Migrations

Migrations run automatically on startup. To run manually:

```bash
cargo run --bin migrate
```

### Schema

See `services/indexer/migrations/001_initial_schema.sql` for complete schema.

**Key Tables:**
```sql
-- Merkle tree nodes
CREATE TABLE merkle_nodes (
    level INTEGER NOT NULL,
    index INTEGER NOT NULL,
    hash BYTEA NOT NULL,
    PRIMARY KEY (level, index)
);

-- Encrypted note outputs
CREATE TABLE encrypted_outputs (
    leaf_index INTEGER PRIMARY KEY,
    encrypted_data BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## Observability

### Structured Logging

The indexer uses `tracing` for structured logs:

```bash
# Set log level
RUST_LOG=info cargo run

# Available levels
RUST_LOG=trace   # Very verbose
RUST_LOG=debug   # Debug info
RUST_LOG=info    # Default
RUST_LOG=warn    # Warnings only
RUST_LOG=error   # Errors only
```

**Log Output Includes:**
- HTTP request/response spans
- Database query timing
- Merkle tree operations
- Artifact access
- Error traces with context

### Performance Metrics

Key metrics to monitor:

- **Merkle tree operations**: Append, root computation, proof generation
- **Database queries**: Connection pool usage, query latency
- **HTTP requests**: Endpoint latency, error rates
- **Memory**: Tree cache size, connection pool size

### CloudWatch Integration

Optional AWS CloudWatch logging:

```bash
CLOUDWATCH_ENABLED=true
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_REGION=us-east-1
CLOUDWATCH_LOG_GROUP=Cloak
```

**Reference:** `services/indexer/src/cloudwatch.rs`

## Production Considerations

### Security

- **Admin Endpoints**: Must be disabled or access-controlled in production
- **CORS**: Configure allowed origins appropriately
- **Rate Limiting**: Adjust per your expected traffic
- **Database Credentials**: Use secrets management, never commit credentials

### Performance

- **Database Indexing**: Ensure indexes on `merkle_nodes(level, index)`
- **Connection Pooling**: Tune `SQLx` pool size based on load
- **Response Caching**: Consider caching Merkle root responses
- **CDN**: Serve static artifacts via CDN

### Scaling

- **Read Replicas**: PostgreSQL read replicas for high query volume
- **Horizontal Scaling**: Multiple indexer instances (read-only API)
- **Write Singleton**: Only one instance should ingest deposits
- **Load Balancer**: Distribute API requests across instances

## Troubleshooting

### Database Connection Errors

```
Error: Failed to connect to database
```

**Solutions:**
- Verify PostgreSQL is running: `docker compose ps`
- Check connection string in `.env`
- Ensure database exists: `psql -U cloak -d cloak`
- Check firewall/network settings

### Migration Failures

```
Error: Migration failed
```

**Solutions:**
- Drop and recreate database (dev only): `POST /admin/reset`
- Check migration files in `migrations/`
- Verify database permissions
- Review migration logs

### High Memory Usage

**Causes:**
- Large Merkle tree in memory
- Connection pool too large
- Memory leaks (rare)

**Solutions:**
- Reduce tree cache size (if implemented)
- Tune SQLx pool size
- Monitor with `RUST_LOG=debug`
- Restart service periodically

## Related Documentation

- **[Off-Chain Overview](./overview.md)** - Service architecture
- **[Relay Service](./relay.md)** - Withdraw orchestration
- **[Indexer API Reference](../api/indexer.md)** - Detailed API specs
- **[Operations Runbook](../operations/runbook.md)** - Production operations
- **[Merkle Tree Design](../zk/merkle.md)** - Cryptographic details
