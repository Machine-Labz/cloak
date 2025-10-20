# PoW Integration Guide - Relay Service

**Last Updated**: 2025-10-19  
**Status**: Ready for Integration  
**Architecture**: Ore-Inspired (Relay queries on-chain for claims)

---

## Overview

This guide shows how to integrate the ClaimFinder into the relay's withdraw flow. The relay discovers available PoW claims from independent miners and uses them when building transactions.

---

## Step 1: Add ClaimFinder to Relay State

### File: `services/relay/src/main.rs`

```rust
mod claim_manager; // Actually exports ClaimFinder

use claim_manager::ClaimFinder;
use std::sync::Arc;
use tokio::sync::RwLock;

// In your main function or app state initialization:
let claim_finder = if let Some(ref registry_id) = config.solana.scramble_registry_program_id {
    let registry_program_id = Pubkey::from_str(registry_id)
        .context("Invalid scramble registry program ID")?;
    
    Some(Arc::new(ClaimFinder::new(
        config.solana.rpc_url.clone(),
        registry_program_id,
    )))
} else {
    None
};

// Add to app state or pass to services
let app_state = AppState {
    // ... existing fields ...
    claim_finder,
};
```

---

## Step 2: Update SolanaService

### File: `services/relay/src/solana/service.rs` (or equivalent)

```rust
use crate::claim_manager::{ClaimFinder, compute_batch_hash};
use std::sync::Arc;

pub struct SolanaService {
    // ... existing fields ...
    claim_finder: Option<Arc<ClaimFinder>>,
}

impl SolanaService {
    pub fn new(
        // ... existing params ...
        claim_finder: Option<Arc<ClaimFinder>>,
    ) -> Self {
        Self {
            // ... existing fields ...
            claim_finder,
        }
    }
}
```

---

## Step 3: Integrate into Withdraw Flow

### File: `services/relay/src/api/withdraw.rs` or `services/relay/src/worker/processor.rs`

```rust
use crate::claim_manager::compute_batch_hash;
use crate::solana::transaction_builder::{
    build_withdraw_transaction_with_pow,
    build_withdraw_transaction, // Legacy fallback
};

// In your withdraw handler:
pub async fn process_withdraw(
    &self,
    job: WithdrawJob,
) -> Result<String, Error> {
    // 1. Generate ZK proof (existing code)
    let proof = self.prove(&job).await?;
    
    // 2. Check if PoW is enabled
    if let Some(ref claim_finder) = self.claim_finder {
        // PoW path
        self.withdraw_with_pow(job, proof, claim_finder).await
    } else {
        // Legacy path (no PoW)
        self.withdraw_legacy(job, proof).await
    }
}

async fn withdraw_with_pow(
    &self,
    job: WithdrawJob,
    proof: ProofData,
    claim_finder: &ClaimFinder,
) -> Result<String, Error> {
    // 1. Compute batch hash (k=1 for MVP: one job per batch)
    let job_id = job.request_id.to_string();
    let batch_hash = compute_batch_hash(&job_id);
    
    tracing::info!("🔍 Looking for PoW claim for batch_hash: {}", hex::encode(&batch_hash));
    
    // 2. Find available claim on-chain
    let claim = claim_finder
        .find_claim(&batch_hash)
        .await?
        .ok_or_else(|| {
            Error::NoClaimAvailable(format!(
                "No PoW claims available for batch_hash {}. Please wait for miners.",
                hex::encode(&batch_hash)
            ))
        })?;
    
    tracing::info!(
        "✅ Found claim from miner: {} (expires at slot {}, pda: {})",
        claim.miner_authority,
        claim.mined_slot,
        claim.claim_pda,
    );
    
    // 3. Build transaction with PoW accounts
    let tx = build_withdraw_transaction_with_pow(
        // Proof data
        proof.groth16_bytes,
        proof.public_inputs,
        job.recipient,
        job.amount,
        batch_hash,
        
        // Standard accounts
        self.config.program_id,
        self.pool_pda,
        self.treasury_pda,
        self.roots_ring_pda,
        self.nullifier_shard_pda,
        job.recipient,
        
        // PoW accounts (from claim)
        self.config.scramble_registry_program_id.unwrap(),
        claim.claim_pda,
        claim.miner_pda,
        claim.registry_pda,
        claim.miner_authority, // ← Miner receives fee share!
        
        // Transaction params
        self.fee_payer,
        self.get_recent_blockhash().await?,
        self.config.priority_micro_lamports,
    )?;
    
    // 4. Submit transaction
    let signature = self.submit_transaction(tx).await?;
    
    tracing::info!("🎉 Withdraw submitted with PoW claim: {}", signature);
    
    Ok(signature.to_string())
}

async fn withdraw_legacy(
    &self,
    job: WithdrawJob,
    proof: ProofData,
) -> Result<String, Error> {
    // Existing withdraw logic without PoW
    let tx = build_withdraw_transaction(
        proof.groth16_bytes,
        proof.public_inputs,
        job.recipient,
        job.amount,
        // ... standard accounts ...
        self.fee_payer,
        self.get_recent_blockhash().await?,
        self.config.priority_micro_lamports,
    )?;
    
    let signature = self.submit_transaction(tx).await?;
    Ok(signature.to_string())
}
```

---

## Step 4: Error Handling

### File: `services/relay/src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ... existing errors ...
    
    #[error("No PoW claim available: {0}")]
    NoClaimAvailable(String),
    
    #[error("Claim expired or fully consumed")]
    ClaimNotUsable,
    
    #[error("Failed to query on-chain claims: {0}")]
    ClaimQueryFailed(String),
}
```

### In your API handler:

```rust
match process_withdraw(job).await {
    Ok(signature) => HttpResponse::Ok().json(WithdrawResponse { signature }),
    Err(Error::NoClaimAvailable(msg)) => {
        HttpResponse::ServiceUnavailable().json(ErrorResponse {
            error: "NO_CLAIMS_AVAILABLE",
            message: msg,
            retry_after_seconds: Some(30), // User can retry
        })
    }
    Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
        error: "WITHDRAW_FAILED",
        message: e.to_string(),
        retry_after_seconds: None,
    }),
}
```

---

## Step 5: Configuration

### File: `config.toml`

```toml
[solana]
rpc_url = "https://api.devnet.solana.com"
program_id = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"

# Enable PoW (optional)
scramble_registry_program_id = "scramb1eReg1stryPoWM1n1ngSo1anaC1oak11111111"

# NO miner_keypair_path! Relay queries on-chain for claims.
```

---

## Step 6: Testing

### Local Testing (Without Miners)

```bash
# Start relay with PoW disabled
cargo run --bin relay

# Test withdraw (should use legacy path)
curl -X POST http://localhost:3000/api/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1000000000,
    "recipient": "...",
    "nf_hex": "...",
    "root_hex": "..."
  }'
```

### Testing with Miners (Localnet)

```bash
# Terminal 1: Start localnet
solana-test-validator

# Terminal 2: Deploy programs
just deploy-local

# Terminal 3: Initialize registry
cd programs/scramble-registry
./init-localnet.sh

# Terminal 4: Run miner
cd packages/cloak-miner
cargo run -- mine \
  --keypair ../../miner.json \
  --registry scramb1e... \
  --continuous

# Terminal 5: Start relay with PoW enabled
cd services/relay
cargo run

# Terminal 6: Test withdraw
curl -X POST http://localhost:3000/api/withdraw -d '{...}'
```

---

## Step 7: Monitoring

### Add Metrics

```rust
use prometheus::{IntCounter, IntGauge, register_int_counter, register_int_gauge};

lazy_static! {
    static ref CLAIMS_FOUND: IntCounter = register_int_counter!(
        "cloak_claims_found_total",
        "Total PoW claims found on-chain"
    ).unwrap();
    
    static ref CLAIMS_NOT_FOUND: IntCounter = register_int_counter!(
        "cloak_claims_not_found_total",
        "Total PoW claim queries that found nothing"
    ).unwrap();
    
    static ref WITHDRAW_WITH_POW: IntCounter = register_int_counter!(
        "cloak_withdraws_with_pow_total",
        "Total withdraws using PoW claims"
    ).unwrap();
    
    static ref WITHDRAW_LEGACY: IntCounter = register_int_counter!(
        "cloak_withdraws_legacy_total",
        "Total withdraws without PoW (legacy)"
    ).unwrap();
}

// In your withdraw handler:
if let Some(ref claim_finder) = self.claim_finder {
    match claim_finder.find_claim(&batch_hash).await? {
        Some(claim) => {
            CLAIMS_FOUND.inc();
            WITHDRAW_WITH_POW.inc();
            // ... use claim ...
        }
        None => {
            CLAIMS_NOT_FOUND.inc();
            return Err(Error::NoClaimAvailable(/* ... */));
        }
    }
} else {
    WITHDRAW_LEGACY.inc();
    // ... legacy path ...
}
```

### Logs to Monitor

```
✅ Good:
- "✅ Found available claim: <pda> (consumed X/Y)"
- "🎉 Withdraw submitted with PoW claim: <sig>"

⚠️ Warning:
- "⚠️  No available claims found for batch_hash: <hash>"

❌ Error:
- "Failed to query claims: <error>"
- "Claim expired or fully consumed"
```

---

## Step 8: Production Checklist

- [ ] PoW enabled in config (`scramble_registry_program_id` set)
- [ ] Miners running and creating claims
- [ ] ClaimFinder integrated into withdraw flow
- [ ] Error handling for "no claims available"
- [ ] Metrics for claim discovery and usage
- [ ] Logs for debugging claim issues
- [ ] Fallback to legacy withdraw if PoW fails (optional)
- [ ] User-facing error messages explaining retry

---

## Troubleshooting

### Issue: "No PoW claims available"

**Cause**: No miners running or mining wrong batch patterns

**Fix**:
```bash
# Check if miners are registered
solana account <miner_pda>

# Check if any claims exist
solana program accounts <registry_program_id> --output json | jq '.[] | select(.account.data.length == 256)'

# Start a miner
cloak-miner mine --keypair ~/miner.json --registry <id> --continuous
```

### Issue: Claims expired before use

**Cause**: Claim window too short or relay too slow

**Fix**:
```bash
# Increase claim_window in registry (admin instruction)
cloak-registry update-params \
  --claim-window 600  # 600 slots = ~4 minutes
```

### Issue: High RPC costs from `get_program_accounts`

**Cause**: Querying all accounts is expensive

**Fix**: Use memcmp filters (see POW_CORRECT_ARCHITECTURE.md section on "Mempool Filters")

---

## Next Steps

After integration:

1. **Test on Devnet**: Deploy to devnet, run miners, test withdraws
2. **Optimize**: Add memcmp filters to reduce RPC load
3. **Monitor**: Set up Grafana dashboard for claim metrics
4. **Scale**: Consider claim marketplace API for faster discovery
5. **Batch**: Implement k>1 batches for efficiency

---

## Summary

✅ **What We Did**:
- Added ClaimFinder to relay state
- Query on-chain for available claims
- Build PoW-enabled transactions
- Handle "no claims" gracefully
- Maintain legacy fallback

✅ **What Relay Does**:
- Generates ZK proofs
- Queries on-chain for claims
- Submits transactions
- **Does NOT mine!**

✅ **What Miners Do**:
- Run independently
- Mine claims 24/7
- Earn fees when claims used

**Architecture**: Correct separation of concerns! 🎊

