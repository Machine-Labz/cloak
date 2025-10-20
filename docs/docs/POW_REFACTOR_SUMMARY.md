# PoW Architecture Refactor - Summary

**Date**: 2025-10-19  
**Status**: ✅ Complete - Corrected to Ore-Inspired Architecture  
**Issue**: Relay was incorrectly positioned as a miner  
**Solution**: Separated relay (client) from miners (independent)

---

## 🔥 What Was Wrong

### Original Implementation

```rust
// ❌ WRONG: Relay had miner keypair
pub struct SolanaConfig {
    pub scramble_registry_program_id: Option<String>,
    pub miner_keypair_path: Option<String>,  // ← BAD!
}

// ❌ WRONG: ClaimManager tracked mining state
pub struct ClaimManager {
    miner_keypair: Keypair,
    active_claims: HashMap<[u8; 32], ClaimState>,
}

impl ClaimManager {
    pub async fn get_or_mine_claim(&mut self, batch_hash: &[u8; 32]) 
        -> Result<(Pubkey, u64)> {
        // Would mine if no claim found ← WRONG!
    }
}
```

**Problems**:
1. Relay was a miner AND a prover (conflict of interest)
2. Miner keypair managed by relay (centralization)
3. ClaimManager implied relay owns claims (wrong ownership)
4. Doesn't match Ore model (CLI miners independent)

---

## ✅ What's Correct Now

### Corrected Architecture

```rust
// ✅ CORRECT: Relay only knows where to query
pub struct SolanaConfig {
    pub scramble_registry_program_id: Option<String>,
    // NO miner_keypair_path! Relay doesn't mine!
}

// ✅ CORRECT: ClaimFinder discovers claims
pub struct ClaimFinder {
    rpc_client: RpcClient,
    registry_program_id: Pubkey,
}

impl ClaimFinder {
    pub async fn find_claim(&self, batch_hash: &[u8; 32]) 
        -> Result<Option<AvailableClaim>, Error> {
        // Queries on-chain for available claims ← CORRECT!
    }
}

// ✅ CORRECT: Claims owned by miners
pub struct AvailableClaim {
    pub claim_pda: Pubkey,
    pub miner_authority: Pubkey,  // ← Miner earns fees!
    // ...
}
```

**Benefits**:
1. Clear separation: relay=prover, miner=claimer
2. Decentralized: anyone can run miners
3. Matches Ore: independent CLI miners
4. Scalable: multiple miners compete

---

## 📁 Files Changed

### Modified Files

#### `services/relay/src/config.rs`
- ✅ Removed `miner_keypair_path` from `SolanaConfig`
- ✅ Removed entire `MinerConfig` struct
- ✅ Added comment explaining separation

#### `services/relay/src/claim_manager.rs`
- ✅ Renamed `ClaimManager` → `ClaimFinder`
- ✅ Replaced `get_or_mine_claim()` → `find_claim()`
- ✅ Removed mining logic, added RPC query logic
- ✅ Added `parse_claim_account()` to deserialize on-chain data
- ✅ Changed from state tracking to discovery pattern
- ✅ Updated tests to match new architecture

### New Documentation Files

#### `docs/POW_CORRECT_ARCHITECTURE.md` ✨
- Explains the corrected Ore-inspired architecture
- Shows clear diagrams of separation
- Compares wrong vs correct patterns
- Economic model for miners
- Deployment scenarios

#### `docs/POW_INTEGRATION_GUIDE.md` ✨
- Step-by-step integration into relay
- Code snippets for each step
- Error handling patterns
- Testing procedures
- Monitoring and metrics

#### `docs/POW_REFACTOR_SUMMARY.md` ✨ (this file)
- Summary of what changed and why
- Before/after comparison
- Migration guide

---

## 🔄 Architecture Comparison

### Before (Wrong)

```
┌─────────────────────────┐
│       RELAY             │
│  • Generates ZK proofs  │
│  • Mines PoW claims ❌  │
│  • Submits transactions │
│  • Owns miner keypair ❌│
└─────────────────────────┘
```

### After (Correct)

```
┌─────────────────────────┐
│   INDEPENDENT MINERS    │
│  • Run cloak-miner CLI  │
│  • Mine claims 24/7     │
│  • Earn fees            │
└──────────┬──────────────┘
           │ Claims on-chain
           ↓
┌─────────────────────────┐
│     ON-CHAIN REGISTRY   │
│  • Stores claims        │
│  • Tracks status        │
└──────────┬──────────────┘
           │ Query claims
           ↓
┌─────────────────────────┐
│       RELAY             │
│  • Generates ZK proofs  │
│  • Queries for claims ✅│
│  • Submits transactions │
│  • NO mining! ✅        │
└─────────────────────────┘
```

---

## 🚀 How to Use

### For Relay Operators

**No Action Needed (Mining-Wise)!**

```toml
# config.toml
[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
program_id = "c1oak..."

# Enable PoW (relay will query for claims)
scramble_registry_program_id = "scramb1e..."

# That's it! No miner keypair needed.
```

The relay will automatically:
1. Query on-chain for available claims
2. Use them in withdraw transactions
3. Return "no claims available" if none found

### For Miners (New Opportunity!)

Anyone can run a miner and earn fees:

```bash
# 1. Generate keypair
solana-keygen new -o ~/miner.json

# 2. Register miner
cloak-miner register \
  --keypair ~/miner.json \
  --registry scramb1e...

# 3. Start mining
cloak-miner mine \
  --keypair ~/miner.json \
  --registry scramb1e... \
  --continuous \
  --threads 8

# 4. Earn fees when claims are used! 💰
```

---

## 🎯 Integration Checklist

### Relay Service Integration

- [x] Remove `miner_keypair_path` from config
- [x] Create `ClaimFinder` struct
- [x] Implement `find_claim()` method
- [x] Add `compute_batch_hash()` helper
- [ ] Wire `ClaimFinder` into main.rs
- [ ] Update withdraw handler to use `find_claim()`
- [ ] Add error handling for "no claims available"
- [ ] Add metrics for claim discovery
- [ ] Test with live miners

### Testing

- [x] Unit tests for `compute_batch_hash()`
- [x] Unit tests for `parse_claim_account()`
- [ ] Integration test: relay + miner + withdraw
- [ ] Load test: multiple miners competing
- [ ] Chaos test: miner drops mid-withdraw

### Documentation

- [x] POW_CORRECT_ARCHITECTURE.md
- [x] POW_INTEGRATION_GUIDE.md
- [x] POW_REFACTOR_SUMMARY.md (this file)
- [x] Updated inline code comments
- [ ] Update main README.md
- [ ] Create operator runbook

---

## 📊 Impact Analysis

### For Users

✅ **No change** - withdraw API remains the same

Possible error:
```json
{
  "error": "NO_CLAIMS_AVAILABLE",
  "message": "No PoW claims available. Please try again in 30 seconds.",
  "retry_after_seconds": 30
}
```

### For Relay Operators

✅ **Simplified** - no miner management
✅ **Reduced risk** - separation of concerns
✅ **Better UX** - "no claims" vs long mining wait

### For Miners (New Participants!)

✅ **Opportunity** - earn fees by mining
✅ **Permissionless** - anyone can register
✅ **Competitive** - first to mine gets fees

### For Protocol

✅ **Decentralization** - multiple miners
✅ **Scalability** - miners scale independently
✅ **Security** - no relay mining manipulation

---

## 🐛 Known Issues & Limitations

### Issue 1: `get_program_accounts` is Expensive

**Problem**: Querying all accounts for every withdraw is slow and costly.

**Solution**: Add memcmp filters to only query claims with matching batch_hash.

```rust
let filters = vec![
    RpcFilterType::Memcmp(Memcmp {
        offset: 40, // batch_hash offset
        bytes: MemcmpEncodedBytes::Bytes(batch_hash.to_vec()),
        encoding: None,
    }),
    RpcFilterType::Memcmp(Memcmp {
        offset: 188, // status offset
        bytes: MemcmpEncodedBytes::Bytes(vec![1]), // Revealed
        encoding: None,
    }),
];
```

**Status**: Planned enhancement, works fine for MVP

### Issue 2: No Claim Available

**Problem**: If no miners are running, withdraws fail.

**Solution**: Relay operators should:
1. Run a dedicated miner for reliability
2. Set up monitoring alerts
3. Show user-friendly error with retry

**Status**: Expected behavior, documented

### Issue 3: Claim Expiry Race

**Problem**: Claim might expire between query and submission.

**Solution**: Shield-pool CPI will fail gracefully with `ClaimExpired` error. Relay retries with new claim.

**Status**: Handled by error handling

---

## 🔮 Future Enhancements

### 1. Claim Marketplace API

Instead of querying on-chain, miners advertise via API:

```rust
GET /api/claims/available?batch_hash=<hash>

Response: {
  "claims": [
    { "pda": "...", "miner": "...", "expires_at": 123 }
  ]
}
```

**Benefits**: Faster discovery, less RPC load

### 2. Claim Pooling

Miners pre-mine claims for popular patterns:

```rust
// Miner predicts common patterns
let popular_patterns = ["daily_withdraw", "large_amount", "weekend"];
for pattern in popular_patterns {
    mine_claim(pattern_to_batch_hash(pattern)).await?;
}
```

**Benefits**: Lower latency for users

### 3. Dynamic Pricing

Miners set prices, relay chooses cheapest:

```rust
pub struct AvailableClaim {
    pub pda: Pubkey,
    pub miner: Pubkey,
    pub price_bps: u16,  // ← Miner's fee share ask
}
```

**Benefits**: Market-driven fees

---

## 📚 Related Documents

1. **`docs/pow-scrambler-gate.md`** - Complete technical spec
2. **`docs/pow-architecture.md`** - Original architecture doc
3. **`docs/pow-implementation-status.md`** - Implementation checklist
4. **`docs/POW_CORRECT_ARCHITECTURE.md`** - This refactor's architecture (NEW)
5. **`docs/POW_INTEGRATION_GUIDE.md`** - How to integrate (NEW)
6. **`packages/cloak-miner/README.md`** - Miner CLI documentation

---

## ✅ Summary

### What Changed
- Relay: from miner to client ✅
- ClaimManager → ClaimFinder ✅
- Mine → Query ✅
- Ownership: relay → miners ✅

### Why Changed
- Match Ore architecture ✅
- Separation of concerns ✅
- Decentralization ✅
- Scalability ✅

### What's Next
1. Wire ClaimFinder into relay service
2. Test with live miners on devnet
3. Monitor claim availability
4. Deploy to mainnet

**Status**: Ready for integration! 🎊

---

**Questions?** See `docs/POW_INTEGRATION_GUIDE.md` for step-by-step instructions.

**Want to Mine?** See `packages/cloak-miner/README.md` to start earning fees!

