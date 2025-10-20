---
title: Indexer Service
description: Rust Axum service that maintains the append-only Merkle tree, stores encrypted notes, and serves artifacts.
---

# Indexer Service

The indexer is a Rust/Axum microservice that ingests on-chain deposit events, maintains the Merkle tree, and exposes HTTP APIs for clients and relayers.

Source: [`services/indexer`](https://github.com/cloak-labz/cloak/tree/main/services/indexer)

## Responsibilities

- Subscribe to `shield-pool` deposit logs and append commitments to the Merkle tree.
- Persist encrypted outputs for client note discovery.
- Serve Merkle roots, inclusion proofs, and encrypted notes over HTTP.
- Host SP1 withdraw artifacts (proof parameters, verifying key bundles).
- Provide admin endpoints for root insertion and database reset (development only).

## Technology Stack

- **Axum** web framework with Tower HTTP middleware.
- **SQLx** for async PostgreSQL access (connection pooling, migrations).
- **Tokio** runtime, structured logging via `tracing`.
- **BLAKE3** for Merkle hashing.
- Optional in-process SP1 proof generation endpoint (`POST /api/v1/prove`).

## Configuration

Copy `env.example` to `.env` and adjust the following:

```env
DB_HOST=localhost
DB_PORT=5432
DB_NAME=cloak_indexer
DB_USER=cloak
DB_PASSWORD=secret
PORT=3001
TREE_HEIGHT=32
SOLANA_RPC_URL=http://127.0.0.1:8899
SHIELD_POOL_PROGRAM_ID=<program-id>
```

Set `RUST_LOG=info` to view structured logs.

## Core Endpoints (`/api/v1`)

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/health` | Service health check (uninstrumented). |
| `POST` | `/deposit` | Ingest commitment + encrypted output (primarily through WebSocket feed). |
| `GET` | `/merkle/root` | Returns `{ root, nextIndex }`. |
| `GET` | `/merkle/proof/:index` | Returns Merkle path for the specified index. |
| `GET` | `/notes/range?start=...&end=...&limit=...` | Paginates encrypted outputs. |
| `GET` | `/artifacts/withdraw/:version` | Lists available SP1 artifact bundle names. |
| `GET` | `/artifacts/files/:version/:filename` | Streams artifact files. |
| `POST` | `/prove` | (Optional) Generate an SP1 proof server-side with rate limiting.

### Admin Endpoints

Available in development builds to seed/testing state:

- `POST /admin/push-root`
- `POST /admin/insert-leaf`
- `POST /admin/reset`

## Internal Modules

- `config.rs` – Environment loader, strongly typed with defaults.
- `database/` – SQLx connection pool, migrations, and storage implementation (`PostgresTreeStorage`).
- `merkle.rs` – BLAKE3 tree implementation with dynamic height.
- `artifacts.rs` – File management for SP1 assets.
- `server/` – Routes, middleware (CORS, request size limit, timeouts), rate limiter, handlers.

## Running Locally

```bash
cd services/indexer
cp env.example .env
docker compose up -d postgres
cargo run
```

Use `curl http://localhost:3001/health` to verify it is running.

## Observability

- Logs include request spans via `TraceLayer` and custom middleware.
- `sqlx` warnings are emitted when queries exceed thresholds.
- Rate limiter prevents abusive proof generation (3/hour per client by default).

## Related Documentation

- [`RUNBOOK.md`](../operations/runbook.md) – Operational guidance for validators running shared indexers.
- [`Wildcard Mining Overview`](../pow/overview.md) – Integration with wildcard claims system.
