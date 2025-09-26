# Cloak Relay Service - Complete Running & Testing Guide

This guide provides **comprehensive instructions** for setting up, running, and testing the complete Cloak Relay Service with full PostgreSQL and Redis integration.

## Infrastructure Setup

### 1. Start Required Services
```bash
# Start PostgreSQL and Redis using Docker
docker compose up -d

# Verify services are running
docker compose ps

# Expected output:
# NAME            COMMAND                  SERVICE    STATUS
# relay_postgres  "docker-entrypoint.s…"   postgres   Up
# relay_redis     "docker-entrypoint.s…"   redis      Up
```

### 2. Service Health Checks
```bash
# Check PostgreSQL health
docker exec relay_postgres pg_isready -U postgres

# Check Redis health
docker exec relay_redis redis-cli ping

# Check Docker logs if issues
docker compose logs postgres
docker compose logs redis
```

## Running the Service

### Production Mode (Full Feature Set)

#### 1. Set Environment Variables
```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/relay"
export REDIS_URL="redis://localhost:6379"
export RUST_LOG=info
```

#### 2. Build and Run
```bash
# Build with database support
DATABASE_URL="postgres://postgres:postgres@localhost:5432/relay" cargo build

# Run the service
DATABASE_URL="postgres://postgres:postgres@localhost:5432/relay" \
REDIS_URL="redis://localhost:6379" \
RUST_LOG=info \
/home/$USER/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo run
```

#### 3. Verify Service Started
Look for these logs:
```
INFO relay: Starting Cloak Relay Service
INFO relay::db: Connecting to database: postgres://postgres:postgres@localhost:5432/relay
INFO relay::db: Database connection established
INFO relay::db: Running database migrations
INFO relay::db: Database migrations completed
INFO relay: Relay service listening on 0.0.0.0:3002
```

## Testing Guide

### 1. Basic Service Health

#### Test Service Availability
```bash
# Test basic connectivity
curl -s http://localhost:3002/

# Expected response:
{
  "service": "Cloak Relay",
  "version": "0.1.0",
  "status": "running",
  "endpoints": {
    "health": "GET /health",
    "withdraw": "POST /withdraw",
    "status": "GET /status/:id"
  }
}
```

#### Test Health Endpoint
```bash
# Health check with formatted output
curl -s http://localhost:3002/health

# Expected response:
{
  "status": "ok",
  "message": "Relay service is healthy",
  "timestamp": "2025-09-20T23:47:29.596421023+00:00"
}
```

### 2. Database Integration Testing

#### Verify Database Schema
```bash
# Connect to PostgreSQL and check tables
docker exec -it relay_postgres psql -U postgres -d relay -c "\dt"

# Expected output:
#          List of relations
#  Schema |   Name    | Type  | Owner 
# --------+-----------+-------+---------
#  public | jobs      | table | postgres
#  public | nullifiers| table | postgres
```

#### Check Database Structure
```bash
# Describe jobs table
docker exec -it relay_postgres psql -U postgres -d relay -c "\d jobs"

# Describe nullifiers table  
docker exec -it relay_postgres psql -U postgres -d relay -c "\d nullifiers"

# Check job status enum
docker exec -it relay_postgres psql -U postgres -d relay -c "\dT+ job_status"
```

### 3. Redis Queue Testing

#### Verify Redis Connectivity
```bash
# Test Redis connection
docker exec relay_redis redis-cli ping
# Expected: PONG

# Check Redis info
docker exec relay_redis redis-cli info server | head -10
```

#### Monitor Redis Queues
```bash
# Check queue keys
docker exec relay_redis redis-cli keys "cloak:jobs:*"

# Check queue sizes
docker exec relay_redis redis-cli zcard cloak:jobs:main
docker exec relay_redis redis-cli zcard cloak:jobs:retry
docker exec relay_redis redis-cli zcard cloak:jobs:processing
docker exec relay_redis redis-cli zcard cloak:jobs:dead
```

### 4. API Endpoint Testing

#### Test Withdraw Endpoint (Full Integration)
```bash
# Submit a withdraw request
curl -s -X POST http://localhost:3002/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "outputs": [
      {
        "recipient": "11111111111111111111111111111112",
        "amount": 990000
      }
    ],
    "policy": {
      "fee_bps": 100
    },
    "public_inputs": {
      "root": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "nf": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
      "amount": 1000000,
      "fee_bps": 100,
      "outputs_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    },
    "proof_bytes": "aGVsbG8gd29ybGQ="
  }'

# Expected response:
{
  "success": true,
  "data": {
    "request_id": "uuid-here",
    "status": "queued",
    "message": "Withdraw request received and queued for processing"
  },
  "error": null
}
```

#### Verify Database Record Creation
```bash
# Check if job was created in database
docker exec relay_postgres psql -U postgres -d relay -c \
  "SELECT request_id, status, created_at, amount FROM jobs ORDER BY created_at DESC LIMIT 5;"

# Check nullifier was created
docker exec relay_postgres psql -U postgres -d relay -c \
  "SELECT encode(nullifier, 'hex') as nullifier_hex, job_id, created_at FROM nullifiers ORDER BY created_at DESC LIMIT 5;"
```

#### Verify Redis Queue Entry
```bash
# Check if job was added to Redis queue
docker exec relay_redis redis-cli zrange cloak:jobs:main 0 -1 WITHSCORES

# Check queue size
docker exec relay_redis redis-cli zcard cloak:jobs:main
```

#### Test Status Endpoint
```bash
# Get the request_id from the withdraw response above, then:
REQUEST_ID="your-request-id-here"
curl -s http://localhost:3002/status/$REQUEST_ID

# Expected response (if job exists in database):
{
  "success": true,
  "data": {
    "request_id": "uuid-here",
    "status": "queued",
    "tx_id": null,
    "error": null,
    "created_at": "2025-09-20T23:47:37.876188794+00:00",
    "completed_at": null
  },
  "error": null
}

# Test with non-existent job
curl -s http://localhost:3002/status/00000000-0000-0000-0000-000000000000

# Expected response:
{
  "error": true,
  "message": "Not found"
}
```

### 5. Validation Testing

#### Test Invalid Requests
```bash
# Test invalid conservation (outputs + fee != amount)
curl -s -X POST http://localhost:3002/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "outputs": [{"recipient": "11111111111111111111111111111112", "amount": 999999}],
    "policy": {"fee_bps": 100},
    "public_inputs": {
      "root": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "nf": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "amount": 1000000,
      "fee_bps": 100,
      "outputs_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    },
    "proof_bytes": "aGVsbG8gd29ybGQ="
  }' 

# Expected error response:
{
  "error": true,
  "message": "Validation error: Conservation check failed: outputs + fee != amount"
}
```

#### Test Double-Spend Prevention
```bash
# Submit the same nullifier twice
curl -s -X POST http://localhost:3002/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "outputs": [{"recipient": "11111111111111111111111111111112", "amount": 990000}],
    "policy": {"fee_bps": 100},
    "public_inputs": {
      "root": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "nf": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
      "amount": 1000000,
      "fee_bps": 100,
      "outputs_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    },
    "proof_bytes": "aGVsbG8gd29ybGQ="
  }'

# Expected error response:
{
  "error": true,
  "message": "Validation error: Nullifier already exists (double spend)"
}
```

### 6. Automated Test Script

#### Run Comprehensive Test Suite
```bash
# Make the test script executable
chmod +x examples/test_api.sh

# Run the full test suite
./examples/test_api.sh
```

The test script will:
1. Test service health
2. Submit a withdraw request
3. Extract the request ID
4. Query the status endpoint
5. Display all request/response data

### 7. Load Testing (Optional)

#### Simple Load Test
```bash
# Test multiple concurrent requests
for i in {1..10}; do
  curl -s -X POST http://localhost:3002/withdraw \
    -H "Content-Type: application/json" \
    -d "{
      \"outputs\": [{\"recipient\": \"11111111111111111111111111111112\", \"amount\": 990000}],
      \"policy\": {\"fee_bps\": 100},
      \"public_inputs\": {
        \"root\": \"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",
        \"nf\": \"$(openssl rand -hex 32)\",
        \"amount\": 1000000,
        \"fee_bps\": 100,
        \"outputs_hash\": \"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\"
      },
      \"proof_bytes\": \"aGVsbG8gd29ybGQ=\"
    }" &
done
wait

# Check how many jobs were created
docker exec relay_postgres psql -U postgres -d relay -c \
  "SELECT COUNT(*) as total_jobs FROM jobs;"
```

## Monitoring & Observability

### Database Monitoring
```bash
# Monitor active connections
docker exec relay_postgres psql -U postgres -d relay -c \
  "SELECT pid, usename, application_name, state, query_start FROM pg_stat_activity WHERE state != 'idle';"

# Check table sizes
docker exec relay_postgres psql -U postgres -d relay -c \
  "SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(tablename::text)) as size 
   FROM pg_tables WHERE schemaname = 'public';"

# Monitor job statistics
docker exec relay_postgres psql -U postgres -d relay -c \
  "SELECT status, COUNT(*) FROM jobs GROUP BY status;"
```

### Redis Monitoring
```bash
# Redis info
docker exec relay_redis redis-cli info stats

# Monitor queue sizes
docker exec relay_redis redis-cli eval "
local keys = redis.call('keys', 'cloak:jobs:*')
for i=1,#keys do
  local size = redis.call('zcard', keys[i])
  redis.log(redis.LOG_NOTICE, keys[i] .. ': ' .. size)
end
return keys
" 0
```

### Service Logs
```bash
# Real-time service logs with different levels
RUST_LOG=debug cargo run 2>&1 | tee service.log

# Filter specific component logs
RUST_LOG=relay::db=debug,relay::queue=debug cargo run

# Monitor HTTP requests
RUST_LOG=tower_http=debug cargo run
```