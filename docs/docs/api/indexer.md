---
title: Indexer API Reference
description: HTTP endpoints exposed by the Cloak indexer service with request/response schemas.
---

# Indexer API Reference

Base URL: `http://<host>:3001`

## Health

- **GET `/health`** → `{ "status": "ok", "timestamp": ISO8601 }`

## Deposits & Notes

- **POST `/api/v1/deposit`**
  - Request body
    ```json
    {
      "leafCommit": "hex-commitment",
      "encryptedOutput": "base64",
      "txSignature": "optional signature",
      "slot": 12345
    }
    ```
  - Response: `202 Accepted` on success.

- **GET `/api/v1/notes/range?start=<u64>&end=<u64>&limit=<u16>`**
  - Returns encrypted outputs in the specified range.
  - Response body
    ```json
    {
      "notes": [
        {
          "leafCommit": "...",
          "encryptedOutput": "...",
          "slot": 12345
        }
      ],
      "nextStart": 43
    }
    ```

## Merkle Tree

- **GET `/api/v1/merkle/root`** → `{ "root": "hex", "nextIndex": 42 }`
- **GET `/api/v1/merkle/proof/:index`**
  - Response body
    ```json
    {
      "pathElements": ["hex32", ...],
      "pathIndices": [0, 1, ...],
      "leaf": "hex32",
      "root": "hex32"
    }
    ```

## Artifacts

- **GET `/api/v1/artifacts/withdraw/:version`** → lists available filenames (proof params, wasm, etc.).
- **GET `/api/v1/artifacts/files/:version/:filename`** → streams the binary artifact.

## Proof Generation (Optional)

- **POST `/api/v1/prove`** (rate limited)
  - Request body matches SP1 CLI input (private/public JSON, outputs array).
  - Response returns proof bundle + public inputs.

## Admin Endpoints (Development)

- **POST `/api/v1/admin/push-root`** → `{ "root": "hex32" }`
- **POST `/api/v1/admin/insert-leaf`** → Manually append commitment for testing.
- **POST `/api/v1/admin/reset`** → Clears database state.

## Error Format

Errors follow:

```json
{
  "error": {
    "code": "string",
    "message": "human readable"
  }
}
```

Refer to [`services/indexer/src/error.rs`](https://github.com/cloak-labz/cloak/blob/main/services/indexer/src/error.rs) for detailed codes.
