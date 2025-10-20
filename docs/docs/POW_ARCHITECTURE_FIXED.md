# PoW Architecture - FIXED ✅

**Date**: 2025-10-19  
**Issue**: Relay was mining (wrong!)  
**Fix**: Relay now queries on-chain for claims from independent miners (correct!)

---

## 🎯 TL;DR

### Before ❌
```
Relay = Prover + Miner (conflict of interest)
```

### After ✅
```
Relay = Prover (queries on-chain for claims)
Miners = Independent (cloak-miner CLI, earn fees)
```

---

## 📝 What Changed

### Code Changes

1. **`services/relay/src/config.rs`**
   - ✅ Removed `miner_keypair_path` from `SolanaConfig`
   - ✅ Removed entire `MinerConfig` struct
   - ✅ Relay only needs `scramble_registry_program_id` to query

2. **`services/relay/src/claim_manager.rs`**
   - ✅ Renamed `ClaimManager` → `ClaimFinder`
   - ✅ Changed from mining to querying
   - ✅ `get_or_mine_claim()` → `find_claim()`
   - ✅ Queries `get_program_accounts` for available claims
   - ✅ Filters for: matching batch_hash, revealed status, not expired, not consumed

3. **New Documentation**
   - ✅ `docs/POW_CORRECT_ARCHITECTURE.md` - Ore-inspired architecture
   - ✅ `docs/POW_INTEGRATION_GUIDE.md` - How to integrate
   - ✅ `docs/POW_REFACTOR_SUMMARY.md` - What changed and why

---

## 🚀 How It Works Now

### Step 1: Miners Mine Independently

```bash
# Anyone can run this 24/7
cloak-miner mine \
  --keypair ~/my-miner.json \
  --registry scramb1e... \
  --continuous
```

This creates revealed claims on-chain.

### Step 2: User Requests Withdraw

```bash
curl -X POST http://relay.cloak.network/withdraw -d '{...}'
```

### Step 3: Relay Finds Claim

```rust
let batch_hash = compute_batch_hash(&job_id);
let claim = claim_finder.find_claim(&batch_hash).await?;
```

Relay queries on-chain, finds available claim.

### Step 4: Relay Uses Claim

```rust
let tx = build_withdraw_transaction_with_pow(
    proof, inputs, recipient, amount, batch_hash,
    // ... standard accounts ...
    claim.claim_pda,      // ← From on-chain query
    claim.miner_pda,      // ← From on-chain query
    claim.miner_authority, // ← Miner earns fee!
    // ...
)?;
```

### Step 5: Transaction Executes

- Shield-pool validates proof ✅
- CPI to `consume_claim` ✅
- Fee split: 80% protocol, 20% miner ✅
- Funds sent to recipient ✅

**Winner**: Miner earns fee! 💰

---

## ✅ Verification

### Compiles
```bash
cd services/relay
cargo check --lib
# ✅ Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.63s
```

### Architecture Correct
- ✅ Relay queries (doesn't mine)
- ✅ Miners independent (cloak-miner CLI)
- ✅ ClaimFinder discovers (doesn't manage)
- ✅ Miner earns fees (not relay)
- ✅ Matches Ore model

---

## 📊 Status

### Completed ✅
- [x] Remove miner config from relay
- [x] Create ClaimFinder (query-based)
- [x] Implement `find_claim()` method
- [x] Parse on-chain claim accounts
- [x] Add `compute_batch_hash()` helper
- [x] Write architecture documentation
- [x] Write integration guide
- [x] Verify compilation

### Next Steps 🚧
- [ ] Wire ClaimFinder into `main.rs`
- [ ] Update withdraw handler
- [ ] Add "no claims available" error
- [ ] Test with live miners
- [ ] Deploy to devnet

**Estimated Time**: 1-2 hours of integration work

---

## 📚 Documentation Index

1. **`docs/POW_CORRECT_ARCHITECTURE.md`** ⭐
   - Read this first! Explains correct architecture
   - Diagrams, economic model, deployment scenarios

2. **`docs/POW_INTEGRATION_GUIDE.md`** ⭐
   - Step-by-step integration into relay
   - Code snippets for each step
   - Error handling, testing, monitoring

3. **`docs/POW_REFACTOR_SUMMARY.md`**
   - What changed and why
   - Before/after comparison
   - Impact analysis

4. **`docs/pow-scrambler-gate.md`**
   - Original technical spec (still valid!)
   - Instruction layouts, constants, test vectors

5. **`packages/cloak-miner/README.md`**
   - How to run miners
   - Mining commands, examples

---

## 🎉 Summary

**Problem**: Relay was mining (centralized, conflict of interest)

**Solution**: Relay queries on-chain for claims from independent miners

**Result**: 
- ✅ Decentralized (anyone can mine)
- ✅ Scalable (miners scale independently)
- ✅ Correct (matches Ore model)
- ✅ Ready for integration

**Next**: Follow `docs/POW_INTEGRATION_GUIDE.md` to complete wiring.

---

**Architecture Status**: ✅ FIXED - Ready for Integration! 🎊

