# Worker Implementation for Relay Service

## Problem

The relay service was accepting withdraw requests and successfully enqueueing jobs into Redis, but jobs were stuck in "queued" status indefinitely. After investigating, the root cause was identified:

**The relay service only ran the HTTP API server but had no worker task to actually dequeue and process jobs from the Redis queue.**

## Solution

Implemented a complete worker module that:

1. **Continuously polls the Redis queue** for new jobs
2. **Dequeues jobs** and processes them asynchronously
3. **Updates job status** through the database (queued → processing → completed/failed)
4. **Handles retries** with exponential backoff for failed jobs
5. **Moves permanently failed jobs** to a dead letter queue
6. **Runs concurrently** with the HTTP API server using `tokio::spawn`

## Architecture

### Components

#### 1. Worker Module (`services/relay/src/worker/mod.rs`)

The main worker loop that:
- Polls the Redis queue at configurable intervals (default: 1 second)
- Spawns async tasks for each dequeued job
- Limits concurrent job processing (default: 10 jobs)
- Uses a semaphore to prevent resource exhaustion

```rust
pub struct Worker {
    state: AppState,
    poll_interval: Duration,
    max_concurrent_jobs: usize,
}
```

**Configuration:**
- `poll_interval`: How often to check for new jobs (default: 1s)
- `max_concurrent_jobs`: Maximum number of jobs processing simultaneously (default: 10)

#### 2. Processor Module (`services/relay/src/worker/processor.rs`)

Handles individual job processing:

**Job Processing Flow:**
1. **Fetch job** from database by ID
2. **Check status** - skip if already completed/failed
3. **Update status to "processing"**
4. **Process withdraw transaction** (currently a placeholder)
5. **On success:**
   - Update job status to "completed"
   - Store nullifier to prevent double-spending
6. **On failure:**
   - If retries remain: requeue with exponential backoff
   - If max retries exceeded: mark as failed and move to dead letter queue

**Retry Logic:**
- Max retries: 3 (configurable)
- Backoff: 30s × 2^retry_count (e.g., 30s → 60s → 120s)
- Failed jobs after max retries → dead letter queue

### Integration with Main Service

In `services/relay/src/main.rs`, the worker is spawned as a background task:

```rust
// Spawn the worker task to process jobs
let worker_state = app_state.clone();
tokio::spawn(async move {
    let worker = worker::Worker::new(worker_state)
        .with_poll_interval(std::time::Duration::from_secs(1))
        .with_max_concurrent_jobs(10);
    
    worker.run().await;
});
```

This runs **concurrently** with the HTTP server, so:
- HTTP API accepts new withdraw requests
- Worker processes queued jobs in the background

## Current Status

### ✅ Implemented

- [x] Worker polling loop
- [x] Job dequeuing from Redis
- [x] Job status updates (queued → processing → completed/failed)
- [x] Retry logic with exponential backoff
- [x] Dead letter queue for failed jobs
- [x] Nullifier storage to prevent double-spending
- [x] Concurrent job processing with semaphore
- [x] Integration with main service
- [x] Error handling and logging

### 🚧 TODO

- [ ] **Actual Solana transaction building** in `process_withdraw()`
  - Currently returns a mock signature
  - Need to integrate with Solana client
  - Build withdraw instruction with proof and public inputs
  - Sign with relay keypair
  - Submit to Solana and wait for confirmation
  
- [ ] Configuration from environment variables
  - Poll interval
  - Max concurrent jobs
  - Retry settings

- [ ] Metrics and monitoring
  - Job processing times
  - Success/failure rates
  - Queue size monitoring

- [ ] Graceful shutdown
  - Wait for in-flight jobs to complete
  - Handle SIGTERM/SIGINT

## Testing

To test the worker:

1. **Start the relay service:**
   ```bash
   cd services/relay
   cargo run
   ```

2. **Submit a withdraw request:**
   ```bash
   curl -X POST http://localhost:3002/withdraw \
     -H "Content-Type: application/json" \
     -d @test_withdraw.json
   ```

3. **Check the logs:**
   You should see:
   - ✅ Job enqueued
   - 📦 Job dequeued by worker
   - 🔄 Processing job
   - ✅ Job completed (or retry messages if it fails)

4. **Check job status:**
   ```bash
   curl http://localhost:3002/status/{request_id}
   ```

## Logs

The worker emits detailed logs at each stage:

```
🚀 Worker started
   Poll interval: 1s
   Max concurrent jobs: 10
   
📦 Dequeued job: <job-id>
🔄 Processing job: <job-id>
   Request ID: <request-id>
   Retry count: 0
   
📝 Job <job-id> status updated to processing
🔐 Building withdraw transaction for job <job-id>
📦 Processing withdraw transaction
   Nullifier: <hex>
   Root hash: <hex>
   Amount: <amount>
   
✅ Job <job-id> completed successfully
   Transaction signature: <signature>
```

## Next Steps

The **critical next step** is implementing the actual Solana transaction submission in `process_withdraw()`:

1. Parse proof bytes and public inputs from the job
2. Build a Solana transaction with the withdraw instruction
3. Sign it with the relay's keypair
4. Submit to Solana RPC
5. Wait for confirmation
6. Return the real transaction signature

This requires integrating with:
- `solana-client` for transaction submission
- Your shield-pool program's withdraw instruction
- The relay keypair for signing

## Summary

This implementation resolves the "jobs stuck in queued" issue by adding a worker component that actively processes jobs from the Redis queue. Jobs now flow through the complete lifecycle: queued → processing → completed/failed, with proper retry handling and error recovery.

