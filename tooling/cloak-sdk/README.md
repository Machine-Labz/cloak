# Cloak SDK

TypeScript SDK for the Cloak Protocol - Private transactions on Solana using zero-knowledge proofs.

## Features

- 🔒 **Private Transfers**: Send SOL privately using zero-knowledge proofs
- 👥 **Multi-Recipient**: Support for 1-5 recipients in a single transaction
- 🔐 **Type-Safe**: Full TypeScript support with comprehensive types
- 🌐 **Cross-Platform**: Works in browser and Node.js
- 📦 **Zero Dependencies**: Minimal external dependencies (only Solana + crypto)
- ⚡ **Simple API**: Easy-to-use high-level client

## Installation

```bash
npm install @cloak/sdk @solana/web3.js
# or
yarn add @cloak/sdk @solana/web3.js
# or
pnpm add @cloak/sdk @solana/web3.js
```

## Quick Start

```typescript
import { CloakSDK } from "@cloak/sdk";
import { Connection, PublicKey } from "@solana/web3.js";

// Initialize client
const client = new CloakSDK({
  network: "devnet",
  programId: new PublicKey("YOUR_PROGRAM_ID"),
  poolAddress: new PublicKey("YOUR_POOL_ADDRESS"),
  commitmentsAddress: new PublicKey("YOUR_COMMITMENTS_ADDRESS"),
  // Option A: Single API URL (recommended when reverse proxying indexer+relay)
  apiUrl: "https://api.your-cloak.example.com",
  // Option B: Separate services (if not using a single URL)
  // indexerUrl: "https://your-indexer.com",
  // relayUrl: "https://your-relay.com",
});

// Connect to Solana
const connection = new Connection("https://api.devnet.solana.com");

// ==================================================================
// OPTION 1: Deposit only (save note for later)
// ==================================================================
const depositResult = await client.deposit(
  connection,
  wallet, // Wallet with sendTransaction method
  1_000_000_000, // 1 SOL in lamports
  {
    onProgress: (status) => console.log(status),
  }
);
console.log("Note saved:", depositResult.note);
// Save this note securely! You can withdraw later.

// ==================================================================
// OPTION 2: Private Transfer (complete flow: deposit + withdraw)
// ==================================================================
// This is the main use case - privately send funds to recipients!

// Generate a note (not deposited yet)
const note = client.generateNote(1_000_000_000); // 1 SOL

// privateTransfer handles EVERYTHING:
// - Deposits the note if not already deposited
// - Waits for confirmation
// - Generates ZK proof
// - Transfers to recipients
const transferResult = await client.privateTransfer(
  connection,
  wallet,
  note,
  [
    { recipient: new PublicKey("ADDR1"), amount: 500_000_000 },
    { recipient: new PublicKey("ADDR2"), amount: 492_500_000 }, // After fees
  ],
  {
    relayFeeBps: 50, // 0.5% relay fee
    onProgress: (status) => console.log(status),
    onProofProgress: (progress) => console.log(`Proof: ${progress}%`),
  }
);

console.log("Transfer complete:", transferResult.signature);
```

Note on API configuration:
- Provide a single `apiUrl` when your deployment proxies both indexer and relay behind the same origin (recommended).
- If `apiUrl` is not provided, you must provide both `indexerUrl` and `relayUrl`.

## Core Concepts

### Notes

A **Cloak Note** is a cryptographic commitment representing a private amount of SOL. It contains:

- `commitment`: Blake3 hash that commits to the amount and secrets
- `sk_spend`: Spending secret key (keep this safe!)
- `r`: Randomness value
- `amount`: Amount in lamports
- Metadata: signature, leafIndex, merkleProof (after deposit)

**Important**: Treat notes like cash - anyone with the note can withdraw the funds!

### Private Transfer Flow

The `privateTransfer` method handles the **complete flow automatically**:

1. **Check**: Is the note already deposited?
2. **Deposit** (if needed): Deposit SOL and create commitment
3. **Wait**: Transaction confirms and indexer processes it
4. **Prove**: Generate zero-knowledge proof
5. **Transfer**: Submit to relay and send funds to recipients
6. **Complete**: Funds are sent to recipients privately

The proof ensures that:
- You own the note (know the spending key)
- The note exists in the Merkle tree
- You haven't spent it before (nullifier check)
- Outputs sum correctly (amount conservation)

All without revealing which note you're spending!

### Usage Patterns

**Pattern 1: Deposit now, withdraw later**
```typescript
// Save funds privately
const result = await client.deposit(connection, wallet, 1_000_000_000);
saveNoteSecurely(result.note); // Store for later

// Later: withdraw using the saved note
const withdrawResult = await client.withdraw(
  connection, wallet, loadNote(), recipientAddress
);
```

**Pattern 2: Private transfer (deposit + immediate withdraw)**
```typescript
// Send funds privately to recipients in one call
const note = client.generateNote(1_000_000_000);
const result = await client.privateTransfer(
  connection,
  wallet,
  note,
  [
    { recipient: addr1, amount: 500_000_000 },
    { recipient: addr2, amount: 492_500_000 }
  ]
);
// Everything handled automatically!
```

## API Reference

### CloakSDK

Main client for interacting with Cloak Protocol.

#### Constructor

```typescript
new CloakSDK(config: CloakConfig)
```

#### Methods

##### `deposit()`

Deposit SOL into the protocol and create a private note.

```typescript
async deposit(
  connection: Connection,
  payer: Wallet,
  amountLamports: number,
  options?: DepositOptions
): Promise<DepositResult>
```

**Example:**
```typescript
const result = await client.deposit(connection, wallet, 1_000_000_000);
console.log("Leaf index:", result.leafIndex);
// Save result.note securely!
```

##### `privateTransfer()`

Execute a complete private transfer with 1-5 recipients.

**Handles the full flow**: If the note isn't deposited yet, it deposits it first, waits for confirmation, then proceeds with the withdrawal.

```typescript
async privateTransfer(
  connection: Connection,
  payer: Wallet,
  note: CloakNote,
  recipients: MaxLengthArray<Transfer, 5>,
  options?: TransferOptions
): Promise<TransferResult>
```

**Type-safe recipient array** (1-5 elements):
```typescript
type Transfer = { recipient: PublicKey; amount: number };
type MaxLengthArray<T, Max extends number, A extends T[] = []> =
  A['length'] extends Max ? A : A | MaxLengthArray<T, Max, [T, ...A]>;
```

**Example:**
```typescript
// Generate note (not deposited)
const note = client.generateNote(1_000_000_000);

// privateTransfer handles deposit + withdrawal
const result = await client.privateTransfer(
  connection,
  wallet,
  note,
  [
    { recipient: addr1, amount: 500_000_000 },
    { recipient: addr2, amount: 400_000_000 },
    { recipient: addr3, amount: 92_500_000 }, // After protocol fees
  ],
  { relayFeeBps: 50 } // 0.5% relay fee
);
```

##### `withdraw()`

Convenience method for withdrawing to a single recipient.

**Handles the full flow**: Deposits if needed, then withdraws to one recipient.

```typescript
async withdraw(
  connection: Connection,
  payer: Wallet,
  note: CloakNote,
  recipient: PublicKey,
  options?: WithdrawOptions
): Promise<TransferResult>
```

**Example:**
```typescript
const note = client.generateNote(1_000_000_000);
const result = await client.withdraw(
  connection,
  wallet,
  note,
  recipientAddress,
  { withdrawAll: true } // Withdraw full amount minus fees
);
```

##### `generateNote()`

Generate a new note without depositing (for testing or pre-generation).

```typescript
generateNote(amountLamports: number): CloakNote
```

##### `parseNote()`

Parse a note from JSON string.

```typescript
parseNote(jsonString: string): CloakNote
```

##### `exportNote()`

Export a note to JSON string.

```typescript
exportNote(note: CloakNote, pretty?: boolean): string
```

### Fee Calculation

```typescript
import { calculateFee, getDistributableAmount, formatAmount } from "@cloak/sdk";

const amount = 1_000_000_000; // 1 SOL
const fee = calculateFee(amount); // Protocol fee
const distributable = getDistributableAmount(amount); // Amount after fees

console.log(`Fee: ${formatAmount(fee)} SOL`);
console.log(`Distributable: ${formatAmount(distributable)} SOL`);
```

**Fee Structure:**
- Fixed: 0.0025 SOL (2.5M lamports)
- Variable: 0.5% of amount
- Total: `FIXED + floor(amount * 0.005)`

### Crypto Utilities

```typescript
import {
  generateCommitment,
  computeNullifier,
  computeOutputsHash,
  hexToBytes,
  bytesToHex,
} from "@cloak/sdk";

// Generate commitment
const commitment = generateCommitment(amount, r, skSpend);

// Compute nullifier (prevents double-spending)
const nullifier = computeNullifier(skSpend, leafIndex);

// Compute outputs hash
const outputsHash = computeOutputsHash([
  { recipient: addr1, amount: amount1 },
  { recipient: addr2, amount: amount2 },
]);
```

## Advanced Usage

### Custom Storage

The SDK doesn't handle persistence - you control where notes are stored:

```typescript
// Browser localStorage
const noteJson = client.exportNote(note);
localStorage.setItem(`note-${note.commitment}`, noteJson);

// File system (Node.js)
import fs from "fs";
fs.writeFileSync(`note-${note.commitment}.json`, client.exportNote(note, true));

// Database
await db.notes.create({ commitment: note.commitment, data: client.exportNote(note) });
```

### Progress Tracking

```typescript
await client.privateTransfer(
  note,
  recipients,
  {
    onProgress: (status) => {
      console.log(status);
      // "Fetching Merkle proof..."
      // "Computing cryptographic values..."
      // "Generating zero-knowledge proof..."
      // "Submitting to relay service..."
    },
    onProofProgress: (progress) => {
      console.log(`Proof generation: ${progress}%`);
      // Update progress bar, etc.
    },
  }
);
```

### Direct Service Access

Use service clients directly for lower-level operations:

```typescript
import { IndexerService, ProverService, RelayService } from "@cloak/sdk";

const indexer = new IndexerService("https://indexer.example.com");
const prover = new ProverService("https://prover.example.com");
const relay = new RelayService("https://relay.example.com");

// Get Merkle root
const { root, next_index } = await indexer.getMerkleRoot();

// Get Merkle proof
const proof = await indexer.getMerkleProof(leafIndex);

// Generate proof
const proofResult = await prover.generateProof(inputs);

// Submit withdrawal
const signature = await relay.submitWithdraw(params);
```

## TypeScript Types

The SDK is fully typed. Import types as needed:

```typescript
import type {
  CloakNote,
  CloakConfig,
  Transfer,
  TransferResult,
  DepositResult,
  MerkleProof,
} from "@cloak/sdk";
```

## Security Considerations

⚠️ **Important Security Notes:**

1. **Keep notes secure**: Treat them like private keys. Anyone with the note can withdraw funds.

2. **Backup notes**: If you lose a note, the funds are unrecoverable.

3. **Verify recipients**: Double-check recipient addresses before transfer.

4. **Amount validation**: Always validate that outputs sum to expected amounts.

5. **Network isolation**: Don't mix notes from different networks (devnet/mainnet).

6. **Proof privacy**: This SDK sends private inputs to a backend prover. For full privacy in production, consider client-side proof generation.

## Error Handling

```typescript
try {
  const result = await client.privateTransfer(note, recipients);
  console.log("Success:", result.signature);
} catch (error) {
  if (error.message.includes("Note must be deposited")) {
    console.error("Note not yet deposited");
  } else if (error.message.includes("Proof generation failed")) {
    console.error("ZK proof generation failed");
  } else if (error.message.includes("Relay")) {
    console.error("Relay service error");
  } else {
    console.error("Unknown error:", error);
  }
}
```

## Examples

See the [examples](./examples) directory for complete working examples:

- Basic deposit and withdrawal
- Multi-recipient transfers
- Note management and storage
- Error handling
- Progress tracking

## Contributing

Contributions are welcome! Please open an issue or PR.

## License

MIT

## Support

For questions and support:
- GitHub Issues: [github.com/cloak/sdk/issues](https://github.com/cloak/sdk/issues)
- Documentation: [docs.cloak.xyz](https://docs.cloak.xyz)
