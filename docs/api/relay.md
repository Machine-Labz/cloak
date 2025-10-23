---
title: Relay API Reference
description: REST endpoints exposed by the Cloak relay for withdraw jobs, validator agents, and health checks.
---

# Relay API Reference

**Base URL:** `http://<host>:3002` (default: `http://localhost:3002`)

**Version:** Provided in service metadata endpoint

## General Information

All requests and responses use `application/json` content type unless otherwise specified.

Binary data is encoded as:
- **Hex strings** for hashes, roots, nullifiers (64 characters = 32 bytes)
- **Base64** for proof bytes and encoded data
- **Base58** for Solana addresses (recipients)

Timestamps follow ISO 8601 format.

## Service Information

### GET `/`

Returns service metadata, version, and available endpoints.

**Success Response (200 OK):**
```json
{
  "service": "Cloak Relay",
  "version": "0.1.0",
  "status": "running",
  "endpoints": {
    "health": "GET /health",
    "withdraw": "POST /withdraw",
    "status": "GET /status/:id",
    "validator_jobs": "POST /jobs/withdraw",
    "validator_job": "GET /jobs/:job_id",
    "submit": "POST /submit"
  }
}
```

**Reference:** `services/relay/src/main.rs:193-205`

### GET `/health`

Health check endpoint verifying database and Redis connectivity.

**Success Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00.000Z",
  "database": "connected",
  "redis": "connected",
  "worker": "running"
}
```

**Error Response (503 Service Unavailable):**
```json
{
  "status": "unhealthy",
  "error": "Database connection failed"
}
```

**Reference:** `services/relay/src/api/mod.rs:52-54`

## Withdraw API

### POST `/withdraw`

Submit a withdraw request with SP1 proof for processing.

**Request Body:**
```json
{
  "outputs": [
    {
      "recipient": "recipient_pubkey_base58",
      "amount": 400000
    },
    {
      "recipient": "recipient_pubkey_base58",
      "amount": 594000
    }
  ],
  "policy": {
    "fee_bps": 60
  },
  "public_inputs": {
    "root": "hex64_merkle_root",
    "nf": "hex64_nullifier",
    "amount": 1000000,
    "fee_bps": 60,
    "outputs_hash": "hex64_outputs_hash"
  },
  "proof_bytes": "base64_encoded_sp1_proof"
}
```

**Field Requirements:**

**`outputs`** (array, required):
- Minimum 1 output, maximum 10 outputs
- Each output has:
  - `recipient`: Base58 Solana pubkey (32-44 characters)
  - `amount`: u64 lamports (positive, non-zero)
- Sum of output amounts must equal `amount - fee`

**`policy`** (object, required):
- `fee_bps`: u16 basis points (0-500, where 60 = 0.6%)

**`public_inputs`** (object, required):
- `root`: 64-char hex (32 bytes) - Merkle root from indexer
- `nf`: 64-char hex (32 bytes) - Unique nullifier
- `amount`: u64 lamports - Total amount being withdrawn
- `fee_bps`: u16 - Must match `policy.fee_bps`
- `outputs_hash`: 64-char hex (32 bytes) - BLAKE3 hash of outputs

**`proof_bytes`** (string, required):
- Base64-encoded SP1 proof bundle
- After decoding, should be 260 bytes (Groth16 proof)
- Or full SP1 bundle that extracts to 260 bytes

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "queued",
    "message": "Withdraw request received and queued for processing"
  }
}
```

**Error Responses:**

`400 Bad Request` - Validation error:
```json
{
  "success": false,
  "error": {
    "code": "ValidationError",
    "message": "No outputs specified"
  }
}
```

`400 Bad Request` - Invalid proof:
```json
{
  "success": false,
  "error": {
    "code": "ValidationError",
    "message": "Invalid SP1 proof bundle"
  }
}
```

`500 Internal Server Error` - Server error:
```json
{
  "success": false,
  "error": {
    "code": "InternalServerError",
    "message": "Failed to queue job"
  }
}
```

**Notes:**
- The nullifier is NOT checked for duplicates at submission time
- Duplicate checking happens on-chain during transaction execution
- Jobs are queued in Redis for background processing
- Use `GET /status/:request_id` to track job progress

**Reference:** `services/relay/src/api/withdraw.rs`

### GET `/status/:id`

Check the status of a withdraw job.

**Path Parameters:**
- `id`: UUID of the request (from POST /withdraw response)

**Example Request:**
```
GET /status/550e8400-e29b-41d4-a716-446655440000
```

**Success Response (200 OK) - Queued:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "queued",
  "tx_id": null,
  "error": null,
  "created_at": "2024-01-01T12:00:00.000Z",
  "completed_at": null
}
```

**Success Response (200 OK) - Processing:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "processing",
  "tx_id": null,
  "error": null,
  "created_at": "2024-01-01T12:00:00.000Z",
  "completed_at": null
}
```

**Success Response (200 OK) - Completed:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "tx_id": "5j7s...signature...xyz",
  "error": null,
  "created_at": "2024-01-01T12:00:00.000Z",
  "completed_at": "2024-01-01T12:00:05.234Z"
}
```

**Success Response (200 OK) - Failed:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "failed",
  "tx_id": null,
  "error": "Nullifier already used on-chain",
  "created_at": "2024-01-01T12:00:00.000Z",
  "completed_at": "2024-01-01T12:00:03.456Z"
}
```

**Error Response (404 Not Found):**
```json
{
  "success": false,
  "error": {
    "code": "NotFound",
    "message": "Job not found"
  }
}
```

**Status Values:**
- `queued` - Job is waiting in Redis queue
- `processing` - Worker is currently processing the job
- `completed` - Transaction confirmed on-chain successfully
- `failed` - Job failed (see `error` field for details)

**Reference:** `services/relay/src/api/status.rs`

## Validator Agent Endpoints

Advanced API for validator operations and external integrations.

**Full Documentation:** See [`validator-agent.md`](./validator-agent.md) for complete specifications.

### POST `/jobs/withdraw`

Create a structured withdraw job (validator-agent format).

**Brief Overview:**
- Alternative to `POST /withdraw` with more control
- Structured job format with deadlines and fee caps
- Returns job ID for tracking

**See:** [`validator-agent.md`](./validator-agent.md#post-jobswithdraw)

**Reference:** `services/relay/src/api/validator_agent.rs`

### GET `/jobs/:job_id`

Fetch job details and artifacts.

**Path Parameters:**
- `job_id`: UUID of the job

**Brief Overview:**
- Returns job status and metadata
- Includes proof artifacts when job is done
- Status values: `queued`, `running`, `done`, `failed`

**See:** [`validator-agent.md`](./validator-agent.md#get-jobsjob_id)

**Reference:** `services/relay/src/api/validator_agent.rs`

### POST `/submit`

Submit a pre-signed Solana transaction for broadcast.

**Request Body:**
```json
{
  "tx_bytes_base64": "base64_encoded_transaction"
}
```

**Success Response (200 OK):**
```json
{
  "signature": "transaction_signature_base58",
  "slot": 123456
}
```

**Use Case:** For workflows where transaction signing happens externally.

**See:** [`validator-agent.md`](./validator-agent.md#post-submit)

**Reference:** `services/relay/src/api/validator_agent.rs`

## Orchestration API

### POST `/orchestrate/withdraw`

Higher-level API that coordinates multiple operations in a single call.

**Purpose:**
- Wraps validation, queueing, and submission
- Coordinates proof verification and claim finding
- Returns orchestrated job ID for tracking

**Request Body:**
```json
{
  "outputs": [...],
  "policy": {...},
  "public_inputs": {...},
  "proof_bytes": "..."
}
```

**Success Response (200 OK):**
```json
{
  "job_id": "uuid",
  "status": "orchestrated",
  "message": "Withdraw orchestration started"
}
```

**Notes:**
- This is a higher-level abstraction over the standard withdraw API
- Provides additional coordination logic
- Useful for complex multi-step workflows

**Reference:** `services/relay/src/planner/orchestrator.rs`

## Development Helpers

### POST `/jobs/:job_id/prove-local`

Generate an SP1 proof locally using the host CLI (development/testing only).

**Path Parameters:**
- `job_id`: UUID of the job

**Purpose:**
- Development helper for testing without external proof generation
- Triggers local SP1 host CLI proof generation
- Should NOT be used in production

**Success Response (200 OK):**
```json
{
  "success": true,
  "message": "Local proof generation triggered",
  "job_id": "uuid"
}
```

**Production:** This endpoint should be disabled in production environments.

**Reference:** `services/relay/src/api/prove_local.rs`

## Error Responses

All error responses follow a consistent format:

**Standard Error Format:**
```json
{
  "success": false,
  "error": {
    "code": "ErrorCode",
    "message": "Human-readable error description"
  }
}
```

**Common HTTP Status Codes:**
- `400 Bad Request` - Invalid request data, validation errors
- `404 Not Found` - Job or resource not found
- `413 Payload Too Large` - Request body exceeds size limits
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server-side errors
- `503 Service Unavailable` - Service temporarily unavailable (DB/Redis down)

**Error Codes Reference:**

See `services/relay/src/error.rs` for detailed error types:

**Validation Errors (`400`):**
- `ValidationError` - Invalid request data
- `InvalidRequest` - Malformed request structure
- `InvalidProof` - SP1 proof verification failed
- `OutputsMismatch` - Outputs hash doesn't match
- `AmountMismatch` - Amount conservation violation

**Resource Errors (`404`):**
- `NotFound` - Job or resource not found
- `NullifierNotFound` - Nullifier not in database

**PoW Errors (`503`):**
- `NoClaimsAvailable` - No PoW claims available from miners
- `ClaimExpired` - Found claim has expired
- `ClaimConsumed` - Found claim already fully consumed

**On-Chain Errors (`500`):**
- `TransactionFailed` - Solana transaction failed
- `SimulationFailed` - Transaction simulation failed
- `RpcError` - Solana RPC error
- `NullifierAlreadyUsed` - Nullifier exists on-chain

**Server Errors (`500`):**
- `InternalServerError` - Generic server error
- `DatabaseError` - Database operation failed
- `QueueError` - Redis queue operation failed

## Rate Limiting

**Current Status:** No rate limiting implemented on public endpoints.

**Future Considerations:**
- Per-IP rate limiting for `/withdraw` endpoint
- Per-user rate limiting (if authentication added)
- Queue depth limiting to prevent overload

## CORS

Cross-Origin Resource Sharing is configured permissively:

**Development:** All origins allowed (`*`)
**Production:** Configure specific allowed origins via environment

**Configuration:**
```toml
[server]
cors_origins = ["https://app.cloak.network", "https://wallet.example.com"]
```

## Authentication

**Current Status:** No authentication required for public endpoints.

**Future Considerations:**
- API key authentication for validator endpoints
- JWT tokens for user-specific operations
- Rate limiting tied to authenticated users

## Webhooks

**Current Status:** Not implemented.

**Future Considerations:**
- Webhook callbacks for job status changes
- Event notifications for completed/failed jobs
- Configurable webhook URLs per job

## Related Documentation

- **[Relay Service](../offchain/relay.md)** - Service architecture and deployment
- **[Validator Agent API](./validator-agent.md)** - Advanced API specifications
- **[Operations Guide](../operations/runbook.md)** - Production operations
- **[Withdraw Workflow](../workflows/withdraw.md)** - End-to-end withdraw flow
- **[PoW Withdraw](../workflows/pow-withdraw.md)** - PoW-gated withdrawals
- **[Cloak Miner](../packages/cloak-miner.md)** - Mining documentation
