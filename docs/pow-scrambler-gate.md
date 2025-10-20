# PoW-Gated Scrambler System (Ore-Inspired)

## Implementation Status

**Last Updated**: 2025-10-19

### ✅ Completed
- ✅ **On-chain program** (`programs/scramble-registry/`): All instructions implemented and tested
- ✅ **Shield-pool CPI integration**: Withdraw instruction calls `consume_claim` (lines 121-172)
- ✅ **Relay transaction builders**: Both legacy (437B) and PoW-enabled (469B) variants
- ✅ **Batch commitment**: Single-job hashing implemented (`packages/cloak-miner/`)
- ✅ **PDA derivation**: Helper functions for all PDAs
- ✅ **Error handling**: Complete error enum with PoW-specific errors

### 🚧 In Progress
- 🚧 **Off-chain miner**: `packages/cloak-miner/` has skeleton, needs integration with relay
- 🚧 **Claim management**: Lifecycle tracking (mine → reveal → consume) needs relay integration
- 🚧 **Fee distribution**: CPI implemented, but treasury → miner transfer needs wiring

### 📋 Planned
- 📋 **Multi-job batches** (k>1): Merkle tree and on-chain inclusion proofs
- 📋 **Difficulty retargeting**: EMA-based adjustment algorithm
- 📋 **Shield-pool program ID binding**: Store in registry, verify CPI caller
- 📋 **Monitoring**: Metrics for claim pool health, mining success rate

### 🔗 File References
- On-chain: `programs/scramble-registry/src/`
- Shield-pool: `programs/shield-pool/src/instructions/withdraw.rs`
- Relay builders: `services/relay/src/solana/transaction_builder.rs`
- Miner lib: `packages/cloak-miner/src/`

---

## Overview

The PoW-gated scrambler system provides **Sybil-resistant** access to withdraw batching rights using **slot-bound proof-of-work**. Scramblers earn fee shares by mining for the right to batch withdrawals, preventing spam, precomputation attacks, and ensuring fair access.

## Design Goals

1. **Sybil Resistance**: Require computational work tied to recent slot hashes
2. **Precomputation Prevention**: Bind PoW to SlotHashes sysvar (Ore-style)
3. **Fair Access**: EMA-based difficulty adjustment targeting ~1 winner per W slots
4. **Withholding Prevention**: Time-bound claim/reveal windows with strict expiry
5. **Fee Distribution**: Scramblers earn bounded share of withdraw fees
6. **Decentralization**: Anyone can register and mine

## Architecture

### On-Chain Components (Pinocchio Program)

#### 1. **ScrambleRegistry** (singleton PDA)
- **Seed**: `[b"scramble_registry"]`
- **Size**: ~128 bytes (rent-exempt: ~0.0009 SOL)
- **Fields**:
  ```rust
  pub struct ScrambleRegistry {
      pub admin: Pubkey,                    // 32 bytes
      pub current_difficulty: [u8; 32],     // 256-bit LE target (H must be < this)
      pub last_retarget_slot: u64,          // When difficulty was last adjusted
      pub solutions_observed: u64,          // Claims mined since last retarget
      pub target_interval_slots: u64,       // Target: 1 solution per W slots (e.g., W=100)
      pub fee_share_bps: u16,               // Scrambler fee share (≤ 5000 = 50%)
      pub reveal_window: u64,               // Slots to reveal after mining (e.g., 10)
      pub claim_window: u64,                // Slots to consume claim after reveal (e.g., 100)
      pub max_k: u16,                       // Max batch size (DoS limit, e.g., 20)
      pub min_difficulty: [u8; 32],         // Floor to prevent trivial mining
      pub max_difficulty: [u8; 32],         // Ceiling to ensure feasibility
      pub total_claims: u64,
      pub active_claims: u64,
  }
  ```

#### 2. **Miner** (PDA per authority, anti-key-grinding)
- **Seed**: `[b"miner", miner_authority]`
- **Size**: ~64 bytes
- **Fields**:
  ```rust
  pub struct Miner {
      pub authority: Pubkey,                // Registered miner authority (immutable)
      pub total_mined: u64,
      pub total_consumed: u64,
      pub registered_at_slot: u64,
  }
  ```
- **Purpose**: Tie miner to a single authority; prevent key-space grinding by requiring registration

#### 3. **Claim** (PDA per miner + batch)
- **Seed**: `[b"claim", miner_authority, batch_hash, mined_slot_le:u64]`
- **Size**: ~256 bytes (rent-exempt: ~0.002 SOL)
- **Fields**:
  ```rust
  pub struct Claim {
      pub miner_authority: Pubkey,          // 32 bytes
      pub batch_hash: [u8; 32],             // BLAKE3 of batch descriptor
      pub slot: u64,                        // Slot when mined
      pub slot_hash: [u8; 32],              // From SlotHashes sysvar (anti-precompute)
      pub nonce: u128,                      // 128-bit nonce (16 bytes)
      pub proof_hash: [u8; 32],             // BLAKE3(preimage)
      pub mined_at_slot: u64,               // Block timestamp
      pub revealed_at_slot: u64,            // When revealed (0 = not revealed)
      pub consumed_count: u16,              // How many withdraws consumed
      pub max_consumes: u16,                // Batch size k (≤ registry.max_k)
      pub expires_at_slot: u64,             // revealed_at + claim_window
      pub status: ClaimStatus,              // Mined | Revealed | Active | Consumed | Expired
      pub _reserved: [u8; 32],              // Future use
  }

  pub enum ClaimStatus {
      Mined,      // Created but not revealed
      Revealed,   // Revealed within window, ready to consume
      Active,     // Being consumed (alias for Revealed)
      Consumed,   // Fully consumed
      Expired,    // Failed to reveal or consume in time
  }
  ```

### Proof-of-Work Specification

#### Preimage Construction
```rust
// Domain tag
const DOMAIN: &[u8] = b"CLOAK:SCRAMBLE:v1";

// Preimage (fixed 129 bytes)
let preimage = [
    DOMAIN,                    // 17 bytes
    slot.to_le_bytes(),        //  8 bytes (u64 LE)
    slot_hash,                 // 32 bytes (from SlotHashes)
    miner_pubkey,              // 32 bytes (from Miner.authority)
    batch_hash,                // 32 bytes
    nonce.to_le_bytes(),       // 16 bytes (u128 LE)
].concat();

// Hash
let proof_hash: [u8; 32] = BLAKE3(preimage);
```

#### Difficulty Comparison
```rust
// Convert proof_hash and difficulty target to 256-bit LE integers
let h = u256::from_le_bytes(proof_hash);
let target = u256::from_le_bytes(registry.current_difficulty);

// Valid iff H < target (lower hash = more work)
assert!(h < target);
```

**Endianness**: Both `proof_hash` and `current_difficulty` are interpreted as **little-endian 256-bit integers**.

#### SlotHashes Verification
- **Sysvar**: `SlotHashes` (account: `SysvarS1otHashes111111111111111111111111111`)
- **Lookup**: `slot_hashes.get(slot)` returns the hash for recent slots (~300 slots)
- **Requirement**: Both `mine_claim` and `reveal_claim` must verify `(slot, slot_hash)` against SlotHashes
- **Anti-precompute**: Miners cannot precompute work for future slots

### Difficulty Retargeting

#### Algorithm: EMA-based adjustment
```rust
// Target: 1 solution per `target_interval_slots` (e.g., W=100)
let elapsed_slots = current_slot - registry.last_retarget_slot;
let solutions = registry.solutions_observed;

// Expected solutions in elapsed period
let expected = elapsed_slots / registry.target_interval_slots;

// Ratio: actual / expected
let ratio = solutions as f64 / expected as f64;

// New difficulty (clamp change to ±20% per epoch)
let adjustment = ratio.clamp(0.8, 1.2);
let new_difficulty = (current_difficulty_u256 as f64 * adjustment) as u256;

// Enforce bounds
new_difficulty = new_difficulty.clamp(registry.min_difficulty, registry.max_difficulty);
```

**Retarget Frequency**: Every `N` slots or `M` solutions (whichever comes first), e.g., every 1000 slots or 100 solutions.

**Initialization**:
- `min_difficulty`: ~2^200 (very hard, prevents spam)
- `max_difficulty`: ~2^255 (trivial, ensures mining is always possible)
- `current_difficulty`: start at median

### Off-Chain Components (Relay/Scrambler)

#### 1. **Batch Descriptor**
```rust
pub struct BatchDescriptor {
    pub version: u8,                      // Protocol version
    pub round: u64,                       // Monotonic round counter
    pub roots_window: Vec<[u8; 32]>,      // Valid roots (recent N roots)
    pub jobs_root: [u8; 32],              // Merkle root of job commits
    pub k: u16,                           // Batch size (≤ max_k)
    pub policy: BatchPolicy,              // Selection criteria
    pub expiry_slot: u64,                 // Deadline
}

pub enum BatchPolicy {
    FIFO,           // First-in-first-out
    HighestFee,     // Prioritize by fee
    Random,         // Random selection
}
```

**Batch Hash Computation**:
```rust
let batch_hash = BLAKE3([
    b"CLOAK:BATCH:v1",
    version.to_le_bytes(),
    round.to_le_bytes(),
    roots_window.concat(),           // Sorted roots
    jobs_root,
    k.to_le_bytes(),
    policy.to_bytes(),
    expiry_slot.to_le_bytes(),
].concat());
```

#### 2. **Job Commitment**

**✅ IMPLEMENTED** (single-job batches): `packages/cloak-miner/src/batch.rs`

```rust
/// Compute batch hash from job IDs
pub fn compute_batch_hash(job_ids: &[String]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for job_id in job_ids {
        hasher.update(job_id.as_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Single-job convenience wrapper (MVP, k=1)
pub fn compute_single_job_hash(job_id: &str) -> [u8; 32] {
    compute_batch_hash(&[job_id.to_string()])
}
```

**Current Status**: MVP uses **k=1** (one claim per job). The batch_hash is simply `BLAKE3(job_id)`.

**Future Enhancement** (for k>1 batches): Merkle tree over job commits with on-chain inclusion proof verification:
```rust
// Per-job commitment (planned for multi-job batches)
let job_commit = BLAKE3([
    b"CLOAK:JOB:v1",
    public_104,                       // 104-byte public inputs
    recipient_addr_32,                // 32-byte recipient pubkey
    recipient_amount.to_le_bytes(),   // u64 LE
].concat());

// Jobs root (Merkle tree over sorted job commits)
jobs_root = merkle_root(sorted(job_commits));
```

#### 3. **Mining Loop** (Off-Chain)
```rust
// Fetch recent slot and slot_hash from RPC
let slot = rpc.get_slot().await?;
let slot_hash = rpc.get_slot_hash(slot).await?;

// Fetch current difficulty
let registry = fetch_scramble_registry().await?;
let target = u256::from_le_bytes(registry.current_difficulty);

// Build batch descriptor and compute batch_hash
let batch_hash = compute_batch_hash(&descriptor);

// Mine
let mut nonce: u128 = fastrand::u128(..);
loop {
    let preimage = build_preimage(slot, slot_hash, miner_authority, batch_hash, nonce);
    let proof_hash = BLAKE3(preimage);
    let h = u256::from_le_bytes(proof_hash);

    if h < target {
        // Found valid PoW!
        return MineResult { slot, slot_hash, nonce, proof_hash };
    }
    nonce = nonce.wrapping_add(1);
}
```

## Protocol Flow

### Phase 0: Miner Registration (One-Time)
1. Scrambler calls `register_miner` with their authority pubkey
2. Creates `Miner` PDA with immutable authority
3. **Anti-key-grinding**: Cannot change authority post-registration

### Phase 1: Mining
1. Scrambler builds `BatchDescriptor` from pending jobs
2. Computes `batch_hash = BLAKE3(batch_descriptor)`
3. Fetches current `(slot, slot_hash)` from RPC
4. Mines: finds `nonce` where `BLAKE3(preimage) < difficulty`
5. Submits `mine_claim` instruction

### Phase 2: Reveal (Two Paths)

#### Option A: Two-Step (mine → reveal)
1. `mine_claim`: Creates claim with status `Mined`
2. Within `reveal_window` slots, scrambler calls `reveal_claim`
3. On-chain verifies:
   - `current_slot <= mined_at_slot + reveal_window`
   - SlotHashes lookup for `(slot, slot_hash)` still valid
4. Claim status → `Revealed`, set `expires_at_slot = current_slot + claim_window`

#### Option B: Single-Step (claim_pow, optional)
1. Combined `claim_pow` instruction (mine + reveal in one tx)
2. Immediately activates claim if within reveal window
3. **Simpler**: Fewer transactions, faster activation
4. **Tradeoff**: Must submit within reveal window from mining slot

### Phase 3: Batch Execution (Withdraw CPI)
1. For each withdraw in batch:
   - **Shield-pool withdraw instruction runs**:
     1. Verify SP1 proof (Groth16 260B)
     2. Check withdraw invariants:
        - Root in `roots_window`
        - Nullifier not spent
        - `outputs_hash` matches
        - Fee conservation holds
     3. **After all checks pass**, CPI to `consume_claim`:
        - Verify claim is `Revealed` and not expired
        - Verify `consumed_count < max_consumes`
        - Increment `consumed_count`
        - Transfer `withdraw_fee * fee_share_bps / 10000` to `miner_authority`
        - If `consumed_count == max_consumes`: status → `Consumed`

**CPI Ordering**: `consume_claim` is called **after** SP1 verification and all invariants, so a failing withdraw cannot increment consumption or leak rewards.

2. **Fee Calculation** (explicit formula from `withdraw.rs` lines 101-109):
   ```rust
   // Total fee computation
   let fixed_fee = 2_500_000; // 0.0025 SOL
   let variable_fee = (public_amount * 5) / 1_000; // 0.5% of public_amount
   let total_fee = fixed_fee + variable_fee;
   
   // Conservation check
   let recipient_amount = public_amount - total_fee;
   ```
   
   **Key**: `public_amount` is the amount from public inputs (amount being withdrawn from pool).
   `recipient_amount` = `public_amount` - `total_fee`.

3. **Fee Share Transfer** (🚧 TODO: needs implementation):
   ```rust
   // After consume_claim succeeds, transfer scrambler share
   let scrambler_share = total_fee
       .checked_mul(registry.fee_share_bps as u64)
       .unwrap()
       .checked_div(10_000)
       .unwrap();

   // Transfer from treasury to miner_authority
   // NOTE: Currently treasury receives full fee; scrambler share transfer TODO
   unsafe {
       *treasury_info.borrow_mut_lamports_unchecked() += total_fee - scrambler_share;
       *miner_authority_info.borrow_mut_lamports_unchecked() += scrambler_share;
   }
   ```
   
   **Current Status**: Shield-pool transfers `total_fee` to treasury (line 178). Scrambler fee share **not yet distributed** - this is a remaining TODO.

### Phase 4: Expiry Handling
- **Not revealed in time**: Claim status remains `Mined`, expires automatically; no state cleanup needed (rent reclaim optional)
- **Not fully consumed**: If `current_slot > expires_at_slot` and `consumed_count < max_consumes`, remaining slots are forfeited; no partial refunds

## Instructions

### 1. `initialize_registry`
- **Accounts**: `[registry (init), admin (signer, pays), system_program]`
- **Args**:
  ```rust
  pub struct InitializeRegistryArgs {
      pub initial_difficulty: [u8; 32],
      pub min_difficulty: [u8; 32],
      pub max_difficulty: [u8; 32],
      pub target_interval_slots: u64,
      pub fee_share_bps: u16,          // ≤ 5000 (50%)
      pub reveal_window: u64,          // e.g., 10 slots
      pub claim_window: u64,           // e.g., 100 slots
      pub max_k: u16,                  // e.g., 20
  }
  ```
- **Logic**: Creates singleton registry with parameters; sets admin

### 2. `register_miner`
- **Accounts**: `[miner (init), authority (signer, pays), system_program]`
- **Args**: None (authority is signer)
- **Logic**:
  - Create `Miner` PDA with `authority = signer.key()`
  - Immutable after creation (anti-key-grinding)

### 3. `mine_claim`
- **Accounts**:
  ```
  [
    claim (init),
    miner,
    miner_authority (signer, pays rent),
    registry (mut),
    slot_hashes (sysvar),
    system_program
  ]
  ```
- **Args**:
  ```rust
  pub struct MineClaimArgs {
      pub batch_hash: [u8; 32],
      pub slot: u64,
      pub slot_hash: [u8; 32],
      pub nonce: u128,
      pub proof_hash: [u8; 32],
      pub max_consumes: u16,           // Batch size k (≤ max_k)
  }
  ```
- **Logic**:
  1. Verify `slot_hash` against `SlotHashes` sysvar at `slot`
  2. Recompute preimage and hash:
     ```rust
     let preimage = build_preimage(slot, slot_hash, miner.authority, batch_hash, nonce);
     let recomputed = BLAKE3(preimage);
     assert_eq!(proof_hash, recomputed);
     ```
  3. Verify difficulty: `u256::from_le_bytes(proof_hash) < u256::from_le_bytes(registry.current_difficulty)`
  4. Verify `max_consumes <= registry.max_k`
  5. Create `Claim` with status `Mined`, set `mined_at_slot = current_slot`
  6. Increment `registry.solutions_observed`
  7. Charge small fee or require rent-exempt minimum (~0.002 SOL) to deter spam

- **DoS Protection**: CU budget ~30k (one BLAKE3 + SlotHashes lookup + minimal writes)

### 4. `reveal_claim`
- **Accounts**:
  ```
  [
    claim (mut),
    miner,
    miner_authority (signer),
    registry,
    slot_hashes (sysvar)
  ]
  ```
- **Args**: None
- **Logic**:
  1. Verify `current_slot <= claim.mined_at_slot + registry.reveal_window`
  2. Re-verify `(claim.slot, claim.slot_hash)` against SlotHashes (may have rotated out; handle gracefully)
  3. Update `claim.revealed_at_slot = current_slot`
  4. Set `claim.expires_at_slot = current_slot + registry.claim_window`
  5. Update `claim.status = Revealed`

**Note**: If SlotHashes no longer contains the slot (>300 slots old), fail gracefully with error.

### 5. `claim_pow` (Optional Single-Step)
- **Accounts**: Same as `mine_claim` + `reveal_claim` combined
- **Args**: `MineClaimArgs` (same as `mine_claim`)
- **Logic**:
  1. Perform `mine_claim` validation
  2. Immediately perform `reveal_claim` validation
  3. Create claim with status `Revealed` and set `expires_at_slot`
- **Benefit**: One transaction, faster activation
- **Constraint**: Must be called within `reveal_window` of mining slot

### 6. `consume_claim` (CPI from shield-pool withdraw)
- **Accounts**:
  ```
  [
    claim (mut),
    registry,
    miner_authority (receives fee share),
    treasury (pays fee share),
    system_program
  ]
  ```
- **Args**:
  ```rust
  pub struct ConsumeClaimArgs {
      pub withdraw_fee: u64,
  }
  ```
- **Logic**:
  1. Verify claim is `Revealed` and `current_slot <= expires_at_slot`
  2. Verify `consumed_count < max_consumes`
  3. Increment `consumed_count`
  4. Compute scrambler share: `withdraw_fee * fee_share_bps / 10_000`
  5. Transfer share from `treasury` to `miner_authority`
  6. If `consumed_count == max_consumes`: `status = Consumed`

**Called From**: Shield-pool `withdraw` instruction **after** SP1 proof verification and all invariants pass.

### 7. `adjust_difficulty` (Admin or Automated)
- **Accounts**: `[registry (mut), admin (signer)]`
- **Args**:
  ```rust
  pub struct AdjustDifficultyArgs {
      pub new_difficulty: [u8; 32],
  }
  ```
- **Logic**:
  1. Verify signer is `registry.admin` (or automated condition: elapsed slots > retarget interval)
  2. Apply EMA-based retarget (see algorithm above)
  3. Clamp to `[min_difficulty, max_difficulty]`
  4. Update `registry.current_difficulty`, reset `solutions_observed`, set `last_retarget_slot`

## Security Considerations

### 1. **SlotHashes Binding (Anti-Precompute)**
- Mining is bound to recent slot hashes from SlotHashes sysvar
- Prevents precomputing work for future slots
- SlotHashes rotates every ~300 slots; claims older than this cannot be revealed

### 2. **Difficulty Bounds**
- `min_difficulty` prevents trivial mining
- `max_difficulty` ensures mining is always feasible
- EMA retarget clamped to ±20% per epoch prevents extreme swings

### 3. **Anti-Key-Grinding**
- Miner PDA ties work to a single `authority` (immutable)
- Cannot mine with different keys to explore key space
- Nonce is 128 bits (sufficient search space per authority)

### 4. **Replay Prevention**
- Claim PDA seeds include `[miner_authority, batch_hash, mined_slot]`
- Cannot reuse same claim across different slots or batches
- Each slot+batch requires fresh mining

### 5. **Withholding Prevention**
- Strict `reveal_window`: must reveal within N slots or claim expires
- Strict `claim_window`: must consume within M slots after reveal or forfeit
- No carryover or partial refunds

### 6. **DoS Resistance**
- `max_k` bounds batch size to prevent huge claims
- Rent-exempt minimum on claims deters spam
- Difficulty adjustment prevents flooding during low-difficulty periods

### 7. **Fee Share Bounds**
- `fee_share_bps` capped at 5000 (50%) to prevent extraction
- Admin adjustments require governance or time-lock (future)

### 8. **CPI Ordering (Withdraw Integration)**
- `consume_claim` called **after** SP1 verification and all invariants
- Failed withdraws cannot increment consumption or transfer fees
- Ensures scrambler only earns for valid, successful withdraws

## Economic Model

### Fee Distribution (Example: 0.0025 SOL base + 0.5% variable)
```
Total Fee = 2_500_000 + (amount * 5 / 1000) lamports

Distribution:
- Protocol Treasury: 50%  (1_250_000 + ...)
- Scrambler (miner): 20%  (500_000 + ...)   ← fee_share_bps = 2000
- Prover:            20%  (500_000 + ...)
- LP/Liquidity:      10%  (250_000 + ...)
```

### Scrambler Revenue (per batch, k=10)
```
Per withdraw: fee * 0.20
Per batch:    fee * 0.20 * k = 10 * (500_000 + ...) lamports

Mining cost:  electricity + opportunity cost
Profit:       Revenue - Mining Cost

Break-even difficulty ≈ f(hardware, electricity price, fee level)
```

## Compatibility with shield-pool MVP

### Account Additions (Withdraw Instruction)
**Existing accounts** (unchanged):
```
[pool, treasury, roots_ring, nullifier_shard, recipient, system_program]
```

**New accounts** (for PoW integration):
```
[
  claim,                 // Claim PDA
  miner,                 // Miner PDA
  miner_authority,       // Scrambler's authority (receives fee share)
  scramble_registry,     // Registry PDA
  scramble_program,      // Scramble registry program ID
  slot_hashes,           // SlotHashes sysvar (for future on-chain verify if needed)
]
```

**Total accounts**: 6 existing + 6 new = **12 accounts**

### Withdraw Data Layout

**Standard (no PoW)**: 437 bytes after discriminant
```
[proof:260][public:104][nf-dup:32][num_outputs:1][recipient:32][amount:8]
```

**With PoW**: 469 bytes after discriminant (adds 32-byte batch_hash)
```
[proof:260][public:104][nf-dup:32][num_outputs:1][recipient:32][amount:8][batch_hash:32]
```

**Current Implementation Status**:
- Shield-pool withdraw **supports both layouts** (checks data.len())
- Relay transaction builder has **both variants**:
  - `build_withdraw_ix_body()` - 437 bytes (legacy)
  - `build_withdraw_ix_body_with_pow()` - 469 bytes (PoW-enabled)
- PoW path **fully implemented** in shield-pool (lines 121-172 of withdraw.rs)

### CPI Flow

**✅ FULLY IMPLEMENTED** in `programs/shield-pool/src/instructions/withdraw.rs` (lines 121-172):

1. Shield-pool `withdraw` validates all invariants (SP1 proof, root, nullifier, outputs_hash, fee conservation)
2. **After all checks pass**, extract PoW data:
   ```rust
   let batch_hash: &[u8; 32] = &*(data.as_ptr().add(437) as *const [u8; 32]);
   let miner_authority: &[u8; 32] = &*(miner_data.as_ptr().add(8) as *const [u8; 32]);
   ```
3. Build `consume_claim` CPI instruction:
   ```rust
   // Instruction data: discriminator(1) + miner_authority(32) + batch_hash(32) = 65 bytes
   let mut consume_ix_data = [0u8; 65];
   consume_ix_data[0] = 4; // consume_claim discriminator
   consume_ix_data[1..33].copy_from_slice(miner_authority);
   consume_ix_data[33..65].copy_from_slice(batch_hash);
   ```
4. CPI to `scramble_registry::consume_claim` with accounts:
   - `claim_pda` (writable)
   - `miner_pda` (writable)
   - `registry_pda` (writable)
   - `pool_pda` (signer for CPI auth)
   - `clock` (readonly)
5. Scramble program verifies claim, increments consumed count, marks as Consumed if fully used
6. Withdraw completes normally with lamport transfers

**Security**: Failed SP1 verification or invariant check prevents CPI call entirely. Scramblers only earn fees for valid, successful withdraws.

## Appendix A: Constants & Offsets Reference

### Instruction Data Layouts

#### Withdraw Instruction (discriminant = 2)
```
Legacy (437 bytes):
  0-259:   SP1 proof (260 bytes)
260-363:   Public inputs (104 bytes)
364-395:   Nullifier duplicate (32 bytes)
    396:   Number of outputs (1 byte, always 1 in MVP)
397-428:   Recipient address (32 bytes)
429-436:   Recipient amount (8 bytes, u64 LE)

With PoW (469 bytes):
  0-259:   SP1 proof (260 bytes)
260-363:   Public inputs (104 bytes)
364-395:   Nullifier duplicate (32 bytes)
    396:   Number of outputs (1 byte, always 1 in MVP)
397-428:   Recipient address (32 bytes)
429-436:   Recipient amount (8 bytes, u64 LE)
437-468:   Batch hash (32 bytes) ← NEW
```

#### Consume Claim CPI (discriminant = 4)
```
65 bytes total:
   0:      Discriminator (1 byte, value = 4)
 1-32:     Miner authority (32 bytes)
33-64:     Batch hash (32 bytes)
```

### PDA Seeds

```rust
// Shield Pool PDAs
pool_pda:            [b"pool"]
treasury_pda:        [b"treasury"]
roots_ring_pda:      [b"roots_ring"]
nullifier_shard_pda: [b"nullifier_shard"]

// Scramble Registry PDAs
registry_pda:        [b"scramble_registry"]
miner_pda:           [b"miner", authority:32]
claim_pda:           [b"claim", miner_authority:32, batch_hash:32, slot_le:8]
```

### Fee Constants

```rust
// From shield-pool/src/instructions/withdraw.rs
const FIXED_FEE: u64 = 2_500_000;           // 0.0025 SOL
const VARIABLE_FEE_BPS: u64 = 5;            // 0.5% = 5/1000
const VARIABLE_FEE_DIVISOR: u64 = 1_000;

// Formula
total_fee = FIXED_FEE + (public_amount * VARIABLE_FEE_BPS / VARIABLE_FEE_DIVISOR)
recipient_amount = public_amount - total_fee

// Example (1 SOL withdraw)
public_amount = 1_000_000_000 lamports
total_fee = 2_500_000 + (1_000_000_000 * 5 / 1_000) = 7_500_000 lamports (0.0075 SOL)
recipient_amount = 992_500_000 lamports (0.9925 SOL)
```

### Compute Unit Limits

```rust
// From transaction_builder.rs
const CU_LIMIT: u32 = 1_000_000;            // 1M CU per transaction

// Estimated CU consumption (from spec)
mine_claim:     ~30,000 CU  (BLAKE3 + SlotHashes + writes)
reveal_claim:   ~10,000 CU  (minimal validation + state update)
consume_claim:  ~15,000 CU  (arithmetic + counter updates)
withdraw:      ~800,000 CU  (SP1 verify + merkle + CPI + transfers)
```

### Domain Tags (BLAKE3)

```rust
// PoW preimage
const DOMAIN_POW: &[u8] = b"CLOAK:SCRAMBLE:v1";  // 17 bytes

// Batch descriptor (future)
const DOMAIN_BATCH: &[u8] = b"CLOAK:BATCH:v1";

// Job commitment (future, for k>1 batches)
const DOMAIN_JOB: &[u8] = b"CLOAK:JOB:v1";
```

### PoW Preimage Layout

```rust
// Fixed 137 bytes (129 without domain, includes domain in spec)
// Note: Spec says 129 bytes, but includes 17-byte domain = 137 total
DOMAIN             17 bytes   b"CLOAK:SCRAMBLE:v1"
slot                8 bytes   u64 LE
slot_hash          32 bytes   from SlotHashes sysvar
miner_pubkey       32 bytes   from Miner.authority
batch_hash         32 bytes   BLAKE3(job_ids)
nonce              16 bytes   u128 LE
-----
Total             137 bytes
```

### Account Sizes (Rent-Exempt Minimums)

```rust
// Scramble Registry State Accounts
ScrambleRegistry:  196 bytes  (~0.0014 SOL rent-exempt)
Miner:              72 bytes  (~0.0006 SOL rent-exempt)
Claim:             256 bytes  (~0.0019 SOL rent-exempt)

// Shield Pool State Accounts (for reference)
Pool:              ~100 bytes
RootsRing:         ~2KB (stores 1024 roots)
NullifierShard:    ~variable (grows with nullifiers)
```

### Time Windows (Default Values from Spec)

```rust
// SlotHashes retention
SLOT_HASHES_WINDOW: ~300 slots  (~2.5 minutes at 400ms/slot)

// Claim lifecycle windows (configurable in registry)
reveal_window:      150 slots   (~1 minute default)
claim_window:       300 slots   (~2 minutes default)

// Difficulty retargeting (planned)
target_interval:    600 slots   (~4 minutes default, 1 solution per interval)
retarget_frequency: 1000 slots  (~6.7 minutes) or 100 solutions
```

### Error Codes

#### Shield Pool Errors (0x1000-0x1070 range)
```rust
0x1064: InvalidMinerAccount
0x1065: InvalidClaimAccount  
0x1066: ConsumClaimFailed
```

#### Scramble Registry Errors (from error.rs)
```rust
0: InvalidProofHash
1: DifficultyNotMet
2: SlotHashMismatch
3: SlotHashNotFound
4: RevealWindowExpired
5: ClaimWindowExpired
17: UnauthorizedMiner
24: BatchHashMismatch
```

---

## Appendix B: Testing & Golden Vectors

### Test Vectors

#### Vector 1: Toy Difficulty (High Target)
```
Domain:      "CLOAK:SCRAMBLE:v1"
Slot:        12345678 (LE: 0x4e61bc00000000)
Slot Hash:   0x1234...abcd (32 bytes, from SlotHashes)
Miner:       0x5678...ef01 (32 bytes)
Batch Hash:  0x9abc...def0 (32 bytes)
Nonce:       0x00000000000000000000000000000042 (128-bit LE)

Preimage (129 bytes):
[DOMAIN || slot_le || slot_hash || miner || batch_hash || nonce_le]

BLAKE3(preimage) = 0xabcd...1234 (32 bytes)

Difficulty Target (toy): 0xffff...ffff (2^256 - 1, trivial)
Comparison: u256(0xabcd...1234) < u256(0xffff...ffff) → PASS
```

#### Vector 2: Endianness Check
```
proof_hash bytes (LE):  [0x42, 0x00, ..., 0x00] = 0x42 as u256
target bytes (LE):      [0x80, 0x00, ..., 0x00] = 0x80 as u256
Comparison: 0x42 < 0x80 → PASS
```

#### Vector 3: Boundary Slots (Reveal Window)
```
Mined at slot:     1000
Reveal window:     10 slots
Valid reveal:      slots 1000..=1010
Boundary (exact):  slot 1010 → PASS
Boundary (late):   slot 1011 → FAIL (expired)
```

### CU Budget Estimates
- `mine_claim`: ~30k CU (BLAKE3 + SlotHashes lookup + account writes)
- `reveal_claim`: ~10k CU (minimal validation + state update)
- `claim_pow`: ~40k CU (combined)
- `consume_claim`: ~15k CU (arithmetic + transfer)

### Edge Cases
1. **SlotHashes rotation**: Claim mined at slot 1000, but revealed at slot 1400 (>300 slots)
   - SlotHashes no longer contains slot 1000
   - Reveal fails with error (cannot verify slot_hash)

2. **Exact expiry boundary**: `current_slot == expires_at_slot`
   - Check uses `<=` or `<` (document clearly; suggest `<` for strict expiry)

3. **Multiple claims per miner**: Different `batch_hash` or `slot` → different Claim PDAs
   - Allowed; each claim is independent

4. **Zero-consume claims**: Claim revealed but never consumed before expiry
   - Forfeited; no refunds; rent can be reclaimed by miner

## Implementation Phases

### Phase 1: Minimal On-Chain Registry (Current Focus)
- [x] Design specification (this document)
- [ ] Pinocchio program: `programs/scramble-registry/`
- [ ] State structs: `ScrambleRegistry`, `Miner`, `Claim`
- [ ] Instructions: `initialize_registry`, `register_miner`, `mine_claim`, `reveal_claim`, `consume_claim`
- [ ] Tests: mining, reveal, consume, expiry, difficulty retarget
- [ ] SlotHashes integration

### Phase 2: Off-Chain Mining (Relay)
- [ ] Module: `services/relay/src/scrambler/`
- [ ] BLAKE3 mining loop with SlotHashes fetching
- [ ] Batch descriptor builder
- [ ] Auto-reveal on successful mine
- [ ] Difficulty monitoring and adaptive mining

### Phase 3: Shield-Pool CPI Integration
- [ ] Update `shield-pool-upstream` withdraw instruction
- [ ] Add `consume_claim` CPI after SP1 verification
- [ ] Fee share transfer from treasury to miner_authority
- [ ] Tests: end-to-end withdraw with scrambler fee share

### Phase 4: Enhancements
- [ ] Automated difficulty retargeting (cron or per-instruction)
- [ ] Metrics/monitoring (claims per epoch, difficulty trend)
- [ ] Multi-claim parallel mining strategies
- [ ] Merkle inclusion proofs for `jobs_root` enforcement (future)

## File Structure
```
programs/scramble-registry/
├── Cargo.toml
├── src/
│   ├── lib.rs                         # Entry point
│   ├── state/
│   │   ├── mod.rs
│   │   ├── registry.rs                # ScrambleRegistry
│   │   ├── miner.rs                   # Miner
│   │   └── claim.rs                   # Claim
│   ├── instructions/
│   │   ├── mod.rs
│   │   ├── initialize_registry.rs
│   │   ├── register_miner.rs
│   │   ├── mine_claim.rs
│   │   ├── reveal_claim.rs
│   │   ├── claim_pow.rs               # Optional single-step
│   │   ├── consume_claim.rs
│   │   └── adjust_difficulty.rs
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── blake3.rs                  # PoW verification
│   │   ├── difficulty.rs              # u256 comparison
│   │   └── slot_hashes.rs             # Sysvar helpers
│   ├── error.rs
│   ├── constants.rs
│   └── tests/
│       ├── mod.rs
│       ├── mining.rs
│       ├── reveal.rs
│       ├── consume.rs
│       └── difficulty.rs

services/relay/src/scrambler/
├── mod.rs
├── miner.rs                           # BLAKE3 mining loop
├── batch.rs                           # BatchDescriptor builder
├── client.rs                          # On-chain client (mine/reveal/consume)
└── difficulty.rs                      # Monitor & adapt

programs/shield-pool-upstream/src/
└── instructions/
    └── withdraw.rs                    # Add consume_claim CPI
```

## Next Actions: MVP Completion Punch-List

### Critical Path (Must Complete for PoW-Gated Withdraws)

#### 1. ✅ Shield-Pool Program ID Binding
**Status**: 🚧 Partially Complete
**Files**: 
- `programs/scramble-registry/src/state/registry.rs` - Add `shield_pool_program: Pubkey` field
- `programs/scramble-registry/src/instructions/initialize.rs` - Accept program ID as param
- `programs/scramble-registry/src/instructions/consume_claim.rs` - Verify CPI caller

**Changes Needed**:
```rust
// In ScrambleRegistry
pub shield_pool_program: Pubkey,  // NEW: store authorized caller

// In consume_claim (line 35-38)
// Replace generic "is_signer" check with:
if shield_pool_program.key() != registry.shield_pool_program() {
    return Err(ScrambleError::UnauthorizedCaller.into());
}
```

#### 2. 🚧 Fee Share Distribution
**Status**: TODO
**File**: `programs/shield-pool/src/instructions/withdraw.rs`

**Changes Needed** (after line 172, before line 174):
```rust
// After consume_claim CPI succeeds, split fee
unsafe {
    // Fetch registry to get fee_share_bps
    let registry_data = registry_pda_info.try_borrow_data()?;
    let fee_share_bps = u16::from_le_bytes(
        registry_data[80..82].try_into().unwrap() // Offset to fee_share_bps
    );
    
    let scrambler_share = (total_fee as u128 * fee_share_bps as u128 / 10_000) as u64;
    let protocol_share = total_fee - scrambler_share;
    
    // Extract miner authority from miner PDA (offset 8)
    let miner_data = miner_pda_info.try_borrow_data()?;
    let miner_authority_bytes: &[u8; 32] = &*(miner_data.as_ptr().add(8) as *const [u8; 32]);
    let miner_authority = Pubkey::from(miner_authority_bytes);
    
    // Distribute fees
    *treasury_info.borrow_mut_lamports_unchecked() += protocol_share;
    // Transfer scrambler_share to miner_authority (needs account in withdraw accounts)
}
```

**Account Changes**: Add `miner_authority_account` to withdraw instruction accounts (writable).

#### 3. 🚧 Relay Worker Integration
**Status**: Transaction builders ready, worker needs update
**File**: `services/relay/src/worker/processor.rs` (or equivalent)

**Changes Needed**:
```rust
use crate::solana::transaction_builder::{
    build_withdraw_transaction_with_pow,
    derive_scramble_registry_pdas,
};

// In withdraw handler
async fn process_withdraw_job(&mut self, job: WithdrawJob) -> Result<()> {
    // 1. Compute batch hash for this job (k=1 for MVP)
    let batch_hash = cloak_miner::batch::compute_single_job_hash(&job.id);
    
    // 2. Get or mine claim (TODO: implement ClaimManager)
    let (claim_pda, mined_slot) = self.claim_manager
        .get_or_mine_claim(&batch_hash)
        .await?;
    
    // 3. Derive PoW PDAs
    let (registry_pda, miner_pda, _) = derive_scramble_registry_pdas(
        &self.config.scramble_registry_program_id,
        &self.config.miner_keypair.pubkey(),
        &batch_hash,
        mined_slot,
    );
    
    // 4. Build transaction with PoW
    let tx = build_withdraw_transaction_with_pow(
        proof, public_inputs, recipient, amount, batch_hash,
        shield_pool_program, pool_pda, roots_ring, nullifier_shard, treasury, recipient,
        scramble_registry_program, claim_pda, miner_pda, registry_pda,
        fee_payer, blockhash, priority_fee,
    )?;
    
    // 5. Submit and track
    self.submit_tx(tx).await?;
    self.claim_manager.record_consume(&batch_hash);
    Ok(())
}
```

#### 4. 🚧 ClaimManager Implementation
**Status**: Skeleton exists in `packages/cloak-miner/`, needs relay integration
**New File**: `services/relay/src/claim_manager.rs`

**Core API**:
```rust
pub struct ClaimManager {
    rpc_client: RpcClient,
    miner_keypair: Keypair,
    registry_program_id: Pubkey,
    active_claims: HashMap<[u8; 32], ClaimState>,
}

impl ClaimManager {
    /// Get existing usable claim or mine+reveal new one
    pub async fn get_or_mine_claim(&mut self, batch_hash: &[u8; 32]) 
        -> Result<(Pubkey, u64)>;
    
    /// Record that a claim was consumed (decrement available count)
    pub fn record_consume(&mut self, batch_hash: &[u8; 32]);
    
    /// Background task: monitor and refresh claims before expiry
    pub async fn maintain_pool(&mut self);
}
```

**Integration**: 
- Add to relay `Config`: `miner_keypair_path`, `scramble_registry_program_id`
- Initialize in `main.rs`, share via `Arc<Mutex<ClaimManager>>`

#### 5. 📋 SlotHashes Verification Enhancement
**Status**: Basic implementation exists, needs robustness
**File**: `programs/scramble-registry/src/instructions/mine_claim.rs`

**Current**: Checks if slot_hash matches SlotHashes entry.
**Enhance**: Add clear error messages for:
- Slot too old (>300 slots)
- Slot not yet in SlotHashes
- SlotHashes parse failure

#### 6. 📋 Testing & Validation
**Files**: Create integration tests

**Tests Needed**:
- [ ] Full flow: mine → reveal → withdraw → consume
- [ ] Claim expiry handling
- [ ] Fee distribution verification
- [ ] Batch hash mismatch rejection
- [ ] Unauthorized CPI rejection

**Test Environment**: Use Mollusk or solana-program-test with both programs deployed.

---

### Optional Enhancements (Post-MVP)

#### A. Multi-Job Batches (k>1)
- Implement Merkle tree over job commits
- Add on-chain inclusion proof verification
- Batch coordination in relay

#### B. Difficulty Retargeting
- Implement EMA-based adjustment algorithm
- Add `retarget_difficulty` instruction
- Automate retargeting (cron or permissionless trigger)

#### C. Monitoring & Observability
- Prometheus metrics: claim pool size, mine success rate, fee distribution
- Alerts: claim expiry, mining failures
- Dashboard: difficulty trend, active claims

#### D. Multi-Threaded Mining
- Parallelize nonce search across CPU cores
- GPU acceleration for high difficulty

---

## Quick Start for Next Developer

### 1. Test Current State
```bash
# Build all programs
anchor build

# Run scramble-registry tests
cargo test -p scramble-registry

# Run cloak-miner tests
cargo test -p cloak-miner
```

### 2. Deploy to Localnet
```bash
# Start validator
solana-test-validator

# Deploy programs
anchor deploy

# Initialize registry
programs/scramble-registry/init-localnet.sh
```

### 3. Complete Fee Distribution
See **Critical Path #2** above - add fee split logic after line 172 in `withdraw.rs`.

### 4. Wire Up Relay
See **Critical Path #3** above - update worker to use `build_withdraw_transaction_with_pow`.

### 5. Test End-to-End
```bash
# Mine a claim
cloak-miner mine --timeout 30

# Submit withdraw via relay API
curl -X POST http://localhost:3000/api/withdraw -d '{...}'

# Verify claim consumed
solana account <claim_pda>
```
