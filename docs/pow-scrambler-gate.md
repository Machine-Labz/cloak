# PoW-Gated Scrambler System (Ore-Inspired)

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
```rust
// Per-job commitment (deterministic)
let job_commit = BLAKE3([
    b"CLOAK:JOB:v1",
    public_104,                       // 104-byte public inputs
    recipient_addr_32,                // 32-byte recipient pubkey
    recipient_amount.to_le_bytes(),   // u64 LE
].concat());

// Jobs root (Merkle tree over sorted job commits)
jobs_root = merkle_root(sorted(job_commits));
```

**Note**: In MVP, Merkle inclusion proofs are not enforced on-chain but can be added in future. Off-chain scramblers commit to jobs via `jobs_root`.

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

2. **Fee Share Transfer**:
   ```rust
   let scrambler_share = withdraw_fee
       .checked_mul(registry.fee_share_bps as u64)
       .unwrap()
       .checked_div(10_000)
       .unwrap();

   // Transfer from pool/treasury to miner_authority
   invoke_signed(
       &system_instruction::transfer(treasury, miner_authority, scrambler_share),
       &[treasury_info, miner_authority_info, system_program_info],
       &[treasury_seeds],
   )?;
   ```

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

### Withdraw Data Layout (Unchanged)
- **437 bytes** after discriminant (same as before)
- No changes to proof/public/recipient/amount encoding
- PoW integration is purely account-level (CPIs, not data)

### CPI Flow
1. Shield-pool `withdraw` validates all invariants
2. After SP1 verify + invariant checks pass, CPI to `scramble_registry::consume_claim`
3. Scramble program transfers fee share to miner
4. Withdraw completes normally

**No breaking changes to existing withdraw logic**; PoW is an additive enhancement.

## Testing Appendix

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

## Next Actions
1. ✅ Design specification complete
2. Create `programs/scramble-registry/` skeleton
3. Implement state structs with exact sizes
4. Implement `mine_claim` with SlotHashes verification
5. Add BLAKE3 + u256 difficulty comparison utils
6. Write unit tests with test vectors
7. Build off-chain miner in Relay
