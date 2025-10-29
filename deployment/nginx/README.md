# Nginx Reverse Proxy for Cloak Services

This nginx service acts as a reverse proxy for the Cloak indexer and relay services, routing requests appropriately and preparing the setup for AWS CloudFront integration.

## Architecture

```
CloudFront (https://api.cloaklabz.xyz)
    ↓
Nginx (port 80)
    ├─→ Indexer (port 3001) - /api/v1/*, /, /health
    └─→ Relay (port 3002) - /withdraw, /status/*, /jobs/*, etc.
```

## Route Mapping

### Indexer Routes
- `/api/v1/*` → Indexer service (all indexer API endpoints)
- `/` → Indexer service (API info)
- `/health` → Indexer service (health check)

### Relay Routes
- `/withdraw` → Relay service
- `/status/*` → Relay service
- `/jobs/*` → Relay service (includes `/jobs/:job_id/prove-local`)
- `/backlog` → Relay service
- `/orchestrate/*` → Relay service
- `/submit` → Relay service
- `/relay/health` → Relay service (health check)
- `/metrics` → Relay metrics (Prometheus endpoint on port 9090)

## Usage

Include nginx when starting services:

```bash
# From project root
docker compose -f deployment/compose.yml up nginx indexer relay --build

# Or from deployment directory
cd deployment
docker compose up nginx indexer relay --build
```

Or if you only want to rebuild nginx:

```bash
# From project root
docker compose -f deployment/compose.yml up nginx --build

# Or from deployment directory
cd deployment
docker compose up nginx --build
```

## CloudFront Configuration

When setting up CloudFront distribution pointing to `https://api.cloaklabz.xyz`:

1. **Origin**: Point to your EC2 instance or load balancer (port 80)
2. **Behaviors**: 
   - Default Cache Behavior: Forward all headers, don't cache API requests
   - Consider setting cache policies to "CachingDisabled" for dynamic content
3. **SSL/TLS**: Use AWS Certificate Manager (ACM) for HTTPS certificate
4. **Origin Protocol Policy**: HTTP Only (nginx listens on port 80)

## Health Checks

- Nginx health check: `http://your-server/health` (proxied to indexer)
- Relay health check: `http://your-server/relay/health`

## Logs

Nginx logs are available in the container:
- Access logs: `/var/log/nginx/access.log`
- Error logs: `/var/log/nginx/error.log`

To view logs:
```bash
docker logs cloak-nginx
```

## Troubleshooting

If nginx fails to start:
1. Check that indexer and relay services are running
2. Verify nginx container logs: `docker logs cloak-nginx`
3. Test nginx config syntax: `docker exec cloak-nginx nginx -t`
4. Verify upstream connectivity: `docker exec cloak-nginx wget -O- http://indexer:3001/health`

