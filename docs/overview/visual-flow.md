---
title: Visual Flow
description: Comprehensive sequence diagrams and ASCII representations for Cloak deposits, withdrawals, PoW mining, and system interactions.
---

# Visual Flow

This page provides detailed visual representations of all major Cloak workflows, including sequence diagrams, data flow charts, and component interaction patterns. These diagrams illustrate the complete transaction lifecycle from user interaction to on-chain execution.

## Deposit Flow

### Complete Deposit Sequence

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Indexer    │    │  Shield Pool    │    │   Merkle     │
│   Wallet    │    │   Service    │    │    Program      │    │    Tree      │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Generate       │                   │                   │
       │    secrets        │                   │                   │
       │ (sk_spend, r)     │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 2. Compute        │                   │                   │
       │    commitment     │                   │                   │
       │ C = H(amt||r||pk) │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 3. Encrypt        │                   │                   │
       │    output         │                   │                   │
       │    payload        │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 4. Submit         │                   │                   │
       │    deposit tx     │                   │                   │
       ├──────────────────────────────────────►│                   │
       │                   │                   │                   │
       │                   │ 5. Listen for     │                   │
       │                   │    DepositCommit  │                   │
       │                   │◄──────────────────┤                   │
       │                   │                   │                   │
       │                   │ 6. Append to      │                   │
       │                   │    Merkle tree    │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │                   │ 7. Store          │                   │
       │                   │    encrypted      │                   │
       │                   │    output         │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │ 8. Confirmation   │                   │                   │
       │◄──────────────────┤                   │                   │
```

**Detailed Steps:**

1. **Secret Generation** - Client generates `sk_spend` (32 bytes) and `r` (32 bytes)
2. **Commitment Computation** - Calculate `C = BLAKE3(amount || r || pk_spend)`
3. **Output Encryption** - Encrypt recipient data with `pk_spend`
4. **Transaction Submission** - Send deposit instruction + SOL transfer
5. **Event Detection** - Indexer listens for `DepositCommit` events
6. **Tree Update** - Append commitment to Merkle tree at next leaf index
7. **Storage** - Store encrypted output for future note discovery
8. **Confirmation** - Client receives transaction confirmation

### Deposit Data Structures

```rust
// Client-side deposit preparation
pub struct DepositData {
    pub amount: u64,                    // Deposit amount in lamports
    pub sk_spend: [u8; 32],            // Note secret key
    pub r: [u8; 32],                   // Randomness
    pub pk_spend: [u8; 32],            // Derived public key
    pub commitment: [u8; 32],          // Computed commitment
    pub encrypted_output: Vec<u8>,     // Encrypted recipient data
}

// On-chain deposit instruction
pub struct DepositInstruction {
    pub commitment: [u8; 32],          // Merkle tree commitment
    pub encrypted_output: Vec<u8>,     // Encrypted output payload
}
```

## Withdrawal Flow

### Standard Withdrawal Sequence

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Relay      │    │   SP1 Prover    │    │   Solana     │
│   Wallet    │    │   Service    │    │                 │    │   Programs   │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Discover       │                   │                   │
       │    spendable      │                   │                   │
       │    notes          │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 2. Fetch Merkle  │                   │                   │
       │    proof          │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 3. Prepare        │                   │                   │
       │    witness        │                   │                   │
       │    data           │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 4. Generate ZK    │                   │                   │
       │    proof          │                   │                   │
       ├──────────────────────────────────────►│                   │
       │                   │                   │                   │
       │ 5. Submit         │                   │                   │
       │    withdraw       │                   │                   │
       │    request        │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 6. Validate       │                   │
       │                   │    inputs         │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 7. Check          │                   │
       │                   │    nullifiers     │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 8. Enqueue job    │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 9. Process job    │                   │
       │                   │    (worker)       │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 10. Build tx      │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │                   │ 11. Submit tx     │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │ 12. Confirmation  │                   │                   │
       │◄──────────────────┤                   │                   │
```

**Detailed Steps:**

1. **Note Discovery** - Scan encrypted outputs, decrypt with `sk_spend`
2. **Merkle Proof** - Fetch inclusion proof from indexer API
3. **Witness Preparation** - Prepare circuit witness data
4. **ZK Proof Generation** - Generate Groth16 proof with SP1
5. **Job Submission** - Submit withdraw request to relay
6. **Input Validation** - Verify proof format and public inputs
7. **Nullifier Check** - Ensure nullifier hasn't been spent
8. **Job Queuing** - Add job to Redis queue for processing
9. **Worker Processing** - Background worker picks up job
10. **Transaction Building** - Construct Solana withdraw transaction
11. **Transaction Submission** - Submit to Solana network
12. **Confirmation** - Receive transaction confirmation

### PoW-Enhanced Withdrawal Sequence

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Relay      │    │  Scramble       │    │   Shield     │
│   Wallet    │    │   Worker     │    │  Registry       │    │   Pool       │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Submit         │                   │                   │
       │    withdraw       │                   │                   │
       │    (with PoW)     │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 2. Find           │                   │
       │                   │    wildcard       │                   │
       │                   │    claim          │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 3. Verify claim   │                   │
       │                   │    availability   │                   │
       │                   │◄──────────────────┤                   │
       │                   │                   │                   │
       │                   │ 4. Build tx       │                   │
       │                   │    with PoW       │                   │
       │                   │    claim          │                   │
       │                   ├──────────────────────────────────────►│
       │                   │                   │                   │
       │                   │ 5. Consume claim  │                   │
       │                   │    via CPI         │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 6. Execute        │                   │
       │                   │    withdraw       │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │ 7. Confirmation   │                   │                   │
       │◄──────────────────┤                   │                   │
```

**PoW-Specific Steps:**

1. **PoW Request** - Submit withdraw with PoW preference enabled
2. **Claim Discovery** - Find available wildcard claim in registry
3. **Claim Validation** - Verify claim is revealed and not expired
4. **Transaction Building** - Include claim consumption in withdraw tx
5. **CPI Call** - Execute `consume_claim` via cross-program invocation
6. **Withdraw Execution** - Complete standard withdraw with PoW proof
7. **Confirmation** - Receive confirmation with claim consumption details

## PoW Mining Flow

### Complete Mining Lifecycle

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Miner     │    │   Mining     │    │  Scramble       │    │   Relay      │
│   Client    │    │   Engine     │    │  Registry       │    │   Workers    │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Register       │                   │                   │
       │    miner          │                   │                   │
       ├──────────────────────────────────────►│                   │
       │                   │                   │                   │
       │ 2. Start mining   │                   │                   │
       │    wildcard       │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 3. Fetch         │                   │
       │                   │    difficulty     │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 4. Mine claim    │                   │
       │                   │    (BLAKE3)       │                   │
       │                   │    nonce loop     │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 5. Submit mine   │                   │
       │                   │    transaction    │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 6. Wait for      │                   │
       │                   │    reveal window  │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │ 7. Reveal claim  │                   │
       │                   │    preimage       │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │                   │ 8. Consume claim  │
       │                   │                   │◄──────────────────┤
       │                   │                   │                   │
       │ 9. Update stats   │                   │                   │
       │◄──────────────────┤                   │                   │
```

**Mining Steps:**

1. **Miner Registration** - Register with scramble registry program
2. **Mining Initialization** - Start BLAKE3 mining with wildcard batch hash
3. **Difficulty Fetch** - Get current difficulty target from registry
4. **Claim Mining** - Iterate nonces until `BLAKE3(preimage) < target`
5. **Mine Submission** - Submit `mine_claim` instruction with hash
6. **Reveal Window** - Wait for reveal window to open
7. **Claim Revelation** - Submit `reveal_claim` with full preimage
8. **Claim Consumption** - Relay workers consume claims via CPI
9. **Statistics Update** - Track mining performance and claim usage

### Mining Data Structures

```rust
// Mining preimage structure
pub struct ClaimPreimage {
    pub domain: [u8; 17],              // "CLOAK:SCRAMBLE:v1"
    pub slot: u64,                    // Current slot
    pub slot_hash: [u8; 32],          // Slot hash
    pub miner_pubkey: [u8; 32],       // Miner public key
    pub batch_hash: [u8; 32],         // [0; 32] for wildcard
    pub nonce: u128,                  // Mining nonce
    pub hash: [u8; 32],               // Resulting hash
}

// Claim account structure
pub struct ClaimAccount {
    pub miner_pda: Pubkey,            // Miner account
    pub miner_authority: Pubkey,      // Miner authority
    pub hash: [u8; 32],              // Claim hash
    pub mined_slot: u64,             // Slot when mined
    pub revealed_slot: Option<u64>,  // Slot when revealed
    pub batch_hash: [u8; 32],        // Batch hash (wildcard = [0; 32])
    pub consumption_count: u8,       // Times consumed
    pub max_consumptions: u8,       // Max consumptions
    pub expires_at_slot: u64,       // Expiration slot
}
```

## System Integration Flows

### Service Communication Pattern

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Indexer    │    │     Relay       │    │   Solana     │
│   Apps      │    │   Service    │    │    Service      │    │   Network    │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ HTTP APIs         │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ PostgreSQL        │                   │
       │                   │◄─────────────────►│                   │
       │                   │                   │                   │
       │                   │                   │ Redis Queue       │
       │                   │                   │◄─────────────────►│
       │                   │                   │                   │
       │                   │                   │ RPC Calls        │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ Event Listening   │
       │                   │◄──────────────────┤                   │
```

### Database Interaction Flow

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Indexer   │    │ PostgreSQL   │    │     Relay       │    │     Redis    │
│   Service   │    │  Database    │    │    Service      │    │    Queue    │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Store Merkle   │                   │                   │
       │    nodes          │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │ 2. Store          │                   │                   │
       │    encrypted      │                   │                   │
       │    outputs        │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │                   │ 3. Store job      │
       │                   │                   │    state          │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ 4. Queue job      │
       │                   │                   │    for processing │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ 5. Update job    │
       │                   │                   │    status         │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ 6. Cache claims   │
       │                   │                   ├──────────────────►│
```

## Error Handling Flows

### Withdrawal Error Recovery

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Relay      │    │   Worker        │    │   Solana     │
│   Wallet    │    │   Service    │    │   Process       │    │   Network    │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Submit         │                   │                   │
       │    withdraw       │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 2. Validate       │                   │
       │                   │    (fails)        │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │ 3. Error          │                   │                   │
       │    response       │                   │                   │
       │◄──────────────────┤                   │                   │
       │                   │                   │                   │
       │ 4. Retry with     │                   │                   │
       │    fixes          │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 5. Process        │                   │
       │                   │    successfully   │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │                   │ 6. Submit tx      │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ 7. Tx fails      │
       │                   │                   │◄──────────────────┤
       │                   │                   │                   │
       │                   │ 8. Retry with    │                   │
       │                   │    backoff       │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │                   │ 9. Success       │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │ 10. Confirmation  │                   │                   │
       │◄──────────────────┤                   │                   │
```

## Performance Optimization Flows

### Parallel Processing Pattern

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Client    │    │   Relay      │    │   Worker Pool   │    │   SP1       │
│   Batch     │    │   Service    │    │   (Multiple)    │    │   Provers   │
└─────────────┘    └──────────────┘    └─────────────────┘    └──────────────┘
       │                   │                   │                   │
       │ 1. Submit         │                   │                   │
       │    batch          │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │                   │                   │
       │                   │ 2. Split into     │                   │
       │                   │    jobs           │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │                   │
       │                   │                   │ 3. Parallel      │
       │                   │                   │    processing    │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ 4. Collect       │
       │                   │                   │    results       │
       │                   │                   │◄──────────────────┤
       │                   │                   │                   │
       │                   │ 5. Aggregate     │                   │
       │                   │    results        │                   │
       │                   │◄──────────────────┤                   │
       │                   │                   │                   │
       │ 6. Batch          │                   │                   │
       │    confirmation   │                   │                   │
       │◄──────────────────┤                   │                   │
```

## Related Documentation

- **[System Architecture](./system-architecture.md)** - Complete architectural overview
- **[Technology Stack](./tech-stack.md)** - Technical implementation details
- **[Deposit Workflow](../workflows/deposit.md)** - Detailed deposit process
- **[Withdrawal Workflow](../workflows/withdraw.md)** - Standard withdrawal process
- **[PoW Withdrawal Workflow](../workflows/pow-withdraw.md)** - PoW-enhanced withdrawals
- **[Operations Runbook](../operations/runbook.md)** - Operational procedures
