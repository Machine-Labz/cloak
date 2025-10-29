# Testing Nginx Setup Locally

## Quick Start

1. **Start all services:**
   ```bash
   # From project root
   docker compose -f deployment/compose.yml up nginx indexer relay postgres --build
   
   # Or from deployment directory
   cd deployment
   docker compose up nginx indexer relay postgres --build
   ```

2. **In another terminal, run the test script:**
   ```bash
   # From project root
   ./deployment/nginx/test.sh
   
   # Or from deployment directory
   cd deployment
   ./nginx/test.sh
   ```

3. **Or test manually with curl:**
   ```bash
   # Test indexer
   curl http://localhost/
   curl http://localhost/health
   curl http://localhost/api/v1/merkle/root
   
   # Test relay
   curl http://localhost/relay/health
   curl http://localhost/backlog
   curl http://localhost/withdraw  # Will need POST with body
   ```

## Manual Testing Commands

### Indexer Endpoints
```bash
# Root (API info)
curl -v http://localhost/

# Health check
curl -v http://localhost/health

# API v1 endpoints
curl -v http://localhost/api/v1/merkle/root
curl -v http://localhost/api/v1/notes/range
```

### Relay Endpoints
```bash
# Health check (via nginx rewrite)
curl -v http://localhost/relay/health

# Backlog status
curl -v http://localhost/backlog

# Metrics (if enabled)
curl -v http://localhost/metrics
```

### Test with Pretty JSON Output
```bash
curl -s http://localhost/ | jq .
curl -s http://localhost/health | jq .
curl -s http://localhost/relay/health | jq .
```

## Verify Nginx is Working

1. **Check nginx logs:**
   ```bash
   docker logs cloak-nginx
   ```

2. **Check nginx config syntax:**
   ```bash
   docker exec cloak-nginx nginx -t
   ```

3. **Check if nginx can reach upstreams:**
   ```bash
   # Test indexer connection
   docker exec cloak-nginx wget -O- http://indexer:3001/health
   
   # Test relay connection
   docker exec cloak-nginx wget -O- http://relay:3002/health
   ```

## Troubleshooting

### If endpoints return 502 Bad Gateway:
- Ensure indexer and relay containers are running: `docker ps`
- Check if services are healthy: `docker logs cloak-indexer` and `docker logs cloak-relay`
- Verify network connectivity: Services should be on `cloak-network`

### If endpoints return 404:
- Check nginx routing rules match your request path
- Verify the endpoint exists in the target service
- Check nginx access logs: `docker logs cloak-nginx`

### If services won't start:
- Check postgres is ready: `docker logs cloak-postgres`
- Verify `.env.docker` file exists and has required variables
- Check port conflicts: Ensure ports 80, 3001, 3002, 5434 are not in use

## Expected Results

When testing successfully, you should see:

✅ **Indexer endpoints** return JSON with service information
✅ **Relay endpoints** return appropriate responses (200 OK, or required request body for POST)
✅ **Health checks** return `{"status": "healthy"}` or `{"status": "ok"}`
✅ **API v1 endpoints** return proper JSON responses

## Testing with Custom Port

If nginx is exposed on a different port, adjust the test script:

```bash
   # From project root
   ./deployment/nginx/test.sh http://localhost:8080
   
   # Or from deployment directory
   ./nginx/test.sh http://localhost:8080
```

Or update your `compose.yml` port mapping if needed.

