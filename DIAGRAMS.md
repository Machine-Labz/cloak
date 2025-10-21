# Cloak Architecture & System Diagrams

This document aggregates all the core architectural diagrams and transaction flows of the Cloak protocol, including on-chain programs, relayers, miners, and the zk-proof pipeline.

---

## 1. System Overview (Data & Control Planes)

### High-Level Flow
```
User Wallet → Web Frontend → Indexer (SP1 Prover) → Relay → shield-pool → Receiver
                        ↳ PostgreSQL (Merkle tree, notes, job queue)
PoW Miners → scramble-registry → shield-pool (claim consumption)
System SlotHashes → scramble-registry
```

### Detailed Architecture
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                USER SIDE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│  │   User      │───▶│   Web App   │───▶│  Indexer    │                      │
│  │  Wallet     │    │  (Next.js)  │    │ (Rust API)  │                      │
│  │ (Phantom)   │    │             │    │             │                      │
│  └─────────────┘    └─────────────┘    └─────────────┘                      │
│                           │                   │                             │
│                           ▼                   ▼                             │
│                    ┌─────────────┐    ┌─────────────┐                      │
│                    │ SP1 Prover  │    │ PostgreSQL  │                      │
│                    │ (TEE/Local) │    │ (Merkle DB) │                      │
│                    └─────────────┘    └─────────────┘                      │
│                           │                   │                             │
│                           ▼                   ▼                             │
│                    ┌─────────────┐    ┌─────────────┐                      │
│                    │    Relay     │───▶│ PostgreSQL  │                      │
│                    │ (Job Proc.)  │    │ (Job Queue) │                      │
│                    └─────────────┘    └─────────────┘                      │
│                           │                                               │
│                           ▼                                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                               ON-CHAIN                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│  │ shield-pool │───▶│ scramble-   │◀───│ PoW Miners  │                      │
│  │ (ZK verify) │    │ registry    │    │(cloak-miner)│                      │
│  └─────────────┘    └─────────────┘    └─────────────┘                      │
│         │                   │                   │                           │
│         ▼                   ▼                   │                           │
│  ┌─────────────┐    ┌─────────────┐             │                           │
│  │  Receiver   │    │ SlotHashes  │             │                           │
│  │  Account    │    │   Sysvar    │             │                           │
│  └─────────────┘    └─────────────┘             │                           │
│                                                 │                           │
│                                                 ▼                           │
│                                          ┌─────────────┐                    │
│                                          │ engine +    │                    │
│                                          │ manager     │                    │
│                                          │ (submit     │                    │
│                                          │  claims)    │                    │
│                                          └─────────────┘                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Withdraw Transaction Flow

### Process Overview
```
User → Web App → Indexer (SP1 Prover) → Relay → shield-pool → scramble-registry
```

### Detailed Step-by-Step Flow
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              WITHDRAW FLOW                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │   User      │ 1. Submit withdraw request                                │
│  │  Wallet     │    (outputs, policy, public inputs)                       │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Web App   │ 2. Call Indexer /api/v1/prove                            │
│  │  (Next.js)  │    with private inputs                                    │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │  Indexer    │ 3. Generate SP1 ZK proof                                 │
│  │ (SP1 Prover)│    (TEE or local proving, 30-180s)                       │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Web App   │ 4. Submit proof to Relay /withdraw                       │
│  │  (Next.js)  │    endpoint                                               │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │    Relay    │ 5. Validate proof & find PoW claim                       │
│  │ (Job Proc.) │    via ClaimFinder                                        │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │    Relay    │ 6. Construct PoW-enabled transaction                    │
│  │ (Job Proc.) │    (469 bytes)                                            │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Solana RPC  │ 7. Send transaction with retry logic                     │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ shield-pool │ 8. Verify Groth16 proof & public inputs                  │
│  │ (ZK verify) │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ scramble-   │ 9. CPI to consume valid PoW claim                        │
│  │ registry    │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ shield-pool │ 10. Distribute lamports                                  │
│  │ (ZK verify) │     (receiver, treasury, miner fees)                     │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │    Relay    │ 11. Confirm slot & signature to user                    │
│  │ (Job Proc.) │                                                           │
│  └─────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow Diagram
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   User      │    │   Web App   │    │  Indexer    │
│  Wallet     │───▶│  (Next.js)  │───▶│ (SP1 Prover)│
└─────────────┘    └─────────────┘    └─────────────┘
                           │                   │
                           │                   ▼
                           │            ┌─────────────┐
                           │            │ PostgreSQL  │
                           │            │ (Merkle DB) │
                           │            └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │    Relay    │
                    │ (Job Proc.) │
                    └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ Solana RPC  │
                    └─────────────┘
                           │
                           ▼
                    ┌─────────────┐    ┌─────────────┐
                    │ shield-pool │───▶│ scramble-   │
                    │ (ZK verify) │    │ registry    │
                    └─────────────┘    └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  Receiver   │
                    │  Account    │
                    └─────────────┘
```

---

## 3. PoW Miner Lifecycle

### Miner Architecture
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLOAK MINER                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│  │   Engine    │    │   Batch     │    │  Manager    │                      │
│  │ (BLAKE3     │───▶│ (Job Commit │───▶│ (RPC Submit │                      │
│  │  Search)    │    │  + Wildcard │    │  + Claim    │                      │
│  │             │    │  Support)   │    │  Tracking)  │                      │
│  └─────────────┘    └─────────────┘    └─────────────┘                      │
│         │                   │                   │                           │
│         ▼                   ▼                   ▼                           │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│  │ SlotHash    │    │ Wildcard    │    │ scramble-   │                      │
│  │ Anti-       │    │ Claims      │    │ registry    │                      │
│  │ Precomp.    │    │ (batch_hash │    │ (On-chain)  │                      │
│  │             │    │  = [0; 32]) │    │             │                      │
│  └─────────────┘    └─────────────┘    └─────────────┘                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Claim Lifecycle Flow
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            CLAIM LIFECYCLE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │ Registration│ ← One-time miner registration                             │
│  │ (register_  │                                                           │
│  │  miner)     │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Mining    │ ← Continuous BLAKE3 mining                               │
│  │ (mine_claim │   with SlotHash verification                             │
│  │  + reveal_  │                                                           │
│  │  claim)     │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Active    │ ← Available for withdraws                                │
│  │   Claims    │   (wildcard claims)                                       │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Consumed    │ ← Used by shield-pool withdraws                          │
│  │   Claims    │   (miner earns fees)                                     │
│  └─────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Mining Process Detail
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MINING PROCESS                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │   Fetch     │ 1. Get current difficulty from registry                   │
│  │ Difficulty  │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Fetch     │ 2. Get recent SlotHash (anti-precomputation)             │
│  │ SlotHash    │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Mine      │ 3. BLAKE3 mining with difficulty target                  │
│  │ Valid       │    (137-byte preimage)                                   │
│  │ Nonces      │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Submit    │ 4. mine_claim transaction (commit to hash)               │
│  │ mine_claim  │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Submit    │ 5. reveal_claim transaction (reveal solution)           │
│  │ reveal_claim│                                                           │
│  └─────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### On-Chain Program Interactions
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ scramble-   │◀───│ PoW Miners  │    │ shield-pool │
│ registry    │    │(cloak-miner)│    │ (withdraw)  │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ register_   │    │ mine_claim  │    │ consume_    │
│ miner()     │    │ reveal_     │    │ claim()     │
│             │    │ claim()     │    │ (CPI)       │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Miner PDA   │    │ Claim PDA   │    │ Fee         │
│ Created     │    │ Created     │    │ Distribution│
└─────────────┘    └─────────────┘    └─────────────┘
```

---

## 4. Deposit Flow & Indexing

```
User Wallet → shield-pool::deposit → Ledger
                   ↓
            Indexer (ingests leaves, emits Merkle roots)
                   ↓
            PostgreSQL (Merkle tree storage)
                   ↓
            Indexer API → Web App (Prover / Relay / Wallets)
```

```
╔═══════════════════════════════════════╗
║ Deposit creates encrypted note event  ║
║ Indexer stores leaf + updates root    ║
║ PostgreSQL maintains Merkle tree      ║
║ Clients query latest valid root       ║
║ Web App generates proofs using roots  ║
╚═══════════════════════════════════════╝
```

---

## 5. Withdraw Instruction Layout (PoW-Enabled: 469 bytes)

| Byte Range | Field            | Size (B) | Description                           |
| ---------- | ---------------- | -------- | ------------------------------------- |
| 0–259      | Proof            | 260      | Groth16 fragment                      |
| 260–363    | Public Inputs    | 104      | root, nullifier, outputs_hash, amount |
| 364–395    | Nullifier Dup    | 32       | repeated nullifier                    |
| 396        | Num Outputs      | 1        | number of outputs                     |
| 397–428    | Recipient Pubkey | 32       | recipient account                     |
| 429–436    | Amount           | 8        | recipient amount (little endian)      |
| 437–468    | Batch Hash       | 32       | PoW batch hash (wildcard = [0; 32])  |

**Legacy Format (437 bytes)**: Omits batch hash field for backward compatibility.

---

## 6. Hashing & Proof Extraction

```
Outputs Hash = BLAKE3(address || amount_le)
Public Inputs = root || nullifier || outputs_hash || amount_le
SP1 proof bundle → Groth16 fragment (260 B)
```

```
┌─────────────────────┐
│ Outputs Hash (BLAKE3) ───────────────▶ outputs_hash[32B]
└─────────────────────┘
┌────────────────────────────────────────────┐
│ Public Inputs (104 B)                      │
│  root ∥ nf ∥ outputs_hash ∥ amount_le      │
└────────────────────────────────────────────┘
SP1 bundle ──▶ Groth16 fragment (260 B)
```

---

## 7. Transaction Submission Pipeline

```
Relay → Simulate (CU/fees)
     → if CU too high → Backoff + bump priority fee
     → send via RPC/Jito
     → if RPC error → retry with jitter
     → poll confirmation
     → record slot + fees
```

```
[Relay submit]
   │
   ▼
Simulate CU / fee caps
   │
   ├── no → Backoff + bump fee
   └── yes
         ▼
     Send RPC/Jito
         │
         ├── fail → Retry
         └── ok
               ▼
           Confirm signature
               ▼
           Record slot + fees
```

---

## 8. Full Data Plane Summary

| Component                              | Language       | Role                                                                               |
| -------------------------------------- | -------------- | ---------------------------------------------------------------------------------- |
| **shield-pool**                        | Rust (Pinocchio) | Verifies Groth16, handles deposits & withdrawals, invokes CPI to scramble-registry |
| **scramble-registry**                  | Rust (Pinocchio) | Manages PoW mining claims, miner registration, and claim consumption windows      |
| **cloak-miner**                        | Rust           | Standalone PoW miner that performs BLAKE3 mining and submits wildcard claims      |
| **relay**                              | Rust           | Job processor with database polling, validates proofs, finds PoW claims, submits transactions |
| **indexer**                            | Rust           | Maintains Merkle trees, stores notes, provides SP1 proving service                |
| **web**                                | TypeScript     | Next.js frontend with crypto library, indexer client, and withdraw integration    |
| **zk-guest-sp1**                       | Rust + SP1 SDK | Zero-knowledge proof generation and verification (guest program)                  |
| **zk-verifier-program**                | Rust           | On-chain proof verification using SP1-Solana integration                          |
| **cloak-proof-extract**                | Rust           | Proof extraction utilities for Groth16 fragment generation                       |
| **vkey-generator**                     | Rust           | Verification key generation for SP1 circuits                                     |

---

## 9. SP1 Proving Pipeline

### High-Level Flow
```
Web App → Indexer → SP1 Prover → Proof → Relay
   │         │         │
   │         │         ├── TEE Private Proving (preferred)
   │         │         └── Local Proving (fallback)
   │         │
   │         └── PostgreSQL (Merkle tree data)
   │
   └── Private Inputs (witness data)
```

### Detailed Proving Process
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            SP1 PROVING PIPELINE                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │   Web App   │ 1. User submits withdraw request                         │
│  │  (Next.js)  │    with private inputs (witness data)                    │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │  Indexer    │ 2. Receives /api/v1/prove request                       │
│  │ (Rust API)  │    with witness data                                     │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ PostgreSQL  │ 3. Fetch Merkle tree data                                │
│  │ (Merkle DB) │    (roots, paths, commitments)                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ SP1 TEE     │ 4. Generate proof using TEE Private Proving             │
│  │ Private     │    (30-180 seconds)                                      │
│  │ Proving     │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │  Indexer    │ 5. Return proof bundle and execution report             │
│  │ (Rust API)  │    (proof, public inputs, cycles, syscalls)             │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Web App   │ 6. Submit proof to Relay /withdraw endpoint             │
│  │  (Next.js)  │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │    Relay    │ 7. Validate proof and construct transaction             │
│  │ (Job Proc.) │                                                           │
│  └─────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### TEE vs Local Proving
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PROVING METHODS                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────┐    ┌─────────────────────────────────┐  │
│  │         TEE PROVING             │    │        LOCAL PROVING           │  │
│  │                                 │    │                                 │  │
│  │  ┌─────────────┐                │    │  ┌─────────────┐                │  │
│  │  │ Private     │                │    │  │ Local SP1   │                │  │
│  │  │ Witness     │───▶ TEE        │    │  │ Prover      │───▶ Local      │  │
│  │  │ Data        │    Enclave     │    │  │ Process     │    Machine     │  │
│  │  └─────────────┘                │    │  └─────────────┘                │  │
│  │         │                       │    │         │                       │  │
│  │         ▼                       │    │         ▼                       │  │
│  │  ┌─────────────┐                │    │  ┌─────────────┐                │  │
│  │  │ ZK Proof    │                │    │  │ ZK Proof    │                │  │
│  │  │ + Attestation│                │    │  │ (No         │                │  │
│  │  │             │                │    │  │ Attestation)│                │  │
│  │  └─────────────┘                │    │  └─────────────┘                │  │
│  │                                 │    │                                 │  │
│  │  ✅ Private data protection     │    │  ⚠️  Private data on server     │  │
│  │  ✅ Cryptographic attestation   │    │  ✅ No additional hardware      │  │
│  │  ✅ Faster proving times        │    │  ✅ Same proof format           │  │
│  │  ⚠️  Requires TEE hardware      │    │  ⚠️  Slower proving times       │  │
│  └─────────────────────────────────┘    └─────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 10. TEE Integration Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Web App   │    │   Indexer    │    │ SP1 TEE     │
│             │    │              │    │             │
│ Private     │───▶│ /api/v1/prove │───▶│ Private     │
│ Inputs      │    │              │    │ Proving     │
│             │    │              │    │             │
│ Proof       │◀───│ Proof Bundle  │◀───│ ZK Proof    │
│ Bundle      │    │              │    │             │
└─────────────┘    └──────────────┘    └─────────────┘
```

**TEE Benefits:**
- Private witness data never leaves secure enclave
- Cryptographic attestation of proof generation
- Protection against malicious server operators
- Faster proving times (optimized hardware)

**Fallback Strategy:**
- If TEE unavailable → Local SP1 proving
- Same proof format and verification process
- Maintains protocol compatibility

---

## 11. ClaimFinder System

```
Relay → ClaimFinder → RPC Query → Filter Claims → Select Wildcard
   │         │           │            │
   │         │           │            └── batch_hash = [0; 32]
   │         │           │
   │         │           └── Parse account data (256 bytes)
   │         │
   │         └── Query scramble-registry accounts
   │
   └── Construct PoW-enabled transaction
```

**Wildcard Claims:**
- Miners create claims with `batch_hash = [0; 32]`
- Relay accepts any wildcard claim for any withdrawal
- On-chain program skips batch hash validation for wildcards
- Enables demand-driven mining without coordination

---

## 13. Database Polling Job Processing

### Architecture Overview
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DATABASE POLLING SYSTEM                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │    Relay    │ ← Job creation and API endpoints                          │
│  │ (Job Proc.) │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ PostgreSQL  │ ← Job queue and state management                         │
│  │ (Job Queue) │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │   Worker    │ ← Background polling and processing                      │
│  │  Process    │                                                           │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Solana RPC  │ ← Transaction submission                                 │
│  └─────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Job Processing Flow
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            JOB PROCESSING FLOW                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │ Job        │ 1. Relay creates job record in PostgreSQL                 │
│  │ Creation   │    with 'queued' status                                   │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Worker     │ 2. Background worker polls database                      │
│  │ Polling    │    for queued jobs                                        │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Job        │ 3. Worker processes jobs directly                         │
│  │ Processing │    (no message queue)                                    │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Status     │ 4. Job status updated in database                        │
│  │ Updates    │    ('processing' → 'completed'/'failed')                  │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ Retry      │ 5. Failed jobs requeued with exponential backoff          │
│  │ Logic      │                                                           │
│  └─────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Benefits Comparison
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DATABASE POLLING BENEFITS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────┐                                         │
│  │        DATABASE POLLING         │                                         │
│  │                                 │                                         │
│  │  ✅ Simplicity                  │                                         │
│  │  ✅ Durability (ACID)           │                                         │
│  │  ✅ Observability               │                                         │
│  │  ✅ Scalability                 │                                         │
│  │  ✅ No additional services      │                                         │
│  │  ⚠️  Polling overhead           │                                         │
│  └─────────────────────────────────┘                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 14. Glossary (Core Concepts)

| Term                | Meaning                                                            |
| ------------------- | ------------------------------------------------------------------ |
| **Scrambler**       | PoW-based validator ensuring fair anonymization & liquidity cycles |
| **Claim**           | Proof-of-work record with reveal and consumption windows           |
| **Wildcard Claim**  | PoW claim with batch_hash = [0; 32] for universal compatibility   |
| **ClaimFinder**     | Relay component that discovers available PoW claims for withdrawals |
| **Root Ring**       | Merkle root aggregation ring for shield-pool consistency           |
| **Nullifier Shard** | Prevents double spends via unique nullifier tracking               |
| **Outputs Hash**    | BLAKE3 digest of output address and amount                         |
| **ZK Proof**        | SP1-based Groth16 proof verifying private correctness              |
| **TEE Proving**      | Trusted Execution Environment for private proof generation         |
| **CPI**             | Cross-Program Invocation (shield-pool → scramble-registry)         |
| **Pinocchio**       | Solana program framework used by shield-pool and scramble-registry |
| **SP1**             | Succinct's zero-knowledge proof system for circuit verification   |
