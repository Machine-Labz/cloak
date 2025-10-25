---
title: Operations Runbook
description: Comprehensive operational handbook for running Cloak infrastructure in production, including deployment, monitoring, troubleshooting, and maintenance procedures.
---

# Cloak Operations Runbook

This comprehensive runbook provides detailed operational procedures for running Cloak's privacy-preserving infrastructure in production. It covers deployment, monitoring, troubleshooting, and maintenance for all system components including indexer, relay, miners, and on-chain programs.

## Table of Contents

- [Cloak Operations Runbook](#cloak-operations-runbook)
  - [Table of Contents](#table-of-contents)
  - [System Overview](#system-overview)
    - [Core Components](#core-components)
    - [Service Dependencies](#service-dependencies)
  - [Deployment Architecture](#deployment-architecture)
    - [Production Environment](#production-environment)
    - [Development Environment](#development-environment)
  - [Infrastructure Provisioning](#infrastructure-provisioning)
    - [Hardware Requirements](#hardware-requirements)
    - [Cloud Provider Configurations](#cloud-provider-configurations)
  - [Service Configuration](#service-configuration)
    - [Indexer Configuration](#indexer-configuration)
    - [Relay Configuration](#relay-configuration)
    - [Miner Configuration](#miner-configuration)
  - [Monitoring \& Alerting](#monitoring--alerting)
    - [Metrics Collection](#metrics-collection)
    - [Alerting Rules](#alerting-rules)
    - [Dashboard Configuration](#dashboard-configuration)
  - [Operational Procedures](#operational-procedures)
    - [Service Startup Procedures](#service-startup-procedures)
    - [Health Check Procedures](#health-check-procedures)
    - [Backup Procedures](#backup-procedures)

## System Overview

### Core Components

**On-Chain Programs:**
- **Shield Pool** - Core privacy protocol with ZK proof verification
- **Scramble Registry** - PoW claim management and miner coordination

**Off-Chain Services:**
- **Indexer Service** - Merkle tree maintenance and note discovery
- **Relay Service** - Withdrawal processing and PoW integration
- **Web Application** - User interface and admin tooling

**Mining Infrastructure:**
- **Cloak Miner** - Standalone PoW mining client
- **Mining Pools** - Distributed mining coordination

### Service Dependencies

```mermaid
graph TB
    subgraph "Service Dependencies"
        subgraph "Client Layer"
            WA[Web App]
            CLI[CLI Tools]
            API[APIs]
        end
        
        subgraph "Service Layer"
            IDX[Indexer]
            REL[Relay]
            WRK[Workers]
        end
        
        subgraph "Blockchain Layer"
            RPC[Solana RPC]
            PROG[Programs]
            EVT[Events]
        end
        
        subgraph "Storage Layer"
            PG[PostgreSQL]
            DB[Database]
            MT[Merkle Tree]
        end
        
        subgraph "Queue Layer"
            Q[Database Queue]
            J[Jobs]
            C[Claims]
        end
        
        subgraph "Monitoring Layer"
            PROM[Prometheus]
            GRAF[Grafana]
            ALERT[Alerts]
        end
    end
    
    WA <--> IDX
    CLI <--> REL
    API <--> WRK
    
    IDX <--> RPC
    REL <--> PROG
    WRK <--> EVT
    
    IDX --> PG
    REL --> DB
    WRK --> MT
    
    REL --> Q
    WRK --> J
    REL --> C
    
    IDX --> PROM
    REL --> GRAF
    WRK --> ALERT
```

## Deployment Architecture

### Production Environment

**High Availability Setup:**
```yaml
# Production deployment architecture
production:
  indexer:
    replicas: 3
    load_balancer: true
    database: postgres_cluster
    monitoring: prometheus
    
  relay:
    replicas: 5
    workers: 10
    queue: database_cluster
    database: postgres_cluster
    
  miners:
    instances: 20
    pool_coordination: database_cluster
    monitoring: prometheus
    
  storage:
    postgres:
      primary: 1
      replicas: 2
      backup: s3
      
    database:
      cluster: 6_nodes
      persistence: wal
      
  monitoring:
    prometheus: 2_replicas
    grafana: 2_replicas
    alertmanager: 1_replica
```

### Development Environment

**Local Development:**
```yaml
# docker-compose.yml for local development
version: '3.8'
services:
  indexer:
    build: ./services/indexer
    ports: ["3001:3001"]
    environment:
      - DATABASE_URL=postgresql://cloak:password@postgres:5432/indexer
      - RPC_URL=https://api.devnet.solana.com
    depends_on: [postgres]
    
  relay:
    build: ./services/relay
    ports: ["3002:3002"]
    environment:
      - DATABASE_URL=postgresql://cloak:password@postgres:5432/relay
      - RPC_URL=https://api.devnet.solana.com
    depends_on: [postgres]
    
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=cloak
      - POSTGRES_USER=cloak
      - POSTGRES_PASSWORD=password
    volumes: ["postgres_data:/var/lib/postgresql/data"]
    
  prometheus:
    image: prom/prometheus
    ports: ["9090:9090"]
    volumes: ["./prometheus.yml:/etc/prometheus/prometheus.yml"]
    
  grafana:
    image: grafana/grafana
    ports: ["3000:3000"]
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

## Infrastructure Provisioning

### Hardware Requirements

**Indexer Service:**
```yaml
indexer_production:
  cpu: 8-16 vCPU
  memory: 32-64 GB RAM
  storage: 500 GB SSD
  network: 1 Gbps
  
indexer_development:
  cpu: 2-4 vCPU
  memory: 8-16 GB RAM
  storage: 100 GB SSD
  network: 100 Mbps
```

**Relay Service:**
```yaml
relay_production:
  cpu: 16-32 vCPU
  memory: 64-128 GB RAM
  storage: 200 GB SSD
  network: 1 Gbps
  
relay_development:
  cpu: 4-8 vCPU
  memory: 16-32 GB RAM
  storage: 50 GB SSD
  network: 100 Mbps
```

**Mining Infrastructure:**
```yaml
miner_production:
  cpu: 32-64 vCPU (high frequency)
  memory: 64-128 GB RAM
  storage: 100 GB SSD
  network: 1 Gbps
  
miner_development:
  cpu: 8-16 vCPU
  memory: 16-32 GB RAM
  storage: 50 GB SSD
  network: 100 Mbps
```

### Cloud Provider Configurations

**AWS Configuration:**
```yaml
aws_production:
  indexer:
    instance_type: c5.2xlarge
    ebs_volume: gp3-500gb
    vpc: private_subnet
    security_groups: [cloak-indexer-sg]
    
  relay:
    instance_type: c5.4xlarge
    ebs_volume: gp3-200gb
    vpc: private_subnet
    security_groups: [cloak-relay-sg]
    
  miners:
    instance_type: c5.8xlarge
    ebs_volume: gp3-100gb
    vpc: private_subnet
    security_groups: [cloak-miner-sg]
    
  database:
    rds_instance: db.r5.2xlarge
    storage: 1000gb-gp3
    multi_az: true
    backup_retention: 7_days
    
  cache:
    elasticache_database: r6g.2xlarge
    cluster_mode: enabled
    nodes: 6
```

**GCP Configuration:**
```yaml
gcp_production:
  indexer:
    machine_type: c2-standard-8
    disk_type: ssd-persistent
    disk_size: 500gb
    network: private_vpc
    
  relay:
    machine_type: c2-standard-16
    disk_type: ssd-persistent
    disk_size: 200gb
    network: private_vpc
    
  miners:
    machine_type: c2-standard-32
    disk_type: ssd-persistent
    disk_size: 100gb
    network: private_vpc
    
  database:
    cloud_sql: db-n1-standard-8
    storage: 1000gb-ssd
    high_availability: true
    backup_enabled: true
    
  cache:
    memorystore_database: 8gb-standard
    tier: standard
    nodes: 6
```

## Service Configuration

### Indexer Configuration

**Environment Variables:**
```bash
# Core configuration
INDEXER_PORT=3001
INDEXER_HOST=0.0.0.0
INDEXER_LOG_LEVEL=info

# Database configuration
DATABASE_URL=postgresql://user:pass@localhost:5432/indexer
DATABASE_POOL_SIZE=20
DATABASE_TIMEOUT=30s

# Solana configuration
RPC_URL=https://api.mainnet-beta.solana.com
RPC_TIMEOUT=30s
RPC_RETRIES=3

# Merkle tree configuration
MERKLE_TREE_DEPTH=32
MERKLE_BATCH_SIZE=1000
MERKLE_CACHE_SIZE=10000

# Performance tuning
WORKER_THREADS=8
ASYNC_RUNTIME_THREADS=16
```

**Configuration File:**
```toml
# config/indexer.toml
[server]
port = 3001
host = "0.0.0.0"
workers = 8

[database]
url = "postgresql://user:pass@localhost:5432/indexer"
pool_size = 20
timeout = "30s"
max_lifetime = "3600s"

[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
timeout = "30s"
retries = 3
rate_limit = 100

[merkle]
depth = 32
batch_size = 1000
cache_size = 10000
update_interval = "1s"

[logging]
level = "info"
format = "json"
```

### Relay Configuration

**Environment Variables:**
```bash
# Core configuration
RELAY_PORT=3002
RELAY_HOST=0.0.0.0
RELAY_LOG_LEVEL=info

# Database configuration
DATABASE_URL=postgresql://user:pass@localhost:5432/relay
DATABASE_POOL_SIZE=30
DATABASE_TIMEOUT=30s

# Solana configuration
RPC_URL=https://api.mainnet-beta.solana.com
RPC_TIMEOUT=30s
RPC_RETRIES=3

# Worker configuration
WORKER_COUNT=10
WORKER_TIMEOUT=300s
WORKER_RETRIES=3

# PoW configuration
POW_ENABLED=true
POW_CLAIM_CACHE_TTL=300s
POW_CLAIM_SEARCH_TIMEOUT=10s

# Transaction configuration
PRIORITY_FEE_MICROLAMPORTS=1000
COMPUTE_BUDGET_CU=1000000
JITO_ENABLED=true
JITO_TIP_LAMPORTS=10000
```

**Configuration File:**
```toml
# config/relay.toml
[server]
port = 3002
host = "0.0.0.0"
workers = 10

[database]
url = "postgresql://user:pass@localhost:5432/relay"
pool_size = 30
timeout = "30s"

[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
timeout = "30s"
retries = 3

[workers]
count = 10
timeout = "300s"
retries = 3
backoff_multiplier = 2.0

[pow]
enabled = true
claim_cache_ttl = "300s"
search_timeout = "10s"

[transaction]
priority_fee_microlamports = 1000
compute_budget_cu = 1000000
jito_enabled = true
jito_tip_lamports = 10000
```

### Miner Configuration

**Environment Variables:**
```bash
# Core configuration
MINER_LOG_LEVEL=info
MINER_KEYPAIR_PATH=/etc/cloak/miner-keypair.json

# Solana configuration
RPC_URL=https://api.mainnet-beta.solana.com
RPC_TIMEOUT=30s

# Mining configuration
MINING_MODE=wildcard
MINING_DIFFICULTY=1000000
MINING_THREADS=16
MINING_BATCH_SIZE=1000000

# Registry configuration
REGISTRY_PROGRAM_ID=scramb1eReg1stryPoWM1n1ngSo1anaC1oak11111111
MINER_PDA_SEED=miner

# Performance configuration
HASHRATE_REPORT_INTERVAL=60s
STATS_UPDATE_INTERVAL=10s
```

**Configuration File:**
```toml
# config/miner.toml
[miner]
keypair_path = "/etc/cloak/miner-keypair.json"
log_level = "info"

[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
timeout = "30s"

[mining]
mode = "wildcard"
difficulty = 1000000
threads = 16
batch_size = 1000000

[registry]
program_id = "scramb1eReg1stryPoWM1n1ngSo1anaC1oak11111111"
miner_pda_seed = "miner"

[performance]
hashrate_report_interval = "60s"
stats_update_interval = "10s"
```

## Monitoring & Alerting

### Metrics Collection

**Prometheus Configuration:**
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "cloak_alerts.yml"

scrape_configs:
  - job_name: 'cloak-indexer'
    static_configs:
      - targets: ['indexer:3001']
    metrics_path: '/metrics'
    scrape_interval: 5s
    
  - job_name: 'cloak-relay'
    static_configs:
      - targets: ['relay:3002']
    metrics_path: '/metrics'
    scrape_interval: 5s
    
  - job_name: 'cloak-miners'
    static_configs:
      - targets: ['miner1:8080', 'miner2:8080', 'miner3:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
    
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 30s
```

**Key Metrics:**
```rust
// Indexer metrics
lazy_static! {
    static ref DEPOSITS_TOTAL: Counter = Counter::new("deposits_total", "Total deposits processed").unwrap();
    static ref MERKLE_UPDATES_TOTAL: Counter = Counter::new("merkle_updates_total", "Total Merkle tree updates").unwrap();
    static ref PROOF_REQUESTS_TOTAL: Counter = Counter::new("proof_requests_total", "Total proof requests").unwrap();
    static ref PROOF_GENERATION_TIME: Histogram = Histogram::new("proof_generation_seconds", "Proof generation time").unwrap();
    static ref DATABASE_CONNECTIONS: Gauge = Gauge::new("database_connections_active", "Active database connections").unwrap();
}

// Relay metrics
lazy_static! {
    static ref WITHDRAWALS_TOTAL: Counter = Counter::new("withdrawals_total", "Total withdrawals processed").unwrap();
    static ref JOBS_QUEUED: Gauge = Gauge::new("jobs_queued", "Jobs in queue").unwrap();
    static ref JOBS_PROCESSING: Gauge = Gauge::new("jobs_processing", "Jobs being processed").unwrap();
    static ref JOBS_COMPLETED: Counter = Counter::new("jobs_completed_total", "Jobs completed").unwrap();
    static ref JOBS_FAILED: Counter = Counter::new("jobs_failed_total", "Jobs failed").unwrap();
    static ref POW_CLAIMS_FOUND: Counter = Counter::new("pow_claims_found_total", "PoW claims found").unwrap();
    static ref POW_CLAIMS_CONSUMED: Counter = Counter::new("pow_claims_consumed_total", "PoW claims consumed").unwrap();
    static ref TRANSACTION_SUCCESS_RATE: Gauge = Gauge::new("transaction_success_rate", "Transaction success rate").unwrap();
}

// Miner metrics
lazy_static! {
    static ref MINING_HASHRATE: Gauge = Gauge::new("mining_hashrate", "Current mining hashrate").unwrap();
    static ref CLAIMS_MINED_TOTAL: Counter = Counter::new("claims_mined_total", "Total claims mined").unwrap();
    static ref CLAIMS_REVEALED_TOTAL: Counter = Counter::new("claims_revealed_total", "Total claims revealed").unwrap();
    static ref CLAIMS_CONSUMED_TOTAL: Counter = Counter::new("claims_consumed_total", "Total claims consumed").unwrap();
    static ref MINING_DIFFICULTY: Gauge = Gauge::new("mining_difficulty", "Current mining difficulty").unwrap();
}
```

### Alerting Rules

**Alert Configuration:**
```yaml
# cloak_alerts.yml
groups:
  - name: cloak_critical
    rules:
      - alert: HighJobQueueBacklog
        expr: jobs_queued > 100
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High job queue backlog"
          description: "Job queue has {{ $value }} jobs queued for more than 5 minutes"
          
      - alert: LowTransactionSuccessRate
        expr: transaction_success_rate < 0.95
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Low transaction success rate"
          description: "Transaction success rate is {{ $value }}% for more than 5 minutes"
          
      - alert: DatabaseConnectionFailure
        expr: database_connections_active == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database connection failure"
          description: "No active database connections for more than 1 minute"
          
      - alert: HighProofGenerationTime
        expr: histogram_quantile(0.95, proof_generation_seconds) > 120
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High proof generation time"
          description: "95th percentile proof generation time is {{ $value }}s"
          
      - alert: NoPoWClaimsAvailable
        expr: pow_claims_found_total == 0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "No PoW claims available"
          description: "No PoW claims found for more than 10 minutes"
          
      - alert: LowMiningHashrate
        expr: mining_hashrate < 1000000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low mining hashrate"
          description: "Mining hashrate is {{ $value }} H/s for more than 5 minutes"
```

### Dashboard Configuration

**Grafana Dashboard:**
```json
{
  "dashboard": {
    "title": "Cloak Operations Dashboard",
    "panels": [
      {
        "title": "Job Queue Status",
        "type": "stat",
        "targets": [
          {
            "expr": "jobs_queued",
            "legendFormat": "Queued"
          },
          {
            "expr": "jobs_processing",
            "legendFormat": "Processing"
          }
        ]
      },
      {
        "title": "Transaction Success Rate",
        "type": "gauge",
        "targets": [
          {
            "expr": "transaction_success_rate",
            "legendFormat": "Success Rate"
          }
        ]
      },
      {
        "title": "Proof Generation Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, proof_generation_seconds)",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, proof_generation_seconds)",
            "legendFormat": "p95"
          }
        ]
      },
      {
        "title": "Mining Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "mining_hashrate",
            "legendFormat": "Hashrate (H/s)"
          },
          {
            "expr": "mining_difficulty",
            "legendFormat": "Difficulty"
          }
        ]
      }
    ]
  }
}
```

## Operational Procedures

### Service Startup Procedures

**Indexer Service:**
```bash
#!/bin/bash
# scripts/start-indexer.sh

# 1. Check prerequisites
echo "Checking prerequisites..."
if ! command -v postgres &> /dev/null; then
    echo "PostgreSQL not found. Please install PostgreSQL 14+"
    exit 1
fi

if ! command -v solana &> /dev/null; then
    echo "Solana CLI not found. Please install Solana CLI"
    exit 1
fi

# 2. Verify database connection
echo "Verifying database connection..."
psql $DATABASE_URL -c "SELECT 1;" || {
    echo "Database connection failed"
    exit 1
}

# 3. Run database migrations
echo "Running database migrations..."
sqlx migrate run --database-url $DATABASE_URL

# 4. Verify Solana RPC connection
echo "Verifying Solana RPC connection..."
solana cluster-version --url $RPC_URL || {
    echo "Solana RPC connection failed"
    exit 1
}

# 5. Start indexer service
echo "Starting indexer service..."
cargo run --bin indexer --release
```

**Relay Service:**
```bash
#!/bin/bash
# scripts/start-relay.sh

# 1. Check prerequisites
echo "Checking prerequisites..."
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL client not found. Please install PostgreSQL client"
    exit 1
fi

# 2. Verify database connection
echo "Verifying database connection..."
psql $DATABASE_URL -c "SELECT 1;" || {
    echo "Database connection failed"
    exit 1
}

# 3. Start relay service
echo "Starting relay service..."
cargo run --bin relay --release
```

**Miner Service:**
```bash
#!/bin/bash
# scripts/start-miner.sh

# 1. Check prerequisites
echo "Checking prerequisites..."
if [ ! -f "$MINER_KEYPAIR_PATH" ]; then
    echo "Miner keypair not found at $MINER_KEYPAIR_PATH"
    exit 1
fi

# 2. Verify Solana RPC connection
echo "Verifying Solana RPC connection..."
solana cluster-version --url $RPC_URL || {
    echo "Solana RPC connection failed"
    exit 1
}

# 3. Register miner if not already registered
echo "Checking miner registration..."
cloak-miner status --keypair $MINER_KEYPAIR_PATH --rpc-url $RPC_URL || {
    echo "Registering miner..."
    cloak-miner register --keypair $MINER_KEYPAIR_PATH --rpc-url $RPC_URL
}

# 4. Start mining
echo "Starting mining..."
cloak-miner mine --mode wildcard --keypair $MINER_KEYPAIR_PATH --rpc-url $RPC_URL
```

### Health Check Procedures

**Service Health Checks:**
```bash
#!/bin/bash
# scripts/health-check.sh

# Indexer health check
check_indexer() {
    local response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3001/api/v1/health)
    if [ "$response" = "200" ]; then
        echo "✅ Indexer is healthy"
        return 0
    else
        echo "❌ Indexer health check failed (HTTP $response)"
        return 1
    fi
}

# Relay health check
check_relay() {
    local response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3002/health)
    if [ "$response" = "200" ]; then
        echo "✅ Relay is healthy"
        return 0
    else
        echo "❌ Relay health check failed (HTTP $response)"
        return 1
    fi
}

# Database health check
check_database() {
    psql $DATABASE_URL -c "SELECT 1;" > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Database is healthy"
        return 0
    else
        echo "❌ Database health check failed"
        return 1
    fi
}

# Solana RPC health check
check_solana() {
    solana cluster-version --url $RPC_URL > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Solana RPC is healthy"
        return 0
    else
        echo "❌ Solana RPC health check failed"
        return 1
    fi
}

# Run all health checks
echo "Running health checks..."
check_indexer
check_relay
check_database
check_solana
```

### Backup Procedures

**Database Backup:**
```bash
#!/bin/bash
# scripts/backup-database.sh

BACKUP_DIR="/backups/postgres"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/cloak_backup_$DATE.sql"

# Create backup directory
mkdir -p $BACKUP_DIR

# Create database backup
echo "Creating database backup..."
pg_dump $DATABASE_URL > $BACKUP_FILE

# Compress backup
echo "Compressing backup..."
gzip $BACKUP_FILE

# Upload to S3 (if configured)
if [ ! -z "$S3_BACKUP_BUCKET" ]; then
    echo "Uploading backup to S3..."
    aws s3 cp "$BACKUP_FILE.gz" "s3://$S3_BACKUP_BUCKET/"
fi

# Cleanup old backups (keep last 7 days)
echo "Cleaning up old backups..."
find $BACKUP_DIR -name "cloak_backup_*.sql.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_FILE.gz"
```

**Configuration Backup:**
```bash
#!/bin/bash
# scripts/backup-config.sh

CONFIG_DIR="/etc/cloak"
BACKUP_DIR="/backups/config"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup configuration files
echo "Backing up configuration files..."
tar -czf "$BACKUP_DIR/cloak_config_$DATE.tar.gz" -C $CONFIG_DIR .

# Upload to S3 (if configured)
if [ ! -z "$S3_BACKUP_BUCKET" ]; then
    echo "Uploading config backup to S3..."
    aws s3 cp "$BACKUP_DIR/cloak_config_$DATE.tar.gz" "s3://$S3_BACKUP_BUCKET/"
fi

echo "Configuration backup completed: $BACKUP_DIR/cloak_config_$DATE.tar.gz"
```
