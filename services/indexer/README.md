# Cloak Indexer Service

A microservice that maintains an append-only Merkle tree and serves proofs for the Cloak privacy protocol on Solana.

## Features

- **Merkle Tree Management**: Maintains a binary Merkle tree with height 32 using BLAKE3 hashing
- **REST API**: Provides endpoints for tree roots, proofs, and encrypted note queries  
- **PostgreSQL Storage**: Persistent storage for tree nodes and encrypted outputs
- **Docker Support**: Containerized deployment with Docker Compose
- **Health Monitoring**: Built-in health checks and monitoring endpoints

## API Endpoints

### Core Endpoints

- `POST /api/v1/deposit` - **Main deposit ingestion endpoint** - accepts leaf commits and encrypted outputs
- `GET /api/v1/merkle/root` - Get current tree root and next leaf index
- `GET /api/v1/merkle/proof/:index` - Generate Merkle proof for leaf at index
- `GET /api/v1/notes/range?start=<n>&end=<n>&limit=<n>` - Get encrypted outputs in range
- `GET /api/v1/artifacts/withdraw/:version` - Get SP1 guest ELF and verification key (TODO)

### Utility Endpoints

- `GET /health` - Service health check
- `GET /` - API information and available endpoints

### Admin Endpoints (Development)

- `POST /api/v1/admin/push-root` - Manually push a tree root
- `POST /api/v1/admin/insert-leaf` - Manually insert a leaf commitment

## Quick Start

```bash
cd services/indexer && cp env.example .env
docker-compose up -d postgres && bun run dev
curl http://localhost:3001/health  # Should return "healthy"
```

### 🛠️ Local Development
```bash
cd services/indexer
bun install && cp env.example .env
docker-compose up -d postgres
bun run dev
```

## Configuration

Key environment variables (see `env.example`):
- `DB_*` - PostgreSQL connection settings
- `SOLANA_RPC_URL` - Solana RPC endpoint  
- `SHIELD_POOL_PROGRAM_ID` - **Set after deploying your program**
- `PORT` - HTTP server port (default: 3001)
- `TREE_HEIGHT` - Merkle tree height (default: 32)

## Database Schema

- `merkle_tree_nodes` - Tree nodes by level/index
- `notes` - Encrypted outputs with metadata  
- `indexer_metadata` - Service state
- `artifacts` - SP1 guest ELF and verification keys

## Integration

**Deposit Flow**: Relayer → `/deposit` → Tree update → Proof generation  
**Withdraw Flow**: Relayer → `/merkle/proof/:index` → ZK verification  

**Health**: `curl http://localhost:3001/health`

## Testing

### 🚀 Quick Test (2 minutes)
```bash
# Start services
docker-compose up -d postgres
cd services/indexer && bun run dev

# Run automated test
bun run test:integration
```

**Expected output:** `🎉 ALL TESTS PASSED! Indexer is working correctly.`

### 🧪 Manual Testing
```bash
# Health check
curl http://localhost:3001/health

# Make deposit
curl -X POST http://localhost:3001/api/v1/deposit \
  -H "Content-Type: application/json" \
  -d '{"leafCommit":"1111111111111111111111111111111111111111111111111111111111111111","encryptedOutput":"dGVzdA=="}'

# Get merkle root & proof
curl http://localhost:3001/api/v1/merkle/root
curl http://localhost:3001/api/v1/merkle/proof/0

# Query notes & artifacts
curl "http://localhost:3001/api/v1/notes/range?start=0&end=10"
curl http://localhost:3001/api/v1/artifacts/withdraw/v2.0.0
```

### 🐳 Docker Testing
```bash
# Full stack
docker-compose up -d
bun run test:integration

# Production build
bun run docker:build && bun run docker:run
```

### 🧩 Unit Tests
```bash
bun test                    # All tests
bun test --coverage         # With coverage
bun test --watch           # Watch mode
```


## TODOs

### Required for Solana Integration
- [ ] **Deploy Shield Pool Program** - Update `SHIELD_POOL_PROGRAM_ID` in `.env` 
- [ ] Add Solana program log listener for automatic deposit detection
- [ ] Implement webhook interface for event ingestion

### Production Readiness  
- [ ] Add authentication for admin endpoints
- [ ] Implement rate limiting and DDoS protection
- [ ] Add metrics and observability (Prometheus/Grafana)
- [ ] Performance optimization for large trees (>10k deposits)

## Current Status ✅

**MVP COMPLETE** - All core functionality implemented:
- ✅ Deposit ingestion via `/deposit` endpoint
- ✅ Merkle tree operations (root, proof generation)
- ✅ Notes storage and range queries  
- ✅ SP1 artifact hosting with SHA-256 integrity
- ✅ PostgreSQL storage layer
- ✅ Docker containerization
- ✅ Comprehensive test suite