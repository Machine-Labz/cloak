# PoW Architecture - Corrected (Ore-Inspired) ✅

**Last Updated**: 2025-10-19  
**Status**: Correctly separated - Miners independent from Relay

---

## 🎯 Core Principle: Separation of Concerns

```
┌─────────────────────────────────────────────┐
│        INDEPENDENT MINERS                   │
│  • Run cloak-miner CLI 24/7                 │
│  • Compete for mining claims                │
│  • Earn fees when claims are consumed       │
│  • Own their miner keypairs                 │
└──────────────┬──────────────────────────────┘
               │
               │ Claims created on-chain
               ↓
┌─────────────────────────────────────────────┐
│           ON-CHAIN REGISTRY                 │
│  • Stores revealed claims                   │
│  • Tracks claim status & expiry             │
│  • Manages fee distribution                 │
└──────────────┬──────────────────────────────┘
               │
               │ Relay queries for claims
               ↓
┌─────────────────────────────────────────────┐
│           RELAY SERVICE                     │
│  • Generates ZK proofs                      │
│  • Queries on-chain for available claims    │
│  • Builds tx referencing claim              │
│  • Submits tx                               │
│  • NO MINING! Just a client                 │
└─────────────────────────────────────────────┘
```

---

## ❌ What Was Wrong (Original Implementation)

I incorrectly gave the relay a `miner_keypair_path` config, which would make it:
- Mine claims itself
- Compete with independent miners
- Violate separation of concerns

This is like having a stock exchange (relay) also be a trader (miner) - conflict of interest!

---

## ✅ What's Correct Now

### Relay as Client-Only

```rust
pub struct SolanaConfig {
    // ✅ CORRECT: Just needs to know where to query
    pub scramble_registry_program_id: Option<String>,
    
    // ❌ REMOVED: No miner keypair!
    // pub miner_keypair_path: Option<String>,
}
```

### ClaimFinder (Not ClaimManager)

```rust
/// Discovers available claims from independent miners
pub struct ClaimFinder {
    rpc_client: RpcClient,
    registry_program_id: Pubkey,
}

impl ClaimFinder {
    /// Query on-chain for usable claims
    pub async fn find_claim(
        &self,
        batch_hash: &[u8; 32],
    ) -> Result<Option<AvailableClaim>, Error> {
        // 1. Query getProgramAccounts
        // 2. Filter for:
        //    - Matching batch_hash
        //    - Status = Revealed
        //    - Not expired
        //    - Not fully consumed
        // 3. Return first available claim
    }
}
```

### AvailableClaim Structure

```rust
pub struct AvailableClaim {
    pub claim_pda: Pubkey,
    pub miner_pda: Pubkey,
    pub miner_authority: Pubkey,  // ← Miner earns fees, not relay!
    pub mined_slot: u64,
    pub registry_pda: Pubkey,
}
```

---

## 🔄 Complete Flow

### 1. Miner Mines Independently

```bash
# Miner runs 24/7 on their own server
cloak-miner mine \
  --keypair ~/miner.json \
  --registry scramb1e... \
  --continuous
```

This creates claims on-chain with status `Revealed`.

### 2. User Requests Withdraw

```bash
curl -X POST http://relay.cloak.network/withdraw -d '{
  "amount": 1000000000,
  "recipient": "...",
  "nf_hex": "...",
  "root_hex": "..."
}'
```

### 3. Relay Finds Available Claim

```rust
// In relay's withdraw handler
let job_id = request_id.to_string();
let batch_hash = compute_batch_hash(&job_id);

// Query on-chain for claims
let claim = claim_finder.find_claim(&batch_hash).await?
    .ok_or_else(|| Error::NoClaimAvailable)?;

// Found a claim! Miner is claim.miner_authority
info!("Using claim from miner: {}", claim.miner_authority);
```

### 4. Relay Builds Transaction

```rust
let tx = build_withdraw_transaction_with_pow(
    proof,
    public_inputs,
    recipient,
    amount,
    batch_hash,
    // Standard accounts
    program_id, pool, treasury, roots, nullifier_shard, recipient,
    // PoW accounts (from claim)
    registry_program_id,
    claim.claim_pda,      // ← From on-chain query
    claim.miner_pda,      // ← From on-chain query
    claim.registry_pda,   // ← From on-chain query
    claim.miner_authority, // ← Miner earns fee!
    // Other
    fee_payer, blockhash, priority_fee,
)?;
```

### 5. Transaction Executes

1. Shield-pool validates proof
2. CPI to `consume_claim` (increments counter)
3. Fee split:
   - Protocol: 80% → treasury
   - Miner: 20% → `claim.miner_authority` ✅
4. Funds sent to recipient

**Key**: Miner earns fee, not relay!

---

## 📊 Economic Model

### Miner Revenue

```
Per Withdraw:
  Fee = 0.0025 SOL + 0.5% of amount
  Miner Share = Fee × 20% = (0.0025 + 0.005×amount) × 0.2

For 1 SOL withdraw:
  Fee = 0.0075 SOL
  Miner Earns = 0.0015 SOL (20%)
```

### Miner Costs

```
Mining:
  - Electricity (CPU/GPU)
  - RPC costs (fetching slot hashes)
  
Transaction Fees:
  - mine_claim: ~5,000 lamports (0.000005 SOL)
  - reveal_claim: ~5,000 lamports (0.000005 SOL)
  
Total Cost per Claim: ~0.00001 SOL
```

### Profitability

```
Break-even:
  Earnings > Costs
  0.0015 SOL > 0.00001 SOL ✓

Profit Margin:
  (0.0015 - 0.00001) / 0.0015 = 99.3%
```

**Result**: Very profitable for miners at current difficulty!

---

## 🚀 Deployment Scenarios

### Scenario A: Public Miner Pool

```
Multiple miners compete:
  - MinerA mines claims for batch_hash=0xABC...
  - MinerB mines claims for batch_hash=0xDEF...
  - Relay queries on-chain, uses whoever has claim available
  - First miner with revealed claim gets the fee
```

### Scenario B: Dedicated Miner (Simple)

```
Single miner runs 24/7:
  - Operator runs both relay + miner on same server
  - Miner creates claims continuously
  - Relay queries on-chain (still separated!)
  - All fees go to single miner
```

### Scenario C: Miner Marketplace (Future)

```
Off-chain coordination:
  - Miners advertise available claims via API
  - Relay queries marketplace instead of on-chain
  - Faster discovery, less RPC load
  - Miners can pre-mine popular batch patterns
```

---

## 🛠️ Integration Guide

### For Relay Operators

```toml
# config.toml
[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
program_id = "c1oak..."
scramble_registry_program_id = "scramb1e..."  # Enable PoW

# NO miner_keypair_path needed!
```

```rust
// In main.rs
let claim_finder = if let Some(ref registry_id) = config.scramble_registry_program_id {
    let registry_program_id = Pubkey::from_str(registry_id)?;
    Some(ClaimFinder::new(
        config.rpc_url.clone(),
        registry_program_id,
    ))
} else {
    None
};
```

### For Miners

```bash
# 1. Generate miner keypair
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
```

### For Users

```bash
# Just request withdraw - relay handles everything
curl -X POST http://relay/withdraw -d '{...}'

# If no claims available, relay returns error:
# "No PoW claims available. Please try again later."
```

---

## 🔍 Debugging

### No Claims Available?

```bash
# Check if any miners are running
solana program show scramb1eReg... | grep "Data"

# Query claims manually
solana program accounts scramb1eReg... --output json | jq

# Check claim status
solana account <claim_pda>
```

### Claim Expired?

```rust
// ClaimFinder automatically filters expired claims
// Check registry config for:
reveal_window: 150 slots (~1 min)
claim_window: 300 slots (~2 min)
```

---

## 📈 Future Enhancements

### 1. Mempool Filters (Performance)

```rust
// Instead of get_program_accounts, use memcmp filters
let filters = vec![
    RpcFilterType::Memcmp(Memcmp {
        offset: 40, // batch_hash offset
        bytes: MemcmpEncodedBytes::Base58(bs58::encode(batch_hash).into_string()),
        encoding: None,
    }),
];

let accounts = rpc.get_program_accounts_with_config(
    &registry_program_id,
    RpcProgramAccountsConfig {
        filters: Some(filters),
        ..Default::default()
    },
).await?;
```

### 2. Claim Marketplace API

```rust
// Off-chain service aggregates available claims
GET /api/claims/available?batch_hash=<hash>

Response: {
  "claims": [
    {
      "claim_pda": "...",
      "miner_authority": "...",
      "expires_at": 1234567890,
      "price": 0.0015  // Optional: dynamic pricing
    }
  ]
}
```

### 3. Multi-Job Batches (k>1)

```rust
// Miner mines for common patterns
let batch_hash = compute_batch_hash(&["job-001", "job-002", "job-003"]);
// Claim can be used for any of the 3 jobs
```

---

## ✅ Checklist: Is Your Architecture Correct?

- ✅ Relay queries on-chain for claims (doesn't mine)
- ✅ Relay has NO miner keypair
- ✅ Miners run independently (cloak-miner CLI)
- ✅ Miners own their keypairs
- ✅ Miner earns fee (not relay)
- ✅ ClaimFinder discovers, doesn't manage
- ✅ Separation: relay=prover, miner=claimer

---

## 🎉 Summary

**Before (Wrong)**:
```
Relay = Prover + Miner  ❌
```

**After (Correct)**:
```
Relay = Prover (client of miners)  ✅
Miners = Independent competitors  ✅
```

This matches the Ore model where:
- Ore CLI mines ORE tokens independently
- dApps use existing tokens, don't mine
- Miners compete, users benefit from market

**Status**: Architecture corrected! Relay is now a proper client. 🎊

