---
title: Relay API Reference
description: REST endpoints exposed by the Cloak relay for withdraw jobs, validator agents, and health checks.
---

# Relay API Reference

Base URL: `http://<host>:3002`

## Service Metadata

- **GET `/`** → Service descriptor with version and available endpoints.
- **GET `/health`** → `{ "status": "ok", "timestamp": ISO8601 }`.

## Withdraw Submission

- **POST `/withdraw`**
  - Request body
    ```json
    {
      "outputs": [
        { "recipient": "base58", "amount": 1000000 }
      ],
      "policy": { "fee_bps": 50 },
      "public_inputs": {
        "root": "hex32",
        "nf": "hex32",
        "amount": 1000000,
        "fee_bps": 50,
        "outputs_hash": "hex32"
      },
      "proof_bytes": "base64(SP1 bundle)"
    }
    ```
  - Response: `{ "success": true, "data": { "request_id": "uuid", "status": "queued" } }`

- **GET `/status/:request_id`**
  - Response body
    ```json
    {
      "success": true,
      "data": {
        "request_id": "uuid",
        "status": "completed",
        "tx_id": "optional signature",
        "error": null
      }
    }
    ```

## Validator Agent Endpoints

See [`validator-agent.md`](./validator-agent.md) for detailed schema. Summary:

- **POST `/jobs/withdraw`** – Structured job submission.
- **GET `/jobs/:job_id`** – Fetch current state.
- **POST `/submit`** – Submit signed transaction payloads.
- **POST `/jobs/:job_id/prove-local`** – Development endpoint to trigger local proof generation.

## Orchestration

- **POST `/orchestrate/withdraw`** – Planner entrypoint that wraps validation, queueing, and submission. Returns orchestrated job ID.

## Error Format

Errors follow

```json
{
  "success": false,
  "error": {
    "code": "string",
    "message": "human readable"
  }
}
```

Specific error codes come from `services/relay/src/error.rs` (e.g., `InvalidRequest`, `NoClaimsAvailable`, `SimulationFailed`).
