# Relay Service Architecture

## Overview

The relay service is now a multi-component system that processes private withdraw transactions for the Cloak protocol.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        RELAY SERVICE                            │
│                                                                 │
│  ┌───────────────────┐              ┌────────────────────────┐ │
│  │                   │              │                        │ │
│  │   HTTP API        │              │     Worker Loop       │ │
│  │   (Port 3002)     │              │                        │ │
│  │                   │              │  • Polls every 1s      │ │
│  │  POST /withdraw   │              │  • Max 10 concurrent   │ │
│  │  GET  /status/:id │              │  • Spawned at startup  │ │
│  │  GET  /health     │              │                        │ │
│  │                   │              │                        │ │
│  └─────────┬─────────┘              └───────────┬────────────┘ │
│            │                                    │              │
│            │ enqueue()                dequeue() │              │
│            ▼                                    ▼              │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    Redis Queue                            │ │
│  │                                                            │ │
│  │  • Main Queue:     cloak:relay:jobs                      │ │
│  │  • Retry Queue:    cloak:relay:retry                     │ │
│  │  • Dead Letter:    cloak:relay:dlq                       │ │
│  │  • Processing:     cloak:relay:processing                │ │
│  └──────────────────────────────────────────────────────────┘ │
│                               ▲                                │
│                               │                                │
│                               │ update status                  │
│                               │                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                 PostgreSQL Database                       │ │
│  │                                                            │ │
│  │  Tables:                                                  │ │
│  │  • jobs         - Job records & status                    │ │
│  │  • nullifiers   - Spent nullifiers (anti-double-spend)   │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                               │
                               │ submit transaction
                               ▼
                    ┌─────────────────────┐
                    │   Solana Network    │
                    │                     │
                    │  Shield Pool Program│
                    └─────────────────────┘
```

## Data Flow

### 1. Withdraw Request (User → API)

```
User
  │
  ├─► POST /withdraw
  │   {
  │     "proof": "<base64>",
  │     "public_inputs": "<base64>",
  │     "outputs": {...},
  │     "fee_bps": 100
  │   }
  │
  └─► HTTP API
       │
       ├─► Create job in database (status: queued)
       │
       ├─► Enqueue job message to Redis
       │   {
       │     "job_id": "<uuid>",
       │     "request_id": "<uuid>",
       │     "priority": 100,
       │     "retry_count": 0
       │   }
       │
       └─► Return 202 Accepted
           {
             "request_id": "<uuid>",
             "status": "queued"
           }
```

### 2. Job Processing (Worker)

```
Worker Loop (background task)
  │
  ├─► Poll Redis queue (every 1s)
  │
  ├─► Dequeue job message
  │
  ├─► Spawn async task (semaphore-controlled)
  │    │
  │    ├─► Fetch job from database
  │    │
  │    ├─► Update status: queued → processing
  │    │
  │    ├─► Build Solana transaction
  │    │    • Parse proof & public inputs
  │    │    • Create withdraw instruction
  │    │    • Sign with relay keypair
  │    │
  │    ├─► Submit to Solana
  │    │
  │    ├─► On Success:
  │    │    ├─► Update status: processing → completed
  │    │    ├─► Store transaction signature
  │    │    └─► Save nullifier (prevent double-spend)
  │    │
  │    └─► On Failure:
  │         ├─► If retries remain:
  │         │    ├─► Increment retry_count
  │         │    ├─► Requeue with delay (30s × 2^retry_count)
  │         │    └─► Update status: processing → queued
  │         │
  │         └─► If max retries exceeded:
  │              ├─► Update status: processing → failed
  │              ├─► Move to dead letter queue
  │              └─► Store error message
  │
  └─► Continue polling
```

### 3. Status Check (User → API)

```
User
  │
  └─► GET /status/{request_id}
       │
       └─► Query database for job
            │
            └─► Return job status
                {
                  "request_id": "<uuid>",
                  "status": "completed|processing|queued|failed",
                  "tx_id": "<signature>",
                  "created_at": "<timestamp>",
                  "completed_at": "<timestamp>"
                }
```

## Job State Machine

```
           ┌─────────┐
           │ QUEUED  │ ◄──────────────┐
           └────┬────┘                │
                │                     │
                │ worker dequeues     │ retry
                │                     │
                ▼                     │
         ┌─────────────┐              │
         │ PROCESSING  │──────────────┘
         └──────┬──────┘
                │
       ┌────────┴────────┐
       │                 │
       ▼                 ▼
  ┌──────────┐      ┌────────┐
  │COMPLETED │      │ FAILED │
  └──────────┘      └────────┘
```

## Components

### 1. HTTP API Server
- **File:** `services/relay/src/main.rs`
- **Port:** 3002
- **Endpoints:**
  - `POST /withdraw` - Submit withdraw request
  - `GET /status/:id` - Check job status
  - `GET /health` - Health check

### 2. Worker
- **File:** `services/relay/src/worker/mod.rs`
- **Function:** Background job processor
- **Runs:** Concurrently with HTTP server via `tokio::spawn`

### 3. Processor
- **File:** `services/relay/src/worker/processor.rs`
- **Function:** Individual job processing logic
- **Handles:** Transaction building, submission, retries

### 4. Redis Queue
- **File:** `services/relay/src/queue/redis_queue.rs`
- **Queues:**
  - `cloak:relay:jobs` - Main job queue
  - `cloak:relay:retry` - Retry queue (with delays)
  - `cloak:relay:dlq` - Dead letter queue
  - `cloak:relay:processing` - In-flight jobs

### 5. Database Repository
- **File:** `services/relay/src/db/repository.rs`
- **Operations:**
  - Job CRUD
  - Status updates
  - Nullifier management

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/relay

# Redis
REDIS_URL=redis://localhost:6379

# Solana
SOLANA_RPC_URL=https://api.devnet.solana.com
RELAY_KEYPAIR_PATH=/path/to/keypair.json

# Worker (future)
WORKER_POLL_INTERVAL_MS=1000
WORKER_MAX_CONCURRENT_JOBS=10
WORKER_MAX_RETRIES=3
```

### Worker Settings

Currently hardcoded in `main.rs`:
```rust
.with_poll_interval(Duration::from_secs(1))
.with_max_concurrent_jobs(10)
```

Max retries hardcoded in `processor.rs`:
```rust
let max_retries = 3;
```

## Performance Characteristics

### Throughput
- **Max concurrent jobs:** 10 (configurable)
- **Poll interval:** 1 second
- **Theoretical max:** ~10 jobs/second (assuming 1s per job)

### Latency
- **Queue latency:** < 1 second (poll interval)
- **Processing latency:** Depends on Solana confirmation time (~400-800ms)
- **Total latency:** ~1-2 seconds per withdraw

### Retry Strategy
- **Initial retry delay:** 30 seconds
- **Backoff:** Exponential (2^retry_count)
- **Max retries:** 3
- **Total max delay:** 30s + 60s + 120s = 210s (3.5 minutes)

## Error Handling

### Job Failures
1. **Transient errors** → Retry with backoff
2. **Permanent errors** → Dead letter queue
3. **All errors** → Logged with context

### System Failures
- **Database down** → Worker logs error, continues polling
- **Redis down** → Worker logs error, retries connection
- **Solana RPC down** → Job fails, retries with backoff

## Monitoring

### Logs
- Worker lifecycle events (start, dequeue, process)
- Job status transitions
- Error details with context
- Transaction signatures

### Metrics (TODO)
- Jobs processed per minute
- Average processing time
- Success/failure rates
- Queue depth
- Retry counts

## Future Enhancements

1. **Solana Integration**
   - [ ] Implement actual transaction building
   - [ ] Keypair management
   - [ ] RPC retry logic
   - [ ] Transaction confirmation tracking

2. **Configuration**
   - [ ] Move hardcoded values to environment variables
   - [ ] Runtime configuration updates

3. **Monitoring**
   - [ ] Prometheus metrics
   - [ ] Grafana dashboards
   - [ ] Alerting

4. **Resilience**
   - [ ] Graceful shutdown
   - [ ] Circuit breakers for external services
   - [ ] Rate limiting per user

5. **Optimization**
   - [ ] Batch processing for related jobs
   - [ ] Priority queue for urgent withdrawals
   - [ ] Dynamic worker scaling

