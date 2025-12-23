# Unstake Implementation - TODO

## ⚠️ CRITICAL ISSUE: Missing Stake Authority Signature

### Problem

The current unstake implementation is **incomplete** and will **fail on-chain** because it's missing the `stake_authority` signature.

**Why it fails:**
- Solana's Stake program requires the `stake_authority` to sign any transaction that withdraws funds from a stake account
- The relay currently only signs with its `fee_payer` keypair
- Attempting to submit a transaction without the `stake_authority` signature will result in: `Error: Transaction signature verification failure`

### Current Flow (BROKEN)

```
1. Frontend generates ZK proof ✅
2. Frontend sends proof + config to relay ✅  
3. Relay creates UnstakeToPool transaction ✅
4. Relay signs ONLY with fee_payer ❌ INCOMPLETE
5. Relay submits transaction → FAILS ❌
```

### Required Flow (2-Phase Signing)

Similar to the Stake flow (WithdrawStake + Delegate), unstake needs **two signatures**:

```
Phase 1 - Frontend:
1. Generate ZK proof ✅
2. Create UnstakeToPool transaction
3. User signs transaction as stake_authority 🔐
4. Send partially signed transaction to relay

Phase 2 - Relay:
1. Receive partially signed transaction
2. Add fee_payer signature  
3. Submit complete transaction to blockchain ✅
```

### Implementation Tasks

#### 1. **Frontend Changes** (`services/web/app/privacy/page.tsx`)

After proof generation, before calling relay:

```typescript
// Build UnstakeToPool transaction
const unstakeTx = await buildUnstakeToPoolTransaction({
  proof: proofBytes,
  publicInputs: public104,
  stakeAccount: stakeAccountPubkey,
  programId,
  poolPda,
  rootsRingPda,
});

// User signs as stake_authority
const partiallySignedTx = await sendTransaction(unstakeTx, connection, {
  skipPreflight: true,
  preflightCommitment: "confirmed",
});

// Serialize and send to relay
const serializedTx = unstakeTx.serialize({
  requireAllSignatures: false,
  verifySignatures: false,
}).toString("base64");
```

#### 2. **Relay API Changes** (`services/relay/src/api/unstake.rs`)

Already supports `partially_signed_tx` field:

```rust
pub struct UnstakeRequest {
    pub proof: String,
    pub public_inputs: UnstakePublicInputs,
    pub unstake: UnstakeConfig,
    pub partially_signed_tx: Option<String>, // ✅ Added
}
```

#### 3. **Relay Worker Changes** (`services/relay/src/solana/mod.rs`)

Update `submit_unstake_to_pool` to:

```rust
async fn submit_unstake_to_pool(&self, job: &Job, unstake_config: &UnstakeConfig) -> Result<Signature, Error> {
    // Check if job has partially_signed_tx in outputs_json
    if let Some(partially_signed_tx_b64) = job.outputs_json.get("partially_signed_tx") {
        // Deserialize transaction
        let tx_bytes = base64::decode(partially_signed_tx_b64)?;
        let mut transaction = Transaction::deserialize(&tx_bytes)?;
        
        // Add relay's signature as fee_payer
        transaction.partial_sign(&[self.fee_payer], recent_blockhash);
        
        // Submit with both signatures
        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        return Ok(signature);
    }
    
    // Fallback: old flow (will fail)
    return Err(Error::ValidationError("Missing stake_authority signature".to_string()));
}
```

#### 4. **Helper Function** (`services/web/lib/solana-tx-builder.ts`)

```typescript
export async function buildUnstakeToPoolTransaction(params: {
  proof: Uint8Array;
  publicInputs: Uint8Array;
  stakeAccount: PublicKey;
  stakeAuthority: PublicKey;
  programId: PublicKey;
  poolPda: PublicKey;
  rootsRingPda: PublicKey;
  feePayer: PublicKey;
  recentBlockhash: string;
}): Promise<Transaction> {
  // Build instruction data: [proof (260)] [public_inputs (104)] [stake_account (32)]
  const data = Buffer.concat([
    params.proof,
    params.publicInputs,
    params.stakeAccount.toBuffer(),
  ]);
  
  // Build UnstakeToPool instruction
  const instruction = new TransactionInstruction({
    programId: params.programId,
    keys: [
      // ... account metas ...
    ],
    data,
  });
  
  // Build transaction
  const tx = new Transaction();
  tx.add(instruction);
  tx.feePayer = params.feePayer;
  tx.recentBlockhash = params.recentBlockhash;
  
  return tx;
}
```

### Alternative: Simpler Approach

Use the existing `/jobs/withdraw` + `/submit` flow that already supports 2-phase signing:

1. POST `/jobs/withdraw` with unstake config → returns `job_id` + unsigned transaction
2. Frontend gets user to sign
3. POST `/submit` with `job_id` + signed transaction

### Testing

Once implemented, test:

1. ✅ ZK proof generation
2. ✅ Transaction creation
3. ✅ User signature (stake_authority)
4. ✅ Relay signature (fee_payer)
5. ✅ On-chain submission
6. ✅ Transaction confirmed in Solana Explorer
7. ✅ Funds moved from stake account to shield pool
8. ✅ New commitment added to Merkle tree

### References

- Stake flow implementation: `handleStake()` in `services/web/app/privacy/page.tsx`
- Solana Stake program: https://docs.solana.com/developing/runtime-facilities/programs#stake-program
- Transaction building: `services/relay/src/solana/transaction_builder.rs`

---

**Status:** 🚧 In Progress  
**Priority:** 🔴 Critical  
**Blocker:** Yes - unstake currently fails on-chain without stake_authority signature

