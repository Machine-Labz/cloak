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
npm install @cloaklabz/sdk @solana/web3.js
# or
yarn add @cloaklabz/sdk @solana/web3.js
# or
pnpm add @cloaklabz/sdk @solana/web3.js
```

**Note**: For swap functionality, you'll also need `@solana/spl-token`:
```bash
npm install @solana/spl-token
```

## Quick Start

```typescript
import { CloakSDK, generateNote } from "@cloaklabz/sdk";
import { Connection, Keypair } from "@solana/web3.js";

// Initialize connection and keypair
const connection = new Connection("https://api.devnet.solana.com");
const keypair = Keypair.fromSecretKey(/* your secret key */);

// Initialize client - only need network and keypair!
const client = new CloakSDK({
  keypairBytes: keypair.secretKey,
  network: "devnet", // Optional, defaults to "devnet"
});

// ==================================================================
// OPTION 1: Deposit only (save note for later)
// ==================================================================
const depositResult = await client.deposit(
  connection,
  keypair,
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
// Generate a note (not deposited yet)
const note = generateNote(1_000_000_000); // 1 SOL

// privateTransfer handles EVERYTHING:
// - Deposits the note if not already deposited
// - Waits for confirmation
// - Generates ZK proof using artifact-based flow (private inputs never pass through backend)
// - Transfers to recipients
const transferResult = await client.privateTransfer(
  connection,
  note,
  [
    { recipient: new PublicKey("ADDR1"), amount: 500_000_000 },
    { recipient: new PublicKey("ADDR2"), amount: 492_500_000 }, // After fees
  ],
  {
    onProgress: (status) => console.log(status),
    onProofProgress: (progress) => console.log(`Proof: ${progress}%`),
  }
);

console.log("Transfer complete:", transferResult.signature);
```

**API Configuration:**
- The SDK uses environment variables or defaults for API URLs
- Set `CLOAK_API_URL` environment variable to override the default
- Defaults to `http://localhost` for local development

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
import { generateNote } from "@cloaklabz/sdk";

// Save funds privately
const result = await client.deposit(connection, keypair, 1_000_000_000);
saveNoteSecurely(result.note); // Store for later

// Later: withdraw using the saved note
const withdrawResult = await client.withdraw(
  connection, loadNote(), recipientAddress
);
```

**Pattern 2: Private transfer (deposit + immediate withdraw)**
```typescript
import { generateNote } from "@cloaklabz/sdk";

// Send funds privately to recipients in one call
const note = generateNote(1_000_000_000);
const result = await client.privateTransfer(
  connection,
  note,
  [
    { recipient: addr1, amount: 500_000_000 },
    { recipient: addr2, amount: 492_500_000 }
  ]
);
// Everything handled automatically!
```

**Pattern 3: Swap SOL for tokens privately**
```typescript
import { generateNote } from "@cloaklabz/sdk";

// Swap SOL for tokens privately
const note = generateNote(1_000_000_000);
const result = await client.swap(
  connection,
  note,
  recipientPublicKey,
  {
    outputMint: "BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k",
    slippageBps: 100,
    getQuote: async (amount, mint, slippage) => {
      // Your swap quote logic
    },
  }
);
```

## API Reference

### CloakSDK

Main client for interacting with Cloak Protocol.

#### Constructor

```typescript
new CloakSDK(config: {
  keypairBytes: Uint8Array;  // Required: Solana keypair secret key
  network?: Network;          // Optional: "devnet" | "testnet" | "mainnet" (default: "devnet")
  cloakKeys?: CloakKeyPair;   // Optional: Cloak protocol keys (auto-generated if not provided)
  storage?: StorageAdapter;   // Optional: Storage adapter for notes (default: in-memory)
  programId?: PublicKey;      // Optional: Cloak program ID (uses default if not provided)
})
```

#### Methods

##### `deposit()`

Deposit SOL into the protocol and create a private note.

```typescript
async deposit(
  connection: Connection,
  payer: Keypair | WalletAdapter,
  amountLamports: number,
  options?: DepositOptions
): Promise<DepositResult>
```

**Example:**
```typescript
const result = await client.deposit(connection, keypair, 1_000_000_000, {
  onProgress: (status) => console.log(status),
});
console.log("Leaf index:", result.leafIndex);
// Save result.note securely!
```

##### `privateTransfer()`

Execute a complete private transfer with 1-5 recipients.

**Handles the full flow**: If the note isn't deposited yet, it deposits it first, waits for confirmation, then proceeds with the withdrawal.

```typescript
async privateTransfer(
  connection: Connection,
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
const note = generateNote(1_000_000_000);

// privateTransfer handles deposit + withdrawal
const result = await client.privateTransfer(
  connection,
  note,
  [
    { recipient: addr1, amount: 500_000_000 },
    { recipient: addr2, amount: 400_000_000 },
    { recipient: addr3, amount: 92_500_000 }, // After protocol fees
  ],
  {
    onProgress: (status) => console.log(status),
    onProofProgress: (progress) => console.log(`Proof: ${progress}%`),
  }
);
```

##### `send()`

Convenience method that wraps `privateTransfer()` with a simpler API.

```typescript
async send(
  connection: Connection,
  note: CloakNote,
  recipients: MaxLengthArray<Transfer, 5>,
  options?: TransferOptions
): Promise<TransferResult>
```

**Example:**
```typescript
const result = await client.send(
  connection,
  note,
  [
    { recipient: addr1, amount: 500_000_000 },
    { recipient: addr2, amount: 492_500_000 },
  ]
);
```

##### `withdraw()`

Convenience method for withdrawing to a single recipient.

**Handles the full flow**: Deposits if needed, then withdraws to one recipient.

```typescript
async withdraw(
  connection: Connection,
  note: CloakNote,
  recipient: PublicKey,
  options?: WithdrawOptions
): Promise<TransferResult>
```

**Example:**
```typescript
const note = generateNote(1_000_000_000);
const result = await client.withdraw(
  connection,
  note,
  recipientAddress,
  { withdrawAll: true } // Withdraw full amount minus fees
);
```

##### `swap()`

Swap SOL for tokens privately using zero-knowledge proofs.

**Handles the full flow**: Deposits if needed, fetches swap quote, generates proof, and executes swap.

```typescript
async swap(
  connection: Connection,
  note: CloakNote,
  recipient: PublicKey,
  options: SwapOptions
): Promise<SwapResult>
```

**SwapOptions:**
```typescript
interface SwapOptions extends TransferOptions {
  outputMint: string;                    // Output token mint address
  slippageBps?: number;                  // Slippage tolerance in basis points (default: 100)
  minOutputAmount?: number;               // Minimum output amount (if not using getQuote)
  getQuote?: (amountLamports: number, outputMint: string, slippageBps: number) => Promise<{
    outAmount: number;
    minOutputAmount: number;
  }>;
}
```

**Example:**
```typescript
const note = generateNote(1_000_000_000);
const result = await client.swap(
  connection,
  note,
  recipientPublicKey,
  {
    outputMint: "BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k",
    slippageBps: 100, // 1% slippage
    getQuote: async (amount, mint, slippage) => {
      // Fetch quote from swap API (e.g., Orca, Jupiter)
      const response = await fetch(
        `https://pools-api.devnet.orca.so/swap-quote?from=So11111111111111111111111111111111111111112&to=${mint}&amount=${amount}&slippageBps=${slippage}`
      );
      const data = await response.json();
      return {
        outAmount: parseInt(data.data.swap.outputAmount),
        minOutputAmount: Math.floor(parseInt(data.data.swap.outputAmount) * (10000 - slippage) / 10000),
      };
    },
    onProgress: (status) => console.log(status),
  }
);
```

##### `generateNote()`

Generate a new note without depositing (for testing or pre-generation).

**Note**: This is a standalone function, not a method on the client.

```typescript
import { generateNote } from "@cloaklabz/sdk";

const note = generateNote(amountLamports: number): CloakNote
```

##### `parseNote()`

Parse a note from JSON string.

**Note**: This is a standalone function, not a method on the client.

```typescript
import { parseNote } from "@cloaklabz/sdk";

const note = parseNote(jsonString: string): CloakNote
```

##### `exportNote()`

Export a note to JSON string.

**Note**: This is a standalone function, not a method on the client.

```typescript
import { exportNote } from "@cloaklabz/sdk";

const json = exportNote(note: CloakNote, pretty?: boolean): string
```

### Fee Calculation

```typescript
import { calculateFee, getDistributableAmount, formatAmount } from "@cloaklabz/sdk";

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
} from "@cloaklabz/sdk";

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
import { IndexerService, ArtifactProverService, RelayService } from "@cloaklabz/sdk";

const indexer = new IndexerService("https://indexer.example.com");
// Use ArtifactProverService for privacy (private inputs never pass through backend)
const prover = new ArtifactProverService("https://indexer.example.com");
const relay = new RelayService("https://relay.example.com");

// Get Merkle root
const { root, next_index } = await indexer.getMerkleRoot();

// Get Merkle proof
const proof = await indexer.getMerkleProof(leafIndex);

// Generate proof using artifact-based flow (private inputs uploaded directly to TEE)
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
  SwapOptions,
  SwapResult,
} from "@cloaklabz/sdk";
```

## Security Considerations

⚠️ **Important Security Notes:**

1. **Keep notes secure**: Treat them like private keys. Anyone with the note can withdraw funds.

2. **Backup notes**: If you lose a note, the funds are unrecoverable.

3. **Verify recipients**: Double-check recipient addresses before transfer.

4. **Amount validation**: Always validate that outputs sum to expected amounts.

5. **Network isolation**: Don't mix notes from different networks (devnet/mainnet).

6. **Proof privacy**: This SDK uses artifact-based proof generation where private inputs are uploaded directly to TEE, never passing through the backend in plaintext. This provides true privacy-preserving withdrawals.

## Error Handling

```typescript
try {
  const result = await client.privateTransfer(connection, note, recipients);
  console.log("Success:", result.signature);
} catch (error) {
  if (error.message.includes("Note must be deposited")) {
    console.error("Note not yet deposited");
  } else if (error.message.includes("Proof generation failed")) {
    console.error("ZK proof generation failed");
  } else if (error.message.includes("Relay")) {
    console.error("Relay service error");
  } else if (error.message.includes("@solana/spl-token")) {
    console.error("Swap requires @solana/spl-token package");
  } else {
    console.error("Unknown error:", error);
  }
}
```

## Examples

See the [examples](./examples) directory for complete working examples:

- **deposit-example.ts** - Deposit SOL to create a private note
- **transfer-example.ts** - Private transfers to multiple recipients
- **send-example.ts** - Using the `send()` convenience method
- **withdraw-example.ts** - Withdraw previously deposited notes
- **swap-example.ts** - Swap SOL for tokens privately

Run examples:
```bash
npm run example:deposit
npm run example:transfer
npm run example:send
npm run example:withdraw
npm run example:swap
```

## Contributing

Contributions are welcome! Please open an issue or PR.

## License

MIT

## Support

For questions and support:
- GitHub Issues: [github.com/cloak/sdk/issues](https://github.com/cloak/sdk/issues)
- Documentation: [docs.cloak.xyz](https://docs.cloak.xyz)
