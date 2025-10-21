# Relay Service

The relay service is responsible for accepting withdraw requests with ZK proofs, validating them, and submitting transactions to the Solana network.

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│    API      │───▶│    Queue    │───▶│  Processor  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                           │                                     │
                           ▼                                     ▼
                   ┌─────────────┐                       ┌─────────────┐
                   │ PostgreSQL  │                       │   Solana    │
                   │  Database   │                       │   Network   │
                   └─────────────┘                       └─────────────┘
```

## Features

- **ZK Proof Validation**: Validates SP1 proofs for withdraw requests
- **Double-spend Prevention**: Tracks nullifiers to prevent double spending
- **Job Queue**: Database-backed job queue with retry logic and exponential backoff
- **Transaction Submission**: Submits withdraw transactions to Solana with confirmation
- **Observability**: Comprehensive metrics, logging, and health checks
- **Database Persistence**: PostgreSQL for job state and nullifier storage

## API Endpoints

### POST /withdraw

Submit a withdraw request with ZK proof.

**Request:**
```json
{
  "outputs": [
    {
      "recipient": "base58-encoded-pubkey",
      "amount": 1000000
    }
  ],
  "policy": {
    "fee_bps": 100
  },
  "public_inputs": {
    "root": "hex-encoded-root-hash",
    "nf": "hex-encoded-nullifier",
    "amount": 1000000,
    "fee_bps": 100,
    "outputs_hash": "hex-encoded-outputs-hash"
  },
  "proof_bytes": "base64-encoded-proof"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "request_id": "uuid",
    "status": "queued"
  }
}
```

### GET /status/:request_id

Get the status of a withdraw request.

**Response:**
```json
{
  "success": true,
  "data": {
    "request_id": "uuid",
    "status": "completed",
    "tx_id": "solana-transaction-id",
    "error": null
  }
}
```

### GET /health

Health check endpoint.

**Response:**
- `200 OK` - All services healthy
- `503 Service Unavailable` - One or more services unhealthy

## Configuration

Configuration is handled via environment variables or config files:

```bash
# Server
RELAY_SERVER__PORT=3002
RELAY_SERVER__HOST=0.0.0.0

# Database
RELAY_DATABASE__URL=postgres://postgres:postgres@localhost:5432/relay

# Solana
RELAY_SOLANA__RPC_URL=http://127.0.0.1:8899
RELAY_SOLANA__PROGRAM_ID=your-program-id
```

## Development Setup

### Prerequisites

- Rust 1.70+
- PostgreSQL 15+
- Docker & Docker Compose (optional)

### Quick Start

1. **Start dependencies with Docker:**
   ```bash
   cd services/relay
   docker-compose up -d
   ```

2. **Build and run:**
   ```bash
   cargo build
   cargo run
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

### Manual Setup

1. **Start PostgreSQL:**
   ```bash
   # Create database
   createdb relay
   
   # Migrations will run automatically on startup
   ```

2. **Configure environment:**
   ```bash
   export RELAY_DATABASE__URL=postgres://postgres:postgres@localhost:5432/relay
   export RELAY_SOLANA__RPC_URL=http://localhost:8899  # For local validator
   ```

## Job Processing

The relay uses a robust job processing system:

1. **Request Validation**: Validates proof format, public inputs, and business logic
2. **Job Creation**: Creates a job record in PostgreSQL
3. **Queue Enqueue**: Enqueues job in Postgres with retry metadata
4. **Background Processing**: Worker processes jobs asynchronously
5. **Transaction Submission**: Submits transactions to Solana with retries
6. **Status Updates**: Updates job status in database

### Job States

- `queued` - Job is waiting to be processed
- `processing` - Job is currently being processed
- `completed` - Job completed successfully
- `failed` - Job failed (max retries exceeded)
- `cancelled` - Job was cancelled

### Retry Logic

- **Exponential Backoff**: Delays increase exponentially (1s, 2s, 4s, ...)
- **Jitter**: Random jitter to prevent thundering herd
- **Max Retries**: Configurable maximum retry attempts (default: 3)
- **Dead Letter Queue**: Failed jobs moved to dead letter queue

## Validation

The service performs comprehensive validation:

### Request Validation
- Output format and addresses
- Fee bounds (0-10%)
- Proof length (256 bytes)
- Public inputs format

### Cryptographic Validation
- Proof format verification
- Nullifier uniqueness
- Root hash format

### Business Logic Validation
- Conservation check: `sum(outputs) + fee == amount`
- Outputs hash verification
- Double-spend prevention

## Observability

### Metrics

Prometheus metrics available at `/metrics`:
- Request counters and latencies
- Job processing metrics
- Queue size and processing time
- Database connection pool metrics
- Solana RPC call metrics

### Logging

Structured logging with tracing:
- Request/response logging
- Job lifecycle events
- Error tracking with context
- Performance metrics

### Health Checks

The `/health` endpoint checks:
- Database connectivity
- Solana RPC connectivity

## Security Considerations

- **Input Validation**: All inputs are thoroughly validated
- **SQL Injection Prevention**: Using parameterized queries
- **Rate Limiting**: TODO - Add rate limiting middleware
- **Error Handling**: Errors don't leak sensitive information

## Production Deployment

### Environment Variables

```bash
# Required
RELAY_DATABASE__URL=postgres://user:pass@host:5432/relay
RELAY_SOLANA__RPC_URL=https://api.mainnet-beta.solana.com
RELAY_SOLANA__PROGRAM_ID=your-mainnet-program-id

# Optional
RELAY_SERVER__PORT=3002
RELAY_METRICS__ENABLED=true
RUST_LOG=relay=info,warn
```

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/relay /usr/local/bin/relay
EXPOSE 3002
CMD ["relay"]
```

### Monitoring

Set up monitoring for:
- Application metrics (Prometheus + Grafana)
- Logs aggregation (ELK stack or similar)
- Database monitoring
- Solana network status

## Troubleshooting

### Common Issues

1. **Database connection failed**
   - Check PostgreSQL is running
   - Verify connection string
   - Check database exists

2. **Solana RPC errors**
   - Check RPC endpoint is accessible
   - Verify network (devnet/mainnet)
   - Check rate limits

3. **Job processing stuck**
   - Check worker is running
   - Check database job status

### Debug Commands

```bash
# Check database jobs
psql -d relay -c "SELECT status, COUNT(*) FROM jobs GROUP BY status;"

# Check recent errors
psql -d relay -c "SELECT request_id, error_message, created_at FROM jobs WHERE status = 'failed' ORDER BY created_at DESC LIMIT 10;"
```

## Development

### Adding New Validation Rules

1. Add validation logic to `src/validation/mod.rs`
2. Add tests for the new validation
3. Update documentation

### Adding New Metrics

1. Define metrics in `src/metrics.rs`
2. Instrument code with metric calls
3. Update Grafana dashboards

### Database Migrations

Migrations are in `migrations/` directory and run automatically on startup.

To create a new migration:
```bash
# Create migration file
touch migrations/XXX_description.sql

# Add your SQL
echo "ALTER TABLE jobs ADD COLUMN new_field TEXT;" > migrations/XXX_description.sql
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

See the LICENSE file in the project root.
