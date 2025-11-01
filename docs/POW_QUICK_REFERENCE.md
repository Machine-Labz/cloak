# PoW Integration Quick Reference Card

**For**: Developers integrating PoW-gated withdraws  
**Last Updated**: 2025-10-19

---

## 🚀 30-Second Overview

The PoW scrambler gate prevents spam by requiring miners to solve BLAKE3 proof-of-work before batching withdrawals. Miners earn fee shares for successful batches.

**Current Status**: 
- ✅ On-chain programs complete
- ✅ Shield-pool CPI integrated  
- 🚧 Relay worker needs wiring
- 🚧 Fee distribution needs implementation

---

## 📐 Data Structures

### Withdraw Instruction (discriminant = 2)

```
┌─────────────────────────────────────────┐
│ Legacy (437 bytes after discriminant)  │
├─────────────────────────────────────────┤
│   0-259: SP1 proof (260 bytes)         │
│ 260-363: Public inputs (104 bytes)     │
│ 364-395: Nullifier dup (32 bytes)      │
│     396: Num outputs (1 byte)           │
│ 397-428: Recipient addr (32 bytes)     │
│ 429-436: Recipient amt (8 bytes)       │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│ With PoW (469 bytes after discriminant) │
├─────────────────────────────────────────┤
│   0-259: SP1 proof (260 bytes)         │
│ 260-363: Public inputs (104 bytes)     │
│ 364-395: Nullifier dup (32 bytes)      │
│     396: Num outputs (1 byte)           │
│ 397-428: Recipient addr (32 bytes)     │
│ 429-436: Recipient amt (8 bytes)       │
│ 437-468: Batch hash (32 bytes) ← NEW   │
└─────────────────────────────────────────┘
```

### Consume Claim CPI (discriminant = 4)

```
┌──────────────────────────────────┐
│ 65 bytes total                   │
├──────────────────────────────────┤
│    0: Discriminator (1 byte) = 4 │
│ 1-32: Miner authority (32 bytes) │
│33-64: Batch hash (32 bytes)      │
└──────────────────────────────────┘
```

---

## 🔑 PDA Seeds

```rust
// Shield Pool
pool_pda            = [b"pool"]
treasury_pda        = [b"treasury"]
roots_ring_pda      = [b"roots_ring"]
nullifier_shard_pda = [b"nullifier_shard"]

// Scramble Registry
registry_pda = [b"scramble_registry"]
miner_pda    = [b"miner", authority:32]
claim_pda    = [b"claim", miner_authority:32, batch_hash:32, slot_le:8]
```

---

## 💰 Fee Calculation

```rust
// Constants
FIXED_FEE = 2_500_000 lamports      // 0.0025 SOL
VARIABLE_FEE = amount * 5 / 1_000   // 0.5%

// Formula
total_fee = FIXED_FEE + VARIABLE_FEE
recipient_amount = public_amount - total_fee

// Example: 1 SOL withdraw
public_amount = 1_000_000_000 lamports
total_fee = 2_500_000 + (1_000_000_000 * 5 / 1_000) = 7_500_000 lamports
recipient_amount = 992_500_000 lamports

// Scrambler share (if fee_share_bps = 2000 = 20%)
scrambler_share = total_fee * 2000 / 10_000 = 1_500_000 lamports
protocol_share = 6_000_000 lamports
```

---

## 🏗️ Building PoW Transactions

### Using Relay Transaction Builders

```rust
use crate::solana::transaction_builder::{
    build_withdraw_transaction_with_pow,
    derive_scramble_registry_pdas,
};

// 1. Compute batch hash (k=1 for MVP)
let batch_hash = cloak_miner::batch::compute_single_job_hash(&job_id);

// 2. Get claim from miner (TODO: implement ClaimManager)
let (claim_pda, mined_slot) = claim_manager.get_or_mine_claim(&batch_hash).await?;

// 3. Derive PDAs
let (registry_pda, miner_pda, _) = derive_scramble_registry_pdas(
    &scramble_registry_program_id,
    &miner_keypair.pubkey(),
    &batch_hash,
    mined_slot,
);

// 4. Build transaction
let tx = build_withdraw_transaction_with_pow(
    groth16_260,           // SP1 proof
    public_104,            // Public inputs
    recipient_addr_32,     // Recipient address
    recipient_amount,      // Amount after fee
    batch_hash,            // ← Batch commitment
    shield_pool_program,   // Shield pool program ID
    pool_pda,              // Pool state
    roots_ring_pda,        // Roots ring
    nullifier_shard_pda,   // Nullifier shard
    treasury,              // Treasury
    recipient,             // Recipient
    scramble_registry_program,  // ← Scramble registry program ID
    claim_pda,             // ← Claim PDA
    miner_pda,             // ← Miner PDA
    registry_pda,          // ← Registry PDA
    fee_payer,             // Fee payer
    blockhash,             // Recent blockhash
    priority_fee,          // Priority fee (micro-lamports per CU)
)?;

// 5. Sign and submit
tx.sign(&[fee_payer_keypair, ...]);
client.send_and_confirm_transaction(&tx).await?;

// 6. Record consumption
claim_manager.record_consume(&batch_hash);
```

---

## 🔄 Claim Lifecycle

```
┌─────────┐ mine_claim()    ┌─────────┐ reveal_claim()  ┌──────────┐
│  Mined  │ ───────────────>│Revealed │ ──────────────> │  Active  │
└─────────┘                 └─────────┘                 └──────────┘
     │                           │                            │
     │ reveal_window             │ claim_window               │ consume_claim()
     │ expires                   │ expires                    ▼
     ▼                           ▼                       ┌──────────┐
┌─────────┐                 ┌─────────┐                 │ Consumed │
│ Expired │                 │ Expired │                 └──────────┘
└─────────┘                 └─────────┘
```

**Windows** (default):
- `reveal_window`: 150 slots (~1 minute)
- `claim_window`: 300 slots (~2 minutes)

---

## 🛠️ Common Tasks

### Check if Claim is Usable

```rust
// Fetch claim account
let claim_data = client.get_account(&claim_pda).await?;
let claim = Claim::deserialize(&claim_data.data)?;

// Check status and expiry
let current_slot = client.get_slot().await?;
let is_usable = 
    claim.status == ClaimStatus::Revealed &&
    current_slot <= claim.expires_at_slot &&
    claim.consumed_count < claim.max_consumes;
```

### Derive Claim PDA

```rust
let (claim_pda, bump) = Pubkey::find_program_address(
    &[
        b"claim",
        miner_authority.as_ref(),
        &batch_hash,
        &mined_slot.to_le_bytes(),
    ],
    &scramble_registry_program_id,
);
```

### Compute Batch Hash

```rust
// Single job (k=1)
let batch_hash = cloak_miner::batch::compute_single_job_hash("job-123");

// Multiple jobs (future)
let job_ids = vec!["job-001".to_string(), "job-002".to_string()];
let batch_hash = cloak_miner::batch::compute_batch_hash(&job_ids);
```

---

## ⚠️ Common Pitfalls

### ❌ DON'T: Use batch_hash from a different job
```rust
// WRONG: Reusing claim for different job
let claim_pda = get_existing_claim(); // From job-001
submit_withdraw_for_job_002(claim_pda); // ❌ Will fail with BatchHashMismatch
```

### ✅ DO: Mine claim per job (or batch)
```rust
// CORRECT: Mine claim for each unique batch
let batch_hash = compute_single_job_hash(&job.id);
let claim_pda = claim_manager.get_or_mine_claim(&batch_hash).await?;
```

### ❌ DON'T: Forget to check expiry
```rust
// WRONG: Using expired claim
let claim_pda = claims_cache.get(&batch_hash); // Might be expired
submit_withdraw(claim_pda); // ❌ Will fail with ClaimExpired
```

### ✅ DO: Validate claim before use
```rust
// CORRECT: Check expiry and consumed count
if claim.is_usable(current_slot) {
    submit_withdraw(claim_pda);
} else {
    // Mine new claim
}
```

### ❌ DON'T: Hardcode offsets
```rust
// WRONG: Brittle offset calculations
let batch_hash = &data[437..469];
```

### ✅ DO: Use constants or builder functions
```rust
// CORRECT: Use transaction builder
let body = build_withdraw_ix_body_with_pow(...);
```

---

## 🐛 Debugging

### Error: BatchHashMismatch (0x18)
**Cause**: Claim was mined for different batch  
**Fix**: Compute correct batch_hash for this job

### Error: ClaimExpired (0x17)
**Cause**: Claim past `expires_at_slot`  
**Fix**: Mine new claim, or extend claim_window in registry

### Error: InvalidMinerAccount (0x1064)
**Cause**: Miner PDA account data malformed  
**Fix**: Verify PDA derivation, check miner is registered

### Error: UnauthorizedMiner (0x11)
**Cause**: Miner authority mismatch in consume_claim  
**Fix**: Pass correct miner_authority (from Miner.authority field)

---

## 📊 Monitoring

### Key Metrics to Track

```rust
// Claim pool health
active_claims_count: usize         // Number of usable claims
claims_expiring_soon: usize        // Claims expiring in next 50 slots
avg_claim_lifetime: Duration       // Time from reveal to consumed

// Mining performance
mine_success_rate: f64             // Successful mines / attempts
avg_mine_time: Duration            // Time to find valid nonce
difficulty_trend: Vec<f64>         // Historical difficulty values

// Fee distribution
total_fees_collected: u64          // Cumulative fees
scrambler_share_paid: u64          // Cumulative scrambler rewards
protocol_share_retained: u64       // Cumulative protocol revenue
```

---

## 📚 Further Reading

- **Wildcard Mining Overview**: [pow/overview.md](pow/overview.md)
- **Integration Guide**: [POW_INTEGRATION_GUIDE.md](POW_INTEGRATION_GUIDE.md)
- **Operations Guide**: [operations/metrics-guide.md](operations/metrics-guide.md)

---

## 🆘 Getting Help

1. Check error code in `programs/scramble-registry/src/error.rs`
2. Review CPI flow in `programs/shield-pool/src/instructions/withdraw.rs` (lines 121-172)
3. Test with localnet: `programs/scramble-registry/init-localnet.sh`
4. Consult golden tests: `packages/zk-guest-sp1/tests/golden.rs`

---

**Pro Tip**: Start with k=1 (one claim per job). Optimize to k>1 batches only after single-job flow works end-to-end.

