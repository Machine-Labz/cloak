---
title: Indexer API Reference
description: HTTP endpoints exposed by the Cloak indexer service with request/response schemas.
---

# Indexer API Reference

**Base URL:** `http://<host>:3001` (default: `http://localhost:3001`)

**Version:** Provided in all responses via `X-Service-Version` header

## General Information

All responses use `application/json` content type unless otherwise specified.

Timestamps follow ISO 8601 format (e.g., `2024-01-01T12:00:00.000Z`).

Binary data is encoded as:
- Hex strings for hashes and commitments (64 characters = 32 bytes)
- Base64 for encrypted data

## Service Information

### GET `/`

Returns service metadata and available endpoints.

**Response:**
```json
{
  "name": "Cloak Indexer API",
  "version": "0.1.0",
  "description": "Merkle tree indexer for Cloak privacy protocol",
  "endpoints": {
    "health": "/health",
    "deposit": "/api/v1/deposit",
    "merkle_root": "/api/v1/merkle/root",
    "merkle_proof": "/api/v1/merkle/proof/:index",
    "notes_range": "/api/v1/notes/range",
    "artifacts": "/api/v1/artifacts/withdraw/:version"
  },
  "documentation": "https://docs.cloak.network/indexer",
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

**Reference:** `services/indexer/src/server/final_handlers.rs:42-63`

### GET `/health`

Health check endpoint with detailed service status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00.000Z",
  "database": {
    "healthy": true
  },
  "merkle_tree": {
    "initialized": true,
    "height": 32
  },
  "version": "0.1.0"
}
```

**Status Codes:**
- `200 OK` - Service is healthy and operational

**Reference:** `services/indexer/src/server/final_handlers.rs:65-78`

## Deposits & Notes

### POST `/api/v1/deposit`

Ingest a new deposit commitment and encrypted note output.

**Request Body:**
```json
{
  "leaf_commit": "0123456789abcdef...", // 64-char hex (32 bytes)
  "encrypted_output": "base64encodeddata...", // Base64 encrypted note
  "tx_signature": "txsig123...", // Optional: Solana transaction signature
  "slot": 12345 // Optional: Slot number
}
```

**Field Requirements:**
- `leaf_commit`: Required, exactly 64 hex characters (32 bytes)
- `encrypted_output`: Required, non-empty Base64 string
- `tx_signature`: Optional, auto-generated if not provided
- `slot`: Optional, auto-generated if not provided

**Success Response (201 Created):**
```json
{
  "success": true,
  "leafIndex": 42,
  "root": "new_merkle_root_hex...",
  "nextIndex": 43,
  "message": "Leaf inserted successfully"
}
```

**Error Responses:**

`400 Bad Request` - Invalid input:
```json
{
  "error": "Leaf commit must be 64 characters"
}
```

`500 Internal Server Error` - Database or tree error:
```json
{
  "error": "Failed to insert leaf into Merkle tree"
}
```

**Notes:**
- This endpoint is primarily for testing. Production deposits come from on-chain event ingestion.
- The leaf is immediately appended to the Merkle tree and stored in PostgreSQL.
- The new Merkle root is computed and returned.

**Reference:** `services/indexer/src/server/final_handlers.rs:80-210`

### GET `/api/v1/notes/range`

Retrieve encrypted note outputs in a specific range (for note scanning/discovery).

**Query Parameters:**
- `start`: Optional, starting leaf index (default: 0)
- `end`: Optional, ending leaf index (default: current max)
- `limit`: Optional, maximum number of results (default: 100, max: 1000)

**Example Request:**
```
GET /api/v1/notes/range?start=0&end=100&limit=50
```

**Success Response (200 OK):**
```json
{
  "notes": [
    {
      "leaf_index": 0,
      "leaf_commit": "hex64...",
      "encrypted_output": "base64...",
      "tx_signature": "txsig...",
      "slot": 1000,
      "created_at": "2024-01-01T12:00:00.000Z"
    },
    {
      "leaf_index": 1,
      "leaf_commit": "hex64...",
      "encrypted_output": "base64...",
      "tx_signature": "txsig...",
      "slot": 1001,
      "created_at": "2024-01-01T12:00:05.000Z"
    }
  ],
  "count": 2,
  "next_start": 2,
  "has_more": true
}
```

**Error Responses:**

`400 Bad Request` - Invalid parameters:
```json
{
  "error": "Invalid range: start must be less than end"
}
```

**Reference:** `services/indexer/src/server/final_handlers.rs` (note scanning handler)

## Merkle Tree

### GET `/api/v1/merkle/root`

Get the current Merkle root and next available leaf index.

**Success Response (200 OK):**
```json
{
  "root": "0123456789abcdef...", // 64-char hex (32 bytes)
  "nextIndex": 42
}
```

**Use Case:** Clients need the current root to create valid Merkle proofs for withdrawals.

**Reference:** `services/indexer/src/server/final_handlers.rs` (get_merkle_root handler)

### GET `/api/v1/merkle/proof/:index`

Generate a Merkle inclusion proof for a specific leaf index.

**Path Parameters:**
- `index`: Leaf index (non-negative integer)

**Example Request:**
```
GET /api/v1/merkle/proof/5
```

**Success Response (200 OK):**
```json
{
  "path_elements": [
    "hex64_sibling_at_level_0",
    "hex64_sibling_at_level_1",
    "hex64_sibling_at_level_2",
    // ... up to tree height (32 levels)
  ],
  "path_indices": [0, 1, 0, 1, 0, ...], // Left (0) or Right (1) at each level
  "leaf": "hex64_leaf_commitment",
  "root": "hex64_current_root"
}
```

**Error Responses:**

`404 Not Found` - Leaf index doesn't exist:
```json
{
  "error": "Leaf index 999 not found in tree"
}
```

`400 Bad Request` - Invalid index:
```json
{
  "error": "Invalid index parameter"
}
```

**Notes:**
- `path_elements` and `path_indices` have the same length (equal to tree height)
- Clients use this proof to demonstrate Merkle inclusion for withdrawals
- The proof is verified on-chain by the shield-pool program

**Reference:** `services/indexer/src/server/final_handlers.rs` (get_merkle_proof handler)

## Artifacts

### GET `/api/v1/artifacts/withdraw/:version`

List available SP1 verification artifacts for a specific circuit version.

**Path Parameters:**
- `version`: Circuit version (e.g., `v1`, `latest`)

**Example Request:**
```
GET /api/v1/artifacts/withdraw/v1
```

**Success Response (200 OK):**
```json
{
  "version": "v1",
  "files": [
    "vkey.bin",
    "proof_params.json",
    "circuit_metadata.json"
  ],
  "base_url": "/api/v1/artifacts/files/v1"
}
```

**Reference:** `services/indexer/src/artifacts.rs`

### GET `/api/v1/artifacts/files/:version/:filename`

Download a specific artifact file.

**Path Parameters:**
- `version`: Circuit version
- `filename`: File name from the artifacts list

**Example Request:**
```
GET /api/v1/artifacts/files/v1/vkey.bin
```

**Success Response (200 OK):**
- Content-Type: `application/octet-stream` (binary files)
- Content-Type: `application/json` (JSON metadata files)
- Response body: Raw file contents

**Error Responses:**

`404 Not Found` - File doesn't exist:
```json
{
  "error": "Artifact file not found"
}
```

**Use Case:** Clients download verification keys and proof parameters needed for SP1 proof generation.

**Reference:** `services/indexer/src/artifacts.rs`

## Proof Generation ⚠️ DEPRECATED

### POST `/api/v1/prove`

**⚠️ DEPRECATED:** This endpoint has been removed in favor of client-side proof generation.

**Status:** Returns `410 GONE`

**Response:**
```json
{
  "success": false,
  "error": "The /api/v1/prove endpoint has been deprecated.",
  "deprecation_notice": "Generate SP1 proofs in the client or wallet. Upload the SP1Stdin to the TEE proving service and submit the resulting proof to the relay.",
  "documentation": "https://docs.cloak.network/architecture/proving"
}
```

**Migration Path:**
1. Use `packages/zk-guest-sp1` host CLI to generate proofs locally
2. OR use SP1 network TEE proving service
3. Submit resulting proof to relay service via `POST /withdraw`

**Reference:** `services/indexer/src/server/prover_handler.rs`

## Admin Endpoints (Development Only)

**⚠️ WARNING:** These endpoints should be disabled or access-controlled in production environments.

### POST `/admin/push-root`

Manually push a Merkle root to the ring buffer (for testing).

**Request Body:**
```json
{
  "root": "hex64_merkle_root"
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "message": "Root pushed successfully"
}
```

### POST `/admin/insert-leaf`

Manually insert a leaf into the Merkle tree (for testing).

**Request Body:**
```json
{
  "leaf_commit": "hex64_commitment"
}
```

**Success Response (201 Created):**
```json
{
  "success": true,
  "leafIndex": 42,
  "root": "new_hex64_root"
}
```

### POST `/admin/reset`

**⚠️ DANGER:** Clears all database state. Cannot be undone!

**Request Body:** Empty or `{}`

**Success Response (200 OK):**
```json
{
  "success": true,
  "message": "Database reset successfully"
}
```

**Production:** This endpoint MUST be disabled in production.

## Error Responses

All error responses follow a consistent format:

**Standard Error Format:**
```json
{
  "error": "Human-readable error message"
}
```

**Common HTTP Status Codes:**
- `400 Bad Request` - Invalid input, validation errors
- `404 Not Found` - Resource not found (leaf index, artifact file)
- `410 Gone` - Deprecated endpoint
- `413 Payload Too Large` - Request body exceeds size limit
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server-side errors (database, tree operations)
- `503 Service Unavailable` - Service temporarily unavailable

**Error Codes Reference:**
See `services/indexer/src/error.rs` for detailed error types:
- `InvalidLeafCommit`
- `InvalidEncryptedOutput`
- `DatabaseError`
- `MerkleTreeError`
- `ArtifactNotFound`
- `ProofGenerationError`

## Rate Limiting

**Note:** The deprecated `/api/v1/prove` endpoint had rate limiting (3 requests/hour per client). Other endpoints currently have no rate limits but this may change in production.

## CORS

Cross-Origin Resource Sharing (CORS) is configured to allow requests from configured origins.

Default development: All origins allowed (`*`)
Production: Configure specific allowed origins via `server.cors_origins` in config.

## Related Documentation

- **[Indexer Service](../offchain/indexer.md)** - Service architecture and deployment
- **[Merkle Tree Design](../zk/merkle.md)** - Cryptographic details
- **[Operations Guide](../operations/runbook.md)** - Production operations
