---
title: Relay Service
description: Axum service + background workers that validate withdraw jobs, integrate PoW claims, and submit Solana transactions.
---

# Relay Service

The relay orchestrates withdraw operations from submission through on-chain execution. It acts as the bridge between users requesting private withdrawals and the Solana blockchain, coordinating proof verification, PoW claim discovery, transaction building, and submission.

Source: `services/relay/`

## Overview

The relay service is responsible for:

1. **Accepting** withdraw requests via HTTP API
2. **Validating** SP1 proofs and public inputs
3. **Queueing** jobs with retry logic and backoff
4. **Finding** available PoW claims from independent miners
5. **Building** Solana transactions with all required accounts
6. **Submitting** transactions to Solana (optionally via Jito)
7. **Tracking** job state and nullifier usage

## Architecture

The relay follows a **queue-based worker architecture** with distinct layers:

```
┌─────────────────────────────────────────────────────────┐
│                     HTTP API (Axum)                      │
│  /withdraw  /status  /jobs/*  /orchestrate  /submit     │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
          ┌────────────┐
          │ Validation │  - Schema validation
          │   Layer    │  - Policy checks
          └─────┬──────┘  - Nullifier pre-check
                │
                ▼
         ┌─────────────┐
         │  PostgreSQL │  - Jobs table
         │  Repository │  - Nullifiers table
         └──────┬──────┘
                │
                ▼
         ┌─────────────┐
         │    Queue    │  - Job queue
         │             │  - Retry logic
         └──────┬──────┘  - Backoff
                │
                ▼
    ┌───────────────────────┐
    │   Background Worker   │
    │                       │
    │  1. Poll queue        │
    │  2. Find claim        │  ───┐
    │  3. Build tx          │     │
    │  4. Submit tx         │     │
    │  5. Update status     │     │
    └───────────────────────┘     │
                                  │
                 ┌────────────────┘
                 │
      ┌──────────▼──────────┐
      │   Claim Manager     │  - Query on-chain claims
      │  (PoW Integration)  │  - Filter available
      └──────────┬──────────┘  - Wildcard matching
                 │
                 ▼
      ┌─────────────────────┐
      │  Solana Service     │  - RPC client
      │                     │  - Transaction builder
      │  - Client           │  - Jito integration
      │  - Transaction      │  - Submission
      │    Builder          │
      │  - Submit           │
      └─────────────────────┘
                 │
                 ▼
         ┌───────────────┐
         │    Solana     │
         │   Programs    │
         └───────────────┘
```

### Module Structure

```
services/relay/src/
├── main.rs                      - Service entry point, routes setup
├── lib.rs                       - Module exports
├── config.rs                    - Configuration from TOML/env
├── error.rs                     - Error types and handling
├── metrics.rs                   - Metrics logging
├── cloudwatch.rs                - AWS CloudWatch integration
├── claim_manager.rs             - PoW claim discovery (key component!)
├── planner.rs                   - Module re-export
├── api/
│   ├── mod.rs                   - API module exports
│   ├── withdraw.rs              - POST /withdraw handler
│   ├── status.rs                - GET /status/:id handler
│   ├── validator_agent.rs       - Validator agent endpoints
│   └── prove_local.rs           - Local proof generation helper
├── planner/
│   └── orchestrator.rs          - Multi-step withdraw orchestration
├── queue/
│   └── mod.rs                   - Queue implementation
├── worker/
│   ├── mod.rs                   - Worker main logic
│   └── processor.rs             - Job processing implementation
├── validation/
│   └── mod.rs                   - Request validation logic
├── db/
│   ├── mod.rs                   - Database module exports
│   ├── models.rs                - Job and nullifier models
│   └── repository.rs            - PostgreSQL repositories
└── solana/
    ├── mod.rs                   - Solana module exports
    ├── client.rs                - RPC client wrapper
    ├── transaction_builder.rs   - Transaction construction
    └── submit.rs                - Transaction submission
```

## Core Components

### 1. Claim Manager (`claim_manager.rs`)

**Critical component** for PoW integration - discovers available claims from miners.

**Purpose:** The relay acts as a CLIENT of the independent miner ecosystem. Miners run `cloak-miner` independently, and the relay queries on-chain for available claims.

**Key Structure:**
```rust
pub struct ClaimFinder {
    rpc_client: RpcClient,
    registry_program_id: Pubkey,
}

pub struct AvailableClaim {
    pub claim_pda: Pubkey,
    pub miner_pda: Pubkey,
    pub miner_authority: Pubkey,
    pub mined_slot: u64,
    pub registry_pda: Pubkey,
}
```

**Claim Discovery Process:**
1. Query all accounts owned by scramble-registry program
2. Filter accounts by size (256 bytes = claim)
3. Parse claim account data
4. Check claim status (must be Revealed)
5. Check expiration (not expired)
6. Check consumption (not fully consumed)
7. Match batch_hash (or accept wildcard `[0; 32]`)
8. Return first available claim

**Metrics Logged:**
- `[METRICS] Claim search started`
- `[METRICS] Query complete: N accounts found in Xms`
- Filtering statistics (size, status, expiration, etc.)

**Reference:** `services/relay/src/claim_manager.rs`

### 2. Worker (`worker/`)

Background task that processes jobs from the queue.

**Configuration:**
- Poll interval: 1 second (default)
- Max concurrent jobs: 10 (default)
- Runs in separate Tokio task

**Processing Flow:**
1. Poll queue for next job
2. Fetch job details from PostgreSQL
3. Find available PoW claim (if enabled)
4. Build Solana transaction with all accounts
5. Simulate transaction (optional)
6. Submit transaction to Solana
7. Wait for confirmation
8. Update job status in database
9. On error: Retry with exponential backoff or mark failed

**Reference:** `services/relay/src/worker/mod.rs`, `services/relay/src/worker/processor.rs`

### 3. Solana Service (`solana/`)

Encapsulates all Solana RPC interactions.

**Components:**
- **Client**: RPC client wrapper with connection management
- **Transaction Builder**: Constructs withdraw transactions with correct accounts
- **Submit**: Handles transaction submission (direct RPC or Jito)

**Key Methods:**
```rust
impl SolanaService {
    pub async fn submit_withdraw_transaction(...) -> Result<Signature>
    pub fn set_claim_finder(&mut self, finder: Option<Arc<ClaimFinder>>)
}
```

**Optional Jito Integration:**
- Bundle submission for priority inclusion
- Configured via `jito_url` in config
- Falls back to standard RPC if unavailable

**Reference:** `services/relay/src/solana/`

## Configuration

Primary configuration lives in `config.toml` (see `services/relay/config.toml` for sample) and environment variables. Key sections:

```toml
[server]
port = 3002
host = "0.0.0.0"

[database]
url = "postgres://postgres:postgres@localhost:5432/relay"

[solana]
rpc_url = "http://127.0.0.1:8899"
program_id = "<shield-pool-program-id>"
scramble_registry_program_id = "<scramble-program-id>"
jito_url = "https://mainnet.block-engine.jito.wtf"  # optional

[pow]
enabled = true
```

Enable detailed logging with `RUST_LOG=info,tower_http=info`.

## HTTP API

### Public Endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/` | Service metadata and version |
| `GET` | `/health` | Health check (DB connectivity) |

**Example Response (`/`):**
```json
{
  "service": "Cloak Relay",
  "version": "0.1.0",
  "status": "running",
  "endpoints": {
    "health": "GET /health",
    "withdraw": "POST /withdraw",
    "status": "GET /status/:id"
  }
}
```

### Withdraw API

#### Submit Withdraw Request

**Endpoint:** `POST /withdraw`

**Request Body:**
```json
{
  "proof": "0x...",              // SP1 Groth16 proof (256 bytes hex)
  "public_inputs": "0x...",      // SP1 public inputs (64 bytes hex)
  "root": "0x...",               // Merkle root (32 bytes hex)
  "nullifier": "0x...",          // Nullifier (32 bytes hex)
  "amount": 1000000,             // Total amount in lamports
  "fee_bps": 60,                 // Fee in basis points (0.6%)
  "outputs_hash": "0x...",       // Hash of outputs (32 bytes hex)
  "outputs": [
    {
      "recipient": "recipient_pubkey_base58",
      "amount": 400000
    },
    {
      "recipient": "recipient_pubkey_base58",
      "amount": 594000
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "request_id": "uuid-here",
    "status": "queued",
    "message": "Withdraw request queued successfully"
  }
}
```

**Reference:** `services/relay/src/api/withdraw.rs`

#### Check Job Status

**Endpoint:** `GET /status/:id`

**Response:**
```json
{
  "request_id": "uuid-here",
  "status": "completed",
  "tx_id": "signature-here",
  "error": null,
  "created_at": "2024-01-01T00:00:00Z",
  "completed_at": "2024-01-01T00:00:05Z"
}
```

**Possible Status Values:**
- `queued` - Job is waiting in queue
- `processing` - Worker is processing the job
- `completed` - Transaction confirmed on-chain
- `failed` - Job failed (see error field)

**Reference:** `services/relay/src/api/status.rs`

### Validator Agent API

Advanced API for validator operations and external integrations.

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/jobs/withdraw` | Create withdraw job (structured format) |
| `GET` | `/jobs/:job_id` | Fetch job details |
| `POST` | `/submit` | Submit pre-signed transaction |

**Documentation:** See [`api/validator-agent.md`](../api/validator-agent.md) for detailed specs.

**Reference:** `services/relay/src/api/validator_agent.rs`

### Orchestration API

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/orchestrate/withdraw` | Multi-step withdraw orchestration |

**Purpose:** Higher-level API that coordinates multiple operations (proof verification, claim finding, transaction building) in a single call.

**Reference:** `services/relay/src/planner/orchestrator.rs`

### Development Helpers

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/jobs/:job_id/prove-local` | Generate proof using local SP1 host CLI |

**Note:** This endpoint is for development/testing only. Production proofs should be generated client-side.

**Reference:** `services/relay/src/api/prove_local.rs`

## Job Lifecycle

### State Machine

```
    ┌──────────┐
    │ Submitted│
    │  (User)  │
    └─────┬────┘
          │
          ▼
    ┌──────────┐
    │Validation│ ──► [Invalid] ──► ❌ Rejected
    └─────┬────┘
          │ [Valid]
          ▼
    ┌──────────┐
    │Persisted │
    │(Database)│
    └─────┬────┘
          │
          ▼
    ┌──────────┐
    │  Queued  │ ◄──┐
    │          │  Retry
    └─────┬────┘    │
          │         │
          ▼         │
    ┌──────────┐    │
    │Processing│ ───┘ [Transient Error]
    │ (Worker) │
    └─────┬────┘
          │
      ┌───┴────┐
      │        │
      ▼        ▼
  [Success] [Fatal Error]
      │        │
      ▼        ▼
┌──────────┐ ┌────────┐
│Completed │ │ Failed │
│    ✅    │ │   ❌   │
└──────────┘ └────────┘
```

### Detailed Flow

#### 1. Validation Phase

**Input Validation:**
- Deserialize JSON request body
- Validate proof length (256 bytes for SP1 Groth16)
- Validate public inputs (64 bytes)
- Validate all hashes (32 bytes each)
- Validate amounts (non-zero, no overflow)
- Validate fee_bps (within allowed range)
- Validate outputs (at least 1, max N)

**Policy Checks:**
- Fee must be within allowed bounds (e.g., 0-500 bps)
- Output count within limits
- Total amount matches sum(outputs) + fee

**Nullifier Pre-Check:**
- Query database for existing nullifier
- Reject if already used (prevents duplicate submission)

**Reference:** `services/relay/src/validation/mod.rs`

#### 2. Persistence Phase

Job inserted into PostgreSQL with:
- `id` (UUID)
- `status` (`queued`)
- `proof`, `public_inputs`, `root`, `nullifier`
- `amount`, `fee_bps`, `outputs_hash`
- `outputs` (JSON array)
- `created_at` (timestamp)
- `tx_signature` (null initially)
- `error_message` (null initially)

**Reference:** `services/relay/src/db/models.rs`

#### 3. Queueing Phase

Job enqueued in database:
- Key: `cloak:jobs:queue`
- Value: Job ID (UUID)
- Initial delay: 0 seconds
- Priority: FIFO (first-in-first-out)

Database queue provides:
- Atomic pop operations
- Exponential backoff on retry
- Dead-letter queue for failed jobs

**Reference:** `services/relay/src/queue/mod.rs`

#### 4. Processing Phase

Worker picks up job and executes:

**Step 1: Fetch Job Data**
- Retrieve full job record from PostgreSQL
- Update status to `processing`

**Step 2: Find PoW Claim (if enabled)**
- Compute batch_hash from job data
- Call `ClaimFinder::find_claim(batch_hash)`
- Wait for available claim
- If no claim available: Retry later

**Step 3: Build Transaction**
- Construct withdraw instruction data
- Gather all required accounts:
  - Pool PDA
  - Treasury PDA
  - Roots ring PDA
  - Nullifier shard PDA
  - Recipient accounts (from outputs)
  - System program
  - Claim PDA (if PoW enabled)
  - Miner PDA (if PoW enabled)
  - Registry PDA (if PoW enabled)
- Build Solana transaction with recent blockhash

**Reference:** `services/relay/src/solana/transaction_builder.rs`

**Step 4: Submit Transaction**
- Simulate transaction (optional)
- Submit via RPC or Jito bundle
- Wait for confirmation (confirmed commitment level)
- Extract transaction signature

**Reference:** `services/relay/src/solana/submit.rs`

**Step 5: Update Status**
- On success: Set status to `completed`, store tx_signature
- On error: Determine if retryable or fatal
  - Retryable: Re-queue with backoff
  - Fatal: Set status to `failed`, store error_message

**Step 6: Commit Nullifier**
- Insert nullifier into database
- Prevents future duplicate submissions

**Reference:** `services/relay/src/worker/processor.rs`

### Retry Logic

**Transient Errors (retry):**
- Network timeout
- RPC node unavailable
- Insufficient compute units
- Blockhash expired
- No PoW claims available (temporary)

**Fatal Errors (no retry):**
- Invalid proof (SP1 verification failed)
- Nullifier already used on-chain
- Root not found in ring buffer
- Insufficient funds in pool
- Invalid transaction structure

**Backoff Strategy:**
- Initial delay: 1 second
- Max delay: 60 seconds
- Multiplier: 2x
- Max retries: 5

## Database Models

### Jobs Table

```sql
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    status VARCHAR(20) NOT NULL,
    proof BYTEA NOT NULL,
    public_inputs BYTEA NOT NULL,
    root BYTEA NOT NULL,
    nullifier BYTEA NOT NULL,
    amount BIGINT NOT NULL,
    fee_bps INTEGER NOT NULL,
    outputs_hash BYTEA NOT NULL,
    outputs JSONB NOT NULL,
    tx_signature VARCHAR(88),
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_nullifier ON jobs(nullifier);
CREATE INDEX idx_jobs_created_at ON jobs(created_at);
```

**Status Values:**
- `queued` - Waiting in database queue
- `processing` - Being processed by worker
- `completed` - Transaction confirmed
- `failed` - Fatal error occurred

**Schema:** `services/relay/migrations/001_init.sql`

### Nullifiers Table

```sql
CREATE TABLE nullifiers (
    nullifier BYTEA PRIMARY KEY,
    job_id UUID REFERENCES jobs(id),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_nullifiers_job_id ON nullifiers(job_id);
```

**Purpose:**
- Pre-commit nullifier tracking
- Prevents duplicate job submission before on-chain confirmation
- Faster lookups than querying Solana

**Reference:** `services/relay/src/db/models.rs`

## PoW Integration Details

### Initialization

`ClaimFinder` is initialized when `scramble_registry_program_id` is configured:

```rust
// In main.rs:102-81
let claim_finder = if let Some(ref registry_id) = config.solana.scramble_registry_program_id {
    let registry_program_id = Pubkey::from_str(registry_id)?;
    let finder = Some(Arc::new(ClaimFinder::new(
        config.solana.rpc_url.clone(),
        registry_program_id,
    )));
    solana_service.set_claim_finder(finder.clone());
    finder
} else {
    None
};
```

**Reference:** `services/relay/src/main.rs:58-81`

### Claim Matching

**Wildcard Claims:**
- batch_hash = `[0; 32]`
- Can be used for ANY withdraw
- Higher competition/demand

**Specific Claims:**
- batch_hash = BLAKE3(job_ids...)
- Only usable for specific batch
- Targeted mining

### Claim Consumption Flow

1. Worker calls `ClaimFinder::find_claim(batch_hash)`
2. ClaimFinder queries all claims from registry
3. Filters for: Revealed status, not expired, not fully consumed
4. Matches batch_hash (or accepts wildcard)
5. Returns `AvailableClaim` with all required PDAs
6. Worker includes claim accounts in withdraw transaction
7. Shield-pool program calls `consume_claim` CPI
8. Miner earns fees when claim is consumed

**Fee Distribution:**
- Configured in scramble-registry
- Paid from withdraw amount
- Sent to miner_authority

### Metrics

Logged with `[METRICS]` prefix:
- `Claim search started`
- `Query complete: N accounts found in Xms`
- Filtering statistics (filtered by size, status, expiration, consumption, batch mismatch)
- `Claim found` / `No claims available`

## Running Locally

### Prerequisites

- PostgreSQL 16+ (local or Docker)
- Rust toolchain
- Solana RPC endpoint (localnet/devnet/mainnet)
- Shield-pool and scramble-registry programs deployed

### Setup Steps

```bash
# Navigate to relay directory
cd services/relay

# Start infrastructure via Docker Compose (from repo root)
cd ../..
docker compose up -d postgres relay

# Or run relay locally:
cd services/relay

# Copy and configure environment
cp config.toml.example config.toml
# Edit config.toml with your settings

# Run the service
cargo run
```

### Configuration

Edit `config.toml`:

```toml
[server]
port = 3002
host = "0.0.0.0"

[database]
url = "postgres://cloak:password@localhost:5434/cloak"

[solana]
rpc_url = "http://127.0.0.1:8899"
program_id = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"
scramble_registry_program_id = "EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6"
# jito_url = "https://mainnet.block-engine.jito.wtf"  # Optional

[pow]
enabled = true
```

**Alternative: Environment Variables**

```bash
DATABASE_URL=postgres://cloak:password@localhost:5434/cloak
SOLANA_RPC_URL=http://127.0.0.1:8899
CLOAK_PROGRAM_ID=c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp
SCRAMBLE_REGISTRY_PROGRAM_ID=EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6
```

### Verify It's Running

```bash
# Health check
curl http://localhost:3002/health
# Expected: "OK"

# Service info
curl http://localhost:3002/
# Expected: JSON with service metadata

# Check worker logs
# You should see: "Worker task spawned and running"
```

### Testing Workflow

```bash
# 1. Ensure indexer is running (for Merkle root)
curl http://localhost:3001/api/v1/merkle/root

# 2. Generate a proof (using zk-guest-sp1)
cd packages/zk-guest-sp1
cargo run --bin cloak-zk -- prove ...

# 3. Submit withdraw request to relay
curl -X POST http://localhost:3002/withdraw \
  -H "Content-Type: application/json" \
  -d @withdraw_request.json

# 4. Check job status
curl http://localhost:3002/status/{job_id}

# 5. Monitor worker logs for processing
# Worker will log claim search, tx building, submission
```

## Observability

### Structured Logging

The relay uses `tracing` for comprehensive structured logging:

```bash
# Set log level
RUST_LOG=info cargo run

# Available levels
RUST_LOG=trace     # Very verbose (includes all sub-crates)
RUST_LOG=debug     # Debug information
RUST_LOG=info      # Default production level
RUST_LOG=warn      # Warnings only
RUST_LOG=error     # Errors only

# Fine-grained control
RUST_LOG=relay=debug,tower_http=info,sqlx=warn
```

**Log Output Includes:**
- HTTP request/response spans with duration
- Job lifecycle events (queued, processing, completed/failed)
- PoW claim search metrics
- Database operations
- Transaction simulation and submission
- Error traces with full context

### Metrics

Key metrics logged with `[METRICS]` prefix:

**Claim Search:**
```
🔍 [METRICS] Claim search started for batch_hash: 12345678
📊 [METRICS] Query complete: 150 accounts found in 234ms
✅ [METRICS] Claim found: claimPda123...
```

**Job Processing:**
```
[METRICS] Job {uuid} started processing
[METRICS] Job {uuid} completed in 3.4s
[METRICS] Transaction confirmed: signature123...
```

**Errors:**
```
❌ [METRICS] Claim query failed after 5.2s: RPC timeout
❌ [METRICS] Job {uuid} failed: Nullifier already used
```

**Reference:** `services/relay/src/metrics.rs`

### CloudWatch Integration

Optional AWS CloudWatch logging:

```bash
CLOUDWATCH_ENABLED=true
AWS_ACCESS_KEY_ID=your_key
AWS_SECRET_ACCESS_KEY=your_secret
AWS_REGION=us-east-1
CLOUDWATCH_LOG_GROUP=Cloak
```

Logs are streamed to CloudWatch with:
- Structured JSON format
- Searchable fields
- Retention policies
- Alerting integration

**Reference:** `services/relay/src/cloudwatch.rs`

### Database Monitoring

Monitor these PostgreSQL metrics:

- **Job queue depth**: Count of `status='queued'`
- **Processing jobs**: Count of `status='processing'`
- **Success rate**: Ratio of `completed` to total
- **Nullifier duplicates**: Failed insertions on `nullifiers` table
- **Average job duration**: `completed_at - created_at`

```sql
-- Queue depth
SELECT COUNT(*) FROM jobs WHERE status = 'queued';

-- Success rate (last 24h)
SELECT
  status,
  COUNT(*) as count,
  COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () as percentage
FROM jobs
WHERE created_at > NOW() - INTERVAL '24 hours'
GROUP BY status;

-- Average processing time
SELECT AVG(EXTRACT(EPOCH FROM (completed_at - created_at))) as avg_seconds
FROM jobs
WHERE status = 'completed' AND completed_at > NOW() - INTERVAL '1 hour';
```

## Production Considerations

### Security

**API Security:**
- Enable HTTPS/TLS termination
- Rate limiting per IP/user
- API key authentication (optional)
- CORS configured for specific origins only

**Database Security:**
- Use strong passwords (secrets management)
- Enable SSL/TLS for PostgreSQL connections
- Restrict network access (firewall rules)
- Regular backups with encryption

**Solana RPC:**
- Use dedicated RPC nodes (not public endpoints)
- Implement retry logic for reliability
- Monitor RPC rate limits
- Consider multiple RPC endpoints for failover

### Performance

**Database Optimization:**
- Connection pooling (tune `max_connections`)
- Index on frequently queried columns
- Vacuum and analyze regularly
- Consider read replicas for high load

**Database Optimization:**
- Tune connection pool settings
- Use read replicas for high load
- Monitor query performance
- Consider database clustering for scalability

**Worker Tuning:**
- Adjust concurrent job limit based on CPU cores
- Tune poll interval for latency vs CPU tradeoff
- Monitor queue lag
- Scale horizontally (multiple worker instances)

**RPC Optimization:**
- Use confirmed commitment level for faster responses
- Implement connection pooling
- Cache recent blockhashes
- Batch RPC requests where possible

### Scaling

**Horizontal Scaling:**
- Multiple worker instances (all polling same database queue)
- Load balancer for HTTP API
- Shared PostgreSQL (single source of truth)
- Stateless workers (no local state)

**Vertical Scaling:**
- Increase worker concurrency limit
- More CPU cores for parallel processing
- More RAM for larger connection pools
- Faster storage for database

**Database Scaling:**
- Read replicas for GET endpoints
- Write/read splitting
- Connection pooling middleware (PgBouncer)
- Partitioning large tables by date

**Queue Scaling:**
- Database clustering for distributed queue
- Multiple queue priorities
- Separate queues for different job types
- Dead-letter queue monitoring

### High Availability

**Service Redundancy:**
- Multiple relay instances behind load balancer
- Health check endpoints for automatic failover
- Graceful shutdown handling
- Blue-green deployment strategy

**Data Redundancy:**
- PostgreSQL streaming replication
- Database clustering for automatic failover
- Regular database backups
- Point-in-time recovery capability

**Monitoring & Alerting:**
- Job queue depth alerts
- Worker health checks
- Transaction failure rate alerts
- RPC availability monitoring
- Database performance alerts

## Troubleshooting

### No PoW Claims Available

**Symptom:**
```
[METRICS] Query complete: 0 accounts found
No claims available for batch_hash: 0000...
```

**Causes:**
- No miners running
- All claims consumed
- Claims expired
- Registry not initialized

**Solutions:**
- Start `cloak-miner` instances
- Wait for miners to reveal claims
- Check claim expiration windows
- Verify registry initialization

### Jobs Stuck in Queue

**Symptom:** Jobs remain in `queued` status indefinitely.

**Causes:**
- Worker not running
- Database connection lost
- Worker crashed

**Solutions:**
```bash
# Check worker is running
ps aux | grep relay

# Check database connection
psql $DATABASE_URL -c "SELECT 1;"

# Restart relay service
systemctl restart relay

# Check worker logs
journalctl -u relay -f
```

### Transaction Failures

**Symptom:** Jobs marked as `failed` with transaction errors.

**Common Errors:**

**"Invalid proof":**
- Proof verification failed in shield-pool
- Check proof generation was successful
- Verify vkey hash matches program

**"Nullifier already used":**
- Nullifier exists on-chain
- Check nullifiers table for duplicates
- Investigate potential replay attack

**"Root not found":**
- Merkle root not in ring buffer
- Push root to shield-pool via admin instruction
- Wait for indexer to sync

**"Insufficient funds":**
- Pool account has insufficient lamports
- Fund the pool account
- Check amount calculations

### Database Connection Errors

**Symptom:**
```
Error: Failed to connect to database
```

**Solutions:**
- Verify PostgreSQL is running: `docker compose ps`
- Check connection string in `config.toml`
- Test connectivity: `psql $DATABASE_URL`
- Check firewall rules
- Verify database exists

### Database Connection Errors

**Symptom:**
```
Error: Database connection refused
```

**Solutions:**
- Verify database is running: `psql $DATABASE_URL -c "SELECT 1;"`
- Check database URL in `config.toml`
- Verify database authentication
- Check network connectivity
- Restart database if needed

### High Memory Usage

**Causes:**
- Large job backlog in database
- Database queue overflow
- Memory leak (rare)
- Too many concurrent workers

**Solutions:**
- Limit concurrent jobs
- Monitor queue depth
- Restart service periodically
- Investigate with profiling tools

### Testing & Development

```bash
# Run unit tests
cargo test -p relay

# Run specific test
cargo test -p relay test_name

# Run with output
cargo test -p relay -- --nocapture

# Integration tests (requires PostgreSQL)
cargo test -p relay --test '*' -- --ignored

# Test with miner integration
cargo test -p relay --test miner_integration
```

**Test Fixtures:**
- Mock RPC client
- In-memory database (SQLite)
- Mock database (or test instance)
- Test keypairs and proofs

**Reference:** `services/relay/tests/`

## Related Documentation

- **[Off-Chain Overview](./overview.md)** - Service architecture
- **[Indexer Service](./indexer.md)** - Merkle tree and proof generation
- **[Relay API Reference](../api/relay.md)** - Detailed API specifications
- **[Validator Agent API](../api/validator-agent.md)** - Advanced API docs
- **[Operations Runbook](../operations/runbook.md)** - Production operations
- **[Metrics Guide](../operations/metrics-guide.md)** - Monitoring and alerting
- **[PoW Overview](../pow/overview.md)** - Mining and claims system
- **[Cloak Miner](../packages/cloak-miner.md)** - Mining CLI documentation
- **[Withdraw Workflow](../workflows/withdraw.md)** - End-to-end withdraw flow
- **[PoW Withdraw](../workflows/pow-withdraw.md)** - PoW-gated withdrawals
