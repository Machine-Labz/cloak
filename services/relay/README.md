# Relay Service

The Relay service is a critical component of the Cloak protocol, responsible for securely processing withdrawal requests and submitting them to the Solana blockchain.

## Features

- **REST API** for submitting and checking withdrawal requests
- **Asynchronous Processing** with job queue
- **Robust Error Handling** with clear error messages
- **Metrics and Monitoring** with Prometheus integration
- **Configuration Management** with environment variable support
- **Input Validation** to ensure data integrity
- **Idempotent Operations** to prevent duplicate processing

## API Endpoints

### `POST /withdraw`

Submit a new withdrawal request.

**Request Body:**
```json
{
  "proof": "base58-encoded-256-byte-proof",
  "public_inputs": "base58-encoded-64-byte-public-inputs",
  "outputs": [
    {
      "recipient": "base58-encoded-recipient-address",
      "amount": 1000
    }
  ],
  "fee_bps": 10
}
```

**Response (202 Accepted):**
```json
{
  "success": true,
  "data": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "queued"
  }
}
```

### `GET /status/:request_id`

Check the status of a withdrawal request.

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "tx_id": "5F1LhTohUMJzVFs3pFw58j5G4d5uGYHddiTb5C2L5E5X"
  }
}
```

### `GET /health`

Health check endpoint.

**Response (200 OK):**
```
OK
```

### `GET /metrics`

Prometheus metrics endpoint.

## Configuration

Configuration is managed through environment variables with the `RELAY_` prefix. Example `.env` file:

```env
# Server configuration
RELAY_SERVER__PORT=3000
RELAY_SERVER__HOST=0.0.0.0
RELAY_SERVER__REQUEST_TIMEOUT_SECONDS=30

# Solana configuration
RELAY_SOLANA__RPC_URL=http://localhost:8899
RELAY_SOLANA__WS_URL=ws://localhost:8900
RELAY_SOLANA__COMMITMENT=confirmed
RELAY_SOLANA__PROGRAM_ID=11111111111111111111111111111111
RELAY_SOLANA__MAX_RETRIES=3
RELAY_SOLANA__RETRY_DELAY_MS=1000

# Database configuration
RELAY_DATABASE__URL=postgres://postgres:postgres@localhost:5432/relay
RELAY_DATABASE__MAX_CONNECTIONS=5

# Metrics configuration
RELAY_METRICS__ENABLED=true
RELAY_METRICS__PORT=9090
RELAY_METRICS__ROUTE=/metrics
```

## Development

### Prerequisites

- Rust 1.70 or later
- PostgreSQL (for job queue and state)
- Solana test validator (for local development)

### Building

```bash
# Build in release mode
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Running with Docker

```bash
docker build -t cloak-relay .
docker run -p 3000:3000 --env-file .env cloak-relay
```

## Architecture

The relay service follows a clean architecture with the following components:

- **API Layer**: Handles HTTP requests and responses
- **Service Layer**: Implements business logic
- **Repository Layer**: Manages data access
- **Infrastructure**: Configuration, logging, and metrics

## Testing

Run the test suite:

```bash
cargo test
```

For integration tests that require a database, you'll need to have a PostgreSQL instance running.

## Monitoring

The service exposes Prometheus metrics at `/metrics` which can be scraped by a Prometheus server. Key metrics include:

- HTTP request counts and durations
- Withdrawal request statuses
- Queue sizes and processing times
- Error rates

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

# Core Functions of the Relayer

At a minimum, the Relayer has to handle these jobs:

1. API Layer
- Expose POST /withdraw and GET /status/:id.
- Accept user-submitted proofs + public inputs.
- Return requestId, eventual txid, and status (queued, executing, settled, failed).

2. Validation Layer (off-chain pre-checks, fast fail)
- Proof format check: lengths (proof=256B, sp1_public=64B).
- Public inputs check:
    - root must match /merkle/root.
    - outputs_hash recomputed from recipients matches declared.
    - Conservation: sum(outputs) + fee == amount.
- Nullifier check: reject duplicate nullifiers early.

3. Transaction Builder
- Construct the raw Solana instruction for withdraw (with Pinocchio layout).
- Add compute budget instruction.
- Add priority fee (configurable).
- Sign and bundle into a transaction.

4. Submission + Retry Logic
- Send tx via RPC.
- Handle transient errors (BlockhashNotFound, Node is behind).
- Retry with backoff.
- Update job state in DB/queue.

5. Result Tracking
- Poll transaction status until finalized.
- Mark job settled with txid or failed with error reason.

6. Observability + Ops
- Metrics: tx throughput, success/fail ratio, avg latency.
- Logging: structured logs (never store private inputs).
- Alerts if backlog > threshold.

## Integration with the Rest of the System
- Frontend / Prover
    - Users generate SP1 proof locally, then call POST /withdraw.
    - They never directly submit to Solana; Relayer handles that.

- Indexer
    - Relayer queries /merkle/root before building tx.
    - Indexer ensures roots are already pushed on-chain, so Relayer’s root always matches.

- On-chain Program (Pinocchio)
    - Relayer submits instruction with: proof[256] + sp1_public[64] + root + nf + amount + fee_bps + outputs_hash + outputs[].
    - Program verifies SP1, checks nullifier, updates state, and executes transfers.

## Performance Requirements
1. Latency
    - User expectation: withdraws settle within ~1–2 slots (0.5s–1s).
    - Relayer should validate + build tx in <100ms.
    - The bottleneck is Solana block production, not Relayer compute.

2. Throughput
    - Even modest hardware can handle thousands of withdraw requests per second (proof verification happens on-chain, not in Relayer).
    - Bottleneck is Solana TPS + RPC capacity, not CPU.

3. Reliability
    - Must be idempotent: the same withdraw job should never submit twice.
    - Needs persistence (DB/queue) so jobs survive restarts.

4. Scaling model
    - Horizontally scalable: multiple Relayer instances can run if they coordinate on nullifier deduplication and job IDs.
    - Stateless API layer, stateful DB backing.

## TL;DR
- The Relayer is basically a secure transaction proxy: validates, builds, and submits proofs to Solana.
- It doesn’t prove anything — proofs are generated client-side, verified on-chain.
- Performance requirements are modest (ms-level validation, single tx submission), but reliability and correctness are critical.
- Integrates tightly with Indexer (roots) and Program (proof verification + state updates).