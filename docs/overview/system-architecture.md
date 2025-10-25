---
title: System Architecture
description: Comprehensive architectural overview of Cloak's privacy-preserving protocol, covering on-chain programs, off-chain services, ZK tooling, and PoW mining infrastructure.
---

# System Architecture

Cloak is a privacy-preserving protocol built on Solana that enables anonymous deposits and withdrawals through zero-knowledge proofs and proof-of-work mining. The system consists of four interconnected domains working together to provide privacy, security, and decentralization.

## High-Level Architecture

```mermaid
graph TB
    subgraph "CLOAK PROTOCOL"
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
        
        subgraph "On-Chain Layer"
            SP[Shield Pool]
            SR[Scramble Registry]
        end
        
        subgraph "Mining Layer"
            POW[PoW]
            WC[Wildcard]
            CLM[Claims]
        end
        
        subgraph "Storage Layer"
            PG[PostgreSQL]
            DB[Database]
            MT[Merkle Tree]
        end
        
        subgraph "ZK Layer"
            SP1G[SP1 Guest]
            SP1H[SP1 Host]
            PRF[Proofs]
        end
        
        subgraph "Blockchain Layer"
            TX[Transactions]
            ACC[Accounts]
            EVT[Events]
        end
        
        subgraph "Mining Pool Layer"
            B3[BLAKE3]
            DIFF[Difficulty]
            NC[Nonces]
        end
    end
    
    WA <--> IDX
    CLI <--> REL
    API <--> WRK
    
    IDX <--> SP
    REL <--> SR
    WRK <--> SP
    
    SP <--> POW
    SR <--> WC
    SP <--> CLM
    
    IDX --> PG
    REL --> DB
    WRK --> MT
    
    SP1G --> SP1H
    SP1H --> PRF
    
    SP --> TX
    SR --> ACC
    SP --> EVT
    
    POW --> B3
    WC --> DIFF
    CLM --> NC
```

## Core Domains

### 1. Client Layer
**Purpose:** User interfaces and developer tooling for interacting with the protocol.

**Components:**
- **Web Application** (`services/web`) - Next.js frontend for deposits, withdrawals, and note management
- **CLI Tools** - Command-line interfaces for advanced users and developers
- **API Clients** - TypeScript/JavaScript libraries for programmatic access

**Key Responsibilities:**
- Generate note secrets and commitments
- Encrypt/decrypt output payloads
- Submit deposit transactions
- Discover spendable notes
- Generate ZK proofs
- Submit withdrawal requests

### 2. Service Layer
**Purpose:** Off-chain coordination, state management, and transaction processing.

**Components:**
- **Indexer Service** (`services/indexer`) - Merkle tree maintenance and note discovery
- **Relay Service** (`services/relay`) - Withdrawal processing and PoW integration
- **Background Workers** - Async job processing and transaction submission

**Key Responsibilities:**
- Maintain Merkle tree state
- Process withdrawal requests
- Discover and consume PoW claims
- Queue management and retry logic
- Transaction simulation and submission

### 3. On-Chain Layer
**Purpose:** Protocol enforcement, fund management, and cryptographic verification.

**Components:**
- **Shield Pool Program** (`programs/shield-pool`) - Core privacy protocol
- **Scramble Registry Program** (`programs/scramble-registry`) - PoW claim management

**Key Responsibilities:**
- Verify ZK proofs on-chain
- Manage nullifier sets
- Enforce Merkle root validation
- Distribute funds to recipients
- Track PoW claim lifecycle

### 4. Mining Layer
**Purpose:** Decentralized proof-of-work infrastructure for enhanced privacy.

**Components:**
- **Cloak Miner** (`packages/cloak-miner`) - Standalone PoW mining client
- **Mining Pool** - Distributed mining infrastructure
- **Claim Registry** - On-chain claim storage and validation

**Key Responsibilities:**
- Mine wildcard claims
- Submit and reveal claims
- Provide consumable PoW proofs
- Maintain mining difficulty

## Data Flow Architecture

### Deposit Flow
```text
┌─────────────┐     ┌─────────────┐     ┌───────────────┐    ┌────────────┐
│    Client   │     │   Indexer   │     │  Shield Pool  │    │   Merkle   │
│             │     │   Service   │     │    Program    │    │    Tree    │
└─────────────┘     └─────────────┘     └───────────────┘    └────────────┘
       │                   │                   │                   │
       │ 1. Generate       │                   │                   │
       │    commitment     │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 2. Submit deposit │                   │                   │
       ├──────────────────────────────────────►│                   │
       │                   │                   │                   │
       │                   │ 3. Listen events  │                   │
       │                   │◄──────────────────┤                   │
       │                   │                   │                   │
       │                   │ 4. Update tree    │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │ 5. Confirmation   │                   │                   │
       │◄──────────────────┤                   │                   │
```

**Step-by-Step Process:**
1. **Commitment Generation** - Client generates `C = BLAKE3(amount || r || pk_spend)`
2. **Transaction Submission** - Submit deposit instruction with SOL transfer
3. **Event Ingestion** - Indexer listens for `DepositCommit` events
4. **Tree Update** - Append commitment to Merkle tree, store encrypted output
5. **Confirmation** - Client receives confirmation of successful deposit

### Withdrawal Flow
```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Relay      │    │   SP1 Prover    │    │   Solana     │
│             │    │   Service    │    │                 │    │   Programs   │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Discover notes │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 2. Get Merkle    │                   │                   │
       │    proof         │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 3. Generate ZK    │                   │                   │
       │    proof          │                   │                   │
       ├──────────────────────────────────────►│                   │
       │                   │                   │                   │
       │ 4. Submit         │                   │                   │
       │    withdraw       │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 5. Find PoW       │                   │
       │                   │    claim          │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │                   │ 6. Execute       │                   │
       │                   │    transaction   │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │ 7. Confirmation   │                   │                   │
       │◄──────────────────┤                   │                   │
```

**Step-by-Step Process:**
1. **Note Discovery** - Scan encrypted outputs, decrypt with `sk_spend`
2. **Merkle Proof** - Fetch inclusion proof from indexer
3. **ZK Proof Generation** - Generate Groth16 proof with SP1
4. **Job Submission** - Submit withdraw request to relay
5. **PoW Claim Discovery** - Find available wildcard claim
6. **Transaction Execution** - Build and submit withdraw transaction
7. **Confirmation** - Receive transaction confirmation

### PoW Mining Flow
```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Miner     │    │   Mining     │    │  Scramble       │    │   Relay      │
│   Client    │    │   Engine     │    │  Registry       │    │   Workers    │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Start mining   │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 2. Mine claim     │                   │
       │                   │    (BLAKE3)       │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 3. Submit mine    │                   │
       │                   │    transaction    │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 4. Reveal claim   │                   │
       │                   │    (after window) │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │                   │ 5. Consume claim  │
       │                   │                   │◄──────────────────┤
       │                   │                   │                   │
       │ 6. Mining stats   │                   │                   │
       │◄──────────────────┤                   │                   │
```

**Step-by-Step Process:**
1. **Mining Initialization** - Start BLAKE3 mining with wildcard batch hash
2. **Claim Discovery** - Find nonce where `BLAKE3(preimage) < difficulty_target`
3. **Mine Transaction** - Submit `mine_claim` instruction to registry
4. **Claim Revelation** - Submit `reveal_claim` after reveal window opens
5. **Claim Consumption** - Relay workers consume claims via CPI calls
6. **Statistics Update** - Track mining performance and claim usage

## Component Responsibilities

### On-Chain Programs

| Program | Purpose | Key Features | Account Structure |
|---------|---------|--------------|-------------------|
| **Shield Pool** | Core privacy protocol | • ZK proof verification<br/>• Nullifier management<br/>• Merkle root validation<br/>• Fund distribution | • Pool PDA (funds)<br/>• Roots ring buffer<br/>• Nullifier shards<br/>• Treasury account |
| **Scramble Registry** | PoW claim management | • Miner registration<br/>• Claim lifecycle<br/>• Wildcard validation<br/>• Consumption tracking | • Miner accounts<br/>• Claim accounts<br/>• Registry state<br/>• Authority accounts |

### Off-Chain Services

| Service | Technology Stack | Responsibilities | Data Stores |
|---------|------------------|------------------|-------------|
| **Indexer** | Rust + Axum + SQLx + PostgreSQL | • Merkle tree maintenance<br/>• Event ingestion<br/>• Note discovery API<br/>• Proof serving | • PostgreSQL (tree nodes)<br/>• Encrypted outputs<br/>• Merkle proofs<br/>• Service state |
| **Relay** | Rust + Axum + PostgreSQL | • Withdrawal processing<br/>• Job queue management<br/>• PoW claim discovery<br/>• Transaction submission | • PostgreSQL (job state)<br/>• Claim cache<br/>• Transaction logs |
| **Web App** | Next.js + Tailwind + shadcn/ui | • User interface<br/>• Wallet integration<br/>• Note management<br/>• Admin tooling | • Browser storage<br/>• Session state<br/>• User preferences<br/>• Transaction history |

### ZK Tooling

| Package | Purpose | Input/Output | Integration |
|---------|---------|-------------|-------------|
| **zk-guest-sp1** | SP1 circuit implementation | • Witness data<br/>• Public inputs | • Guest circuit (RISC-V)<br/>• Host CLI<br/>• Proof generation |
| **vkey-generator** | Verification key extraction | • Guest ELF binary | • VKey hash output<br/>• Program integration<br/>• CI/CD automation |
| **cloak-proof-extract** | Proof parsing utilities | • SP1 proof bundle | • Groth16 extraction<br/>• Public input parsing<br/>• no_std compatibility |

### Mining Infrastructure

| Component | Purpose | Technology | Key Features |
|-----------|---------|------------|--------------|
| **cloak-miner** | Standalone PoW client | Rust + BLAKE3 | • Wildcard mining<br/>• Difficulty adjustment<br/>• Claim submission<br/>• Performance metrics |
| **Mining Pool** | Distributed mining | BLAKE3 + Database | • Load balancing<br/>• Hashrate aggregation<br/>• Claim distribution<br/>• Pool statistics |
| **Claim Registry** | On-chain storage | Solana accounts | • Claim lifecycle<br/>• Consumption limits<br/>• Expiration handling<br/>• Miner rewards |

## Data Storage Architecture

### PostgreSQL Schema

**Indexer Database:**
```sql
-- Merkle tree nodes
CREATE TABLE merkle_nodes (
    id BIGSERIAL PRIMARY KEY,
    level INTEGER NOT NULL,
    index BIGINT NOT NULL,
    hash BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Encrypted outputs for note discovery
CREATE TABLE encrypted_outputs (
    id BIGSERIAL PRIMARY KEY,
    leaf_index BIGINT NOT NULL,
    encrypted_payload BYTEA NOT NULL,
    commitment BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Merkle proofs cache
CREATE TABLE merkle_proofs (
    id BIGSERIAL PRIMARY KEY,
    leaf_index BIGINT NOT NULL,
    root BYTEA NOT NULL,
    path_elements BYTEA[] NOT NULL,
    path_indices INTEGER[] NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

**Relay Database:**
```sql
-- Withdrawal jobs
CREATE TABLE withdraw_jobs (
    id UUID PRIMARY KEY,
    status VARCHAR(20) NOT NULL,
    public_inputs BYTEA NOT NULL,
    proof_bytes BYTEA NOT NULL,
    outputs JSONB NOT NULL,
    policy JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Job processing logs
CREATE TABLE job_logs (
    id BIGSERIAL PRIMARY KEY,
    job_id UUID REFERENCES withdraw_jobs(id),
    status VARCHAR(20) NOT NULL,
    message TEXT,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- PoW claim cache
CREATE TABLE claim_cache (
    claim_pda VARCHAR(44) PRIMARY KEY,
    miner_pda VARCHAR(44) NOT NULL,
    batch_hash BYTEA NOT NULL,
    consumption_count INTEGER NOT NULL,
    max_consumptions INTEGER NOT NULL,
    expires_at_slot BIGINT NOT NULL,
    last_updated TIMESTAMP DEFAULT NOW()
);
```

### Job Queue Data Structures

**Job Processing:**
```sql
-- Withdrawal job queue
INSERT INTO withdraw_jobs (job_id, status, priority, created_at) 
VALUES ('{job_id}', 'queued', 1, NOW());

-- Job processing state
UPDATE withdraw_jobs 
SET status = 'processing', worker_id = '{worker}', started_at = NOW() 
WHERE job_id = '{job_id}';

-- Job completion
UPDATE withdraw_jobs 
SET status = 'completed', tx_signature = '{signature}', completed_at = NOW() 
WHERE job_id = '{job_id}';
```

**Claim Cache:**
```sql
-- Active claims by miner
INSERT INTO active_claims (claim_hash, miner_id, difficulty, created_at) 
VALUES ('{claim_hash}', '{miner}', {diff}, NOW());

-- Claim consumption tracking
INSERT INTO consumed_claims (claim_hash, consumed_at, tx_signature) 
VALUES ('{claim_hash}', NOW(), '{signature}');
```

### Solana Account Layouts

**Shield Pool Account:**
```rust
pub struct ShieldPool {
    pub authority: Pubkey,           // 32 bytes
    pub treasury: Pubkey,            // 32 bytes
    pub total_deposits: u64,         // 8 bytes
    pub total_withdrawals: u64,      // 8 bytes
    pub fee_bps: u16,                // 2 bytes
    pub bump: u8,                    // 1 byte
    // Total: 115 bytes
}
```

**Roots Ring Buffer:**
```rust
pub struct RootsRing {
    pub current_index: u32,           // 4 bytes
    pub roots: [[u8; 32]; 256],      // 8192 bytes
    pub bump: u8,                     // 1 byte
    // Total: 8197 bytes
}
```

**Nullifier Shard:**
```rust
pub struct NullifierShard {
    pub shard_index: u8,             // 1 byte
    pub nullifiers: HashSet<[u8; 32]>, // Variable size
    pub bump: u8,                    // 1 byte
    // Total: Variable (max ~10MB per shard)
}
```

## Security Architecture

### Cryptographic Primitives

**Hash Functions:**
- **BLAKE3-256** - Primary hash function for commitments, nullifiers, and Merkle nodes
- **SHA-256** - Used for Solana account derivation and some legacy compatibility

**Zero-Knowledge Proofs:**
- **Groth16** - zkSNARK proving system with 260-byte proofs
- **SP1** - Succinct zkVM for circuit execution and proof generation
- **RISC-V** - Target architecture for SP1 guest programs

**Encryption:**
- **AES-256-GCM** - Symmetric encryption for output payloads
- **X25519** - Key exchange for encrypted communications

### Access Control

**On-Chain Permissions:**
```rust
// Shield pool authority checks
require!(ctx.accounts.authority.key() == pool.authority, ErrorCode::Unauthorized);

// Admin-only operations
require!(ctx.accounts.admin.key() == ADMIN_PUBKEY, ErrorCode::AdminOnly);

// Signer verification
require!(ctx.accounts.user.is_signer, ErrorCode::MissingSignature);
```

**Service Authentication:**
```rust
// API key authentication
pub struct ApiKeyAuth {
    pub key: String,
    pub permissions: Vec<String>,
    pub rate_limit: u32,
}

// JWT token validation
pub struct JwtAuth {
    pub token: String,
    pub claims: Claims,
    pub expires_at: u64,
}
```

### Privacy Guarantees

**Note Privacy:**
- Commitments hide transaction amounts and recipients
- Nullifiers prevent double-spending without revealing spent notes
- Merkle proofs provide membership without revealing tree structure

**Mining Privacy:**
- Wildcard claims don't reveal specific batch information
- Claim consumption doesn't link to specific withdrawals
- Mining rewards distributed through anonymous mechanisms

**Metadata Protection:**
- No on-chain storage of recipient addresses
- Encrypted output payloads for additional privacy
- Separate nullifier shards to prevent correlation

## Performance Characteristics

### Throughput Metrics

| Operation | Target Performance | Bottlenecks | Optimization Strategies |
|-----------|-------------------|-------------|------------------------|
| **Deposits** | 1000+ TPS | • Solana network<br/>• Event processing | • Batch processing<br/>• Async ingestion<br/>• Connection pooling |
| **Withdrawals** | 100+ TPS | • ZK proof generation<br/>• PoW claim discovery | • Parallel proving<br/>• Claim caching<br/>• Worker scaling |
| **Mining** | 10M+ H/s | • CPU performance<br/>• Memory bandwidth | • Multi-threading<br/>• SIMD optimization<br/>• Hardware acceleration |

### Scalability Considerations

**Horizontal Scaling:**
- Multiple indexer instances with read replicas
- Database clustering for job queue distribution
- Load-balanced relay workers
- Distributed mining pools

**Vertical Scaling:**
- High-memory instances for Merkle tree operations
- GPU acceleration for ZK proof generation
- SSD storage for database performance
- High-bandwidth networking for RPC calls

**Resource Optimization:**
- Connection pooling for database and RPC
- Caching strategies for frequently accessed data
- Batch processing for bulk operations
- Compression for large data structures

## Communication Interfaces

### HTTP APIs

**Indexer API (`/api/v1/*`):**
```typescript
// Merkle tree operations
GET /api/v1/merkle/root                    // Current Merkle root
GET /api/v1/merkle/proof/:leafIndex        // Inclusion proof
GET /api/v1/merkle/leaves/:start/:limit    // Leaf range query

// Note discovery
GET /api/v1/notes/range?start=0&limit=100  // Encrypted outputs
POST /api/v1/notes/decrypt                 // Decryption service

// Service health
GET /api/v1/health                         // Health check
GET /api/v1/metrics                        // Performance metrics
```

**Relay API:**
```typescript
// Withdrawal operations
POST /withdraw                             // Submit withdraw request
GET /status/:requestId                     // Job status
GET /jobs                                   // List jobs

// PoW integration
GET /claims/available                       // Available claims
POST /claims/consume                        // Consume claim

// Admin operations
GET /admin/stats                            // Service statistics
POST /admin/emergency-stop                  // Emergency shutdown
```

### Solana RPC Integration

**Program Interactions:**
```rust
// Shield pool program calls
pub fn deposit(ctx: Context<Deposit>, commitment: [u8; 32]) -> Result<()>;
pub fn withdraw(ctx: Context<Withdraw>, proof: [u8; 260], public_inputs: [u8; 104]) -> Result<()>;
pub fn admin_push_root(ctx: Context<AdminPushRoot>, root: [u8; 32]) -> Result<()>;

// Scramble registry program calls
pub fn register_miner(ctx: Context<RegisterMiner>) -> Result<()>;
pub fn mine_claim(ctx: Context<MineClaim>, hash: [u8; 32]) -> Result<()>;
pub fn reveal_claim(ctx: Context<RevealClaim>, preimage: ClaimPreimage) -> Result<()>;
pub fn consume_claim(ctx: Context<ConsumeClaim>, batch_hash: [u8; 32]) -> Result<()>;
```

**Event Listening:**
```rust
// Indexer event processing
pub async fn process_deposit_event(event: DepositCommitEvent) -> Result<()> {
    let commitment = event.commitment;
    let encrypted_output = event.encrypted_output;
    
    // Update Merkle tree
    self.merkle_tree.append(commitment)?;
    
    // Store encrypted output
    self.store_encrypted_output(encrypted_output)?;
    
    Ok(())
}
```

### WebSocket Subscriptions

**Real-time Updates:**
```typescript
// Client WebSocket connection
const ws = new WebSocket('ws://localhost:3001/ws');

// Subscribe to job status updates
ws.send(JSON.stringify({
    type: 'subscribe',
    channel: 'job_status',
    requestId: 'job-uuid'
}));

// Receive status updates
ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    console.log('Job status:', update.status);
};
```

## Deployment Architecture

### Infrastructure Components

**Core Services:**
```yaml
# docker-compose.yml
services:
  indexer:
    build: ./services/indexer
    ports: ["3001:3001"]
    environment:
      - DATABASE_URL=postgresql://user:pass@postgres:5432/indexer
      - RPC_URL=https://api.mainnet-beta.solana.com
    depends_on: [postgres]

  relay:
    build: ./services/relay
    ports: ["3002:3002"]
    environment:
      - DATABASE_URL=postgresql://user:pass@postgres:5432/relay
      - RPC_URL=https://api.mainnet-beta.solana.com
    depends_on: [postgres]

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=cloak
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes: ["postgres_data:/var/lib/postgresql/data"]

```

**Production Deployment:**
```yaml
# kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloak-indexer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cloak-indexer
  template:
    metadata:
      labels:
        app: cloak-indexer
    spec:
      containers:
      - name: indexer
        image: cloak/indexer:latest
        ports:
        - containerPort: 3001
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: cloak-secrets
              key: database-url
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
```

### Monitoring and Observability

**Metrics Collection:**
```rust
// Prometheus metrics
use prometheus::{Counter, Histogram, Gauge};

lazy_static! {
    static ref DEPOSITS_TOTAL: Counter = Counter::new("deposits_total", "Total deposits").unwrap();
    static ref WITHDRAWALS_TOTAL: Counter = Counter::new("withdrawals_total", "Total withdrawals").unwrap();
    static ref PROOF_GENERATION_TIME: Histogram = Histogram::new("proof_generation_seconds", "Proof generation time").unwrap();
    static ref ACTIVE_JOBS: Gauge = Gauge::new("active_jobs", "Active withdrawal jobs").unwrap();
}
```

**Health Checks:**
```rust
// Service health endpoints
pub async fn health_check() -> Result<Json<HealthStatus>> {
    let db_health = check_database_connection().await?;
    let rpc_health = check_solana_rpc().await?;
    
    Ok(Json(HealthStatus {
        status: if db_health && rpc_health { "healthy" } else { "unhealthy" },
        database: db_health,
        rpc: rpc_health,
        timestamp: Utc::now(),
    }))
}
```

## Related Documentation

- **[Visual Flow](./visual-flow.md)** - Detailed sequence diagrams and flow charts
- **[Technology Stack](./tech-stack.md)** - Complete technical stack overview
- **[Quickstart Guide](./quickstart.md)** - Getting started with Cloak
- **[Operations Runbook](../operations/runbook.md)** - Operational procedures and troubleshooting
- **[Shield Pool Program](../onchain/shield-pool.md)** - On-chain program documentation
- **[Scramble Registry Program](../onchain/scramble-registry.md)** - PoW claim management
- **[Indexer Service](../offchain/indexer.md)** - Merkle tree and note discovery
- **[Relay Service](../offchain/relay.md)** - Withdrawal processing and PoW integration
