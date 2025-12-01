import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  CloakConfig,
  CloakNote,
  DepositOptions,
  DepositResult,
  MaxLengthArray,
  Transfer,
  TransferOptions,
  TransferResult,
  WithdrawOptions,
  MerkleProof,
  SP1ProofInputs,
  Network,
  CloakError,
  ScanNotesOptions,
  ScannedNote,
  SwapOptions,
  SwapResult,
} from "./types";
import { 
  generateNote, 
  generateNoteFromWallet,
  updateNoteWithDeposit, 
  isWithdrawable,
  findNoteByCommitment,
  exportWalletKeys,
  importWalletKeys,
} from "./note-manager";
import { StorageAdapter, MemoryStorageAdapter } from "./storage";
import {
  validateTransfers,
  parseNote,
} from "../utils/validation";
import {
  computeNullifier,
  computeOutputsHash,
  computeSwapOutputsHash,
  hexToBytes,
  bytesToHex,
  isValidHex,
} from "../utils/crypto";
import { getDistributableAmount } from "../utils/fees";
import { IndexerService } from "../services/IndexerService";
import { ProverService } from "../services/ProverService";
import { RelayService } from "../services/RelayService";
import { DepositRecoveryService } from "../services/DepositRecoveryService";
import { createDepositInstruction } from "../solana/instructions";
import { getShieldPoolPDAs } from "../utils/pda";
import { type CloakKeyPair, generateCloakKeys, scanNotesForWallet } from "./keys";
import { prepareEncryptedOutput, prepareEncryptedOutputForRecipient, encodeNoteSimple } from "../helpers/encrypted-output";

export const CLOAK_PROGRAM_ID = new PublicKey("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");
const CLOAK_API_URL = 
  "https://api.cloaklabz.xyz";

/**
 * Main Cloak SDK
 *
 * Provides high-level API for interacting with the Cloak protocol,
 * including deposits, withdrawals, and private transfers.
 */
export class CloakSDK {
  private config: CloakConfig;
  private keypair: Keypair;
  private cloakKeys?: CloakKeyPair;
  private indexer: IndexerService;
  private prover: ProverService;
  private relay: RelayService;
  private depositRecovery: DepositRecoveryService;
  private storage: StorageAdapter;

  /**
  * Create a new Cloak SDK client
   *
   * @param config - Client configuration
   * 
   * @example
   * ```typescript
   * // Enhanced mode with v2.0 features (recommended)
   * const keys = generateCloakKeys();
   * const sdk = new CloakSDK({
   *   keypairBytes: keypair.secretKey,
   *   cloakKeys: keys,
   *   network: "devnet"
   * });
   * 
   * // Legacy mode (v1.0)
   * const sdk = new CloakSDK({
   *   keypairBytes: keypair.secretKey,
   *   network: "devnet"
   * });
   * ```
   */
  constructor(config: {
    keypairBytes: Uint8Array;
    network?: Network;
    cloakKeys?: CloakKeyPair;
    storage?: StorageAdapter;
    programId?: PublicKey;
  }) {
    this.keypair = Keypair.fromSecretKey(config.keypairBytes);
    this.cloakKeys = config.cloakKeys;
    this.storage = config.storage || new MemoryStorageAdapter();

    const apiUrl = CLOAK_API_URL;
    this.indexer = new IndexerService(apiUrl);
    this.prover = new ProverService(apiUrl, 5 * 60 * 1000);
    this.relay = new RelayService(apiUrl);
    this.depositRecovery = new DepositRecoveryService(this.indexer, apiUrl);
    
    // Load keys from storage if not provided (synchronous only for constructor)
    if (!this.cloakKeys) {
      const storedKeys = this.storage.loadKeys();
      if (storedKeys && !(storedKeys instanceof Promise)) {
        this.cloakKeys = storedKeys;
      }
    }

    // Use programId from config if provided, otherwise use default
    const programId = config.programId || CLOAK_PROGRAM_ID;
    const { pool, commitments, rootsRing, nullifierShard, treasury } = getShieldPoolPDAs(programId);

    this.config = {
      network: config.network || "devnet",
      keypairBytes: config.keypairBytes,
      cloakKeys: config.cloakKeys,
      apiUrl,
      programId: programId,
      poolAddress: pool,
      commitmentsAddress: commitments,
      rootsRingAddress: rootsRing,
      nullifierShardAddress: nullifierShard,
      treasuryAddress: treasury,
    };
  }

  /**
   * Deposit SOL into the Cloak protocol
   *
   * Creates a new note (or uses a provided one), submits a deposit transaction,
   * and registers with the indexer.
   *
   * @param connection - Solana connection
   * @param payer - Payer wallet with sendTransaction method
   * @param amountOrNote - Amount in lamports OR an existing note to deposit
   * @param options - Optional configuration
   * @returns Deposit result with note and transaction info
   *
   * @example
   * ```typescript
   * // Generate and deposit in one step
   * const result = await client.deposit(
   *   connection,
   *   wallet,
   *   1_000_000_000,
   *   {
   *     onProgress: (status) => console.log(status)
   *   }
   * );
   *
   * // Or deposit a pre-generated note
   * const note = client.generateNote(1_000_000_000);
   * const result = await client.deposit(connection, wallet, note);
   * ```
   */
  async deposit(
    connection: Connection,
    amountOrNote: number | CloakNote,
    options?: DepositOptions
  ): Promise<DepositResult> {
    try {
      // Privacy warning for testnet
      if (this.config.network !== "mainnet" && !options?.skipPrivacyWarning) {
        console.warn(`
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  PRIVACY WARNING - TESTNET ONLY                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Current privacy level: WEAK (Experimental)                 │
│  Anonymity set: ~10-50 deposits                             │
│  Privacy strength: ~3-6 bits of entropy                     │
│                                                             │
│  DO NOT use for production privacy needs.                   │
│                                                             │
│  Learn more:                                                │
│  https://github.com/cloak-labz/cloak/blob/main/PRIVACY_STATUS.md
│                                                             │
└─────────────────────────────────────────────────────────────┘
        `);
      }

      // Determine if we're using a provided note or generating a new one
      let note: CloakNote;

      if (typeof amountOrNote === "number") {
        options?.onProgress?.("Generating note...");
        note = generateNote(amountOrNote, this.config.network);
      } else {
        note = amountOrNote;
        // Validate the note hasn't been deposited already
        if (note.depositSignature) {
          throw new Error("Note has already been deposited");
        }
      }

    options?.onProgress?.("Checking account balance...");

    // Check account balance before attempting deposit
    const balance = await connection.getBalance(this.keypair.publicKey);
    const requiredAmount = note.amount + 5000; // Amount + estimated transaction fee
    if (balance < requiredAmount) {
      throw new Error(
        `Insufficient balance. Required: ${requiredAmount} lamports (${note.amount} + fees), Available: ${balance} lamports`
      );
    }

    options?.onProgress?.("Creating deposit transaction...");

    // Create deposit instruction
    const commitmentBytes = hexToBytes(note.commitment);
    const programId = this.config.programId || CLOAK_PROGRAM_ID;
    const depositIx = createDepositInstruction({
      programId: programId,
      payer: this.keypair.publicKey,
      pool: this.config.poolAddress!,
      commitments: this.config.commitmentsAddress!,
      amount: note.amount,
      commitment: commitmentBytes,
    });

    // Build transaction
    const { blockhash, lastValidBlockHeight } =
      await connection.getLatestBlockhash();

    const transaction = new Transaction({
      feePayer: this.keypair.publicKey,
      blockhash,
      lastValidBlockHeight,
    }).add(depositIx);

    // Simulate if not skipped
    if (!options?.skipPreflight) {
      options?.onProgress?.("Simulating transaction...");
      const simulation = await connection.simulateTransaction(transaction);

      if (simulation.value.err) {
        const logs = simulation.value.logs?.join("\n") || "No logs";
        throw new Error(
          `Transaction simulation failed: ${JSON.stringify(simulation.value.err)}\nLogs:\n${logs}`
        );
      }
    }

    options?.onProgress?.("Sending transaction...");

    // Send transaction
    const signature = await connection.sendTransaction(transaction, [this.keypair],  {
      skipPreflight: options?.skipPreflight || false,
      preflightCommitment: "confirmed",
      maxRetries: 3,
    });

    options?.onProgress?.("Confirming transaction...");

    // Wait for confirmation
    const confirmation = await connection.confirmTransaction({
      signature,
      blockhash,
      lastValidBlockHeight,
    });

    if (confirmation.value.err) {
      throw new Error(
        `Transaction failed: ${JSON.stringify(confirmation.value.err)}`
      );
    }

    // Get transaction details
    const txDetails = await connection.getTransaction(signature, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });

    const depositSlot = txDetails?.slot ?? 0;

    options?.onProgress?.("Registering with indexer...");

    // Submit to indexer with retry logic for transient backend errors
    const encryptedOutput = this.encodeNote(note, options?.recipientViewKey);
    let depositResponse: { success: boolean; leafIndex?: number; root?: string } | null = null;
    let retries = 0;
    const maxRetries = 3;
    const baseDelayMs = 1000;
    
    while (retries <= maxRetries) {
      try {
        depositResponse = await this.indexer.submitDeposit({
          leafCommit: note.commitment,
          encryptedOutput,
          txSignature: signature,
          slot: depositSlot,
        });
        break; // Success, exit retry loop
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        
        // Retry on Merkle tree inconsistency errors (backend state issues)
        if (errorMessage.includes("Merkle tree") && errorMessage.includes("inconsistent") && retries < maxRetries) {
          retries++;
          const delayMs = baseDelayMs * Math.pow(2, retries - 1); // Exponential backoff
          options?.onProgress?.(`Merkle tree inconsistency detected, retrying in ${delayMs}ms... (attempt ${retries}/${maxRetries})`);
          await new Promise(resolve => setTimeout(resolve, delayMs));
          continue;
        }
        
        // For other errors or max retries reached, throw the error
        throw error;
      }
    }

    if (!depositResponse || !depositResponse.leafIndex || !depositResponse.root) {
      throw new Error("Failed to submit deposit: Indexer did not return leaf index and root");
    }

    const leafIndex = depositResponse.leafIndex;
    const root = depositResponse.root;

    options?.onProgress?.("Fetching Merkle proof...");

    // Fetch Merkle proof
    const merkleProof = await this.indexer.getMerkleProof(leafIndex);

    // Update note with deposit info
    const updatedNote = updateNoteWithDeposit(note, {
      signature,
      slot: depositSlot,
      leafIndex,
      root,
      merkleProof: {
        pathElements: merkleProof.pathElements,
        pathIndices: merkleProof.pathIndices,
      },
    });

      options?.onProgress?.("Deposit complete!");

      return {
        note: updatedNote,
        signature,
        leafIndex,
        root,
      };
    } catch (error) {
      throw this.wrapError(error, "Deposit failed");
    }
  }

  /**
   * Private transfer with up to 5 recipients
   *
   * Handles the complete private transfer flow:
   * 1. If note is not deposited, deposits it first and waits for confirmation
   * 2. Generates a zero-knowledge proof
   * 3. Submits the withdrawal via relay service to recipients
   *
   * This is the main method for performing private transfers - it handles everything!
   *
   * @param connection - Solana connection (required for deposit if not already deposited)
   * @param payer - Payer wallet (required for deposit if not already deposited)
   * @param note - Note to spend (can be deposited or not)
   * @param recipients - Array of 1-5 recipients with amounts
   * @param options - Optional configuration
   * @returns Transfer result with signature and outputs
   *
   * @example
   * ```typescript
   * // Create a note (not deposited yet)
   * const note = client.generateNote(1_000_000_000);
   *
   * // privateTransfer handles the full flow: deposit + withdraw
   * const result = await client.privateTransfer(
   *   connection,
   *   wallet,
   *   note,
   *   [
   *     { recipient: new PublicKey("..."), amount: 500_000_000 },
   *     { recipient: new PublicKey("..."), amount: 492_500_000 }
   *   ],
   *   {
   *     relayFeeBps: 50, // 0.5%
   *     onProgress: (status) => console.log(status),
   *     onProofProgress: (pct) => console.log(`Proof: ${pct}%`)
   *   }
   * );
   * console.log(`Success! TX: ${result.signature}`);
   * ```
   */
  async privateTransfer(
    connection: Connection,
    note: CloakNote,
    recipients: MaxLengthArray<Transfer, 5>,
    options?: TransferOptions
  ): Promise<TransferResult> {
    // Check if note needs to be deposited first
    if (!isWithdrawable(note)) {
      options?.onProgress?.("Note not deposited yet - depositing first...");

      // Deposit the note (pass the existing note to preserve its commitment)
      const depositResult = await this.deposit(connection, note, {
        onProgress: options?.onProgress,
        skipPreflight: false,
      });

      // Use the deposited note for withdrawal
      note = depositResult.note;

      options?.onProgress?.("Deposit complete - proceeding with private transfer...");
    }

    // Calculate the actual protocol fee and convert to basis points
    // The relay validates that fee_bps matches the actual fee charged
    const protocolFee = note.amount - getDistributableAmount(note.amount);
    const feeBps = Math.ceil((protocolFee * 10_000) / note.amount);

    // Calculate distributable amount (after protocol fees)
    const distributableAmount = getDistributableAmount(note.amount);

    // Validate recipients sum to distributable amount
    validateTransfers(recipients, distributableAmount);

    options?.onProgress?.("Fetching Merkle proof...");

    // Use historical Merkle proof from note if available (matches historical root)
    // Otherwise fetch current proof (for notes that don't have historical proof stored)
    let merkleProof: MerkleProof;
    let merkleRoot: string;
    
    if (note.merkleProof && note.root) {
      // Use historical proof and root from when note was deposited
      merkleProof = {
        pathElements: note.merkleProof.pathElements,
        pathIndices: note.merkleProof.pathIndices,
      };
      merkleRoot = note.root;
    } else {
      // Fallback: fetch current proof (for backward compatibility)
      merkleProof = await this.indexer.getMerkleProof(note.leafIndex!);
      merkleRoot = merkleProof.root || (await this.indexer.getMerkleRoot()).root;
    }

    options?.onProgress?.("Computing cryptographic values...");

    // Compute nullifier
    const skSpendBytes = hexToBytes(note.sk_spend);
    const nullifierBytes = computeNullifier(skSpendBytes, note.leafIndex!);
    const nullifierHex = bytesToHex(nullifierBytes);

    // Compute outputs hash
    const outputsHashBytes = computeOutputsHash(recipients);
    const outputsHashHex = bytesToHex(outputsHashBytes);

    options?.onProgress?.("Generating zero-knowledge proof...");

    // Validate required fields
    if (!note.leafIndex && note.leafIndex !== 0) {
      throw new Error("Note must have a leaf index (note must be deposited)");
    }
    if (!merkleProof.pathElements || merkleProof.pathElements.length === 0) {
      throw new Error("Merkle proof is invalid: missing path elements");
    }
    if (merkleProof.pathElements.length !== merkleProof.pathIndices.length) {
      throw new Error("Merkle proof is invalid: path elements and indices length mismatch");
    }
    
    // Validate Merkle path indices are binary (0 or 1 only)
    for (let i = 0; i < merkleProof.pathIndices.length; i++) {
      const idx = merkleProof.pathIndices[i];
      if (idx !== 0 && idx !== 1) {
        throw new Error(`Merkle proof path index at position ${i} must be 0 or 1, got ${idx}`);
      }
    }
    
    // Validate hex strings format
    if (!isValidHex(note.r, 32)) {
      throw new Error("Note r must be 64 hex characters (32 bytes)");
    }
    if (!isValidHex(note.sk_spend, 32)) {
      throw new Error("Note sk_spend must be 64 hex characters (32 bytes)");
    }
    if (!isValidHex(merkleRoot, 32)) {
      throw new Error("Merkle root must be 64 hex characters (32 bytes)");
    }
    
    // Validate Merkle path elements are hex strings
    for (let i = 0; i < merkleProof.pathElements.length; i++) {
      const element = merkleProof.pathElements[i];
      if (typeof element !== "string" || !isValidHex(element, 32)) {
        throw new Error(`Merkle proof path element at position ${i} must be 64 hex characters (32 bytes)`);
      }
    }

    // Prepare proof inputs
    const proofInputs: SP1ProofInputs = {
      privateInputs: {
        amount: note.amount,
        r: note.r,
        sk_spend: note.sk_spend,
        leaf_index: note.leafIndex!,
        merkle_path: {
          path_elements: merkleProof.pathElements,
          path_indices: merkleProof.pathIndices,
        },
      },
      publicInputs: {
        root: merkleRoot,
        nf: nullifierHex,
        outputs_hash: outputsHashHex,
        amount: note.amount,
      },
      outputs: recipients.map((r) => ({
        address: r.recipient.toBase58(),
        amount: r.amount,
      })),
    };

    // Generate proof
    const proofResult = await this.prover.generateProof(proofInputs, {
      onProgress: options?.onProofProgress,
      onStart: () => options?.onProgress?.("Starting proof generation..."),
      onSuccess: () => options?.onProgress?.("Proof generated successfully"),
      onError: (error) => {
        // Log detailed error for debugging
        console.error("[CloakSDK] Proof generation error:", error);
        options?.onProgress?.(`Proof generation error: ${error}`);
      },
    });

    if (!proofResult.success || !proofResult.proof || !proofResult.publicInputs) {
      // The ProverService already extracts and formats the error message
      let errorMessage = proofResult.error || "Proof generation failed";
      
      // Remove redundant "Proof generation failed:" prefix if present
      if (errorMessage.startsWith("Proof generation failed: ")) {
        errorMessage = errorMessage.substring("Proof generation failed: ".length);
      }
      
      // Add context about the note being spent
      errorMessage += `\nNote details: leafIndex=${note.leafIndex}, root=${merkleRoot.slice(0, 16)}..., nullifier=${nullifierHex.slice(0, 16)}...`;
      
      throw new Error(errorMessage);
    }

    options?.onProgress?.("Submitting to relay service...");

    // Submit via relay
    const signature = await this.relay.submitWithdraw(
      {
        proof: proofResult.proof,
        publicInputs: {
          root: merkleRoot,
          nf: nullifierHex,
          outputs_hash: outputsHashHex,
          amount: note.amount,
        },
        outputs: recipients.map((r) => ({
          recipient: r.recipient.toBase58(),
          amount: r.amount,
        })),
        feeBps: feeBps,  // Use calculated protocol fee BPS
      },
      options?.onProgress
    );

    options?.onProgress?.("Transfer complete!");

    return {
      signature,
      outputs: recipients.map((r) => ({
        recipient: r.recipient.toBase58(),
        amount: r.amount,
      })),
      nullifier: nullifierHex,
      root: merkleRoot,
    };
  }

  /**
   * Withdraw to a single recipient
   *
   * Convenience method for withdrawing to one address.
   * Handles the complete flow: deposits if needed, then withdraws.
   *
   * @param connection - Solana connection
   * @param payer - Payer wallet
   * @param note - Note to spend
   * @param recipient - Recipient address
   * @param options - Optional configuration
   * @returns Transfer result
   *
   * @example
   * ```typescript
   * const note = client.generateNote(1_000_000_000);
   * const result = await client.withdraw(
   *   connection,
   *   wallet,
   *   note,
   *   new PublicKey("..."),
   *   { withdrawAll: true }
   * );
   * ```
   */
  async withdraw(
    connection: Connection,
    note: CloakNote,
    recipient: PublicKey,
    options?: WithdrawOptions
  ): Promise<TransferResult> {
    const withdrawAll = options?.withdrawAll ?? true;
    const amount = withdrawAll
      ? getDistributableAmount(note.amount)
      : options?.amount || note.amount;

    if (!withdrawAll && !options?.amount) {
      throw new Error("Must specify amount or set withdrawAll: true");
    }

    return this.privateTransfer(
      connection,
      note,
      [{ recipient, amount }],
      options
    );
  }

  /**
   * Send SOL privately to multiple recipients
   *
   * Convenience method that wraps privateTransfer with a simpler API.
   * Handles the complete flow: deposits if needed, then sends to recipients.
   *
   * @param connection - Solana connection
   * @param note - Note to spend
   * @param recipients - Array of 1-5 recipients with amounts
   * @param options - Optional configuration
   * @returns Transfer result
   *
   * @example
   * ```typescript
   * const note = client.generateNote(1_000_000_000);
   * const result = await client.send(
   *   connection,
   *   note,
   *   [
   *     { recipient: new PublicKey("..."), amount: 500_000_000 },
   *     { recipient: new PublicKey("..."), amount: 492_500_000 }
   *   ]
   * );
   * ```
   */
  async send(
    connection: Connection,
    note: CloakNote,
    recipients: MaxLengthArray<Transfer, 5>,
    options?: TransferOptions
  ): Promise<TransferResult> {
    return this.privateTransfer(connection, note, recipients, options);
  }

  /**
   * Swap SOL for tokens privately
   *
   * Withdraws SOL from a note and swaps it for tokens via the relay service.
   * Handles the complete flow: deposits if needed, generates proof, and submits swap.
   *
   * @param connection - Solana connection
   * @param note - Note to spend
   * @param recipient - Recipient address (will receive tokens)
   * @param options - Swap configuration
   * @returns Swap result with transaction signature
   *
   * @example
   * ```typescript
   * const note = client.generateNote(1_000_000_000);
   * const result = await client.swap(
   *   connection,
   *   note,
   *   new PublicKey("..."), // recipient
   *   {
   *     outputMint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
   *     slippageBps: 100, // 1%
   *     getQuote: async (amount, mint, slippage) => {
   *       // Fetch quote from your swap API
   *       const quote = await fetchSwapQuote(amount, mint, slippage);
   *       return {
   *         outAmount: quote.outAmount,
   *         minOutputAmount: quote.minOutputAmount
   *       };
   *     }
   *   }
   * );
   * ```
   */
  async swap(
    connection: Connection,
    note: CloakNote,
    recipient: PublicKey,
    options: SwapOptions
  ): Promise<SwapResult> {
    try {
      // Check if note needs to be deposited first
      if (!isWithdrawable(note)) {
        options.onProgress?.("Note not deposited yet - depositing first...");

        const depositResult = await this.deposit(connection, note, {
          onProgress: options.onProgress,
          skipPreflight: false,
        });

        note = depositResult.note;
        options.onProgress?.("Deposit complete - proceeding with swap...");
      }

      // Calculate fee_bps for swap-mode withdrawals
      // For swap-mode withdrawals, the relay expects ONLY the variable fee (0.5%) in fee_bps,
      // not the total fee (fixed + variable). The fixed fee is still taken but not encoded in fee_bps.
      // This matches the Rust test implementation in prove_test_swap.rs and services/relay/src/api/withdraw.rs
      const variableFee = Math.floor((note.amount * 5) / 1_000); // 0.5% = 5/1000
      const feeBps = note.amount === 0 
        ? 0 
        : Math.min(Math.floor(((variableFee * 10_000) + note.amount - 1) / note.amount), 65535); // Cap at u16::MAX

      // Calculate withdraw amount (after fees)
      const withdrawAmountLamports = getDistributableAmount(note.amount);
      if (withdrawAmountLamports <= 0) {
        throw new Error("Amount too small after fees");
      }

      // Get swap quote if not provided
      let minOutputAmount: number;
      if (options.minOutputAmount !== undefined) {
        minOutputAmount = options.minOutputAmount;
      } else if (options.getQuote) {
        options.onProgress?.("Fetching swap quote...");
        const quote = await options.getQuote(
          withdrawAmountLamports,
          options.outputMint,
          options.slippageBps || 100
        );
        minOutputAmount = quote.minOutputAmount;
      } else {
        throw new Error(
          "Must provide either minOutputAmount or getQuote function"
        );
      }

      // Get recipient's associated token account
      // Note: This requires @solana/spl-token, but we'll make it optional
      // Users can pass the ATA directly if needed
      let recipientAta: PublicKey;
      try {
        // Try to import getAssociatedTokenAddress dynamically
        // Using type assertion to avoid TypeScript errors during build
        const splTokenModule = await import(
          "@solana/spl-token" as any
        ) as any;
        const getAssociatedTokenAddress = splTokenModule.getAssociatedTokenAddress;
        if (!getAssociatedTokenAddress) {
          throw new Error("getAssociatedTokenAddress not found");
        }
        const outputMint = new PublicKey(options.outputMint);
        recipientAta = await getAssociatedTokenAddress(outputMint, recipient);
      } catch (error) {
        throw new Error(
          `Failed to get associated token account: ${error instanceof Error ? error.message : String(error)}. Please install @solana/spl-token or provide recipientAta in options.`
        );
      }

      options.onProgress?.("Fetching Merkle proof...");

      // Use historical Merkle proof from note if available
      let merkleProof: MerkleProof;
      let merkleRoot: string;

      if (note.merkleProof && note.root) {
        merkleProof = {
          pathElements: note.merkleProof.pathElements,
          pathIndices: note.merkleProof.pathIndices,
        };
        merkleRoot = note.root;
      } else {
        merkleProof = await this.indexer.getMerkleProof(note.leafIndex!);
        merkleRoot =
          merkleProof.root || (await this.indexer.getMerkleRoot()).root;
      }

      options.onProgress?.("Computing cryptographic values...");

      // Compute nullifier
      const skSpendBytes = hexToBytes(note.sk_spend);
      const nullifierBytes = computeNullifier(skSpendBytes, note.leafIndex!);
      const nullifierHex = bytesToHex(nullifierBytes);

      // Compute swap outputs hash
      const outputMint = new PublicKey(options.outputMint);
      const outputsHashBytes = computeSwapOutputsHash(
        outputMint,
        recipientAta,
        minOutputAmount,
        note.amount
      );
      const outputsHashHex = bytesToHex(outputsHashBytes);

      options.onProgress?.("Generating zero-knowledge proof...");

      // Validate required fields
      if (!note.leafIndex && note.leafIndex !== 0) {
        throw new Error("Note must have a leaf index (note must be deposited)");
      }
      if (!merkleProof.pathElements || merkleProof.pathElements.length === 0) {
        throw new Error("Merkle proof is invalid: missing path elements");
      }

      // Prepare proof inputs with swap parameters
      const proofInputs: SP1ProofInputs = {
        privateInputs: {
          amount: note.amount,
          r: note.r,
          sk_spend: note.sk_spend,
          leaf_index: note.leafIndex!,
          merkle_path: {
            path_elements: merkleProof.pathElements,
            path_indices: merkleProof.pathIndices,
          },
        },
        publicInputs: {
          root: merkleRoot,
          nf: nullifierHex,
          outputs_hash: outputsHashHex,
          amount: note.amount,
        },
        outputs: [], // Empty for swaps
        swapParams: {
          output_mint: options.outputMint,
          recipient_ata: recipientAta.toBase58(),
          min_output_amount: minOutputAmount,
        },
      };

      // Generate proof
      const proofResult = await this.prover.generateProof(proofInputs, {
        onProgress: options.onProofProgress,
        onStart: () => options.onProgress?.("Starting proof generation..."),
        onSuccess: () => options.onProgress?.("Proof generated successfully"),
        onError: (error) => {
          console.error("[CloakSDK] Proof generation error:", error);
          options.onProgress?.(`Proof generation error: ${error}`);
        },
      });

      if (!proofResult.success || !proofResult.proof || !proofResult.publicInputs) {
        let errorMessage = proofResult.error || "Proof generation failed";
        if (errorMessage.startsWith("Proof generation failed: ")) {
          errorMessage = errorMessage.substring("Proof generation failed: ".length);
        }
        errorMessage += `\nNote details: leafIndex=${note.leafIndex}, root=${merkleRoot.slice(0, 16)}..., nullifier=${nullifierHex.slice(0, 16)}...`;
        throw new Error(errorMessage);
      }

      options.onProgress?.("Submitting swap to relay service...");

      // Submit swap via relay
      const signature = await this.relay.submitSwap(
        {
          proof: proofResult.proof,
          publicInputs: {
            root: merkleRoot,
            nf: nullifierHex,
            outputs_hash: outputsHashHex,
            amount: note.amount,
          },
          outputs: [
            {
              recipient: recipient.toBase58(),
              amount: withdrawAmountLamports,
            },
          ],
          feeBps: feeBps,
          swap: {
            output_mint: options.outputMint,
            slippage_bps: options.slippageBps || 100,
            min_output_amount: minOutputAmount,
          },
        },
        options.onProgress
      );

      options.onProgress?.("Swap complete!");

      return {
        signature,
        outputs: [
          {
            recipient: recipient.toBase58(),
            amount: withdrawAmountLamports,
          },
        ],
        nullifier: nullifierHex,
        root: merkleRoot,
        outputMint: options.outputMint,
        minOutputAmount: minOutputAmount,
      };
    } catch (error) {
      throw this.wrapError(error, "Swap failed");
    }
  }

  /**
   * Generate a new note without depositing
   *
   * @param amountLamports - Amount for the note
   * @param useWalletKeys - Whether to use wallet keys (v2.0 recommended)
   * @returns New note (not yet deposited)
   */
  generateNote(amountLamports: number, useWalletKeys: boolean = false): CloakNote {
    if (useWalletKeys && this.cloakKeys) {
      return generateNoteFromWallet(amountLamports, this.cloakKeys, this.config.network);
    } else if (useWalletKeys) {
      // Generate keys if not provided but requested
      const keys = generateCloakKeys();
      this.cloakKeys = keys;
      // Save to storage
      const result = this.storage.saveKeys(keys);
      if (result instanceof Promise) {
        // Fire and forget - don't block
        result.catch(() => {});
      }
      return generateNoteFromWallet(amountLamports, keys, this.config.network);
    }
    return generateNote(amountLamports, this.config.network);
  }

  /**
   * Parse a note from JSON string
   *
   * @param jsonString - JSON representation
   * @returns Parsed note
   */
  parseNote(jsonString: string): CloakNote {
    return parseNote(jsonString);
  }

  /**
   * Export a note to JSON string
   *
   * @param note - Note to export
   * @param pretty - Format with indentation
   * @returns JSON string
   */
  exportNote(note: CloakNote, pretty: boolean = false): string {
    return pretty ? JSON.stringify(note, null, 2) : JSON.stringify(note);
  }

  /**
   * Check if a note is ready for withdrawal
   *
   * @param note - Note to check
   * @returns True if withdrawable
   */
  isWithdrawable(note: CloakNote): boolean {
    return isWithdrawable(note);
  }

  /**
   * Get Merkle proof for a leaf index
   *
   * @param leafIndex - Leaf index in tree
   * @returns Merkle proof
   */
  async getMerkleProof(leafIndex: number): Promise<MerkleProof> {
    return this.indexer.getMerkleProof(leafIndex);
  }

  /**
   * Get current Merkle root
   *
   * @returns Current root hash
   */
  async getCurrentRoot(): Promise<string> {
    const response = await this.indexer.getMerkleRoot();
    return response.root;
  }

  /**
   * Get transaction status from relay service
   *
   * @param requestId - Request ID from previous submission
   * @returns Current status
   */
  async getTransactionStatus(requestId: string) {
    return this.relay.getStatus(requestId);
  }

  /**
   * Recover a deposit that completed on-chain but failed to register
   * 
   * Use this when a deposit transaction succeeded but the browser crashed
   * or lost connection before the indexer registration completed.
   * 
   * @param signature - Transaction signature
   * @param commitment - Note commitment hash
   * @param note - Optional: The full note if available
   * @returns Recovery result with updated note
   * 
   * @example
   * ```typescript
   * const result = await sdk.recoverDeposit({
   *   signature: "5Kn4...",
   *   commitment: "abc123...",
   *   note: myNote // optional if you have it
   * });
   * 
   * if (result.success) {
   *   console.log(`Recovered! Leaf index: ${result.leafIndex}`);
   * }
   * ```
   */
  async recoverDeposit(options: {
    signature: string;
    commitment: string;
    note?: CloakNote;
    onProgress?: (status: string) => void;
  }): Promise<{
    success: boolean;
    leafIndex?: number;
    root?: string;
    slot?: number;
    merkleProof?: {
      pathElements: string[];
      pathIndices: number[];
    };
    note?: CloakNote;
    error?: string;
  }> {
    return this.depositRecovery.recoverDeposit(options);
  }
  
  /**
   * Load all notes from storage
   * 
   * @returns Array of saved notes
   */
  async loadNotes(): Promise<CloakNote[]> {
    const notes = this.storage.loadAllNotes();
    return Array.isArray(notes) ? notes : await notes;
  }
  
  /**
   * Save a note to storage
   * 
   * @param note - Note to save
   */
  async saveNote(note: CloakNote): Promise<void> {
    const result = this.storage.saveNote(note);
    if (result instanceof Promise) {
      await result;
    }
  }
  
  /**
   * Find a note by its commitment
   * 
   * @param commitment - Commitment hash
   * @returns Note if found
   */
  async findNote(commitment: string): Promise<CloakNote | undefined> {
    const notes = await this.loadNotes();
    return findNoteByCommitment(notes, commitment);
  }
  
  /**
   * Import wallet keys from JSON
   * 
   * @param keysJson - JSON string containing keys
   */
  async importWalletKeys(keysJson: string): Promise<void> {
    const keys = importWalletKeys(keysJson);
    this.cloakKeys = keys;
    const result = this.storage.saveKeys(keys);
    if (result instanceof Promise) {
      await result;
    }
  }
  
  /**
   * Export wallet keys to JSON
   * 
   * WARNING: This exports secret keys! Store securely.
   * 
   * @returns JSON string with keys
   */
  exportWalletKeys(): string {
    if (!this.cloakKeys) {
      throw new CloakError("No wallet keys available", "wallet", false);
    }
    return exportWalletKeys(this.cloakKeys);
  }

  /**
   * Get the configuration
   */
  getConfig(): CloakConfig {
    return { ...this.config };
  }

  /**
   * Encode note data for indexer storage
   * 
   * Enhanced version that supports encrypted outputs for v2.0 scanning
   */
  private encodeNote(note: CloakNote, recipientViewKey?: string): string {
    // Use encrypted output if Cloak keys are available
    if (recipientViewKey) {
      return prepareEncryptedOutputForRecipient(note, recipientViewKey);
    }
    
    if (this.cloakKeys) {
      return prepareEncryptedOutput(note, this.cloakKeys);
    }
    
    // Fallback to simple encoding
    return encodeNoteSimple(note);
  }
  
  /**
   * Scan blockchain for notes belonging to this wallet (v2.0 feature)
   * 
   * Requires Cloak keys to be configured in the SDK.
   * Fetches encrypted outputs from the indexer and decrypts notes
   * that belong to this wallet.
   * 
   * @param options - Scanning options
   * @returns Array of discovered notes with metadata
   * 
   * @example
   * ```typescript
   * const notes = await sdk.scanNotes({
   *   onProgress: (current, total) => {
   *     console.log(`Scanning: ${current}/${total}`);
   *   }
   * });
   * 
   * console.log(`Found ${notes.length} notes!`);
   * const totalBalance = notes.reduce((sum, n) => sum + n.amount, 0);
   * ```
   */
  async scanNotes(options?: ScanNotesOptions): Promise<ScannedNote[]> {
    if (!this.cloakKeys) {
      throw new CloakError(
        "Note scanning requires Cloak keys. Initialize SDK with: cloakKeys: generateCloakKeys()",
        "validation",
        false
      );
    }
    
    const startIndex = options?.startIndex ?? 0;
    const batchSize = options?.batchSize ?? 100;
    
    // Get total notes from indexer
    const { next_index: totalNotes } = await this.indexer.getMerkleRoot();
    const endIndex = options?.endIndex ?? (totalNotes > 0 ? totalNotes - 1 : 0);
    
    if (totalNotes === 0 || endIndex < startIndex) {
      return [];
    }
    
    const allEncryptedOutputs: string[] = [];
    
    // Fetch encrypted outputs in batches
    for (let start = startIndex; start <= endIndex; start += batchSize) {
      const end = Math.min(start + batchSize - 1, endIndex);
      
      options?.onProgress?.(start, totalNotes);
      
      const { notes } = await this.indexer.getNotesRange(start, end, batchSize);
      allEncryptedOutputs.push(...notes);
    }
    
    options?.onProgress?.(totalNotes, totalNotes);
    
    // Decrypt notes belonging to this wallet
    const foundNoteData = scanNotesForWallet(
      allEncryptedOutputs,
      this.cloakKeys.view
    );
    
    // Convert to CloakNote objects with metadata
    const scannedNotes: ScannedNote[] = foundNoteData.map((noteData) => ({
      version: "2.0",
      amount: noteData.amount,
      commitment: noteData.commitment,
      sk_spend: noteData.sk_spend,
      r: noteData.r,
      timestamp: Date.now(),
      network: this.config.network || "devnet",
      scannedAt: Date.now(),
    }));
    
    return scannedNotes;
  }
  
  /**
   * Wrap errors with better categorization and user-friendly messages
   * 
   * @private
   */
  private wrapError(error: unknown, context: string): CloakError {
    if (error instanceof CloakError) {
      return error;
    }
    
    const errorMessage = error instanceof Error ? error.message : String(error);
    
    // Duplicate deposit
    if (errorMessage.includes("duplicate key") || errorMessage.includes("already deposited")) {
      return new CloakError(
        "This note was already deposited. The transaction succeeded but the indexer has it recorded. Generate a new note or scan for existing notes.",
        "indexer",
        false,
        error instanceof Error ? error : undefined
      );
    }
    
    // Insufficient funds
    if (errorMessage.includes("insufficient funds") || errorMessage.includes("insufficient lamports")) {
      return new CloakError(
        "Insufficient funds for this transaction. Please check your wallet balance.",
        "wallet",
        false,
        error instanceof Error ? error : undefined
      );
    }
    
    // Merkle tree inconsistency (retryable)
    if (errorMessage.includes("Merkle tree") && errorMessage.includes("inconsistent")) {
      return new CloakError(
        "Indexer is temporarily unavailable. Please try again in a moment.",
        "indexer",
        true,
        error instanceof Error ? error : undefined
      );
    }
    
    // Network timeout (retryable)
    if (errorMessage.includes("timeout") || errorMessage.includes("timed out")) {
      return new CloakError(
        "Network timeout. Please check your connection and try again.",
        "network",
        true,
        error instanceof Error ? error : undefined
      );
    }
    
    // Wallet not connected
    if (errorMessage.includes("not connected") || errorMessage.includes("wallet")) {
      return new CloakError(
        "Wallet not connected. Please connect your wallet first.",
        "wallet",
        false,
        error instanceof Error ? error : undefined
      );
    }
    
    // Proof generation failed
    if (errorMessage.includes("proof") && (errorMessage.includes("failed") || errorMessage.includes("error"))) {
      return new CloakError(
        "Zero-knowledge proof generation failed. This is usually temporary - please try again.",
        "prover",
        true,
        error instanceof Error ? error : undefined
      );
    }
    
    // Relay service error
    if (errorMessage.includes("relay") || errorMessage.includes("withdraw")) {
      return new CloakError(
        "Relay service error. Please try again later.",
        "relay",
        true,
        error instanceof Error ? error : undefined
      );
    }
    
    // Generic error
    return new CloakError(
      `${context}: ${errorMessage}`,
      "network",
      false,
      error instanceof Error ? error : undefined
    );
  }
}
