---
title: Validator Agent API
description: Contract for validator-side automation to create withdraw jobs, fetch artifacts, and submit transactions.
---

# Validator Agent API

The validator agent API is a structured interface for automation workflows (validators, operators, or scripts) to interact with the relay. It mirrors the OpenAPI spec in `docs/api/validator-agent.yaml`.

Base URL examples:

- Production: `https://validator.example.com`
- Local development: `http://localhost:3003`

## POST `/jobs/withdraw`

Queues a withdraw job.

- **Request Body**
  ```json
  {
    "public_bin_hex": "<104-byte hex>",
    "outputs": [
      {
        "address_hex32": "hex32",
        "amount_u64": "1000000"
      }
    ],
    "deadline_iso": "2025-01-01T12:00:00Z",
    "payer_hints": {
      "use_jito": true,
      "bundle_tip_lamports": "5000000",
      "cu_limit": 1000000
    },
    "fee_caps": {
      "max_priority_fee_lamports": "2000000",
      "max_total_fee_lamports": "5000000"
    }
  }
  ```
- **Responses**
  - `202 Accepted` → `{ "job_id": "uuid", "status": "queued" }`
  - `400 Bad Request` with validation errors.
  - `413 Payload Too Large` when body exceeds limits (~64 KiB).
  - `429 Too Many Requests` with `Retry-After` header when rate limited.

## GET `/jobs/{job_id}`

Fetches job status and (once ready) artifacts.

- **Response**
  ```json
  {
    "job_id": "uuid",
    "status": "running",
    "artifacts": {
      "proof_hex_260": "...",
      "public_bin_hex_104": "...",
      "tx_bytes_base64": "..."
    },
    "error": null
  }
  ```
- Status values: `queued`, `running`, `done`, `failed`.
- `artifacts` appear when job reaches `done`.

## POST `/submit`

Submits a signed Solana transaction (base64) for broadcast.

- **Request Body** `{ "tx_bytes_base64": "..." }`
- **Response** `{ "signature": "base58", "slot": 123456 }`
- Errors:
  - `400` for malformed base64/transaction.
  - `429` when hitting rate limits.
  - `500` on relay failures.

## Local Proof Endpoint

- **POST `/jobs/{job_id}/prove-local`** – Development helper to trigger local proof generation via the SP1 host CLI.

## Schema Highlights

- **`public_bin_hex`** – 104-byte hex string: `root || nf || outputs_hash || amount_le`.
- **`proof_hex_260`** – 260-byte Groth16 proof (520 hex chars).
- **`tx_bytes_base64`** – Fully built, unsigned transaction bytes for manual signing.
- **`fee_caps`** – Optional guardrails on priority and total fees (string-encoded u64).
- **`payer_hints.use_jito`** – Toggle for Jito bundle submission.

Refer to [`validator-agent.yaml`](./validator-agent.yaml) for exhaustive schema definitions, rate-limit headers, and error models.
