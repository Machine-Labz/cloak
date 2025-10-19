# PoW Scrambler Gate Architecture

**Last Updated:** 2025-01-18
**Status:** ✅ Complete (Ore-style standalone architecture)

## Overview

The Cloak protocol uses a Proof-of-Work (PoW) scrambler gate to rate-limit withdrawals and prevent spam. This document describes the complete architecture, inspired by [Ore](https://ore.supply/).

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         STANDALONE MINER                            │
│                                                                     │
│  ┌──────────────────┐                                              │
│  │  cloak-miner CLI │  ← Miners run this independently             │
│  │                  │                                              │
│  │  • Fetch         │                                              │
│  │    difficulty    │                                              │
│  │  • Fetch         │                                              │
│  │    SlotHash      │                                              │
│  │  • Mine (BLAKE3) │                                              │
│  │  • Submit txs    │                                              │
│  └────────┬─────────┘                                              │
│           │                                                         │
└───────────┼─────────────────────────────────────────────────────────┘
            │
            │ mine_claim, reveal_claim transactions
            ↓
┌─────────────────────────────────────────────────────────────────────┐
│                        ON-CHAIN PROGRAMS                            │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  scramble-registry                                           │  │
│  │                                                              │  │
│  │  State:                                                      │  │
│  │  • RegistryState (difficulty, windows, etc.)                │  │
│  │  • MinerAccount (per miner)                                 │  │
│  │  • ClaimAccount (per claim)                                 │  │
│  │                                                              │  │
│  │  Instructions:                                               │  │
│  │  • register_miner  - Create miner PDA                       │  │
│  │  • mine_claim      - Commit hash                            │  │
│  │  • reveal_claim    - Reveal solution                        │  │
│  │  • consume_claim   - Mark as used (CPI only)                │  │
│  │  • update_difficulty - Admin adjusts difficulty             │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                ↑                                    │
│                                │ consume_claim CPI                  │
│                                │                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  shield-pool                                                 │  │
│  │                                                              │  │
│  │  withdraw instruction:                                       │  │
│  │  • Verify ZK proof                                           │  │
│  │  • Call consume_claim CPI ───────────────────────────────────┤  │
│  │  • Transfer funds to recipient                              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
            ↑
            │ withdraw API call
            │
┌───────────┼─────────────────────────────────────────────────────────┐
│           │              RELAY SERVICE                              │
│  ┌────────┴────────┐                                               │
│  │  API Endpoints  │                                               │
│  │                 │                                               │
│  │  POST /deposit  │                                               │
│  │  POST /withdraw │  ← References existing claim from miners     │
│  └─────────────────┘                                               │
│           │                                                         │
│  ┌────────┴────────┐                                               │
│  │  Worker         │                                               │
│  │                 │                                               │
│  │  • Generate     │                                               │
│  │    ZK proof     │                                               │
│  │  • Build tx     │                                               │
│  │  • Submit tx    │                                               │
│  └─────────────────┘                                               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Roles

### 1. Standalone Miner (`cloak-miner`)

**Location:** `packages/cloak-miner/`

**Purpose:** Independent PoW mining CLI that miners run to earn fees

**Key Features:**
- Ore-style standalone binary
- Continuous mining loop
- Claim lifecycle management (mine → reveal → track)
- Graceful shutdown with Ctrl-C

**Commands:**
```bash
# One-time setup
cloak-miner register --keypair ./miner.json --program-id <ID>

# Start mining
cloak-miner mine --timeout 30 --interval 10

# Check status
cloak-miner status
```

**Modules:**
- `engine.rs` - BLAKE3 mining with nonce search
- `rpc.rs` - Fetch registry state and SlotHashes
- `batch.rs` - Batch commitment (BLAKE3 of job IDs)
- `instructions.rs` - Instruction builders
- `manager.rs` - Claim lifecycle management

**Mining Algorithm:**
```rust
// Preimage (137 bytes)
domain       = "CLOAK:SCRAMBLE:v1"  // 17 bytes
slot         = u64 LE               // 8 bytes
slot_hash    = [u8; 32]             // 32 bytes (anti-precomputation)
miner_pubkey = [u8; 32]             // 32 bytes
batch_hash   = [u8; 32]             // 32 bytes
nonce        = u128 LE              // 16 bytes

// Difficulty check
BLAKE3(preimage) < difficulty_target  // 256-bit little-endian comparison
```

### 2. On-Chain Program (`scramble-registry`)

**Location:** `programs/scramble-registry/`

**Purpose:** Store and validate PoW claims on-chain

**State Accounts:**

#### RegistryState (196 bytes)
```rust
pub struct RegistryState {
    admin: Pubkey,                     // 32 bytes
    current_difficulty: [u8; 32],      // 32 bytes
    reveal_window: u64,                // Slots to reveal after mining
    claim_window: u64,                 // Slots claim is valid after reveal
    max_k: u16,                        // Max jobs per claim
    total_miners: u64,
    total_claims: u64,
    total_consumed: u64,
    last_difficulty_update: i64,
}
```

#### MinerAccount (72 bytes)
```rust
pub struct MinerAccount {
    authority: Pubkey,                 // 32 bytes
    total_mined: u64,
    total_consumed: u64,
    last_mine_slot: u64,
}
```

#### ClaimAccount (129 bytes)
```rust
pub struct ClaimAccount {
    miner: Pubkey,                     // 32 bytes
    batch_hash: [u8; 32],
    proof_hash: [u8; 32],
    slot: u64,
    nonce: u128,
    revealed_at_slot: u64,
    consumed_count: u16,
    max_consumes: u16,
    state: ClaimState,                 // Mined, Revealed, Consumed, Expired
}
```

**Instructions:**

1. **initialize_registry** - Admin only, one-time setup
2. **register_miner** - Create miner PDA
3. **mine_claim** - Submit hash commitment
4. **reveal_claim** - Reveal solution within reveal_window
5. **consume_claim** - Mark as used (CPI only, called by shield-pool)
6. **update_difficulty** - Admin adjusts difficulty

**PDA Derivations:**
```rust
// Registry (singleton)
["registry"], program_id → registry_pda

// Miner
["miner", authority], program_id → miner_pda

// Claim
["claim", miner_authority, batch_hash, slot], program_id → claim_pda
```

### 3. Shield Pool Integration

**Location:** `programs/shield-pool/src/instructions/withdraw.rs`

**Additional Accounts (for PoW):**
```rust
// Standard accounts
pool, treasury, roots_ring, nullifier_shard, recipient, system,

// PoW accounts (NEW)
scramble_program,   // Scramble registry program
claim_pda,          // Claim being consumed
miner_pda,          // Miner who created claim
registry_pda,       // Registry state
clock               // Clock sysvar
```

**Instruction Data (469 bytes total):**
```
0-259:   SP1 proof (260 bytes)
260-363: Public inputs (104 bytes)
364-395: Nullifier (32 bytes)
396:     Number of outputs (1 byte)
397-428: Recipient address (32 bytes)
429-436: Recipient amount (8 bytes)
437-468: batch_hash (32 bytes) ← NEW for PoW
```

**CPI Flow:**
```rust
// In withdraw instruction
unsafe {
    // Extract batch_hash from instruction data
    let batch_hash: &[u8; 32] = &*(data.as_ptr().add(437) as *const [u8; 32]);

    // Extract miner authority from miner PDA
    let miner_data = miner_pda_info.try_borrow_data()?;
    let miner_authority: &[u8; 32] = &*(miner_data.as_ptr().add(8) as *const [u8; 32]);

    // Build consume_claim instruction
    let mut consume_ix_data = [0u8; 65];
    consume_ix_data[0] = 4;  // discriminator
    consume_ix_data[1..33].copy_from_slice(miner_authority);
    consume_ix_data[33..65].copy_from_slice(batch_hash);

    // CPI to scramble-registry
    // Accounts: [claim_pda(W), miner_pda(W), registry_pda(W), pool_pda(S), clock]
    pinocchio::program::invoke(...)?;
}
```

### 4. Relay Service

**Location:** `services/relay/`

**Role:** Proof generation and transaction submission (NO MINING)

**Updated Flow:**
1. User requests withdraw
2. Relay generates ZK proof
3. Relay finds available claim from miners (future: claim marketplace/pool)
4. Relay builds withdraw transaction including:
   - ZK proof
   - PoW claim reference (claim_pda, miner_pda, batch_hash)
5. Submit transaction
6. Shield-pool validates proof AND claim via CPI

**Note:** The relay service still has miner code in `src/miner/` but it's **deprecated**. Miners should use the standalone `cloak-miner` CLI instead.

## Claim Lifecycle

```
┌─────────────┐
│   Miner     │
│ generates   │
│   nonce     │
└──────┬──────┘
       │
       │ mine_claim tx
       ↓
┌─────────────┐
│   MINED     │  ← Hash committed on-chain
│   state     │
└──────┬──────┘
       │
       │ reveal_claim tx (within reveal_window)
       ↓
┌─────────────┐
│  REVEALED   │  ← Solution revealed, can be consumed
│   state     │
└──────┬──────┘
       │
       │ consume_claim CPI (from shield-pool)
       ↓
┌─────────────┐
│  CONSUMED   │  ← Claim used (or partially consumed if k>1)
│   state     │
└──────┬──────┘
       │
       │ claim_window expires
       ↓
┌─────────────┐
│   EXPIRED   │  ← Can no longer be used
│   state     │
└─────────────┘
```

## Economic Model

### Miner Revenue
- Earn fees when claims are consumed
- Fee structure TBD (per-claim or pooled)

### Miner Costs
- `mine_claim` tx: ~5,000 lamports
- `reveal_claim` tx: ~5,000 lamports
- Total: ~0.00001 SOL per claim

### User Costs
- Regular withdraw fee (0.0025 SOL + 0.5%)
- No additional PoW fee (included in standard fee)

### Difficulty Adjustment
- Admin can adjust `current_difficulty` based on:
  - Network demand
  - Average mine time
  - Number of active miners

## Anti-Spam Properties

1. **SlotHash Binding** - Prevents precomputation
2. **Reveal Window** - Prevents frontrunning
3. **Claim Window** - Limits claim lifetime
4. **Difficulty Target** - Rate-limits claim creation
5. **Batch Commitment** - Anti-replay (batch_hash uniqueness)

## Testing

### Unit Tests
```bash
# Miner tests (in cloak-miner package)
cargo test --package cloak-miner

# 11 passing tests:
# - Batch hash determinism
# - PDA derivation
# - Difficulty check logic
# - Mining engine
# - Timeout behavior
# - Preimage construction
```

### Integration Tests
```bash
# Requires localnet + deployed programs
cargo test --package cloak-miner -- --ignored

# Tests:
# - register_miner
# - mine_and_reveal_claim
# - ClaimManager full flow
# - Claim expiry
```

### End-to-End Testing

#### 1. Start Localnet
```bash
solana-test-validator
```

#### 2. Deploy Programs
```bash
# Deploy scramble-registry
anchor deploy --program-name scramble-registry

# Deploy shield-pool
anchor deploy --program-name shield-pool
```

#### 3. Initialize Registry
```bash
# TODO: Add initialization script
```

#### 4. Register and Mine
```bash
# Register miner
cloak-miner register \
  --rpc-url http://localhost:8899 \
  --keypair ./test-miner.json \
  --program-id <PROGRAM_ID>

# Start mining
cloak-miner mine \
  --rpc-url http://localhost:8899 \
  --keypair ./test-miner.json \
  --program-id <PROGRAM_ID>
```

#### 5. Test Withdraw
```bash
# Submit withdraw request to relay
curl -X POST http://localhost:3000/api/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "nullifier": "...",
    "recipient": "...",
    "amount": 1000000000,
    "proof": "..."
  }'
```

## File Structure

```
cloak/
├── packages/
│   └── cloak-miner/              ← NEW: Standalone miner CLI
│       ├── src/
│       │   ├── main.rs           # CLI entry point
│       │   ├── lib.rs
│       │   ├── engine.rs         # Mining engine
│       │   ├── rpc.rs            # RPC helpers
│       │   ├── batch.rs          # Batch commitment
│       │   ├── instructions.rs   # Instruction builders
│       │   └── manager.rs        # Claim lifecycle
│       ├── Cargo.toml
│       └── README.md
│
├── programs/
│   ├── scramble-registry/        # PoW validation program
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state/
│   │       │   ├── registry.rs
│   │       │   ├── miner.rs
│   │       │   └── claim.rs
│   │       ├── instructions/
│   │       │   ├── initialize.rs
│   │       │   ├── register_miner.rs
│   │       │   ├── mine_claim.rs
│   │       │   ├── reveal_claim.rs
│   │       │   ├── consume_claim.rs
│   │       │   └── update_difficulty.rs
│   │       └── utils/
│   │
│   └── shield-pool/
│       └── src/
│           └── instructions/
│               └── withdraw.rs   # Updated with PoW CPI
│
├── services/
│   └── relay/
│       └── src/
│           ├── miner/            # DEPRECATED - use cloak-miner CLI
│           └── worker/           # Proof generation only
│
└── docs/
    ├── pow-architecture.md       ← This file
    └── pow-implementation-status.md
```

## Differences from Ore

| Aspect | Ore | Cloak PoW |
|--------|-----|-----------|
| **Algorithm** | Equix (memory-hard) | BLAKE3 (CPU-bound) |
| **Target** | Global bus (shared) | Per-claim (individual) |
| **Binding** | Recent hash | SlotHash sysvar |
| **Lifecycle** | Mine → Submit | Mine → Reveal → Consume |
| **Difficulty** | Dynamic (bus-based) | Admin-adjusted registry |
| **Revenue** | ORE token rewards | Fee distribution |
| **Purpose** | Token mining | Rate-limiting + spam prevention |

## Future Enhancements

### Short-term
- [ ] Claim enumeration and tracking
- [ ] Fee distribution mechanism
- [ ] Claim marketplace/pool for relay

### Medium-term
- [ ] Multi-threaded mining
- [ ] GPU acceleration
- [ ] Difficulty auto-adjustment
- [ ] Profitability calculator

### Long-term
- [ ] Mining pools
- [ ] Cross-program claim sharing
- [ ] Dynamic fee markets
- [ ] ZK proof of work (hide miner identity)

## References

- [Ore Protocol](https://ore.supply/)
- [Equix PoW](https://github.com/tevador/equix)
- [BLAKE3 Spec](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Solana SlotHashes Sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#slothashes)

---

**Status:** ✅ Complete - Ready for testing and deployment
