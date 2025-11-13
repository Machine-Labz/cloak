---
title: Relay Service
description: Axum service + background workers that validate withdraw jobs, integrate PoW claims, and submit Solana transactions.
---

# Relay Service

The relay orchestrates withdraw jobs from submission through on-chain execution. It exposes HTTP APIs for clients/validators, manages job state in PostgreSQL + Redis, coordinates SP1 artifacts, and interacts with the scramble registry for PoW.

Source: [`services/relay`](https://github.com/cloak-labz/cloak/tree/main/services/relay)

## Architecture

- **API Layer (Axum):** Routes defined under `src/api`. Includes withdraw submission, validator-agent endpoints, status queries, and local prove helpers.
- **Planner:** Higher-level orchestration entrypoints under `src/planner` that can orchestrate multi-step flows.
- **Queue:** `RedisJobQueue` implements the job queue trait with exponential backoff and dead-letter behaviour.
- **Worker:** Background task (`worker::Worker`) that polls Redis, executes jobs concurrently, and updates status.
- **Solana Service:** Encapsulates RPC interactions, transaction building, Jito integration, and PoW `ClaimFinder` wiring.
- **Database:** PostgreSQL repositories for jobs and nullifiers (`PostgresJobRepository`, `PostgresNullifierRepository`).

## Configuration

Primary configuration lives in `config.toml` (see `services/relay/config.toml` for sample) and environment variables. Key sections:

```toml
[server]
port = 3002
host = "0.0.0.0"

[database]
url = "postgres://postgres:postgres@localhost:5432/relay"

[redis]
url = "redis://localhost:6379"

[solana]
rpc_url = "http://127.0.0.1:8899"
program_id = "<shield-pool-program-id>"
scramble_registry_program_id = "<scramble-program-id>"
jito_url = "https://mainnet.block-engine.jito.wtf"  # optional

[pow]
enabled = true
```

Enable detailed logging with `RUST_LOG=info,tower_http=info`.

## API Surface

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/` | Service metadata. |
| `GET` | `/health` | Health check (ensures DB + Redis reachable). |
| `POST` | `/withdraw` | Submit withdraw request (proof bytes + public inputs). |
| `GET` | `/status/:id` | Retrieve job status (queued, processing, completed, failed). |
| `POST` | `/jobs/withdraw` | Validator agent entrypoint (structured job format). |
| `GET` | `/jobs/:job_id` | Validator agent job fetch. |
| `POST` | `/submit` | Submit signed transaction (validator agent). |
| `POST` | `/orchestrate/withdraw` | Planner-managed withdraw orchestration. |
| `POST` | `/jobs/:job_id/prove-local` | Generate proof locally using host CLI (development helper).

All requests/response shapes are documented in [`api/validator-agent.md`](../api/validator-agent.md).

## Job Lifecycle

1. **Validation:** Requests are deserialised, schema-validated, and checked for policy compliance (fee bounds, output count).
2. **Persistence:** Job inserted into PostgreSQL (`jobs` table). Nullifier pre-check prevents duplicates.
3. **Queueing:** Job enqueued in Redis with initial delay/backoff config.
4. **Processing:** Worker fetches job, recomputes hashes, fetches Merkle data, requests PoW claim, builds transaction.
5. **Submission:** Transaction simulated and broadcast. On success, status set to `completed`; otherwise `failed` with error message and optional retry.
6. **Nullifier Commit:** Nullifier recorded in Postgres to block duplicates ahead of on-chain insertion.

## PoW Integration

- `ClaimFinder` initialised when `scramble_registry_program_id` is present.
- Wildcard claims skip batch hash matching; non-wildcard claims require equality.
- Metrics include search durations (`Query complete`) and success/failure counts.

## Observability

- Structured tracing around each HTTP request and worker job.
- Metrics logged with `[METRICS]` prefix (see [Metrics Guide](../operations/metrics-guide.md)).
- PostgreSQL migrations run automatically on startup (see `migrations/`).

## Testing

```bash
cargo test -p relay
```

- Unit tests cover validation, planner logic, and queue behaviour.
- Integration tests under `tests/` exercise the API against test doubles (Postgres/Redis).

## Deployment Checklist

1. Provision PostgreSQL and Redis with appropriate credentials.
2. Configure Solana RPC endpoints (confirmed + finalized), plus optional Jito bundler.
3. Run `cargo run --bin migrate` if using the CLI migration binary.
4. Expose the HTTP server behind TLS/ingress as required.
5. Monitor job backlog, nullifier duplicates, and PoW claim availability.

Refer to the [Runbook](../operations/runbook.md) for operational SLOs and incident response.
