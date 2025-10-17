# Running Relay Service Locally

## Quick Start

### 1. Start Docker Services (Postgres & Redis)

```bash
docker compose up postgres redis -d
```

This starts:
- **Postgres** on `localhost:5434` (mapped from container's 5432)
- **Redis** on `localhost:6380` (mapped from container's 6379)

### 2. Configure Environment

The `.env` file is already configured for local development:

```env
DATABASE_URL=postgresql://cloak:development_password_change_in_production@localhost:5434/cloak_relay
REDIS_URL=redis://localhost:6380
```

**Important:** Use `localhost:5434` and `localhost:6380` when running locally!

### 3. Run Relay Service

```bash
# Development mode
cargo run -p relay

# Or release mode (faster)
cargo run --release -p relay
```

## Port Configuration Summary

| Service   | Inside Docker | From Host (localhost) | Use in .env (local) |
|-----------|---------------|----------------------|---------------------|
| Postgres  | `postgres:5432` | `localhost:5434`    | `localhost:5434` ✅ |
| Redis     | `redis:6379`    | `localhost:6380`    | `localhost:6380` ✅ |
| Relay API | `0.0.0.0:3002`  | `localhost:3002`    | N/A                 |

## Environment Files

### For Local Development (`.env`)
```env
DATABASE_URL=postgresql://cloak:development_password_change_in_production@localhost:5434/cloak_relay
REDIS_URL=redis://localhost:6380
```

### For Docker Deployment (`env.docker`)
```env
DATABASE_URL=postgresql://cloak:development_password_change_in_production@postgres:5432/cloak_relay
REDIS_URL=redis://redis:6379
```

## Verify Services are Running

### Check Postgres
```bash
psql postgresql://cloak:development_password_change_in_production@localhost:5434/cloak_relay -c "SELECT version();"
```

### Check Redis
```bash
docker exec -it cloak-redis redis-cli PING
# Should return: PONG
```

### Check Relay API
```bash
curl http://localhost:3002/health
# Should return: OK
```

## Common Issues

### Issue: "Connection refused (os error 61)"

**Cause:** Using wrong port or Docker services not running

**Fix:**
1. Check Docker services are running:
   ```bash
   docker ps | grep -E "cloak-postgres|cloak-redis"
   ```

2. Verify port mappings in `compose.yml`:
   ```yaml
   postgres:
     ports:
       - "5434:5432"  # host:container
   redis:
     ports:
       - "6380:6379"  # host:container
   ```

3. Update `.env` to use host ports (`5434`, `6380`)

### Issue: "Database cloak_relay does not exist"

**Cause:** Database not initialized

**Fix:**
```bash
# Stop and remove postgres container
docker compose down postgres
docker volume rm cloak_postgres_data

# Start fresh (will run init.sql)
docker compose up postgres -d

# Wait for initialization
docker compose logs -f postgres
# Look for: "database system is ready to accept connections"
```

### Issue: Variable substitution not working in .env

**Cause:** `.env` files don't support `${VAR}` syntax by default

**Fix:** Use full URLs directly:
```env
# ❌ DON'T use variables
DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}

# ✅ DO use full URL
DATABASE_URL=postgresql://cloak:development_password_change_in_production@localhost:5434/cloak_relay
```

## Development Workflow

### 1. Start Dependencies
```bash
docker compose up postgres redis -d
```

### 2. Run Relay (watch mode with cargo-watch)
```bash
cargo watch -x 'run -p relay'
```

### 3. View Logs
```bash
# Relay logs (from cargo output)
# Will show in terminal

# Database logs
docker compose logs -f postgres

# Redis logs
docker compose logs -f redis
```

### 4. Test the Service
```bash
# Run complete flow test
cd tooling/test
cargo run --bin test-complete-flow-rust
```

### 5. Monitor Jobs
```bash
# Check Redis queue
docker exec -it cloak-redis redis-cli ZCARD cloak:jobs:main

# Check database
psql postgresql://cloak:development_password_change_in_production@localhost:5434/cloak_relay \
  -c "SELECT id, status, created_at FROM jobs ORDER BY created_at DESC LIMIT 10;"
```

## Debugging

### Enable Debug Logs
In `.env`:
```env
RUST_LOG=relay=debug,relay::worker=debug,relay::queue=debug
RUST_BACKTRACE=1
```

### Watch Worker Activity
```bash
cargo run -p relay 2>&1 | grep -E "🚀|📦|🔄|✅|❌"
```

### Check Database Connections
```bash
psql postgresql://cloak:development_password_change_in_production@localhost:5434/postgres \
  -c "SELECT datname, numbackends FROM pg_stat_database WHERE datname IN ('cloak_relay', 'cloak_indexer');"
```

### Check Redis Connections
```bash
docker exec -it cloak-redis redis-cli INFO clients
```

## Cleanup

### Stop Services
```bash
docker compose down
```

### Remove All Data (Fresh Start)
```bash
docker compose down -v
```

### Clear Redis Queue
```bash
./cleanup-redis.sh
```

## Summary

✅ **Docker services** expose on different ports than internal  
✅ **Local .env** uses `localhost:5434` and `localhost:6380`  
✅ **Docker env.docker** uses `postgres:5432` and `redis:6379`  
✅ **Always check** port mappings in `compose.yml`  

Now you can develop the relay service locally with hot-reload! 🚀

