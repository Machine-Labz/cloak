---
title: Deposit Workflow
description: Complete guide to creating privacy-preserving deposits in the Cloak shield pool.
---

# Deposit Workflow

The deposit workflow allows users to fund the Cloak shield pool, creating a privacy-preserving commitment that can later be withdrawn without revealing the link to the original deposit. This guide covers the complete end-to-end process from note generation to on-chain confirmation.

## Overview

A deposit creates a **commitment** (a cryptographic hash binding together amount, randomness, and spending key) that is stored on-chain and indexed off-chain in a Merkle tree. The user retains the secret data needed to later prove ownership and withdraw the funds privately.

### Key Concepts

**Note:** A package of secret data that allows the owner to spend a deposit:
- `sk_spend` - Secret spending key (32 bytes)
- `r` - Randomness nonce (32 bytes)
- `amount` - Deposit amount in lamports (u64)
- `leaf_index` - Position in the Merkle tree (assigned after indexing)

**Commitment:** A cryptographic hash binding the note data:
```
C = BLAKE3(amount || r || pk_spend)
where pk_spend = BLAKE3(sk_spend)
```

**Shield Pool:** The on-chain program that:
- Accepts SOL deposits
- Emits commitment events for indexer
- Stores pool balance in a PDA
- Later validates withdraw proofs

**Indexer:** The off-chain service that:
- Monitors deposit events
- Builds Merkle tree of commitments
- Provides roots and proofs for withdrawals
- Stores encrypted note metadata

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Deposit Flow                         │
│                                                         │
│  Client              Shield Pool         Indexer        │
│  ┌────────┐         ┌──────────┐       ┌────────┐     │
│  │ 1. Gen │         │          │       │        │     │
│  │ Note   │         │          │       │        │     │
│  └───┬────┘         │          │       │        │     │
│      │              │          │       │        │     │
│      │ 2. Compute   │          │       │        │     │
│      │ commitment   │          │       │        │     │
│      ├──────────────┼─────────▶│       │        │     │
│      │ 3. Submit    │ 4. Emit  │       │        │     │
│      │ deposit tx   │ event    │       │        │     │
│      │              ├──────────┼──────▶│        │     │
│      │              │          │       │ 5. Add │     │
│      │              │          │       │ to tree│     │
│      │              │          │       └───┬────┘     │
│      │ 6. Query     │          │           │          │
│      │◀─────────────┼──────────┼───────────┘          │
│      │ confirmation │          │                      │
│      │              │          │                      │
│  ┌───▼────┐         │          │                      │
│  │ Store  │         │          │                      │
│  │ Note   │         │          │                      │
│  └────────┘         └──────────┘                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Step 1: Generate Note Secrets

The client generates the cryptographic material needed to create and later spend a commitment.

### Generate Secret Key

```javascript
// Generate random 32-byte spending key
const sk_spend = crypto.getRandomValues(new Uint8Array(32));

// Derive public spending key
const pk_spend = blake3.hash(sk_spend); // 32 bytes

// Generate randomness nonce
const r = crypto.getRandomValues(new Uint8Array(32));

// Store these securely - they're needed for withdrawals
const note = {
  sk_spend: toHex(sk_spend),
  r: toHex(r),
  amount: 1_000_000, // 0.001 SOL
  created_at: new Date().toISOString(),
};

// Save to encrypted storage
saveNoteToWallet(note);
```

### Security Considerations

**Critical:** If you lose `sk_spend` or `r`, the deposit becomes unspendable. The funds will remain locked in the pool forever.

**Best Practices:**
- Store notes in encrypted local storage (browser) or secure enclave (mobile)
- Backup notes to encrypted cloud storage with user password
- Consider multi-sig schemes for high-value deposits
- Never transmit `sk_spend` over network (even encrypted)
- Use secure random number generation (crypto.getRandomValues, not Math.random)

**Example Storage Schema:**
```typescript
interface SecureNoteStorage {
  version: string;
  notes: Array<{
    commitment: string;      // Public commitment hash (for lookup)
    sk_spend: string;        // Encrypted with user password
    r: string;               // Encrypted with user password
    amount: number;
    leaf_index: number | null; // Assigned after indexing
    created_at: string;
    spent: boolean;
  }>;
}
```

## Step 2: Compute Commitment

The commitment is a cryptographic hash that binds the amount, randomness, and spending key together.

### Commitment Computation

```rust
// Rust implementation (matches on-chain verification)
fn compute_commitment(amount: u64, r: &[u8; 32], pk_spend: &[u8; 32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();

    // Concatenate: amount (8 bytes LE) || r (32 bytes) || pk_spend (32 bytes)
    hasher.update(&amount.to_le_bytes());
    hasher.update(r);
    hasher.update(pk_spend);

    // Return 32-byte hash
    *hasher.finalize().as_bytes()
}
```

**JavaScript Implementation:**
```javascript
import { blake3 } from '@noble/hashes/blake3';

function computeCommitment(amount, r, pk_spend) {
  // Serialize amount as 8-byte little-endian
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amount), true);

  // Concatenate all inputs
  const preimage = new Uint8Array([
    ...amountBytes,    // 8 bytes
    ...r,              // 32 bytes
    ...pk_spend,       // 32 bytes
  ]);

  // Hash with BLAKE3
  return blake3(preimage); // Returns 32 bytes
}

// Example
const amount = 1_000_000; // 0.001 SOL
const commitment = computeCommitment(amount, r, pk_spend);
console.log('Commitment:', toHex(commitment));
```

**Encoding Rules:**
- `amount`: u64 little-endian (8 bytes)
- `r`: Raw bytes (32 bytes)
- `pk_spend`: Raw bytes (32 bytes)
- **Total preimage:** 72 bytes
- **Output:** BLAKE3-256 hash (32 bytes)

**Reference:** `docs/zk/encoding.md`

## Step 3: Encrypt Output Payload (Optional)

For wallet discovery and note scanning, encrypt metadata about the deposit.

### Encrypted Output Format

```javascript
// Create encrypted output for indexer storage
const encryptedOutput = {
  version: 1,
  recipient: toBase58(publicKey),
  amount: 1_000_000,
  metadata: {
    created_at: new Date().toISOString(),
    memo: 'Test deposit',
  },
};

// Encrypt with shared secret or symmetric key
const encrypted = await encryptForRecipient(
  JSON.stringify(encryptedOutput),
  recipientPublicKey
);

// Convert to base64 for transmission
const enc_output = btoa(String.fromCharCode(...encrypted));
```

**Note Scanning:**
Wallets can query `GET /api/v1/notes/range` from the indexer and attempt to decrypt each note's `encrypted_output` field. Successfully decrypted notes belong to that wallet.

**Privacy Consideration:**
The encrypted output is publicly visible on-chain and in the indexer database. Ensure encryption is strong and does not leak metadata.

**Reference:** `services/indexer/src/server/final_handlers.rs`

## Step 4: Submit On-Chain Transaction

Build and submit a Solana transaction containing the deposit instruction.

### Transaction Structure

```typescript
import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
  TransactionInstruction
} from '@solana/web3.js';

async function submitDeposit(
  connection: Connection,
  userKeypair: Keypair,
  commitment: Uint8Array,
  encryptedOutput: Uint8Array,
  amount: number
) {
  const POOL_PDA = derivePoolPda(SHIELD_POOL_PROGRAM_ID);

  // 1. Transfer SOL to pool (covers amount + fee)
  const transferIx = SystemProgram.transfer({
    fromPubkey: userKeypair.publicKey,
    toPubkey: POOL_PDA,
    lamports: amount + DEPOSIT_FEE, // Add fee for indexer/relay
  });

  // 2. Deposit instruction
  const depositIx = new TransactionInstruction({
    programId: SHIELD_POOL_PROGRAM_ID,
    keys: [
      { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: POOL_PDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: serializeDepositInstruction({
      leaf_commit: commitment,       // 32 bytes
      enc_output: encryptedOutput,   // Variable length, base64 encoded
    }),
  });

  // Build transaction
  const tx = new Transaction()
    .add(transferIx)
    .add(depositIx);

  // Sign and send
  const signature = await connection.sendTransaction(tx, [userKeypair]);

  // Wait for confirmation
  await connection.confirmTransaction(signature, 'confirmed');

  return signature;
}
```

### Instruction Encoding

**Deposit Instruction Layout:**
```
[Discriminator: 1 byte]     // Instruction index (e.g., 0 for deposit)
[leaf_commit: 32 bytes]     // Commitment hash
[enc_output_len: 4 bytes]   // Length prefix (u32 LE)
[enc_output: N bytes]       // Encrypted output payload
```

**Example (Anchor):**
```rust
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositArgs {
    pub leaf_commit: [u8; 32],
    pub enc_output: Vec<u8>,
}
```

**Reference:** `programs/shield-pool/src/lib.rs:deposit()`

### On-Chain Processing

The shield-pool program:
1. **Validates transfer:** Checks that `amount + fee` lamports arrived
2. **Stores pool balance:** Increments total pool balance
3. **Emits event:** Logs `deposit_commit:{commitment_hex}` for indexer
4. **No Merkle updates:** Tree is maintained off-chain by indexer

**Event Format:**
```
Program log: deposit_commit:a1b2c3d4e5f6789...
```

The indexer monitors these logs to discover new deposits.

**Reference:** `programs/shield-pool/src/instructions/deposit.rs`

## Step 5: Indexer Ingestion

The indexer service monitors the chain for deposit events and maintains the Merkle tree.

### Event Monitoring

The indexer uses one of two strategies:

**WebSocket Subscription (Real-time):**
```rust
// In indexer service
let ws_client = PubsubClient::new(&ws_url).await?;

// Subscribe to program logs
let (mut logs_stream, unsub) = ws_client
    .logs_subscribe(
        RpcTransactionLogsFilter::Mentions(vec![shield_pool_program_id]),
        Some(RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        }),
    )
    .await?;

// Process logs
while let Some(log_result) = logs_stream.next().await {
    if let Ok(log) = log_result {
        process_deposit_log(log).await?;
    }
}
```

**Polling (Fallback):**
```rust
// Poll every 1 second for new signatures
loop {
    let signatures = rpc_client
        .get_signatures_for_address(&shield_pool_program_id)
        .await?;

    for sig_info in signatures {
        if !seen_signatures.contains(&sig_info.signature) {
            process_transaction(&sig_info.signature).await?;
            seen_signatures.insert(sig_info.signature);
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

**Reference:** `services/indexer/src/ingest.rs`

### Merkle Tree Updates

When a deposit event is detected:

```rust
// 1. Parse commitment from log
let commitment = parse_commitment_from_log(&log)?;

// 2. Insert into database
sqlx::query!(
    "INSERT INTO notes (leaf_commit, encrypted_output, tx_signature, slot, created_at)
     VALUES ($1, $2, $3, $4, NOW())",
    &commitment[..],
    encrypted_output,
    tx_signature,
    slot
).execute(&db).await?;

// 3. Append to Merkle tree
let leaf_index = merkle_tree.append_leaf(&commitment)?;

// 4. Get new root
let new_root = merkle_tree.root();

// 5. Store root in ring buffer (last 64 roots kept)
store_root_in_buffer(&new_root, slot).await?;

info!(
    "Inserted leaf {} with commitment {}",
    leaf_index,
    hex::encode(commitment)
);
```

**Merkle Tree Structure:**
- **Height:** 32 levels (supports 2^32 = 4 billion deposits)
- **Hash Function:** BLAKE3-256
- **Parent Node:** `H(left_child || right_child)`
- **Zero Values:** Each empty level has a deterministic zero hash
- **Storage:** PostgreSQL stores all nodes and sibling paths

**Reference:** `services/indexer/src/merkle.rs`

## Step 6: Client Confirmation

After submitting the deposit, the client should confirm it was indexed correctly.

### Confirm Transaction

```javascript
// 1. Wait for transaction confirmation
const confirmation = await connection.confirmTransaction(
  signature,
  'finalized' // Wait for finalized commitment
);

if (confirmation.value.err) {
  throw new Error(`Deposit failed: ${confirmation.value.err}`);
}

console.log('✅ Transaction confirmed in slot', confirmation.context.slot);
```

### Query Indexer

```javascript
// 2. Query current Merkle root
const rootResponse = await fetch('http://localhost:3001/api/v1/merkle/root');
const { root, nextIndex } = await rootResponse.json();

// Your leaf index is nextIndex - 1 (if you just deposited)
const leafIndex = nextIndex - 1;

console.log('📊 Current Merkle root:', root);
console.log('📍 Your leaf index:', leafIndex);
```

### Verify Note Indexed

```javascript
// 3. Query notes to find your commitment
const notesResponse = await fetch(
  `http://localhost:3001/api/v1/notes/range?start=${leafIndex}&limit=1`
);
const { notes } = await notesResponse.json();

// Verify commitment matches
const indexedNote = notes[0];
if (indexedNote.leaf_commit !== toHex(commitment)) {
  throw new Error('Commitment mismatch - indexer may be out of sync');
}

console.log('✅ Note confirmed in indexer');
console.log('📄 Encrypted output:', indexedNote.encrypted_output);
```

### Update Note Storage

```javascript
// 4. Update stored note with leaf index
note.leaf_index = leafIndex;
note.indexed_at = new Date().toISOString();
note.merkle_root = root;

// Save updated note
updateNoteInWallet(note);

console.log('✅ Deposit complete! Note ready for future withdrawal.');
```

**Reference:** `services/indexer/README.md`, `docs/api/indexer.md`

## Complete Example

Here's a full end-to-end example in TypeScript:

```typescript
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { blake3 } from '@noble/hashes/blake3';

async function depositToShieldPool(
  connection: Connection,
  userKeypair: Keypair,
  amount: number
): Promise<DepositReceipt> {
  console.log('🔐 Generating note secrets...');

  // 1. Generate note
  const sk_spend = crypto.getRandomValues(new Uint8Array(32));
  const pk_spend = blake3(sk_spend);
  const r = crypto.getRandomValues(new Uint8Array(32));

  console.log('✅ Secrets generated');

  // 2. Compute commitment
  const commitment = computeCommitment(amount, r, pk_spend);
  console.log('📝 Commitment:', toHex(commitment));

  // 3. Encrypt output (optional)
  const encryptedOutput = await encryptNoteMetadata({
    recipient: userKeypair.publicKey.toBase58(),
    amount,
    created_at: new Date().toISOString(),
  });

  // 4. Submit transaction
  console.log('📤 Submitting deposit transaction...');
  const signature = await submitDeposit(
    connection,
    userKeypair,
    commitment,
    encryptedOutput,
    amount
  );

  console.log('✅ Transaction submitted:', signature);

  // 5. Wait for indexer
  console.log('⏳ Waiting for indexer...');
  await waitForIndexer(commitment, 30_000); // 30 second timeout

  // 6. Get leaf index
  const { root, nextIndex } = await queryMerkleRoot();
  const leafIndex = nextIndex - 1;

  console.log('✅ Deposit indexed at position', leafIndex);

  // 7. Save note
  const note = {
    commitment: toHex(commitment),
    sk_spend: toHex(sk_spend),
    r: toHex(r),
    amount,
    leaf_index: leafIndex,
    merkle_root: root,
    tx_signature: signature,
    created_at: new Date().toISOString(),
    spent: false,
  };

  await saveNote(note);

  return {
    signature,
    commitment: toHex(commitment),
    leaf_index: leafIndex,
    amount,
  };
}

// Helper: Wait for commitment to appear in indexer
async function waitForIndexer(
  commitment: Uint8Array,
  timeout: number
): Promise<void> {
  const start = Date.now();
  const commitmentHex = toHex(commitment);

  while (Date.now() - start < timeout) {
    try {
      // Query recent notes
      const notes = await queryNotesRange(0, 100);

      // Check if our commitment is there
      const found = notes.some(n => n.leaf_commit === commitmentHex);

      if (found) {
        return; // Success!
      }
    } catch (err) {
      console.warn('Indexer query failed, retrying...', err);
    }

    // Wait 1 second before retry
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  throw new Error('Indexer timeout - deposit may not be indexed yet');
}
```

## Failure & Retry Considerations

### Transaction Failures

**Scenario: Transaction Rejected**
```
Error: Transaction simulation failed: InsufficientFunds
```

**Cause:** User doesn't have enough SOL for deposit + fee + transaction fee

**Solution:**
- Check balance before deposit: `connection.getBalance(userPubkey)`
- Ensure user has `amount + DEPOSIT_FEE + 0.005 SOL` (5000 lamports for tx fee)
- Request airdrop on devnet or display error to user

---

**Scenario: Transaction Timeout**
```
Error: Transaction was not confirmed in 60 seconds
```

**Cause:** Network congestion or RPC node issues

**Solution:**
- Check transaction status manually: `connection.getSignatureStatus(signature)`
- Transaction may have succeeded but confirmation was delayed
- Query indexer to see if deposit was processed
- Only retry if transaction definitely failed (not timeout)

---

**Scenario: Commitment Already Exists**
```
Error: Custom program error: 0x1 (CommitmentAlreadyExists)
```

**Cause:** Same commitment submitted twice (same `sk_spend`, `r`, and `amount`)

**Solution:**
- This is actually OK if it's the same deposit attempt
- Deposit instruction is idempotent with same commitment
- If intentional duplicate, generate new `r` to create different commitment

### Indexer Lag

**Scenario: Deposit Not Appearing**

**Symptoms:**
- Transaction confirmed on Solana
- Commitment not found in indexer after 30 seconds
- `GET /api/v1/notes/range` doesn't include the commitment

**Diagnosis:**
```bash
# Check indexer logs
docker logs cloak-indexer -f | grep "deposit_commit"

# Check indexer health
curl http://localhost:3001/health

# Check if indexer is syncing
curl http://localhost:3001/api/v1/merkle/root
# Compare nextIndex with blockchain state
```

**Solutions:**
1. **WebSocket Disconnected:**
   - Indexer logs show WebSocket errors
   - Restart indexer service
   - Check Solana RPC node health

2. **Processing Backlog:**
   - Indexer is behind by multiple blocks
   - Wait longer (up to 5 minutes on mainnet)
   - Check CPU/memory usage of indexer

3. **Database Issues:**
   - Indexer can't write to PostgreSQL
   - Check PostgreSQL logs and connection
   - Verify disk space available

### Client Storage Loss

**Scenario: Lost Note Secrets**

**Impact:** **Permanent loss of funds.** Without `sk_spend` and `r`, the commitment cannot be spent.

**Prevention:**
- Implement encrypted backups (to cloud or export file)
- Warn users during onboarding about importance of backups
- Consider social recovery mechanisms
- Use hardware wallets for high-value deposits

**Note Recovery:**
If you have the commitment hash but lost the secrets:
- Funds are still in the pool
- No one else can spend them either
- They remain locked forever (privacy guarantee = unrecoverability without secrets)

## Next Steps

After successfully depositing, you can:

1. **Wait for Confirmations:**
   - Wait for more blocks to ensure finality
   - Recommended: 32 blocks on mainnet (~16 seconds)
   - Use commitment level `finalized` for maximum security

2. **Plan Withdrawals:**
   - Study the [Withdraw Workflow](./withdraw.md)
   - Generate ZK proof with `zk-guest-sp1` package
   - Submit to relay service

3. **Monitor Your Notes:**
   - Query indexer periodically to track all your notes
   - Build a local wallet UI showing note balances
   - Check for spent notes (nullifiers)

## Security Considerations

**Commitment Uniqueness:**
- Each commitment should be unique (no duplicates)
- Use fresh random `r` for each deposit
- Don't reuse `sk_spend` across different deposits
- Collision probability: ~2^-256 (astronomically small)

**Amount Privacy:**
- On-chain deposit shows exact amount (not private)
- Privacy comes from breaking the link during withdrawal
- Consider depositing round amounts for better anonymity set
- E.g., 0.1 SOL, 0.5 SOL, 1 SOL, 10 SOL

**Timing Analysis:**
- Depositing and withdrawing immediately reveals link
- Wait random delay between deposit and withdrawal
- Use multiple deposits and withdrawals to mix funds
- Consider coordinated multi-user withdrawals

**Metadata Leakage:**
- Encrypted outputs should not leak info via size
- Pad encrypted payloads to fixed size
- Don't include identifying information in metadata
- Use shared encryption schemes (Diffie-Hellman, ECIES)

## Related Documentation

- **[Withdraw Workflow](./withdraw.md)** - How to spend deposited notes
- **[Shield Pool Program](../onchain/shield-pool.md)** - On-chain deposit logic
- **[Indexer Service](../offchain/indexer.md)** - Off-chain Merkle tree maintenance
- **[Indexer API](../api/indexer.md)** - API endpoints for querying notes and proofs
- **[ZK Encoding](../zk/encoding.md)** - Cryptographic encoding specifications
- **[System Architecture](../overview/system-architecture.md)** - Full system overview
