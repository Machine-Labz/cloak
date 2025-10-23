---
title: PoW-Enhanced Withdrawals
description: Complete guide to proof-of-work mining, wildcard claims, and enhanced withdrawal processing with claim consumption.
---

# PoW-Enhanced Withdrawals

PoW-enhanced withdrawals leverage wildcard proof-of-work claims to eliminate the need for pre-computed batch hashes. Miners mine claims with a batch hash of all zeros, allowing the relay to attach any job-specific hash when consuming the claim.

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   cloak-miner   │    │ Scramble Registry │    │   Relay Worker  │
│                 │    │     Program       │    │                 │
│ 1. Mine Claims  │───▶│ 2. Store Claims  │◀───│ 3. Find Claims  │
│ 4. Reveal       │───▶│ 5. Track State   │◀───│ 6. Consume CPI  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Mining Pool   │    │   On-Chain       │    │ Shield Pool     │
│   (BLAKE3)      │    │   Storage        │    │   Program       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Core Concepts

### Wildcard Claims

Wildcard claims use a batch hash of `[0u8; 32]` (all zeros), making them universally applicable to any withdrawal job. This eliminates the need to mine specific claims for each batch hash.

### Claim States

```
unmined → mined → revealed → consumed
    ↓        ↓        ↓         ↓
  start   commit   reveal   withdraw
```

**State Transitions:**
- `unmined` → `mined`: Miner submits `mine_claim` transaction
- `mined` → `revealed`: Miner submits `reveal_claim` transaction (after reveal window opens)
- `revealed` → `consumed`: Relay consumes claim via CPI during withdraw execution

### Mining Algorithm

**Preimage Structure:**
```
Domain:       "CLOAK:SCRAMBLE:v1"  (17 bytes)
Slot:         u64 LE               (8 bytes)
Slot Hash:    [u8; 32]             (32 bytes)
Miner Pubkey: [u8; 32]             (32 bytes)
Batch Hash:   [u8; 32]             (32 bytes) - [0; 32] for wildcard
Nonce:        u128 LE              (16 bytes)
Total:        137 bytes
```

**Validation:**
```rust
let preimage = [
    b"CLOAK:SCRAMBLE:v1",  // 17 bytes
    slot.to_le_bytes(),    // 8 bytes
    slot_hash,             // 32 bytes
    miner_pubkey,          // 32 bytes
    batch_hash,            // 32 bytes (all zeros for wildcard)
    nonce.to_le_bytes(),   // 16 bytes
];

let hash = blake3(&preimage);
let is_valid = hash < difficulty_target;
```

## Step 1: Miner Registration

Before mining, miners must register with the scramble registry program.

### Initialize Miner Account

```bash
# Register new miner
cloak-miner register \
  --keypair ~/.config/solana/miner-keypair.json \
  --rpc-url https://api.mainnet-beta.solana.com

# Expected output
✅ Miner registered successfully
Miner PDA: 7yRtcB7vG8K9mN2pQ1wE4rT6uI8oP3aS5dF7gH9jK2lM4nB6vC8xZ0
Registration slot: 123456789
```

### Verify Registration

```bash
# Check miner status
cloak-miner status \
  --keypair ~/.config/solana/miner-keypair.json \
  --rpc-url https://api.mainnet-beta.solana.com

# Expected output
Miner Status:
├── PDA: 1111111111111111111111111111111111111111111111111111111111111111
├── Authority: 2222222222222222222222222222222222222222222222222222222222222222
├── Registered: Slot 123456789
├── Total Claims: 0
├── Active Claims: 0
└── Status: Active
```

## Step 2: Wildcard Mining

Mine wildcard claims that can be consumed by any withdrawal job.

### Start Mining Process

```bash
# Start wildcard mining
cloak-miner mine \
  --mode wildcard \
  --keypair ~/.config/solana/miner-keypair.json \
  --rpc-url https://api.mainnet-beta.solana.com \
  --difficulty 1000000

# Expected output
🔨 Starting wildcard mining...
📊 Current difficulty: 1,000,000
🎯 Target: 0x00000000000000000000000000000000000000000000000000000000000f4240
⏱️  Mining started at slot 123456790
```

### Mining Implementation

```rust
use blake3::{Hasher, Hash};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub struct WildcardMiner {
    rpc_client: RpcClient,
    miner_keypair: Keypair,
    registry_program_id: Pubkey,
}

impl WildcardMiner {
    pub async fn mine_wildcard_claim(&self, difficulty_target: u64) -> Result<ClaimPreimage> {
        // 1. Fetch current difficulty and slot info
        let difficulty_info = self.fetch_difficulty().await?;
        let slot_info = self.fetch_slot_info().await?;
        
        // 2. Prepare mining context
        let domain = b"CLOAK:SCRAMBLE:v1";
        let slot = slot_info.slot;
        let slot_hash = slot_info.hash;
        let miner_pubkey = self.miner_keypair.pubkey().to_bytes();
        let batch_hash = [0u8; 32]; // Wildcard: all zeros
        
        // 3. Mining loop
        let mut nonce = 0u128;
        let start_time = std::time::Instant::now();
        
        loop {
            // Build preimage
            let mut preimage = Vec::with_capacity(137);
            preimage.extend_from_slice(domain);           // 17 bytes
            preimage.extend_from_slice(&slot.to_le_bytes()); // 8 bytes
            preimage.extend_from_slice(&slot_hash);        // 32 bytes
            preimage.extend_from_slice(&miner_pubkey);     // 32 bytes
            preimage.extend_from_slice(&batch_hash);       // 32 bytes
            preimage.extend_from_slice(&nonce.to_le_bytes()); // 16 bytes
            
            // Hash and check difficulty
            let hash = blake3(&preimage);
            let hash_u64 = u64::from_le_bytes(hash.as_bytes()[0..8].try_into()?);
            
            if hash_u64 < difficulty_target {
                let mining_time = start_time.elapsed();
                let hashrate = nonce as f64 / mining_time.as_secs_f64();
                
                info!("✅ Wildcard claim found!");
                info!("Nonce: {}", nonce);
                info!("Hash: {}", hex::encode(hash.as_bytes()));
                info!("Mining time: {:?}", mining_time);
                info!("Hashrate: {:.2} H/s", hashrate);
                
                return Ok(ClaimPreimage {
                    slot,
                    slot_hash,
                    miner_pubkey: self.miner_keypair.pubkey(),
                    batch_hash,
                    nonce,
                    hash: hash.as_bytes().try_into()?,
                });
            }
            
            nonce += 1;
            
            // Progress reporting
            if nonce % 1_000_000 == 0 {
                let elapsed = start_time.elapsed();
                let rate = nonce as f64 / elapsed.as_secs_f64();
                info!("Mining progress: {} nonces, {:.2} H/s", nonce, rate);
            }
        }
    }
}
```

### Submit Mine Transaction

```rust
pub async fn submit_mine_transaction(&self, preimage: &ClaimPreimage) -> Result<Signature> {
    // 1. Build mine_claim instruction
    let mine_ix = Instruction {
        program_id: self.registry_program_id,
        accounts: vec![
            AccountMeta::new(self.miner_pda, false),
            AccountMeta::new_readonly(self.miner_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: MineClaimData {
            hash: preimage.hash,
        }.try_to_vec()?,
    };
    
    // 2. Build transaction
    let mut tx = Transaction::new_with_payer(
        &[mine_ix],
        Some(&self.miner_keypair.pubkey()),
    );
    
    // 3. Get recent blockhash and sign
    let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;
    tx.sign(&[&self.miner_keypair], recent_blockhash);
    
    // 4. Submit transaction
    let signature = self.rpc_client
        .send_and_confirm_transaction(&tx)
        .await?;
    
    info!("✅ Mine transaction submitted: {}", signature);
    Ok(signature)
}
```

## Step 3: Claim Revelation

After the reveal window opens, miners must reveal their preimages to make claims consumable.

### Check Reveal Window

```bash
# Check if reveal window is open
cloak-miner status \
  --keypair ~/.config/solana/miner-keypair.json \
  --rpc-url https://api.mainnet-beta.solana.com

# Expected output
Active Claims:
├── Claim PDA: 3333333333333333333333333333333333333333333333333333333333333333
├── Hash: 0x4444444444444444444444444444444444444444444444444444444444444444
├── Slot: 123456790
├── Status: mined
├── Reveal Window: Opens at slot 123456850 (60 slots from now)
└── Expires: Slot 123457790 (1000 slots from mine)
```

### Submit Reveal Transaction

```bash
# Reveal claim when window opens
cloak-miner reveal \
  --claim-pda 8zRtcB7vG8K9mN2pQ1wE4rT6uI8oP3aS5dF7gH9jK2lM4nB6vC8xZ1 \
  --keypair ~/.config/solana/miner-keypair.json \
  --rpc-url https://api.mainnet-beta.solana.com

# Expected output
✅ Claim revealed successfully
Transaction: 5xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
Reveal slot: 123456850
Status: revealed
```

### Reveal Implementation

```rust
pub async fn reveal_claim(&self, claim_pda: Pubkey, preimage: &ClaimPreimage) -> Result<Signature> {
    // 1. Verify reveal window is open
    let claim_account = self.rpc_client.get_account(&claim_pda).await?;
    let claim_data = ClaimAccount::try_from_slice(&claim_account.data)?;
    
    let current_slot = self.rpc_client.get_slot().await?;
    if current_slot < claim_data.reveal_window_start {
        return Err(Error::RevealWindowNotOpen);
    }
    
    // 2. Build reveal_claim instruction
    let reveal_ix = Instruction {
        program_id: self.registry_program_id,
        accounts: vec![
            AccountMeta::new(claim_pda, false),
            AccountMeta::new_readonly(self.miner_keypair.pubkey(), true),
        ],
        data: RevealClaimData {
            slot: preimage.slot,
            slot_hash: preimage.slot_hash,
            miner_pubkey: preimage.miner_pubkey,
            batch_hash: preimage.batch_hash,
            nonce: preimage.nonce,
        }.try_to_vec()?,
    };
    
    // 3. Build and submit transaction
    let mut tx = Transaction::new_with_payer(
        &[reveal_ix],
        Some(&self.miner_keypair.pubkey()),
    );
    
    let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;
    tx.sign(&[&self.miner_keypair], recent_blockhash);
    
    let signature = self.rpc_client
        .send_and_confirm_transaction(&tx)
        .await?;
    
    info!("✅ Reveal transaction submitted: {}", signature);
    Ok(signature)
}
```

## Step 4: Relay Claim Discovery

The relay service discovers available wildcard claims for consumption.

### ClaimFinder Implementation

```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub struct ClaimFinder {
    rpc_client: RpcClient,
    registry_program_id: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AvailableClaim {
    pub claim_pda: Pubkey,
    pub miner_pda: Pubkey,
    pub miner_authority: Pubkey,
    pub mined_slot: u64,
    pub revealed_slot: u64,
    pub batch_hash: [u8; 32],
    pub hash: [u8; 32],
    pub consumption_count: u8,
    pub max_consumptions: u8,
    pub expires_at_slot: u64,
}

impl ClaimFinder {
    pub async fn find_wildcard_claim(&self) -> Result<Option<AvailableClaim>> {
        // 1. Get all claim accounts
        let claim_accounts = self.rpc_client
            .get_program_accounts(&self.registry_program_id)
            .await?;
        
        // 2. Filter for wildcard claims
        let mut wildcard_claims = Vec::new();
        
        for (pubkey, account) in claim_accounts {
            if let Ok(claim_data) = ClaimAccount::try_from_slice(&account.data) {
                // Check if it's a wildcard claim (batch_hash = [0; 32])
                if claim_data.batch_hash == [0u8; 32] {
                    wildcard_claims.push((pubkey, claim_data));
                }
            }
        }
        
        // 3. Filter by availability
        let current_slot = self.rpc_client.get_slot().await?;
        let available_claims: Vec<_> = wildcard_claims
            .into_iter()
            .filter(|(_, claim_data)| {
                // Must be revealed
                claim_data.revealed_slot.is_some() &&
                // Not expired
                claim_data.expires_at_slot > current_slot &&
                // Not fully consumed
                claim_data.consumption_count < claim_data.max_consumptions
            })
            .collect();
        
        if available_claims.is_empty() {
            return Ok(None);
        }
        
        // 4. Select best claim (prefer older, less consumed)
        let best_claim = available_claims
            .into_iter()
            .min_by_key(|(_, claim_data)| {
                (claim_data.consumption_count, claim_data.mined_slot)
            });
        
        if let Some((claim_pda, claim_data)) = best_claim {
            Ok(Some(AvailableClaim {
                claim_pda,
                miner_pda: claim_data.miner_pda,
                miner_authority: claim_data.miner_authority,
                mined_slot: claim_data.mined_slot,
                revealed_slot: claim_data.revealed_slot.unwrap(),
                batch_hash: claim_data.batch_hash,
                hash: claim_data.hash,
                consumption_count: claim_data.consumption_count,
                max_consumptions: claim_data.max_consumptions,
                expires_at_slot: claim_data.expires_at_slot,
            }))
} else {
            Ok(None)
        }
    }
    
    pub async fn find_specific_claim(&self, batch_hash: [u8; 32]) -> Result<Option<AvailableClaim>> {
        // Similar to find_wildcard_claim but filters for specific batch_hash
        let claim_accounts = self.rpc_client
            .get_program_accounts(&self.registry_program_id)
            .await?;
        
        let current_slot = self.rpc_client.get_slot().await?;
        
        for (pubkey, account) in claim_accounts {
            if let Ok(claim_data) = ClaimAccount::try_from_slice(&account.data) {
                if claim_data.batch_hash == batch_hash &&
                   claim_data.revealed_slot.is_some() &&
                   claim_data.expires_at_slot > current_slot &&
                   claim_data.consumption_count < claim_data.max_consumptions {
                    
                    return Ok(Some(AvailableClaim {
                        claim_pda: pubkey,
                        miner_pda: claim_data.miner_pda,
                        miner_authority: claim_data.miner_authority,
                        mined_slot: claim_data.mined_slot,
                        revealed_slot: claim_data.revealed_slot.unwrap(),
                        batch_hash: claim_data.batch_hash,
                        hash: claim_data.hash,
                        consumption_count: claim_data.consumption_count,
                        max_consumptions: claim_data.max_consumptions,
                        expires_at_slot: claim_data.expires_at_slot,
                    }));
                }
            }
        }
        
        Ok(None)
    }
}
```

### Integration with Relay Worker

```rust
impl RelayWorker {
    async fn process_withdraw_job(&self, job: &WithdrawJob) -> Result<()> {
        // 1. Find available claim
        let claim_finder = ClaimFinder::new(
            self.rpc_client.clone(),
            self.config.scramble_registry_program_id,
        );
        
        // Try wildcard first, then specific batch hash
        let claim = claim_finder
            .find_wildcard_claim()
            .await?
            .or_else(|| {
                // Fallback to specific claim if wildcard not available
                claim_finder.find_specific_claim(job.batch_hash).await.ok().flatten()
            })
            .ok_or(Error::NoClaimsAvailable)?;
        
        info!("Found claim: {}", claim.claim_pda);
        info!("Miner: {}", claim.miner_authority);
        info!("Consumptions: {}/{}", claim.consumption_count, claim.max_consumptions);
        info!("Expires at slot: {}", claim.expires_at_slot);
        
        // 2. Build withdraw instruction with PoW claim
        let withdraw_ix = self.build_withdraw_instruction_with_pow(job, &claim)?;
        
        // 3. Build transaction
        let mut tx = Transaction::new_with_payer(
            &[withdraw_ix],
            Some(&self.payer.pubkey()),
        );
        
        // 4. Simulate transaction
        let simulation = self.rpc_client.simulate_transaction(&tx).await?;
        if let Some(err) = simulation.value.err {
            return Err(Error::SimulationFailed(err));
        }
        
        // 5. Set compute budget
        let cu_consumed = simulation.value.units_consumed.unwrap_or(200_000);
        tx.add_compute_budget(cu_consumed + 10_000)?;
        
        // 6. Sign and submit
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;
        tx.sign(&[&self.payer], recent_blockhash);
        
        let signature = self.rpc_client
            .send_and_confirm_transaction(&tx)
            .await?;
        
        info!("✅ Withdraw transaction confirmed: {}", signature);
        Ok(())
    }
}
```

## Step 5: On-Chain Claim Consumption

The shield-pool program consumes PoW claims via CPI calls to the scramble registry.

### Withdraw Instruction with PoW

```rust
pub fn withdraw_with_pow(
    ctx: Context<WithdrawWithPow>,
    proof: [u8; 260],
    public_inputs: [u8; 104],
    outputs: Vec<Output>,
) -> Result<()> {
    // 1. Standard withdraw validation
    sp1_solana::verify_proof(&proof, &public_inputs, &VKEY_HASH)?;
    
    let root = &public_inputs[0..32];
    let nf = &public_inputs[32..64];
    let outputs_hash = &public_inputs[64..96];
    let amount = u64::from_le_bytes(public_inputs[96..104].try_into()?);
    
    // 2. Verify root in ring buffer
    require!(ctx.accounts.roots_ring.contains(root), ErrorCode::RootNotFound);
    
    // 3. Check nullifier unused
    require!(!ctx.accounts.nullifier_shard.contains(nf), ErrorCode::NullifierUsed);
    
    // 4. Mark nullifier as spent
    ctx.accounts.nullifier_shard.insert(nf)?;
    
    // 5. Verify outputs hash
    let computed_hash = compute_outputs_hash(&outputs);
    require!(computed_hash == outputs_hash, ErrorCode::OutputsHashMismatch);
    
    // 6. Verify conservation
    let fee = calculate_fee(amount, FEE_BPS);
    let outputs_sum: u64 = outputs.iter().map(|o| o.amount).sum();
    require!(outputs_sum + fee == amount, ErrorCode::AmountMismatch);
    
    // 7. Transfer funds to recipients
    for output in outputs {
        transfer_from_pool(output.address, output.amount)?;
    }
    
    // 8. Transfer fee to treasury
    transfer_from_pool(treasury, fee)?;
    
    // 9. Consume PoW claim via CPI
    let consume_claim_ctx = CpiContext::new(
        ctx.accounts.scramble_registry.to_account_info(),
        ConsumeClaim {
            claim: ctx.accounts.claim.to_account_info(),
            miner: ctx.accounts.miner.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    
    // Compute batch hash from outputs
    let batch_hash = compute_batch_hash(&outputs);
    
    scramble_registry::cpi::consume_claim(
        consume_claim_ctx,
        batch_hash,
    )?;
    
    // 10. Emit event
    emit!(WithdrawEvent {
        nullifier: *nf,
        amount,
        outputs_count: outputs.len() as u8,
        pow_claim: Some(ctx.accounts.claim.key()),
    });
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(proof: [u8; 260], public_inputs: [u8; 104], outputs: Vec<Output>)]
pub struct WithdrawWithPow<'info> {
    // Standard withdraw accounts
    #[account(mut)]
    pub pool: Account<'info, ShieldPool>,
    
    #[account(mut)]
    pub roots_ring: Account<'info, RootsRing>,
    
    #[account(mut)]
    pub nullifier_shard: Account<'info, NullifierShard>,
    
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    
    // PoW claim accounts
    #[account(mut)]
    pub claim: Account<'info, ClaimAccount>,
    
    #[account(mut)]
    pub miner: Account<'info, MinerAccount>,
    
    #[account(signer)]
    pub authority: Signer<'info>,
    
    pub scramble_registry: Program<'info, ScrambleRegistry>,
    
    pub system_program: Program<'info, System>,
}
```

### CPI Implementation

```rust
// In scramble-registry program
pub fn consume_claim(
    ctx: Context<ConsumeClaim>,
    batch_hash: [u8; 32],
) -> Result<()> {
    let claim = &mut ctx.accounts.claim;
    let miner = &mut ctx.accounts.miner;
    
    // 1. Verify claim is revealed
    require!(claim.revealed_slot.is_some(), ErrorCode::ClaimNotRevealed);
    
    // 2. Check claim not expired
    let current_slot = Clock::get()?.slot;
    require!(claim.expires_at_slot > current_slot, ErrorCode::ClaimExpired);
    
    // 3. Check consumption limit
    require!(claim.consumption_count < claim.max_consumptions, ErrorCode::ClaimFullyConsumed);
    
    // 4. Verify batch hash (wildcard allows any hash)
    if claim.batch_hash != [0u8; 32] {
        require!(claim.batch_hash == batch_hash, ErrorCode::BatchHashMismatch);
    }
    
    // 5. Increment consumption count
    claim.consumption_count += 1;
    
    // 6. Update miner stats
    miner.total_consumptions += 1;
    
    // 7. Emit consumption event
    emit!(ClaimConsumedEvent {
        claim: ctx.accounts.claim.key(),
        miner: ctx.accounts.miner.key(),
        batch_hash,
        consumption_count: claim.consumption_count,
        slot: current_slot,
    });
    
    Ok(())
}
```

## Step 6: Complete Workflow Example

Here's a complete end-to-end example of PoW-enhanced withdrawal:

```typescript
import { Connection, Keypair } from '@solana/web3.js';
import { blake3 } from '@noble/hashes/blake3';

async function powEnhancedWithdraw(
  connection: Connection,
  note: SpendableNote,
  recipients: Array<{ address: string; amount: number }>
): Promise<string> {
  console.log('🔨 Starting PoW-enhanced withdrawal...');
  
  // 1. Generate ZK proof (same as standard withdraw)
  console.log('📝 Step 1: Generating ZK proof...');
  const proofResult = await generateProof(note, recipients);
  console.log('✅ Proof generated');
  
  // 2. Submit to relay with PoW preference
  console.log('📤 Step 2: Submitting to relay...');
  
  const requestId = await submitWithdraw({
    outputs: recipients,
    policy: { fee_bps: 60 },
    public_inputs: proofResult.public_inputs,
    proof_bytes: Buffer.from(proofResult.proof_bytes).toString('base64'),
    pow_preference: 'wildcard', // Prefer wildcard claims
  });
  
  console.log('✅ Withdraw submitted');
  console.log('📋 Request ID:', requestId);
  
  // 3. Monitor job status
  console.log('⏳ Step 3: Monitoring job status...');
  
  const txId = await waitForCompletion(requestId, 120_000);
  
  console.log('✅ PoW-enhanced withdraw completed!');
  console.log('🔗 Transaction:', txId);
  console.log(`🌐 Explorer: https://solscan.io/tx/${txId}`);
  
  return txId;
}

// Monitor job with PoW-specific status
async function waitForCompletion(requestId: string, timeout = 120_000) {
  const start = Date.now();
  const pollInterval = 2000;
  
  while (Date.now() - start < timeout) {
    const response = await fetch(`http://localhost:3002/status/${requestId}`);
    const status = await response.json();
    
    console.log(`Status: ${status.status}`);
    
    if (status.status === 'processing') {
      console.log('🔍 Searching for PoW claims...');
      if (status.pow_claim_found) {
        console.log('✅ PoW claim found:', status.pow_claim_pda);
        console.log('⛏️  Miner:', status.pow_miner);
        console.log('📊 Consumptions:', status.pow_consumptions);
      }
    }
    
    if (status.status === 'completed') {
      console.log('✅ Withdraw completed!');
      console.log('Transaction:', status.tx_id);
      if (status.pow_claim_consumed) {
        console.log('🎯 PoW claim consumed:', status.pow_claim_pda);
      }
      return status.tx_id;
    }
    
    if (status.status === 'failed') {
      if (status.error?.code === 'NoClaimsAvailable') {
        console.log('❌ No PoW claims available');
        console.log('💡 Consider running miners or using standard withdraw');
      }
      throw new Error(`Withdraw failed: ${status.error?.message}`);
    }
    
    await new Promise(resolve => setTimeout(resolve, pollInterval));
  }
  
  throw new Error('Withdraw timeout - still processing');
}
```

## Monitoring and Metrics

### Relay Metrics

The relay emits detailed metrics for PoW claim processing:

```rust
// Claim discovery metrics
info!("[METRICS] claim_search_duration_ms={}", search_duration.as_millis());
info!("[METRICS] claim_search_result={}", if claim.is_some() { "found" } else { "not_found" });
info!("[METRICS] claim_type={}", if claim.as_ref().map(|c| c.batch_hash == [0u8; 32]).unwrap_or(false) { "wildcard" } else { "specific" });

// Claim consumption metrics
info!("[METRICS] claim_consumption_success=true");
info!("[METRICS] claim_pda={}", claim.claim_pda);
info!("[METRICS] miner_pda={}", claim.miner_pda);
info!("[METRICS] consumption_count={}", claim.consumption_count);
info!("[METRICS] max_consumptions={}", claim.max_consumptions);
```

### Miner Monitoring

```bash
# Monitor miner performance
cloak-miner status --keypair ~/.config/solana/miner-keypair.json

# Expected output
Miner Status:
├── PDA: 1111111111111111111111111111111111111111111111111111111111111111
├── Authority: 2222222222222222222222222222222222222222222222222222222222222222
├── Registered: Slot 123456789
├── Total Claims: 15
├── Active Claims: 3
├── Total Consumptions: 47
└── Status: Active

Active Claims:
├── Claim PDA: 3333333333333333333333333333333333333333333333333333333333333333
├── Hash: 0x4444444444444444444444444444444444444444444444444444444444444444
├── Slot: 123456790
├── Status: revealed
├── Consumptions: 2/5
├── Expires: Slot 123457790
└── Type: wildcard
```

## Troubleshooting

### Common Issues

**Scenario: No Wildcard Claims Available**
```
{
  "success": false,
  "error": {
    "code": "NoClaimsAvailable",
    "message": "No PoW claims available for consumption"
  }
}
```

**Cause:** No miners have submitted wildcard claims or all claims are consumed/expired

**Solutions:**
1. **Start Mining:**
   ```bash
   # Start wildcard mining
   cloak-miner mine --mode wildcard --difficulty 1000000
   ```

2. **Check Miner Status:**
   ```bash
   # Verify miners are active
   cloak-miner status --keypair ~/.config/solana/miner-keypair.json
   ```

3. **Lower Difficulty:**
   ```bash
   # Use lower difficulty for faster mining
   cloak-miner mine --mode wildcard --difficulty 100000
   ```

4. **Fallback to Standard Withdraw:**
   ```javascript
   // Disable PoW preference
   const requestId = await submitWithdraw({
     // ... other fields
     pow_preference: 'none', // Use standard withdraw
   });
   ```

---

**Scenario: Claim Expired During Processing**
```
{
  "success": false,
  "error": {
    "code": "ClaimExpired",
    "message": "PoW claim expired before consumption"
  }
}
```

**Cause:** Claim expired between discovery and transaction submission

**Solutions:**
1. **Check Claim Expiry:**
   ```bash
   # Verify claim expiration
   solana account <claim_pda> --output json
   ```

2. **Use Fresh Claims:**
   ```bash
   # Mine new claims with longer expiry
   cloak-miner mine --mode wildcard --expiry-slots 2000
   ```

3. **Retry with New Claim:**
   ```javascript
   // Retry submission (will find new claim)
   const retryId = await submitWithdraw(request);
   ```

---

**Scenario: Claim Fully Consumed**
```
Program log: Error: ClaimFullyConsumed
```

**Cause:** Claim has reached maximum consumption limit

**Solutions:**
1. **Check Consumption Count:**
   ```bash
   # Verify consumption status
   solana account <claim_pda> --output json | jq '.consumption_count'
   ```

2. **Mine New Claims:**
   ```bash
   # Generate fresh claims
   cloak-miner mine --mode wildcard
   ```

3. **Increase Max Consumptions:**
   ```rust
   // In scramble-registry program
   pub const MAX_CONSUMPTIONS: u8 = 10; // Increase from default 5
   ```

---

**Scenario: Batch Hash Mismatch**
```
Program log: Error: BatchHashMismatch
```

**Cause:** Non-wildcard claim doesn't match job's batch hash

**Solutions:**
1. **Use Wildcard Claims:**
   ```bash
   # Mine wildcard claims (batch_hash = [0; 32])
   cloak-miner mine --mode wildcard
   ```

2. **Mine Specific Claims:**
   ```bash
   # Mine for specific batch hash
   cloak-miner mine --batch-hash <specific_hash>
   ```

3. **Verify Batch Hash:**
   ```javascript
   // Ensure batch hash matches
   const batchHash = computeBatchHash(recipients);
   console.log('Batch hash:', toHex(batchHash));
   ```

### Performance Optimization

**Mining Optimization:**
```rust
// Use multiple threads for mining
use rayon::prelude::*;

pub fn mine_parallel(&self, difficulty_target: u64) -> Result<ClaimPreimage> {
    let num_threads = num_cpus::get();
    let chunk_size = u128::MAX / num_threads as u128;
    
    (0..num_threads)
        .into_par_iter()
        .find_map_any(|thread_id| {
            let start_nonce = thread_id as u128 * chunk_size;
            let end_nonce = start_nonce + chunk_size;
            
            self.mine_range(start_nonce, end_nonce, difficulty_target)
        })
        .ok_or(Error::MiningFailed)
}
```

**Claim Caching:**
```rust
// Cache available claims to reduce RPC calls
pub struct ClaimCache {
    claims: HashMap<Pubkey, AvailableClaim>,
    last_update: Instant,
    ttl: Duration,
}

impl ClaimCache {
    pub async fn get_available_claim(&mut self, claim_finder: &ClaimFinder) -> Result<Option<AvailableClaim>> {
        if self.last_update.elapsed() > self.ttl {
            self.refresh(claim_finder).await?;
        }
        
        Ok(self.claims.values().find(|claim| claim.is_available()).cloned())
    }
}
```

## Security Considerations

### Claim Security

**Mining Security:**
- Use secure random number generation for nonces
- Verify difficulty targets are reasonable
- Implement rate limiting to prevent spam

**Consumption Security:**
- Enforce consumption limits per claim
- Implement claim expiration
- Verify batch hash matching (except wildcards)

### Economic Security

**Mining Economics:**
- Difficulty should scale with network usage
- Claim expiry prevents indefinite storage
- Consumption limits prevent single-claim abuse

**Relay Security:**
- Verify claim authenticity before consumption
- Implement claim selection algorithms
- Monitor for claim manipulation attempts

## Related Documentation

- **[Cloak Miner Package](../packages/cloak-miner.md)** - Mining implementation details
- **[Scramble Registry Program](../onchain/scramble-registry.md)** - On-chain claim management
- **[Relay Service](../offchain/relay.md)** - Claim discovery and consumption
- **[PoW Overview](../pow/overview.md)** - Proof-of-work system design
- **[Standard Withdraw](./withdraw.md)** - Non-PoW withdrawal workflow
