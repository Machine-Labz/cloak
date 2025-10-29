# Deployment Configuration

This directory contains all deployment-related configuration files for running Cloak services with Docker.

## Structure

```
deployment/
├── compose.yml          # Docker Compose configuration
├── init.sql            # PostgreSQL database initialization script
├── .env.docker         # Environment variables for Docker services
└── nginx/              # Nginx reverse proxy configuration
    ├── Dockerfile
    ├── nginx.conf
    ├── test.sh
    └── test-all-routes.sh
```

## Quick Start

1. **Configure environment variables:**
   ```bash
   cp .env.docker.example .env.docker
   # Edit .env.docker with your configuration
   ```

2. **Start all services:**
   ```bash
   docker compose -f deployment/compose.yml up --build
   ```

   Or from the deployment directory:
   ```bash
   cd deployment
   docker compose up --build
   ```

3. **Start specific services:**
   ```bash
   docker compose -f deployment/compose.yml up nginx indexer relay --build
   ```

## Services

- **postgres**: PostgreSQL 16 database (port 5434)
- **indexer**: Cloak Indexer API service (port 3001)
- **relay**: Cloak Relay service (port 3002, metrics on 9090)
- **nginx**: Reverse proxy (port 80) routing traffic to indexer and relay

## Testing

See `nginx/TESTING.md` for testing instructions and `nginx/test-all-routes.sh` for comprehensive route testing.

## Environment Variables

The `.env.docker` file should contain:
- Database connection settings
- Service ports and configuration
- CloudWatch logging settings (if enabled)
- Solana RPC endpoints
- Other service-specific configurations

## Network

All services are connected via the `cloak-network` Docker network.

## Volumes

- `cloak_postgres_data`: Persistent database storage
- `indexer_logs`: Indexer service logs
- `indexer_artifacts`: Indexer artifacts storage
- `relay_logs`: Relay service logs


