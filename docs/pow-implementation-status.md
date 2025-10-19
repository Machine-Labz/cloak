# PoW Scrambler Gate - Implementation Status

**Last Updated**: 2025-10-18
**Branch**: `feat/pow-scrambler-gate`
**Status**: ~20% Complete - On-chain program done, off-chain miner needed

---

## Table of Contents

1. [Overview](#overview)
2. [Completed Work](#completed-work)
3. [Architecture](#architecture)
4. [Remaining Work](#remaining-work)
5. [Implementation Plan](#implementation-plan)
6. [Open Questions](#open-questions)
7. [References](#references)

---

## Overview

### Goal
Implement Ore-inspired proof-of-work (PoW) gating for scrambler operations to:
- Prevent spam/DoS attacks on scramblers
- Distribute scrambler fees via competitive mining
- Ensure scramblers are economically incentivized
- Bind work to recent SlotHashes (anti-precomputation)

### Key Design Principles
1. **SlotHashes Binding**: Each PoW solution references a recent slot hash from the SlotHashes sysvar (~300 slots, ~2.5 min window)
2. **256-bit Difficulty**: Little-endian comparison for `BLAKE3(preimage) < difficulty_target`
3. **Anti-Key-Grinding**: Miners must register immutable authority pubkey before mining
4. **Claim Lifecycle**: Mined → Revealed (within reveal_window) → Consumed (within claim_window)
5. **Batch Commitments**: Each claim covers k withdraws via `batch_hash = BLAKE3(job_ids)`
6. **EMA Difficulty Retargeting**: Adjust difficulty to maintain target interval (e.g., 1 solution per 600 slots)

---

## Completed Work

### 1. Specification ✅

**File**: `docs/pow-scrambler-gate.md`

**Contents**:
- Complete PoW preimage format (137 bytes):
  ```
  DOMAIN (17) || slot (8) || slot_hash (32) || miner_pubkey (32) ||
  batch_hash (32) || nonce (16)
  ```
- Difficulty comparison algorithm (256-bit LE)
- Anti-key-grinding via immutable Miner PDAs
- Claim lifecycle state machine
- EMA retargeting formula
- Test vectors with known inputs/outputs
- CU estimates for each instruction
- DoS protection mechanisms (max_k batch size limit)
- Fee share distribution (basis points)

**Key Parameters**:
- Domain: `"CLOAK:SCRAMBLE:v1"`
- Preimage size: 137 bytes
- Nonce size: 128 bits (u128)
- Hash function: BLAKE3
- Difficulty bits: 256
- SlotHashes window: ~300 slots (~2.5 minutes)
- Max fee share: 5000 bps (50%)

---

### 2. On-Chain Program ✅

**Location**: `programs/scramble-registry/`

#### Program Structure

**Cargo.toml**:
```toml
[dependencies]
pinocchio = { workspace = true }
pinocchio-token = { workspace = true }
blake3 = { workspace = true }
```

**Build Status**: ✅ Compiles successfully (release mode)

#### Source Files

##### a. State Structs (`src/state/`)

**registry.rs** - ScrambleRegistry (196 bytes):
```rust
pub struct ScrambleRegistry {
    pub discriminator: [u8; 8],
    pub admin: Pubkey,
    pub current_difficulty: [u8; 32],      // 256-bit LE target
    pub last_retarget_slot: u64,
    pub solutions_observed: u64,
    pub target_interval_slots: u64,        // e.g., 600 slots
    pub fee_share_bps: u16,                // ≤ 5000 (50%)
    pub reveal_window: u64,                // e.g., 150 slots
    pub claim_window: u64,                 // e.g., 300 slots
    pub max_k: u16,                        // DoS limit, e.g., 100
    pub min_difficulty: [u8; 32],
    pub max_difficulty: [u8; 32],
    pub total_claims: u64,
    pub active_claims: u64,
}
```
- Seed: `[b"scramble_registry"]`
- Singleton PDA managing global PoW parameters

**miner.rs** - Miner (64 bytes):
```rust
pub struct Miner {
    pub discriminator: [u8; 8],
    pub authority: Pubkey,          // Immutable (anti-key-grinding)
    pub total_mined: u64,
    pub total_consumed: u64,
    pub registered_at_slot: u64,
}
```
- Seed: `[b"miner", miner_authority]`
- One per miner authority
- Authority cannot be changed after registration

**claim.rs** - Claim (256 bytes):
```rust
pub struct Claim {
    pub discriminator: [u8; 8],
    pub miner_authority: Pubkey,
    pub batch_hash: [u8; 32],      // BLAKE3(job_ids)
    pub slot: u64,                  // Referenced slot
    pub slot_hash: [u8; 32],       // From SlotHashes sysvar
    pub nonce: u128,                // Found via PoW
    pub proof_hash: [u8; 32],      // BLAKE3(preimage)
    pub mined_at_slot: u64,
    pub revealed_at_slot: u64,     // 0 = not revealed
    pub consumed_count: u16,       // How many withdraws used this
    pub max_consumes: u16,         // Batch size k
    pub expires_at_slot: u64,      // revealed_at + claim_window
    pub status: u8,                // ClaimStatus enum
    pub _reserved1: [u8; 32],
    pub _reserved2: [u8; 32],
    pub _padding: [u8; 3],
}
```
- Seed: `[b"claim", miner_authority, batch_hash, mined_slot_le]`
- One per miner + batch combination

**ClaimStatus enum**:
```rust
pub enum ClaimStatus {
    Mined = 0,      // Created but not revealed
    Revealed = 1,   // Revealed within window, ready to consume
    Active = 2,     // Being consumed (alias for Revealed)
    Consumed = 3,   // Fully consumed
    Expired = 4,    // Failed to reveal or consume in time
}
```

##### b. Utility Functions (`src/utils/`)

**difficulty.rs** - 256-bit LE comparison:
```rust
/// Compare two 32-byte arrays as 256-bit little-endian integers
/// Returns: true if a < b
pub fn u256_lt(a: &[u8; 32], b: &[u8; 32]) -> bool {
    for i in (0..32).rev() {
        if a[i] < b[i] { return true; }
        else if a[i] > b[i] { return false; }
    }
    false  // Equal
}
```
- ✅ Comprehensive unit tests (simple, high bytes, equal, max values)

**blake3.rs** - Preimage construction and verification:
```rust
/// Build 137-byte PoW preimage
fn build_preimage(
    slot: u64,
    slot_hash: &[u8; 32],
    miner_pubkey: &Pubkey,
    batch_hash: &[u8; 32],
    nonce: u128,
) -> [u8; 137]

/// Compute BLAKE3 hash of preimage
pub fn hash_pow_preimage(...) -> [u8; 32]

/// Verify proof-of-work: recompute hash and compare
pub fn verify_pow(..., expected_hash: &[u8; 32]) -> bool
```
- ✅ Tests for preimage layout, determinism, nonce changes

##### c. Instructions (`src/instructions/`)

**initialize.rs**:
```rust
/// Instruction 0: initialize_registry
/// One-time setup of ScrambleRegistry singleton
/// Args: initial_difficulty, min_difficulty, max_difficulty,
///       target_interval_slots, fee_share_bps, reveal_window,
///       claim_window, max_k

/// Instruction 1: register_miner
/// Create immutable Miner PDA for authority (anti-key-grinding)
```

**mine_claim.rs**:
```rust
/// Instruction 2: mine_claim
/// Submit PoW solution with SlotHashes verification
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [WRITE] Miner PDA
/// 2. [WRITE] ScrambleRegistry PDA
/// 3. [SIGNER] Miner authority
/// 4. [] SlotHashes sysvar
/// 5. [] Clock sysvar
/// 6. [] System program
///
/// Args: slot, slot_hash, batch_hash, nonce, proof_hash, max_consumes
///
/// Verification steps:
/// 1. Verify slot is recent (< 300 slots old)
/// 2. Verify slot_hash matches SlotHashes sysvar entry
/// 3. Verify proof_hash == BLAKE3(preimage)
/// 4. Check difficulty: proof_hash < current_difficulty (256-bit LE)
/// 5. Verify max_consumes ≤ max_k and > 0
/// 6. Initialize Claim PDA with Mined status
```

**SlotHashes Verification**:
```rust
fn verify_slot_hash(
    slot_hashes_sysvar: &AccountInfo,
    target_slot: u64,
    expected_hash: &[u8; 32],
) -> Result<bool, ProgramError> {
    // Parse SlotHashes sysvar:
    // - u64 count
    // - [(u64 slot, [u8; 32] hash)] * count
    // Returns true if slot found and hash matches
}
```

**reveal_claim.rs**:
```rust
/// Instruction 3: reveal_claim
/// Transition Mined → Revealed within reveal_window
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [] ScrambleRegistry PDA
/// 2. [SIGNER] Miner authority
/// 3. [] Clock sysvar
///
/// Checks:
/// - Status == Mined
/// - Authority matches claim.miner_authority
/// - elapsed_slots ≤ reveal_window
/// - Sets revealed_at_slot, expires_at_slot, status = Revealed
```

**consume_claim.rs**:
```rust
/// Instruction 4: consume_claim
/// Consume one unit from revealed claim (CPI-only from shield-pool)
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [WRITE] Miner PDA
/// 2. [WRITE] ScrambleRegistry PDA
/// 3. [SIGNER] Shield-pool program
/// 4. [] Clock sysvar
///
/// Args: expected_miner_authority, expected_batch_hash
///
/// Anti-replay checks:
/// - Verify miner_authority matches expected
/// - Verify batch_hash matches expected
/// - Verify claim is consumable (revealed, not expired, count < max)
/// - Increment consumed_count
/// - If fully consumed, set status = Consumed, decrement active_claims
```

##### d. Error Codes (`src/error.rs`)

25 distinct error codes:
```rust
pub enum ScrambleError {
    InvalidProofHash = 0,
    DifficultyNotMet = 1,
    SlotHashMismatch = 2,
    SlotHashNotFound = 3,
    RevealWindowExpired = 4,
    ClaimWindowExpired = 5,
    AlreadyRevealed = 6,
    NotRevealed = 7,
    FullyConsumed = 8,
    BatchSizeTooLarge = 9,
    FeeShareTooHigh = 10,
    InvalidMinerAuthority = 11,
    MinerNotRegistered = 12,
    InvalidAdminAuthority = 13,
    ArithmeticOverflow = 14,
    InvalidDifficulty = 15,
    InvalidSlotHashesSysvar = 16,
    UnauthorizedMiner = 17,
    SlotTooOld = 18,
    BatchSizeExceedsMaxK = 19,
    InvalidBatchSize = 20,
    SlotNotFound = 21,
    InvalidClaimStatus = 22,
    ClaimExpired = 23,
    BatchHashMismatch = 24,
}
```

##### e. Main Entrypoint (`src/lib.rs`)

Instruction routing via discriminator (first byte):
- 0: initialize_registry
- 1: register_miner
- 2: mine_claim
- 3: reveal_claim
- 4: consume_claim

---

## Architecture

### On-Chain Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     ScrambleRegistry (PDA)                      │
│  - current_difficulty: [u8; 32]                                 │
│  - reveal_window, claim_window                                  │
│  - max_k, fee_share_bps                                         │
│  - Seed: [b"scramble_registry"]                                 │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │
                     ┌────────┴────────┐
                     │                 │
         ┌───────────▼──────────┐  ┌──▼───────────────────┐
         │    Miner (PDA)       │  │   Claim (PDA)        │
         │  - authority         │  │  - miner_authority   │
         │  - total_mined       │  │  - batch_hash        │
         │  Seed: [b"miner",    │  │  - slot, slot_hash   │
         │         authority]   │  │  - nonce, proof_hash │
         └──────────────────────┘  │  - status: Mined →   │
                                   │    Revealed → Consumed│
                                   │  Seed: [b"claim",    │
                                   │         miner,        │
                                   │         batch_hash,   │
                                   │         slot_le]      │
                                   └──────────────────────┘
```

### Off-Chain Mining Flow (TO BE IMPLEMENTED)

```
┌─────────────────────────────────────────────────────────────────┐
│                         Relay Service                           │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  Claim Manager                           │  │
│  │  - Track active claims (revealed, not expired)           │  │
│  │  - Monitor consumed_count, mine when running low         │  │
│  └────────┬─────────────────────────────────────────────────┘  │
│           │                                                     │
│           ▼                                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  Mining Engine                           │  │
│  │  1. Fetch difficulty from ScrambleRegistry               │  │
│  │  2. Fetch recent SlotHash from RPC                       │  │
│  │  3. Compute batch_hash = BLAKE3(job_ids)                 │  │
│  │  4. Brute-force nonce search:                            │  │
│  │     for nonce in 0.. {                                   │  │
│  │       preimage = build_preimage(slot, slot_hash, ...)    │  │
│  │       hash = BLAKE3(preimage)                            │  │
│  │       if hash < difficulty { return nonce }              │  │
│  │     }                                                     │  │
│  └────────┬─────────────────────────────────────────────────┘  │
│           │                                                     │
│           ▼                                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Transaction Submission                      │  │
│  │  1. Submit mine_claim(slot, slot_hash, nonce, ...)       │  │
│  │  2. Wait for confirmation                                │  │
│  │  3. Submit reveal_claim()                                │  │
│  │  4. Claim now ready to consume                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Shield-Pool Withdraw                         │
│                                                                 │
│  1. Verify proof, nullifier, etc.                              │
│  2. CPI to consume_claim(miner_authority, batch_hash)           │
│  3. Transfer funds to recipient                                │
│  4. Scrambler receives fee share (from registry.fee_share_bps)  │
└─────────────────────────────────────────────────────────────────┘
```

### Claim Lifecycle

```
   mine_claim()           reveal_claim()          consume_claim() × k
┌─────────┐            ┌──────────┐             ┌──────────┐
│  Mined  │───────────▶│ Revealed │────────────▶│ Consumed │
└─────────┘            └──────────┘             └──────────┘
    │                       │
    │ reveal_window         │ claim_window
    │ expires               │ expires
    ▼                       ▼
┌─────────┐            ┌─────────┐
│ Expired │            │ Expired │
└─────────┘            └─────────┘
```

**Timing Windows**:
- **Reveal window**: Time after mining to call `reveal_claim` (e.g., 150 slots)
- **Claim window**: Time after reveal to consume all k units (e.g., 300 slots)
- Both windows checked against current slot from Clock sysvar

---

## Remaining Work

### Critical Path (Must Have)

#### 1. Off-Chain Miner ❌ (BLOCKER)

**Location**: `services/relay/src/miner/`

**Files to Create**:

**a. `mod.rs`** - Public API
```rust
pub mod engine;
pub mod manager;
pub mod rpc;
pub mod instructions;

pub use manager::ClaimManager;
pub use engine::{MiningEngine, MiningSolution};
```

**b. `engine.rs`** - Core mining loop
```rust
pub struct MiningEngine {
    difficulty_target: [u8; 32],
    slot: u64,
    slot_hash: [u8; 32],
    miner_pubkey: Pubkey,
    batch_hash: [u8; 32],
}

pub struct MiningSolution {
    pub nonce: u128,
    pub proof_hash: [u8; 32],
}

impl MiningEngine {
    /// Single-threaded brute-force search
    pub fn mine(&self) -> Option<MiningSolution> {
        for nonce in 0u128.. {
            let preimage = self.build_preimage(nonce);
            let hash = blake3::hash(&preimage);
            if u256_lt(&hash, &self.difficulty_target) {
                return Some(MiningSolution {
                    nonce,
                    proof_hash: hash.into(),
                });
            }

            // Timeout after X iterations?
            if nonce % 10_000_000 == 0 {
                // Log progress, check if slot is still recent
            }
        }
        None
    }

    /// Multi-threaded variant (future optimization)
    pub async fn mine_parallel(&self, num_threads: usize) -> MiningSolution {
        // Split nonce space across threads
        // First thread to find solution wins
    }

    fn build_preimage(&self, nonce: u128) -> [u8; 137] {
        // Reuse logic from scramble-registry/src/utils/blake3.rs
        // DOMAIN || slot || slot_hash || miner_pubkey || batch_hash || nonce
    }
}
```

**c. `manager.rs`** - Claim lifecycle management
```rust
pub struct ClaimManager {
    rpc_client: RpcClient,
    miner_keypair: Keypair,
    registry_pubkey: Pubkey,
    active_claims: HashMap<Vec<u8>, ClaimState>,  // batch_hash -> ClaimState
}

pub struct ClaimState {
    pub pda: Pubkey,
    pub batch_hash: [u8; 32],
    pub revealed_at_slot: u64,
    pub expires_at_slot: u64,
    pub consumed_count: u16,
    pub max_consumes: u16,
}

impl ClaimManager {
    /// Get or create a consumable claim for batch
    pub async fn get_claim(&mut self, batch_hash: [u8; 32]) -> Result<Pubkey> {
        // 1. Check if we have usable claim
        if let Some(state) = self.active_claims.get(&batch_hash.to_vec()) {
            if self.is_claim_usable(state).await? {
                return Ok(state.pda);
            }
        }

        // 2. Otherwise, mine and reveal new claim
        self.mine_and_reveal(batch_hash).await
    }

    async fn mine_and_reveal(&mut self, batch_hash: [u8; 32]) -> Result<Pubkey> {
        // Fetch registry state (difficulty, windows)
        let registry = self.fetch_registry().await?;

        // Fetch recent SlotHash
        let (slot, slot_hash) = self.fetch_recent_slot_hash().await?;

        // Run mining engine
        let engine = MiningEngine {
            difficulty_target: registry.current_difficulty,
            slot,
            slot_hash,
            miner_pubkey: self.miner_keypair.pubkey(),
            batch_hash,
        };

        let solution = engine.mine()
            .ok_or_else(|| anyhow!("Mining failed"))?;

        // Submit mine_claim transaction
        let claim_pda = self.submit_mine_claim(
            slot,
            slot_hash,
            batch_hash,
            solution.nonce,
            solution.proof_hash,
            k, // batch size
        ).await?;

        // Wait for confirmation
        self.wait_for_confirmation(&claim_pda).await?;

        // Submit reveal_claim transaction
        self.submit_reveal_claim(&claim_pda).await?;

        // Track in active claims
        self.active_claims.insert(batch_hash.to_vec(), ClaimState {
            pda: claim_pda,
            batch_hash,
            revealed_at_slot: slot,
            expires_at_slot: slot + registry.claim_window,
            consumed_count: 0,
            max_consumes: k,
        });

        Ok(claim_pda)
    }

    async fn is_claim_usable(&self, state: &ClaimState) -> Result<bool> {
        let current_slot = self.get_current_slot().await?;

        // Check not expired
        if current_slot > state.expires_at_slot {
            return Ok(false);
        }

        // Check not fully consumed
        if state.consumed_count >= state.max_consumes {
            return Ok(false);
        }

        Ok(true)
    }

    /// Update consumed count after withdraw
    pub fn record_consume(&mut self, batch_hash: &[u8; 32]) {
        if let Some(state) = self.active_claims.get_mut(&batch_hash.to_vec()) {
            state.consumed_count += 1;
        }
    }
}
```

**d. `rpc.rs`** - RPC helpers
```rust
/// Fetch ScrambleRegistry account and deserialize
pub async fn fetch_registry(
    client: &RpcClient,
    registry_pubkey: &Pubkey,
) -> Result<ScrambleRegistry> {
    let account = client.get_account(registry_pubkey).await?;

    // Deserialize (cast to ScrambleRegistry struct)
    // Note: Need to add Borsh or manual deserialization
    Ok(deserialize_registry(&account.data)?)
}

/// Fetch SlotHashes sysvar and get most recent entry
pub async fn fetch_recent_slot_hash(
    client: &RpcClient,
) -> Result<(u64, [u8; 32])> {
    let slot_hashes_pubkey = sysvar::slot_hashes::id();
    let account = client.get_account(&slot_hashes_pubkey).await?;

    // Parse SlotHashes: u64 count + [(u64, [u8; 32])] entries
    let data = &account.data;
    let count = u64::from_le_bytes(data[0..8].try_into()?);

    if count == 0 {
        return Err(anyhow!("No slot hashes available"));
    }

    // Return most recent (first entry)
    let slot = u64::from_le_bytes(data[8..16].try_into()?);
    let hash: [u8; 32] = data[16..48].try_into()?;

    Ok((slot, hash))
}

/// Get current slot from Clock sysvar
pub async fn get_current_slot(client: &RpcClient) -> Result<u64> {
    let clock = client.get_sysvar::<Clock>().await?;
    Ok(clock.slot)
}
```

**e. `instructions.rs`** - Transaction builders
```rust
/// Derive Claim PDA
pub fn derive_claim_pda(
    program_id: &Pubkey,
    miner_authority: &Pubkey,
    batch_hash: &[u8; 32],
    slot: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"claim",
            miner_authority.as_ref(),
            batch_hash,
            &slot.to_le_bytes(),
        ],
        program_id,
    )
}

/// Derive Miner PDA
pub fn derive_miner_pda(
    program_id: &Pubkey,
    miner_authority: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"miner", miner_authority.as_ref()],
        program_id,
    )
}

/// Build mine_claim instruction
pub fn build_mine_claim_ix(
    program_id: &Pubkey,
    claim_pda: &Pubkey,
    miner_pda: &Pubkey,
    registry_pda: &Pubkey,
    miner_authority: &Pubkey,
    slot: u64,
    slot_hash: [u8; 32],
    batch_hash: [u8; 32],
    nonce: u128,
    proof_hash: [u8; 32],
    max_consumes: u16,
) -> Instruction {
    // Instruction data: discriminator (1) + args
    let mut data = Vec::new();
    data.push(2); // mine_claim discriminator
    data.extend_from_slice(&slot.to_le_bytes());
    data.extend_from_slice(&slot_hash);
    data.extend_from_slice(&batch_hash);
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&proof_hash);
    data.extend_from_slice(&max_consumes.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*claim_pda, false),
            AccountMeta::new(*miner_pda, false),
            AccountMeta::new(*registry_pda, false),
            AccountMeta::new_readonly(*miner_authority, true),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

/// Build reveal_claim instruction
pub fn build_reveal_claim_ix(
    program_id: &Pubkey,
    claim_pda: &Pubkey,
    registry_pda: &Pubkey,
    miner_authority: &Pubkey,
) -> Instruction {
    let mut data = Vec::new();
    data.push(3); // reveal_claim discriminator

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*claim_pda, false),
            AccountMeta::new_readonly(*registry_pda, false),
            AccountMeta::new_readonly(*miner_authority, true),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data,
    }
}
```

**Integration with Relay**:

In `services/relay/src/config.rs`:
```rust
pub struct RelayConfig {
    // ... existing fields ...

    /// Scramble registry program ID
    pub scramble_registry_program_id: Pubkey,

    /// Miner keypair path (for PoW mining)
    pub miner_keypair_path: String,

    /// Number of mining threads (0 = single-threaded)
    pub mining_threads: usize,
}
```

In `services/relay/src/main.rs`:
```rust
// Initialize claim manager
let miner_keypair = read_keypair_file(&config.miner_keypair_path)?;
let claim_manager = ClaimManager::new(
    rpc_client.clone(),
    miner_keypair,
    config.scramble_registry_program_id,
);
```

In `services/relay/src/worker/processor.rs`:
```rust
pub struct WorkerProcessor {
    // ... existing fields ...
    claim_manager: Arc<Mutex<ClaimManager>>,
}

impl WorkerProcessor {
    async fn process_withdraw_job(&mut self, job: WithdrawJob) -> Result<()> {
        // 1. Compute batch commitment (for now, single job = k=1)
        let batch_hash = compute_batch_hash(&[job.id.clone()]);

        // 2. Get or mine claim
        let claim_pda = self.claim_manager
            .lock()
            .await
            .get_claim(batch_hash)
            .await?;

        // 3. Build withdraw transaction (will include consume_claim CPI)
        let tx = self.build_withdraw_tx(job, claim_pda, batch_hash).await?;

        // 4. Submit transaction
        self.submit_tx(tx).await?;

        // 5. Update claim consumed count
        self.claim_manager
            .lock()
            .await
            .record_consume(&batch_hash);

        Ok(())
    }
}

/// Compute batch hash from job IDs
fn compute_batch_hash(job_ids: &[String]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for id in job_ids {
        hasher.update(id.as_bytes());
    }
    *hasher.finalize().as_bytes()
}
```

**TODO**:
- [ ] Implement `MiningEngine` with brute-force search
- [ ] Add BLAKE3 preimage building (copy from scramble-registry)
- [ ] Implement RPC helpers (fetch registry, SlotHashes)
- [ ] Build instruction helpers (mine_claim, reveal_claim)
- [ ] Implement `ClaimManager` with claim tracking
- [ ] Add transaction submission and confirmation waiting
- [ ] Integrate with relay worker processor
- [ ] Add configuration for miner keypair, registry program ID
- [ ] Test end-to-end mining flow on devnet

---

#### 2. Shield-Pool CPI Integration ❌

**Location**: `programs/shield-pool/src/lib.rs` (withdraw instruction)

**Changes Needed**:

Add to withdraw accounts:
```rust
// Existing accounts
let pool_pda = next_account_info(account_info_iter)?;
let treasury_pda = next_account_info(account_info_iter)?;
// ... other accounts ...

// NEW: PoW scrambler accounts
let scramble_registry_program = next_account_info(account_info_iter)?;
let claim_pda = next_account_info(account_info_iter)?;
let miner_pda = next_account_info(account_info_iter)?;
let registry_pda = next_account_info(account_info_iter)?;
```

Add CPI before withdraw:
```rust
// After proof verification, before transfer

// CPI to consume_claim
let consume_ix_data = build_consume_claim_data(
    miner_authority,  // From claim_pda account data
    batch_hash,       // From withdraw args
);

invoke(
    &Instruction {
        program_id: *scramble_registry_program.key,
        accounts: vec![
            AccountMeta::new(*claim_pda.key, false),
            AccountMeta::new(*miner_pda.key, false),
            AccountMeta::new(*registry_pda.key, false),
            AccountMeta::new_readonly(*pool_pda.key, true), // PDA signer
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data: consume_ix_data,
    },
    &[
        claim_pda.clone(),
        miner_pda.clone(),
        registry_pda.clone(),
        pool_pda.clone(),
        clock_sysvar.clone(),
    ],
)?;

// If CPI succeeds, claim is valid and consumed
// Proceed with withdraw transfer
```

Helper function:
```rust
fn build_consume_claim_data(
    miner_authority: [u8; 32],
    batch_hash: [u8; 32],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.push(4); // consume_claim discriminator
    data.extend_from_slice(&miner_authority);
    data.extend_from_slice(&batch_hash);
    data
}
```

**TODO**:
- [ ] Add scrambler accounts to withdraw instruction
- [ ] Implement consume_claim CPI before transfer
- [ ] Extract miner_authority from claim PDA (or pass as arg)
- [ ] Handle CPI errors (claim expired, invalid, etc.)
- [ ] Update withdraw instruction builder in relay
- [ ] Test CPI flow on devnet

---

#### 3. Transaction Builder Updates ❌

**Location**: `services/relay/src/solana/transaction_builder.rs`

Update `build_withdraw_versioned_with_tip` to include PoW accounts:
```rust
pub fn build_withdraw_versioned_with_tip(
    groth16_260: [u8; 260],
    public_104: [u8; 104],
    recipient: Pubkey,
    // ... existing args ...

    // NEW: PoW args
    claim_pda: Pubkey,
    miner_pda: Pubkey,
    registry_pda: Pubkey,
    scramble_registry_program: Pubkey,
    batch_hash: [u8; 32],
) -> Result<VersionedTransaction, Error> {
    // Build withdraw instruction with additional accounts
    let withdraw_ix = Instruction {
        program_id: shield_pool_program,
        accounts: vec![
            // ... existing accounts ...

            // PoW accounts
            AccountMeta::new_readonly(scramble_registry_program, false),
            AccountMeta::new(claim_pda, false),
            AccountMeta::new(miner_pda, false),
            AccountMeta::new(registry_pda, false),
        ],
        data: build_withdraw_data(
            groth16_260,
            public_104,
            batch_hash,  // NEW: include batch_hash
        ),
    };

    // ... rest of transaction building
}
```

**TODO**:
- [ ] Update withdraw instruction builder
- [ ] Add batch_hash to withdraw instruction data
- [ ] Update all callsites in orchestrator/validator_agent

---

#### 4. Integration Tests ❌

**Location**: `programs/scramble-registry/tests/` or `tooling/test/`

**Test Scenarios**:

**a. Full PoW Mining Flow**:
```rust
#[tokio::test]
async fn test_full_pow_flow() {
    // 1. Initialize registry
    // 2. Register miner
    // 3. Mine claim (find valid nonce off-chain)
    // 4. Reveal claim
    // 5. Consume claim (simulate CPI)
    // 6. Verify claim status transitions
}
```

**b. Difficulty Verification**:
```rust
#[test]
fn test_difficulty_rejection() {
    // Submit mine_claim with hash > difficulty
    // Expect DifficultyNotMet error
}
```

**c. SlotHashes Verification**:
```rust
#[test]
fn test_slot_hash_mismatch() {
    // Submit mine_claim with wrong slot_hash
    // Expect SlotHashMismatch error
}

#[test]
fn test_slot_too_old() {
    // Submit mine_claim with slot > 300 slots ago
    // Expect SlotTooOld error
}
```

**d. Claim Lifecycle**:
```rust
#[test]
fn test_reveal_window_expiry() {
    // Mine claim, wait > reveal_window
    // Attempt reveal_claim
    // Expect RevealWindowExpired error
}

#[test]
fn test_claim_window_expiry() {
    // Mine + reveal claim, wait > claim_window
    // Attempt consume_claim
    // Expect ClaimExpired error
}

#[test]
fn test_fully_consumed() {
    // Consume claim k times
    // Attempt k+1 consume
    // Expect FullyConsumed error
}
```

**e. Anti-Replay**:
```rust
#[test]
fn test_batch_hash_mismatch() {
    // Consume claim with wrong batch_hash
    // Expect BatchHashMismatch error
}
```

**f. End-to-End with Shield-Pool**:
```rust
#[tokio::test]
async fn test_withdraw_with_pow() {
    // 1. Mine + reveal claim
    // 2. Call shield-pool withdraw with claim
    // 3. Verify consume_claim CPI succeeds
    // 4. Verify withdraw completes
    // 5. Verify claim consumed_count incremented
}
```

**TODO**:
- [ ] Set up test environment (Mollusk or solana-program-test)
- [ ] Write unit tests for each instruction
- [ ] Write integration tests for full flow
- [ ] Benchmark CU consumption
- [ ] Test on devnet with real RPC

---

### Important (Should Have)

#### 5. Difficulty Retargeting ❌

**Location**: `programs/scramble-registry/src/instructions/retarget.rs`

Not yet implemented. Needs:
```rust
/// Instruction 5: retarget_difficulty
/// Adjust difficulty based on observed solution rate
///
/// Can be called by anyone (permissionless)
///
/// EMA formula:
/// new_difficulty = current_difficulty * (actual_interval / target_interval)
/// Clamped to [min_difficulty, max_difficulty]
pub fn process_retarget_difficulty(accounts: &[AccountInfo]) -> ProgramResult {
    let registry = ScrambleRegistry::from_account(registry_account)?;
    let clock = Clock::get()?;

    let elapsed_slots = clock.slot - registry.last_retarget_slot;
    let solutions = registry.solutions_observed;

    // Calculate actual interval per solution
    let actual_interval = if solutions > 0 {
        elapsed_slots / solutions
    } else {
        elapsed_slots
    };

    // EMA adjustment
    let adjustment_ratio = actual_interval as f64 / registry.target_interval_slots as f64;
    let new_difficulty = adjust_difficulty_u256(
        &registry.current_difficulty,
        adjustment_ratio,
    );

    // Clamp to bounds
    let clamped = clamp_difficulty(
        &new_difficulty,
        &registry.min_difficulty,
        &registry.max_difficulty,
    );

    registry.current_difficulty = clamped;
    registry.last_retarget_slot = clock.slot;
    registry.solutions_observed = 0;

    msg!("Difficulty retargeted");
    Ok(())
}

/// Adjust 256-bit difficulty by ratio (as f64)
fn adjust_difficulty_u256(current: &[u8; 32], ratio: f64) -> [u8; 32] {
    // Convert to big integer, multiply by ratio, convert back
    // This is non-trivial in on-chain Rust without big int libs
    // May need custom implementation or accept integer-only adjustments
}
```

**TODO**:
- [ ] Implement retarget_difficulty instruction
- [ ] Add 256-bit multiplication/division helpers
- [ ] Test retargeting with various solution rates
- [ ] Add cooldown period to prevent spam retargeting
- [ ] Call retarget periodically (cron job or permissionless)

---

#### 6. Admin Controls ❌

**Location**: `programs/scramble-registry/src/instructions/admin.rs`

Not yet implemented. Needs:
```rust
/// Instruction 6: update_params
/// Admin-only: adjust windows, max_k, fee_share
pub fn process_update_params(
    accounts: &[AccountInfo],
    new_reveal_window: Option<u64>,
    new_claim_window: Option<u64>,
    new_max_k: Option<u16>,
    new_fee_share_bps: Option<u16>,
) -> ProgramResult {
    let registry = ScrambleRegistry::from_account(registry_account)?;
    let admin = next_account_info(account_info_iter)?;

    // Verify admin
    if !admin.is_signer() || *admin.key != registry.admin {
        return Err(ScrambleError::InvalidAdminAuthority.into());
    }

    // Update fields
    if let Some(window) = new_reveal_window {
        registry.reveal_window = window;
    }
    // ... etc

    Ok(())
}

/// Instruction 7: transfer_admin
/// Transfer admin authority to new pubkey
pub fn process_transfer_admin(
    accounts: &[AccountInfo],
    new_admin: Pubkey,
) -> ProgramResult {
    // Verify current admin, update registry.admin
}
```

**TODO**:
- [ ] Implement update_params instruction
- [ ] Implement transfer_admin instruction
- [ ] Add validation (e.g., fee_share ≤ MAX_FEE_SHARE_BPS)
- [ ] Build admin CLI tool

---

#### 7. Client SDK / TypeScript Support ❌

**Location**: `packages/cloak-sdk/` (new package) or add to existing SDK

**Needs**:
```typescript
// PDA derivation
export function deriveClaimPda(
  programId: PublicKey,
  minerAuthority: PublicKey,
  batchHash: Buffer,
  slot: BN
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("claim"),
      minerAuthority.toBuffer(),
      batchHash,
      slot.toArrayLike(Buffer, "le", 8),
    ],
    programId
  );
}

// Instruction builders
export function buildMineClaimIx(
  programId: PublicKey,
  claimPda: PublicKey,
  minerPda: PublicKey,
  registryPda: PublicKey,
  minerAuthority: PublicKey,
  args: {
    slot: BN;
    slotHash: Buffer;
    batchHash: Buffer;
    nonce: BN;
    proofHash: Buffer;
    maxConsumes: number;
  }
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from([2]), // discriminator
    args.slot.toArrayLike(Buffer, "le", 8),
    args.slotHash,
    args.batchHash,
    args.nonce.toArrayLike(Buffer, "le", 16),
    args.proofHash,
    Buffer.from(new Uint16Array([args.maxConsumes]).buffer),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: claimPda, isSigner: false, isWritable: true },
      { pubkey: minerPda, isSigner: false, isWritable: true },
      { pubkey: registryPda, isSigner: false, isWritable: true },
      { pubkey: minerAuthority, isSigner: true, isWritable: false },
      { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}

// Off-chain mining (in browser or Node.js)
export async function minePoW(
  difficulty: Buffer,
  slot: BN,
  slotHash: Buffer,
  minerPubkey: PublicKey,
  batchHash: Buffer
): Promise<{ nonce: BN; proofHash: Buffer }> {
  let nonce = new BN(0);
  while (true) {
    const preimage = buildPreimage(slot, slotHash, minerPubkey, batchHash, nonce);
    const hash = blake3(preimage);

    if (u256Lt(hash, difficulty)) {
      return { nonce, proofHash: hash };
    }

    nonce = nonce.add(new BN(1));

    // Yield to event loop periodically
    if (nonce.mod(new BN(10000)).isZero()) {
      await new Promise(resolve => setTimeout(resolve, 0));
    }
  }
}
```

**TODO**:
- [ ] Create TypeScript package for scramble-registry
- [ ] Implement PDA derivation helpers
- [ ] Implement instruction builders
- [ ] Add off-chain mining function (browser/Node.js)
- [ ] Publish to npm

---

#### 8. Monitoring & Observability ❌

**Needs**:
- Metrics for mining success rate
- Alerts for claim expiry
- Dashboard for registry state (difficulty, solutions, active claims)
- Logs for mining attempts, difficulty changes

**TODO**:
- [ ] Add Prometheus metrics to relay
- [ ] Log mining attempts and success rate
- [ ] Monitor claim pool health (count, expiry rate)
- [ ] Alert on mining failures or expired claims

---

### Nice to Have (Future Optimizations)

#### 9. Multi-Threaded Mining ❌

**Location**: `services/relay/src/miner/engine.rs`

```rust
pub async fn mine_parallel(&self, num_threads: usize) -> MiningSolution {
    use tokio::task;

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    for thread_id in 0..num_threads {
        let tx = tx.clone();
        let engine = self.clone();

        task::spawn_blocking(move || {
            let start_nonce = thread_id as u128 * (u128::MAX / num_threads as u128);

            for i in 0.. {
                let nonce = start_nonce + i;
                let preimage = engine.build_preimage(nonce);
                let hash = blake3::hash(&preimage);

                if u256_lt(&hash, &engine.difficulty_target) {
                    let _ = tx.blocking_send(MiningSolution {
                        nonce,
                        proof_hash: hash.into(),
                    });
                    return;
                }
            }
        });
    }

    // First thread to find solution wins
    rx.recv().await.expect("All threads failed")
}
```

**TODO**:
- [ ] Implement parallel mining with thread pool
- [ ] Benchmark single vs multi-threaded performance
- [ ] Make thread count configurable

---

#### 10. GPU Mining Support ❌

For very high difficulty, GPU acceleration may be needed:
- CUDA/OpenCL kernels for BLAKE3 hashing
- Batch nonce search on GPU
- FFI bindings to Rust

**TODO**:
- [ ] Research BLAKE3 GPU implementations
- [ ] Build GPU miner prototype
- [ ] Benchmark vs CPU mining

---

#### 11. Difficulty Tuning & Analysis ❌

**Needs**:
- Historical difficulty data collection
- Analysis of solution times
- Optimal retargeting parameters (interval, bounds)
- Simulation of different difficulty curves

**TODO**:
- [ ] Collect mainnet mining data
- [ ] Analyze solution time distribution
- [ ] Tune initial difficulty and bounds
- [ ] Automate retargeting frequency

---

#### 12. Batch Optimization ❌

**Current Strategy**: Mine one claim per withdraw (k=1)

**Potential Optimization**: Batch multiple withdraws under one claim (k>1)
- Requires coordinating multiple jobs
- Reduces mining overhead
- More complex claim management

**TODO**:
- [ ] Design batch coordination strategy
- [ ] Implement k>1 claim mining
- [ ] Test performance gains

---

## Implementation Plan

### Phase 1: Core Mining (Week 1) ⚠️ IN PROGRESS

**Goal**: Get basic mining working end-to-end

1. ✅ On-chain program complete
2. ⏳ Implement `MiningEngine` with single-threaded search
3. ⏳ Implement RPC helpers (fetch registry, SlotHashes)
4. ⏳ Build instruction helpers (mine_claim, reveal_claim)
5. ⏳ Test mining flow on localnet
6. ⏳ Integrate with relay (basic)

**Deliverable**: Can mine and reveal claims programmatically

---

### Phase 2: Shield-Pool Integration (Week 2)

**Goal**: Connect PoW to actual withdraws

1. ⏳ Add consume_claim CPI to shield-pool withdraw
2. ⏳ Update transaction builder in relay
3. ⏳ Implement `ClaimManager` with claim tracking
4. ⏳ End-to-end test: withdraw with PoW gate
5. ⏳ Deploy to devnet and test with real jobs

**Deliverable**: Withdraws require valid PoW claims

---

### Phase 3: Production Hardening (Week 3)

**Goal**: Make it robust and observable

1. ⏳ Add comprehensive error handling
2. ⏳ Implement claim expiry monitoring
3. ⏳ Add metrics and logging
4. ⏳ Write integration tests
5. ⏳ Performance tuning (multi-threading?)
6. ⏳ Security audit preparation

**Deliverable**: Production-ready PoW system

---

### Phase 4: Optimization & Tooling (Week 4+)

**Goal**: Scale and operationalize

1. ⏳ Implement difficulty retargeting
2. ⏳ Build admin CLI tools
3. ⏳ Create TypeScript SDK
4. ⏳ Multi-threaded/GPU mining (if needed)
5. ⏳ Mainnet deployment plan
6. ⏳ Documentation and runbooks

**Deliverable**: Scalable, well-documented system

---

## Open Questions

### 1. Batch Size Strategy

**Question**: Should we mine claims for k=1 (single withdraw) or k>1 (batch)?

**Options**:
- **k=1 (Simple)**:
  - ✅ Simpler logic, no coordination needed
  - ✅ More flexible (claim per job)
  - ❌ More mining overhead (more claims needed)

- **k>1 (Optimized)**:
  - ✅ Fewer mining operations (amortized cost)
  - ✅ Better performance at scale
  - ❌ Requires coordinating multiple jobs
  - ❌ Claim expires if not all k jobs consumed

**Recommendation**: Start with k=1, optimize to k>1 later if needed

---

### 2. Mining Timing Strategy

**Question**: When should the relay start mining?

**Options**:
- **Proactive (Continuous)**:
  - Mine ahead of time, keep pool of ready claims
  - ✅ No latency when job arrives
  - ❌ Wastes CPU if no jobs arrive
  - ❌ Claims may expire if unused

- **Reactive (On-Demand)**:
  - Mine when job arrives
  - ✅ No wasted work
  - ❌ High latency (mining blocks job processing)

- **Hybrid (Lazy Pool)**:
  - Mine when claim pool drops below threshold
  - ✅ Balance latency and waste
  - ✅ Adapts to job volume

**Recommendation**: Start reactive, add hybrid pooling if latency becomes issue

---

### 3. Multi-Relay Coordination

**Question**: If multiple relay instances run, do they share claims?

**Options**:
- **Independent (Each relay mines own)**:
  - ✅ Simple, no coordination
  - ❌ Duplicate mining work

- **Shared Pool (Coordinate via DB)**:
  - ✅ No duplicate work
  - ❌ Complex synchronization
  - ❌ Single point of failure (DB)

**Recommendation**: Start independent, add pooling only if needed at scale

---

### 4. Difficulty Initialization

**Question**: What should initial difficulty be?

**Considerations**:
- Too easy: Spam risk, not effective DoS protection
- Too hard: Delays legitimate withdraws, poor UX

**Approach**:
- Start conservative (easier difficulty)
- Monitor solution times on devnet
- Adjust based on real data
- Retarget automatically via EMA

**Recommendation**: Initial difficulty = ~10 seconds on commodity CPU

---

### 5. Fee Share Distribution

**Question**: How are scrambler fees distributed?

**From Spec**: `fee_share_bps` (basis points, ≤ 50%)

**Implementation Needed**:
- Shield-pool withdraw calculates fee
- `fee_share_bps` portion goes to scrambler (miner)
- Rest goes to protocol treasury
- Requires transfer in withdraw instruction

**TODO**: Implement fee distribution logic in shield-pool withdraw

---

### 6. Claim Expiry Handling

**Question**: What happens when a claim expires mid-batch?

**Scenario**:
- Claim revealed at slot 1000, expires at slot 1300
- Consumed 5 times successfully
- At slot 1305, attempt 6th consume → expired

**Options**:
- **Fail withdraw**: Return error, user retries
- **Mine new claim inline**: Delays this withdraw, others succeed
- **Skip to next claim**: Requires claim pool

**Recommendation**: Fail withdraw with clear error, user/relay retries with fresh claim

---

### 7. Testing Strategy

**Question**: How to test PoW mining without waiting?

**Options**:
- **Mock difficulty**: Set to very easy (all 0xFF) for tests
- **Deterministic nonces**: Use known test vectors
- **Fast-forward clock**: Manipulate slot numbers

**Recommendation**: Use easy difficulty + test vectors for unit tests, real mining on devnet

---

## References

### Documentation
- **Specification**: `docs/pow-scrambler-gate.md`
- **This Document**: `docs/pow-implementation-status.md`
- **Architecture**: `docs/ARCHITECTURE.md` (if exists)

### Related Code
- **On-Chain Program**: `programs/scramble-registry/`
- **Relay Service**: `services/relay/`
- **Shield-Pool**: `programs/shield-pool/`

### External Resources
- **Ore Program**: Inspiration for PoW design (https://github.com/regolith-labs/ore)
- **SlotHashes Sysvar**: Solana docs on slot hashes
- **BLAKE3**: https://github.com/BLAKE3-team/BLAKE3
- **Pinocchio**: https://github.com/febo/pinocchio

---

## Maintenance

**Update Checklist** (when making changes):
- [ ] Update "Last Updated" timestamp
- [ ] Check off completed items
- [ ] Add new TODOs as discovered
- [ ] Update status percentage
- [ ] Document any architecture changes
- [ ] Resolve/update open questions as decided

**Review Schedule**:
- Daily during active development
- Weekly once in production
- After any major changes

---

## Appendix: File Tree

```
cloak/
├── docs/
│   ├── pow-scrambler-gate.md           ✅ Specification
│   └── pow-implementation-status.md    ✅ This document
│
├── programs/
│   ├── scramble-registry/              ✅ On-chain program
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  ✅ Entrypoint
│   │       ├── constants.rs            ✅ Domain, sizes
│   │       ├── error.rs                ✅ 25 error codes
│   │       ├── state/
│   │       │   ├── mod.rs
│   │       │   ├── registry.rs         ✅ ScrambleRegistry
│   │       │   ├── miner.rs            ✅ Miner
│   │       │   └── claim.rs            ✅ Claim + ClaimStatus
│   │       ├── utils/
│   │       │   ├── mod.rs
│   │       │   ├── difficulty.rs       ✅ u256_lt
│   │       │   └── blake3.rs           ✅ PoW verification
│   │       └── instructions/
│   │           ├── mod.rs
│   │           ├── initialize.rs       ✅ init_registry, register_miner
│   │           ├── mine_claim.rs       ✅ mine_claim
│   │           ├── reveal_claim.rs     ✅ reveal_claim
│   │           └── consume_claim.rs    ✅ consume_claim
│   │
│   └── shield-pool/
│       └── src/
│           └── lib.rs                  ❌ Needs consume_claim CPI
│
└── services/
    └── relay/
        └── src/
            ├── config.rs               ❌ Add miner config
            ├── main.rs                 ❌ Init ClaimManager
            ├── miner/                  ❌ TO BE CREATED
            │   ├── mod.rs
            │   ├── engine.rs           ❌ Core mining loop
            │   ├── manager.rs          ❌ Claim lifecycle
            │   ├── rpc.rs              ❌ Fetch registry/SlotHashes
            │   └── instructions.rs     ❌ TX builders
            ├── worker/
            │   └── processor.rs        ❌ Integrate ClaimManager
            └── solana/
                └── transaction_builder.rs  ❌ Add PoW accounts

Legend:
✅ Complete
⏳ In Progress
❌ Not Started
```

---

**End of Document**
