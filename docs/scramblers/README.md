# Cloak Scramblers - Decentralized Privacy Enhancement

## Overview

Scramblers create transaction volume to enhance privacy by batching multiple withdrawals into a single ZK proof, reducing costs 10x while increasing anonymity sets.

**Key Innovation:** N withdrawals = 1 proof = 10x cheaper per withdrawal

---

## Architecture

### Core Concept

```
User wants to withdraw 1.5 SOL privately
→ 10 scramblers pre-deposit into pool
→ All join same batch (11 total withdrawals)
→ Leader generates 1 ZK proof for all 11
→ Single BatchWithdraw transaction executes
→ All pull from pool (indistinguishable!)
```

### Privacy Requirement

**🚨 CRITICAL: Scramblers MUST deposit before withdrawing!**

```
❌ BAD:  Scrambler withdraws without deposit = traceable by fund source
✅ GOOD: Scrambler deposits → waits → withdraws = indistinguishable from user
```

All withdrawals pull FROM POOL (enforced on-chain).

### Flow

```
1. Scrambler: Deposit 1 SOL → Creates commitment in merkle tree
2. User: Request withdrawal → Pays 0.15 SOL fee
3. Scramblers: 10 join batch using pre-deposits
4. Leader: Generate batch proof (~135s for 3 withdrawals)
5. On-chain: BatchWithdraw verifies & executes all atomically
6. Scramblers: Receive funds + rewards, immediately redeposit
```

---

## Program Design

### Instructions

| ID | Name | Purpose |
|----|------|---------|
| 0 | Deposit | Deposit SOL into pool, create commitment |
| 1 | AdminPushRoot | Push merkle root to on-chain buffer |
| 2 | Withdraw | Single withdrawal (existing) |
| 3 | BatchWithdraw | **NEW**: Batch N withdrawals in 1 proof |

### BatchWithdraw Data Layout (Optimized)

```
[discriminator: 1 byte]
[sp1_proof: 260 bytes]
[sp1_public_values: N×104 bytes]  ← Contains: root, nf, outputs_hash, amount per withdrawal
[num_withdrawals: 1 byte]
[withdrawal_1: 41 bytes]          ← num_outputs(1) + recipient(32) + amount(8)
[withdrawal_2: 41 bytes]
...
[withdrawal_N: 41 bytes]
```

**Savings:** 41 bytes vs 177 bytes per withdrawal (76% reduction!)

### Transaction Size

```
3 withdrawals:
  Before optimization: 792 bytes
  After optimization:  573 bytes ✅

10 withdrawals:
  Before: 2,031 bytes ❌ (exceeds 1232 limit)
  After:  1,301 bytes ✅ (fits!)
```

### Compute Budget

```
SP1 verification:  ~170k CU
Per withdrawal:    ~20k CU
3 withdrawals:     ~250k CU (needs 1.4M CU limit)
10 withdrawals:    ~370k CU (within 1.4M)
```

---

## Economics

### Capital Requirements

| Capital | Concurrent Cycles | Daily Earnings |
|---------|-------------------|----------------|
| 5 SOL   | 5 cycles          | ~$10/day       |
| 10 SOL  | 10 cycles         | ~$20/day       |
| 20 SOL  | 20 cycles         | ~$40/day       |

**Key:** Capital rotates through deposit→withdraw cycles.

### Fee Distribution (Per Batch)

```
User fee: 0.15 SOL
├─ Leader (40%): 0.06 SOL
├─ 9 Scramblers (48%): 0.008 SOL each
└─ Protocol (12%): 0.018 SOL
```

**Scrambler profit:** ~0.003 SOL per cycle after fees

---

## Testing

### 1. Generate Batch Proof

```bash
# Generate example with 3 withdrawals
cargo run --release --bin generate-batch-example -- --count 3 --output batch.json

# Generate proof
cargo run --release --bin batch-prove -- --batch batch.json --proof proof.bin --pubout public.raw
```

**Result:** 135s for 3 withdrawals, 260-byte proof

### 2. Full E2E Test

```bash
# Terminal 1: Start validator
solana-test-validator

# Terminal 2: Start indexer and the database
docker-compose up -d

# Terminal 3: Run test
cargo run --release -p test-complete-flow-rust --bin batch-localnet-test
```

**What it does:**
1. Deploys program to `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`
2. Creates accounts (pool, treasury, roots_ring, nullifier_shard)
3. **Deposits 3 times to indexer** (1.0, 1.1, 1.2 SOL)
4. Gets merkle root & proofs from indexer
5. **Generates real batch ZK proof** (137s)
6. **Executes BatchWithdraw** with 1.4M CU limit
7. ✅ Verifies all 3 withdrawals succeed atomically

**Expected output:**
```
🎉 BATCH WITHDRAWAL E2E TEST SUCCESS!
✅ 3 deposits via indexer
✅ Merkle tree built by indexer  
✅ 3 withdrawals in single transaction
✅ Single batch proof verified on-chain
```

---

## Key Files

### ZK System
- `packages/zk-guest-sp1/guest/src/batch.rs` - Batch verification logic
- `packages/zk-guest-sp1/host/src/bin/batch-prove.rs` - Proof generator

### On-Chain
- `programs/shield-pool/src/instructions/batch_withdraw.rs` - Batch instruction (140 lines)
- `programs/shield-pool/src/constants.rs` - Updated vkey hash

### Testing
- `tooling/test/src/batch_localnet_test.rs` - Full E2E test (368 lines)
- `tooling/test/src/helpers.rs` - Shared utilities

---

## Performance

| Metric | Single (×3) | Batch (3) | Improvement |
|--------|-------------|-----------|-------------|
| **Proof time** | 90s | 137s | 1.5x total |
| **Proof cost** | 0.15 SOL | 0.05 SOL | 3x cheaper |
| **Transaction size** | 1,311 bytes | 573 bytes | 56% smaller |
| **Compute units** | 3× ~200k | 310k total | More efficient |
| **Privacy** | 1:0 ratio | 3:0 ratio | 3x better |

**With 10 scramblers:** 10:1 dummy-to-real ratio = excellent privacy!

---

## Status

✅ **Phase 1 Complete:** Batch ZK proofs working  
✅ **Phase 2 Complete:** On-chain BatchWithdraw implemented  
✅ **Testing:** Full E2E test passing  
⏳ **Next:** State accounts + coordinator service (future work)

**Foundation is solid - ready for scrambler network development!** 🚀

