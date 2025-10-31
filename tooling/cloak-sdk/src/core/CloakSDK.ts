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
} from "./types";
import { generateNote, updateNoteWithDeposit, isWithdrawable } from "./note";
import {
  validateTransfers,
  parseNote,
} from "../utils/validation";
import {
  computeNullifier,
  computeOutputsHash,
  hexToBytes,
  bytesToHex,
  isValidHex,
} from "../utils/crypto";
import { getDistributableAmount } from "../utils/fees";
import { IndexerService } from "../services/IndexerService";
import { ProverService } from "../services/ProverService";
import { RelayService } from "../services/RelayService";
import { createDepositInstruction } from "../solana/instructions";
import { getShieldPoolPDAs } from "../utils/pda";

export const CLOAK_PROGRAM_ID = new PublicKey("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");
const CLOAK_API_URL = 
  // "http://localhost:8000"; 
  "https://api.cloaklabz.xyz";

/**
 * Main Cloak SDK
 *
 * Provides high-level API for interacting with the Cloak protocol,
 * including deposits, withdrawals, and private transfers.
 *
 * @example
 * ```typescript
 * const client = new CloakSDK({
 *   network: "devnet",
 *   keypairBytes: [...],
 * });
 *
 * // Option 1: Deposit only (save note for later)
 * const depositResult = await client.deposit(connection, 1_000_000_000);
 * console.log("Note saved:", depositResult.note);
 * 
 * // Then withdraw using the saved note
 * const withdrawResult = await client.withdraw(connection, depositResult.note, recipientAddress);
 * console.log("Withdrawal complete:", withdrawResult.signature);
 *
 * // Option 2: Private transfer (complete flow: deposit + withdraw)
 * const note = client.generateNote(1_000_000_000);
 * const txResult = await client.privateTransfer(
 *   connection,
 *   note,
 *   [
 *     { recipient: addr1, amount: 500_000_000 },
 *     { recipient: addr2, amount: 492_500_000 }
 *   ]
 * );
 * // privateTransfer automatically deposits if needed, then transfers!
 * ```
 */
export class CloakSDK {
  private config: CloakConfig;
  private keypair: Keypair;
  private indexer: IndexerService;
  private prover: ProverService;
  private relay: RelayService;

  /**
  * Create a new Cloak SDK client
   *
   * @param config - Client configuration
   */
  constructor({keypairBytes, network = "devnet"}: {keypairBytes: Uint8Array, network?: Network}) {
    this.keypair = Keypair.fromSecretKey(keypairBytes);

    this.indexer = new IndexerService(CLOAK_API_URL);
    this.prover = new ProverService(
      CLOAK_API_URL,
      5 * 60 * 1000
    );
    this.relay = new RelayService(CLOAK_API_URL);

    const { pool, commitments, rootsRing, nullifierShard, treasury } = getShieldPoolPDAs();

    this.config = {
      network: network,
      keypairBytes: keypairBytes,
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

    options?.onProgress?.("Creating deposit transaction...");

    // Create deposit instruction
    const commitmentBytes = hexToBytes(note.commitment);
    const depositIx = createDepositInstruction({
      programId: CLOAK_PROGRAM_ID,
      payer: this.keypair.publicKey,
      pool: this.config.poolAddress,
      commitments: this.config.commitmentsAddress,
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
    const encryptedOutput = this.encodeNote(note);
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

    const relayFeeBps = options?.relayFeeBps || 0;

    if (relayFeeBps < 0 || relayFeeBps > 1000) {
      throw new Error("Relay fee must be between 0 and 1000 bps (0-10%)");
    }

    // Calculate distributable amount (after protocol fees)
    const distributableAmount = getDistributableAmount(note.amount);

    // Validate recipients sum to distributable amount
    validateTransfers(recipients, distributableAmount);

    options?.onProgress?.("Fetching Merkle proof...");

    // Get current Merkle proof (in case tree has grown)
    const merkleProof = await this.indexer.getMerkleProof(note.leafIndex!);

    // Use historical root from note (or current if not available)
    const merkleRoot = note.root || merkleProof.root!;

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
    const proofResult = await this.prover.generateProof(proofInputs);

    if (!proofResult.success || !proofResult.proof || !proofResult.publicInputs) {
      // The ProverService already extracts and formats the error message
      let errorMessage = proofResult.error || "Proof generation failed";
      
      // Remove redundant "Proof generation failed:" prefix if present
      if (errorMessage.startsWith("Proof generation failed: ")) {
        errorMessage = errorMessage.substring("Proof generation failed: ".length);
      }
      
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
        feeBps: relayFeeBps,
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
   * Generate a new note without depositing
   *
   * @param amountLamports - Amount for the note
   * @returns New note (not yet deposited)
   */
  generateNote(amountLamports: number): CloakNote {
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
   * Get the configuration
   */
  getConfig(): CloakConfig {
    return { ...this.config };
  }

  /**
   * Encode note data for indexer storage
   */
  private encodeNote(note: CloakNote): string {
    const data = {
      amount: note.amount,
      r: note.r,
      sk_spend: note.sk_spend,
    };

    // Base64 encode
    const json = JSON.stringify(data);

    if (typeof Buffer !== "undefined") {
      return Buffer.from(json).toString("base64");
    } else if (typeof btoa !== "undefined") {
      return btoa(json);
    } else {
      throw new Error("No base64 encoding method available");
    }
  }
}
