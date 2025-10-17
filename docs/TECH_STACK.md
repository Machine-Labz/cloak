# Cloak - Technical Stack & Component Details

## 🏗️ Technology Stack Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND LAYER                           │
├─────────────────────────────────────────────────────────────────┤
│  Framework:       Next.js 14 (React 18)                         │
│  Language:        TypeScript                                    │
│  Styling:         Tailwind CSS + Shadcn UI                      │
│  State:           React Hooks + Context                         │
│  Wallet:          Solana Wallet Adapter                         │
│  Crypto:          @solana/web3.js + WASM Prover                 │
│                                                                 │
│  Key Libraries:                                                 │
│  ├─ @solana/web3.js          : Blockchain interaction          │
│  ├─ @solana/wallet-adapter   : Wallet connection               │
│  ├─ @noble/hashes            : Cryptographic hashing           │
│  ├─ buffer                   : Binary data handling            │
│  └─ Custom WASM prover       : In-browser proof generation     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND SERVICES                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  INDEXER SERVICE                                                │
│  ├─ Language:     Rust 1.75+                                    │
│  ├─ Framework:    Actix-web 4                                   │
│  ├─ Database:     PostgreSQL 15                                 │
│  ├─ ORM:          sqlx (compile-time checked)                   │
│  ├─ Crypto:       blake3                                        │
│  ├─ Async:        tokio                                         │
│  └─ Logging:      tracing + tracing-subscriber                  │
│                                                                 │
│  RELAY SERVICE                                                  │
│  ├─ Language:     Rust 1.75+                                    │
│  ├─ Framework:    Actix-web 4                                   │
│  ├─ Database:     PostgreSQL 15                                 │
│  ├─ ORM:          sqlx                                          │
│  ├─ Solana:       solana-client, solana-sdk                     │
│  └─ Crypto:       blake3, bs58                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    BLOCKCHAIN LAYER                             │
├─────────────────────────────────────────────────────────────────┤
│  Platform:        Solana (v1.17+)                               │
│  Language:        Rust                                          │
│  Framework:       Anchor 0.29                                   │
│  Crypto:          blake3                                        │
│                                                                 │
│  SHIELD-POOL PROGRAM                                            │
│  ├─ Instructions: initialize, deposit, withdraw, admin_push    │
│  ├─ Accounts:     PoolState, MerkleRootState, NullifierSet    │
│  ├─ Validation:   SP1 proof verification                       │
│  └─ Security:     Access controls, input validation            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    ZERO-KNOWLEDGE LAYER                         │
├─────────────────────────────────────────────────────────────────┤
│  Proving System:  SP1 (Succinct)                                │
│  Proof Type:      Groth16 (260 bytes)                           │
│  Target:          RISC-V zkVM                                   │
│  Language:        Rust                                          │
│                                                                 │
│  SP1 GUEST PROGRAM                                              │
│  ├─ Runtime:      SP1 zkVM (RISC-V)                            │
│  ├─ Constraints:  Circuit logic                                │
│  ├─ Crypto:       blake3 (no_std)                              │
│  └─ Output:       ELF binary                                    │
│                                                                 │
│  SP1 HOST PROGRAM                                               │
│  ├─ Runtime:      Native x86/ARM                               │
│  ├─ SDK:          sp1-sdk                                       │
│  ├─ Prover:       SP1 prover (GPU optional)                    │
│  └─ Output:       Groth16 proof + public inputs                │
│                                                                 │
│  WASM PROVER                                                    │
│  ├─ Target:       wasm32-unknown-unknown                       │
│  ├─ Bindings:     wasm-bindgen                                 │
│  ├─ Build:        wasm-pack                                    │
│  └─ Usage:        Browser proof generation                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     DATA STORAGE                                │
├─────────────────────────────────────────────────────────────────┤
│  INDEXER DATABASE (PostgreSQL)                                  │
│  ├─ commitments:       Leaf commitments + metadata             │
│  ├─ merkle_tree:       Tree nodes (all levels)                 │
│  ├─ merkle_roots:      Root history                            │
│  ├─ proof_requests:    Cached proofs                           │
│  └─ deposits:          Deposit events                          │
│                                                                 │
│  RELAY DATABASE (PostgreSQL)                                    │
│  ├─ withdrawal_requests:  Request tracking                     │
│  ├─ used_nullifiers:      Spent note prevention                │
│  ├─ transaction_logs:     Audit trail                          │
│  └─ rate_limits:          Rate limiting state                  │
│                                                                 │
│  ON-CHAIN STORAGE (Solana Accounts)                            │
│  ├─ PoolState:            Global configuration                 │
│  ├─ MerkleRootState:      Current valid root                   │
│  ├─ NullifierSet:         Used nullifiers (HashMap)            │
│  └─ Event Logs:           Deposit/withdraw events              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 Package Structure

```
cloak/
│
├── programs/                      # Solana Programs
│   └── shield-pool/
│       ├── Cargo.toml            # Anchor dependencies
│       └── src/
│           ├── lib.rs            # Program entry point
│           ├── state/
│           │   ├── pool.rs       # PoolState account
│           │   ├── merkle_root.rs
│           │   └── nullifier.rs
│           ├── instructions/
│           │   ├── initialize.rs
│           │   ├── deposit.rs
│           │   ├── withdraw.rs   # SP1 verification
│           │   └── admin_push_root.rs
│           ├── constants.rs      # Fees, limits
│           ├── error.rs          # Custom errors
│           └── utils.rs          # Helper functions
│
├── packages/                      # ZK Components
│   ├── zk-guest-sp1/
│   │   ├── guest/                # Circuit (zkVM)
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── main.rs       # Circuit constraints
│   │   │       └── encoding.rs   # Crypto primitives
│   │   ├── host/                 # Prover
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── lib.rs        # Proving API
│   │   │       ├── encoding.rs   # Input encoding
│   │   │       └── bin/
│   │   │           └── cloak-zk.rs  # CLI tool
│   │   └── out/                  # Build artifacts
│   │       └── public.json       # Verification key
│   │
│   ├── sp1-wasm-prover/          # Browser prover
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs            # WASM bindings
│   │   └── pkg/                  # WASM output
│   │
│   └── vkey-generator/           # VKey utility
│       └── src/
│           └── main.rs
│
├── services/                      # Backend Services
│   ├── indexer/
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── config.rs         # Configuration
│   │       ├── blockchain/
│   │       │   ├── monitor.rs    # Event watcher
│   │       │   └── client.rs     # RPC client
│   │       ├── database/
│   │       │   ├── connection.rs
│   │       │   ├── merkle.rs     # Tree operations
│   │       │   ├── storage.rs    # Data access
│   │       │   └── migrations.rs
│   │       ├── server/
│   │       │   ├── routes.rs     # API endpoints
│   │       │   ├── handlers.rs   # Request handlers
│   │       │   ├── prover_handler.rs
│   │       │   └── rate_limiter.rs
│   │       └── logging.rs
│   │
│   ├── relay/
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── config.rs
│   │       ├── api/
│   │       │   ├── withdraw.rs   # Withdraw endpoint
│   │       │   └── status.rs     # Status endpoint
│   │       ├── db/
│   │       │   ├── mod.rs
│   │       │   └── repository.rs # DB operations
│   │       ├── solana/
│   │       │   ├── client.rs     # Solana client
│   │       │   └── transaction.rs # Tx building
│   │       └── error.rs
│   │
│   └── web/                       # Frontend
│       ├── package.json
│       ├── next.config.mjs
│       ├── app/
│       │   ├── layout.tsx        # Root layout
│       │   ├── page.tsx          # Home page
│       │   └── globals.css
│       ├── components/
│       │   ├── ui/               # Shadcn components
│       │   ├── deposit-form.tsx
│       │   ├── withdraw-form.tsx
│       │   ├── balance-display.tsx
│       │   └── wallet-button.tsx
│       ├── lib/
│       │   ├── solana.ts         # Blockchain utils
│       │   ├── crypto.ts         # Crypto utils
│       │   ├── prover.ts         # WASM prover wrapper
│       │   └── api.ts            # API client
│       └── wasm-prover/
│           └── pkg/              # WASM prover binary
│
├── tooling/                       # Testing & Tools
│   └── test/
│       ├── Cargo.toml
│       └── src/
│           ├── shared.rs         # Common test utils
│           ├── localnet_test.rs  # Local tests
│           └── testnet_test.rs   # Testnet tests
│
├── docs/                          # Documentation
│   ├── README.md
│   ├── ARCHITECTURE_DIAGRAM.md   # This doc!
│   ├── VISUAL_FLOW.md
│   ├── COMPLETE_FLOW_STATUS.md
│   └── zk/
│       ├── design.md
│       ├── circuit-withdraw.md
│       └── ...
│
├── Cargo.toml                     # Workspace config
├── compose.yml                    # Docker Compose
└── justfile                       # Task runner
```

---

## 🔧 Build & Deployment Process

```
┌─────────────────────────────────────────────────────────────────┐
│  SOLANA PROGRAM BUILD                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Development                                                 │
│     ├─ Edit: programs/shield-pool/src/**/*.rs                  │
│     ├─ Build: anchor build                                     │
│     └─ Test: anchor test                                       │
│                                                                 │
│  2. Compilation                                                 │
│     ├─ Target: bpf-solana-solana                               │
│     ├─ Compiler: Solana BPF toolchain                          │
│     ├─ Output: target/deploy/shield_pool.so                    │
│     └─ Size: ~100-200 KB                                       │
│                                                                 │
│  3. Deployment                                                  │
│     ├─ Localnet: anchor deploy --provider.cluster localnet     │
│     ├─ Devnet: anchor deploy --provider.cluster devnet         │
│     ├─ Testnet: solana program deploy ...                      │
│     └─ Mainnet: solana program deploy ... (upgradeable)        │
│                                                                 │
│  4. Verification                                                │
│     ├─ Check: solana program show <PROGRAM_ID>                 │
│     ├─ Logs: solana logs <PROGRAM_ID>                          │
│     └─ Test: Run integration tests                             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  SP1 CIRCUIT BUILD                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Guest Program                                               │
│     ├─ Edit: packages/zk-guest-sp1/guest/src/**/*.rs           │
│     ├─ Build: cargo prove build                                │
│     ├─ Target: riscv32im-succinct-zkvm-elf                     │
│     └─ Output: target/riscv32.../release/cloak-zk-guest        │
│                                                                 │
│  2. Host Program                                                │
│     ├─ Edit: packages/zk-guest-sp1/host/src/**/*.rs            │
│     ├─ Build: cargo build --release                            │
│     └─ Output: target/release/cloak-zk                          │
│                                                                 │
│  3. Verification Key Generation                                 │
│     ├─ Run: cargo run --bin vkey-generator                     │
│     ├─ Uses: SP1 SDK + guest ELF                               │
│     └─ Output: packages/zk-guest-sp1/out/public.json           │
│                                                                 │
│  4. WASM Compilation (for browser)                              │
│     ├─ Build: packages/sp1-wasm-prover/build.sh                │
│     ├─ Tool: wasm-pack                                         │
│     ├─ Target: wasm32-unknown-unknown                          │
│     └─ Output: packages/sp1-wasm-prover/pkg/*.wasm             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  SERVICES DEPLOYMENT                                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Development (Docker Compose)                                   │
│  ├─ Command: docker compose up -d                              │
│  ├─ Services:                                                  │
│  │   ├─ indexer: Rust service                                  │
│  │   ├─ relay: Rust service                                    │
│  │   ├─ postgres_indexer: Database                             │
│  │   ├─ postgres_relay: Database                               │
│  │   └─ web: Next.js frontend                                  │
│  └─ Network: Internal Docker network                           │
│                                                                 │
│  Production                                                     │
│  ├─ Indexer:                                                   │
│  │   ├─ Platform: Kubernetes / Cloud Run                       │
│  │   ├─ Image: FROM rust:1.75-slim                            │
│  │   ├─ Database: Managed PostgreSQL                           │
│  │   └─ Scaling: Horizontal (stateless)                        │
│  │                                                             │
│  ├─ Relay:                                                     │
│  │   ├─ Platform: Kubernetes / Cloud Run                       │
│  │   ├─ Image: FROM rust:1.75-slim                            │
│  │   ├─ Database: Managed PostgreSQL                           │
│  │   └─ Scaling: Horizontal (with distributed locking)         │
│  │                                                             │
│  └─ Frontend:                                                  │
│      ├─ Platform: Vercel / Cloudflare Pages                    │
│      ├─ Framework: Next.js (SSG/ISR)                           │
│      ├─ CDN: Automatic edge distribution                       │
│      └─ Scaling: Automatic                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🧪 Testing Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│  UNIT TESTS                                                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Solana Program                                                 │
│  ├─ cargo test --package shield-pool                           │
│  ├─ Tests: Instruction validation, account constraints         │
│  └─ Mock: BanksClient for blockchain simulation                │
│                                                                 │
│  SP1 Circuit                                                    │
│  ├─ cargo test --package cloak-zk-guest                        │
│  ├─ Tests: Constraint satisfaction, hash functions             │
│  └─ Mock: Test vectors for cryptographic primitives            │
│                                                                 │
│  Services                                                       │
│  ├─ cargo test --package indexer                               │
│  ├─ cargo test --package relay                                 │
│  ├─ Tests: API endpoints, database operations                  │
│  └─ Mock: In-memory database, RPC responses                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  INTEGRATION TESTS                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Localnet Test Suite                                            │
│  ├─ Command: just test-localnet                                │
│  ├─ Binary: tooling/test/localnet_test                         │
│  ├─ Setup:                                                     │
│  │   ├─ Start local validator (solana-test-validator)          │
│  │   ├─ Deploy program to localnet                             │
│  │   ├─ Start indexer service                                  │
│  │   └─ Start relay service                                    │
│  ├─ Tests:                                                     │
│  │   ├─ Initialize pool                                        │
│  │   ├─ Deposit SOL                                            │
│  │   ├─ Verify commitment in tree                              │
│  │   ├─ Generate ZK proof                                      │
│  │   ├─ Submit withdrawal                                      │
│  │   └─ Verify funds received                                  │
│  └─ Assertions:                                                │
│      ├─ Transaction confirmations                              │
│      ├─ Balance changes                                        │
│      ├─ Event emissions                                        │
│      └─ State updates                                          │
│                                                                 │
│  Testnet Test Suite                                             │
│  ├─ Command: just test-testnet                                 │
│  ├─ Binary: tooling/test/testnet_test                          │
│  ├─ Network: Solana Testnet (api.testnet.solana.com)           │
│  ├─ Program: c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp      │
│  └─ Similar tests to localnet but on real testnet              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  END-TO-END TESTS                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Full Flow Test                                                 │
│  ├─ User journey: Connect wallet → Deposit → Wait → Withdraw   │
│  ├─ Components: All services + frontend                        │
│  ├─ Verification: Real SOL transfers, real proofs              │
│  └─ Metrics: Latency, success rate, gas costs                  │
│                                                                 │
│  Performance Test                                               │
│  ├─ Load: Multiple concurrent deposits/withdrawals             │
│  ├─ Stress: High volume transactions                           │
│  └─ Metrics: TPS, proof generation time, database load         │
│                                                                 │
│  Security Test                                                  │
│  ├─ Scenarios:                                                 │
│  │   ├─ Double-spend attempts                                  │
│  │   ├─ Invalid proof submission                               │
│  │   ├─ Nullifier reuse                                        │
│  │   ├─ Merkle proof forgery                                   │
│  │   └─ Front-running attacks                                  │
│  └─ Expected: All attacks properly rejected                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📊 Monitoring & Observability

```
┌─────────────────────────────────────────────────────────────────┐
│  LOGGING                                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Solana Program                                                 │
│  ├─ Tool: msg!() macro, solana logs                            │
│  ├─ Levels: info, debug, error                                 │
│  └─ Monitor: solana logs <PROGRAM_ID>                           │
│                                                                 │
│  Services (Indexer, Relay)                                      │
│  ├─ Library: tracing + tracing-subscriber                      │
│  ├─ Format: JSON structured logs                               │
│  ├─ Levels: trace, debug, info, warn, error                    │
│  ├─ Fields: timestamp, service, request_id, user_id            │
│  └─ Output: stdout (captured by Docker/K8s)                    │
│                                                                 │
│  Log Aggregation                                                │
│  ├─ Tools: Loki, Elasticsearch, Datadog                        │
│  ├─ Query: Filter by service, level, time range                │
│  └─ Alerts: Error rate thresholds                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  METRICS                                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Application Metrics                                            │
│  ├─ Deposits:                                                  │
│  │   ├─ Total count                                            │
│  │   ├─ Success rate                                           │
│  │   ├─ Average amount                                         │
│  │   └─ Latency (p50, p95, p99)                                │
│  │                                                             │
│  ├─ Withdrawals:                                               │
│  │   ├─ Total count                                            │
│  │   ├─ Success rate                                           │
│  │   ├─ Proof generation time                                  │
│  │   └─ Transaction confirmation time                          │
│  │                                                             │
│  ├─ Merkle Tree:                                               │
│  │   ├─ Total commitments                                      │
│  │   ├─ Root update frequency                                  │
│  │   └─ Proof generation latency                               │
│  │                                                             │
│  └─ API:                                                       │
│      ├─ Request rate (RPS)                                     │
│      ├─ Error rate (4xx, 5xx)                                  │
│      └─ Response time                                          │
│                                                                 │
│  System Metrics                                                 │
│  ├─ CPU usage                                                  │
│  ├─ Memory usage                                               │
│  ├─ Database connections                                       │
│  ├─ Disk I/O                                                   │
│  └─ Network I/O                                                │
│                                                                 │
│  Blockchain Metrics                                             │
│  ├─ Transaction success rate                                   │
│  ├─ Compute units used                                         │
│  ├─ Transaction fees                                           │
│  └─ Slot lag (for event monitoring)                            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  ALERTS                                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Critical                                                       │
│  ├─ Service down (health check fails)                          │
│  ├─ Database connection lost                                   │
│  ├─ High error rate (>5%)                                      │
│  └─ Proof verification failures                                │
│                                                                 │
│  Warning                                                        │
│  ├─ High latency (>2s p95)                                     │
│  ├─ Disk space low (<20%)                                      │
│  ├─ Memory usage high (>80%)                                   │
│  └─ Unusual traffic patterns                                   │
│                                                                 │
│  Info                                                          │
│  ├─ New program deployment                                     │
│  ├─ Configuration changes                                      │
│  └─ Scheduled maintenance                                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔐 Security Considerations

```
┌─────────────────────────────────────────────────────────────────┐
│  SMART CONTRACT SECURITY                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ✓ Input Validation                                            │
│    ├─ All amounts checked (min, max, overflow)                 │
│    ├─ Account ownership verified                               │
│    └─ Signature validation on admin functions                  │
│                                                                 │
│  ✓ State Management                                            │
│    ├─ Nullifiers marked atomically                             │
│    ├─ No reentrancy (Solana architecture)                      │
│    └─ Account mutability controlled                            │
│                                                                 │
│  ✓ Access Control                                              │
│    ├─ Admin-only functions (root push, config)                 │
│    ├─ Signer validation                                        │
│    └─ Account discriminators                                   │
│                                                                 │
│  ⚠ Audit Status: Internal review complete                      │
│  📋 TODO: External audit by security firm                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  SERVICE SECURITY                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ✓ API Security                                                │
│    ├─ Rate limiting (per IP, per user)                         │
│    ├─ CORS configured properly                                 │
│    ├─ Input validation and sanitization                        │
│    └─ DDoS protection (Cloudflare)                             │
│                                                                 │
│  ✓ Database Security                                           │
│    ├─ Parameterized queries (sqlx)                             │
│    ├─ Connection pooling                                       │
│    ├─ Encrypted connections (TLS)                              │
│    └─ Backup and recovery                                      │
│                                                                 │
│  ✓ Secret Management                                           │
│    ├─ Environment variables                                    │
│    ├─ No secrets in code                                       │
│    ├─ Keypair file permissions (600)                           │
│    └─ Key rotation procedures                                  │
│                                                                 │
│  ✓ Infrastructure                                              │
│    ├─ HTTPS only (TLS 1.3)                                     │
│    ├─ Firewall rules (minimal exposure)                        │
│    ├─ Security updates automated                               │
│    └─ Intrusion detection                                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  CRYPTOGRAPHIC SECURITY                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ✓ Hash Functions                                              │
│    ├─ BLAKE3-256 (standardized)                                │
│    ├─ Consistent implementation                                │
│    └─ Proper input encoding                                    │
│                                                                 │
│  ✓ Randomness                                                  │
│    ├─ Cryptographically secure RNG                             │
│    ├─ Proper entropy sources                                   │
│    └─ No predictable patterns                                  │
│                                                                 │
│  ✓ ZK Proof System                                             │
│    ├─ SP1 (audited by Succinct)                                │
│    ├─ Groth16 (well-studied)                                   │
│    ├─ Proper constraint system                                 │
│    └─ Trusted setup (if applicable)                            │
│                                                                 │
│  ⚠ Key Management                                              │
│    ├─ User responsible for sk_spend                            │
│    ├─ No key recovery mechanism                                │
│    └─ Loss = permanent loss of funds                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Performance Optimization

```
┌─────────────────────────────────────────────────────────────────┐
│  BLOCKCHAIN OPTIMIZATIONS                                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Program Size                                                   │
│  ├─ Minimize dependencies                                      │
│  ├─ Use anchor-lang efficiently                                │
│  └─ Target size: <200 KB                                       │
│                                                                 │
│  Compute Units                                                  │
│  ├─ Current: ~50K CUs (withdraw)                               │
│  ├─ Limit: 200K CUs per transaction                            │
│  ├─ Optimizations:                                             │
│  │   ├─ Efficient SP1 verifier                                 │
│  │   ├─ Minimal account deserialization                        │
│  │   └─ Batch operations where possible                        │
│  └─ Headroom: 150K CUs (75%)                                   │
│                                                                 │
│  Account Data                                                   │
│  ├─ PoolState: ~1 KB                                           │
│  ├─ MerkleRootState: ~100 bytes                                │
│  ├─ NullifierSet: Grows with usage                             │
│  └─ Rent optimization: Minimal account sizes                   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  PROOF GENERATION                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Current Performance                                            │
│  ├─ CPU (8-core): 30-60 seconds                                │
│  ├─ Memory: ~2-4 GB                                            │
│  └─ Proof size: 260 bytes                                      │
│                                                                 │
│  Optimizations                                                  │
│  ├─ GPU acceleration: 5-10 seconds (optional)                  │
│  ├─ Parallel proving: Multiple proofs simultaneously           │
│  ├─ Circuit optimization: Minimize constraints                 │
│  └─ Caching: Reuse intermediate results                        │
│                                                                 │
│  Future Improvements                                            │
│  ├─ SP1 version upgrades (faster)                              │
│  ├─ WASM prover in browser                                     │
│  └─ Prover-as-a-service (outsource)                            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  DATABASE OPTIMIZATIONS                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Indexing                                                       │
│  ├─ Primary keys: leaf_index, commitment                       │
│  ├─ Indexes: timestamp, nullifier                              │
│  └─ Composite: (level, index) for tree                         │
│                                                                 │
│  Queries                                                        │
│  ├─ Prepared statements (compiled)                             │
│  ├─ Batch inserts for tree updates                             │
│  ├─ Connection pooling                                         │
│  └─ Query caching for frequent reads                           │
│                                                                 │
│  Scaling                                                        │
│  ├─ Read replicas for queries                                  │
│  ├─ Write master for updates                                   │
│  ├─ Partitioning by date/range                                 │
│  └─ Archive old data periodically                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  API OPTIMIZATIONS                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Response Time                                                  │
│  ├─ Target: <100ms (p95)                                       │
│  ├─ Caching: Redis for hot data                                │
│  ├─ CDN: Static responses                                      │
│  └─ Compression: gzip/brotli                                   │
│                                                                 │
│  Throughput                                                     │
│  ├─ Async I/O: tokio runtime                                   │
│  ├─ Connection pooling                                         │
│  ├─ Non-blocking operations                                    │
│  └─ Horizontal scaling                                         │
│                                                                 │
│  Error Handling                                                 │
│  ├─ Graceful degradation                                       │
│  ├─ Circuit breakers                                           │
│  ├─ Retry logic with backoff                                   │
│  └─ Fallback mechanisms                                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📈 Scalability

```
Current Capacity:
├─ Merkle tree: 2^31 = 2.1B commitments
├─ Throughput: ~50 TPS (Solana limited)
├─ Storage: ~100 bytes per commitment
└─ Total: ~200 GB for full tree

Scaling Strategies:
├─ Multiple pools (sharding)
├─ Layer 2 aggregation
├─ Proof batching
└─ Off-chain computation
```

---

## 🎯 Development Commands

```bash
# Build everything
just build

# Test localnet
just start-validator  # Terminal 1
just deploy-local     # Terminal 2
just test-localnet    # Terminal 2

# Test testnet
just test-testnet

# Run services
docker compose up -d

# View logs
docker compose logs -f indexer
docker compose logs -f relay

# Database migrations
sqlx migrate run --database-url $DATABASE_URL

# Generate TypeScript types (frontend)
cd services/web && npm run codegen
```


