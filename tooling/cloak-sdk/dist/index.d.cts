import { PublicKey, Transaction, Connection, Keypair, SendOptions, TransactionInstruction } from '@solana/web3.js';

/**
 * Supported Solana networks
 */
type Network = "localnet" | "devnet" | "mainnet" | "testnet";
/**
 * Minimal wallet adapter interface
 * Compatible with @solana/wallet-adapter-base
 */
interface WalletAdapter {
    publicKey: PublicKey | null;
    signTransaction?<T extends Transaction>(transaction: T): Promise<T>;
    signAllTransactions?<T extends Transaction>(transactions: T[]): Promise<T[]>;
    sendTransaction?(transaction: Transaction, connection: any, options?: any): Promise<string>;
}
/**
 * Cloak-specific error with categorization
 */
declare class CloakError extends Error {
    category: "network" | "indexer" | "prover" | "relay" | "validation" | "wallet" | "environment";
    retryable: boolean;
    originalError?: Error | undefined;
    constructor(message: string, category: "network" | "indexer" | "prover" | "relay" | "validation" | "wallet" | "environment", retryable?: boolean, originalError?: Error | undefined);
}
/**
 * Cloak Note - Represents a private transaction commitment
 *
 * A note contains all the information needed to withdraw funds from the Cloak protocol.
 * Keep this safe and secret - anyone with access to this note can withdraw the funds!
 */
interface CloakNote {
    /** Protocol version */
    version: string;
    /** Amount in lamports */
    amount: number;
    /** Blake3 commitment hash (hex) */
    commitment: string;
    /** Spending secret key (hex, 64 chars) */
    sk_spend: string;
    /** Randomness value (hex, 64 chars) */
    r: string;
    /** Transaction signature from deposit (optional until deposited) */
    depositSignature?: string;
    /** Solana slot when deposited (optional until deposited) */
    depositSlot?: number;
    /** Index in the Merkle tree (optional until deposited) */
    leafIndex?: number;
    /** Historical Merkle root at time of deposit (optional until deposited) */
    root?: string;
    /** Merkle proof at time of deposit (optional until deposited) */
    merkleProof?: MerkleProof;
    /** Creation timestamp */
    timestamp: number;
    /** Network where this note was created */
    network: Network;
}
/**
 * Merkle proof for a leaf in the commitment tree
 */
interface MerkleProof {
    /** Sibling hashes along the path (hex strings) */
    pathElements: string[];
    /** Path directions (0 = left, 1 = right) */
    pathIndices: number[];
    /** Optional root for backward compatibility */
    root?: string;
}
/**
 * Transfer recipient - used in privateTransfer
 */
interface Transfer {
    /** Recipient's Solana public key */
    recipient: PublicKey;
    /** Amount to send in lamports */
    amount: number;
}
/**
 * Type-safe array with maximum length constraint
 * Used to enforce 1-5 recipients in privateTransfer
 */
type MaxLengthArray<T, Max extends number, A extends T[] = []> = A['length'] extends Max ? A : A | MaxLengthArray<T, Max, [T, ...A]>;
/**
 * Result from a private transfer
 */
interface TransferResult {
    /** Solana transaction signature */
    signature: string;
    /** Recipients and amounts that were sent */
    outputs: Array<{
        recipient: string;
        amount: number;
    }>;
    /** Nullifier used (prevents double-spending) */
    nullifier: string;
    /** Merkle root that was proven against */
    root: string;
}
/**
 * Result from a deposit operation
 */
interface DepositResult {
    /** The created note (save this securely!) */
    note: CloakNote;
    /** Solana transaction signature */
    signature: string;
    /** Leaf index in the Merkle tree */
    leafIndex: number;
    /** Current Merkle root */
    root: string;
}
/**
 * Configuration for the Cloak SDK
 */
interface CloakConfig {
    /** Solana network */
    network?: Network;
    /**
     * Keypair bytes for signing (deprecated - use wallet instead)
     * @deprecated Use wallet parameter for better integration
     */
    keypairBytes?: Uint8Array;
    /**
     * Wallet adapter for signing transactions
     * Required unless using keypairBytes
     */
    wallet?: WalletAdapter;
    /**
     * Cloak key pair for v2.0 features (note scanning, encryption)
     * Optional but recommended for full functionality
     */
    cloakKeys?: any;
    /**
     * Single API base URL for both Indexer and Relay services.
     * If provided, it will be used for both services and overrides
     * any `indexerUrl` or `relayUrl` values.
     */
    apiUrl?: string;
    /** Optional: Proof generation timeout in milliseconds (default: 5 minutes) */
    proofTimeout?: number;
    /** Optional: Program ID (defaults to Cloak mainnet program) */
    programId?: PublicKey;
    /** Optional: Pool account address (auto-derived from program ID if not provided) */
    poolAddress?: PublicKey;
    /** Optional: Commitments account address (auto-derived if not provided) */
    commitmentsAddress?: PublicKey;
    /** Optional: Roots ring account address (auto-derived if not provided) */
    rootsRingAddress?: PublicKey;
    /** Optional: Nullifier shard account address (auto-derived if not provided) */
    nullifierShardAddress?: PublicKey;
    /** Optional: Treasury account address (auto-derived if not provided) */
    treasuryAddress?: PublicKey;
}
/**
 * Deposit progress status
 */
type DepositStatus = "generating_note" | "creating_transaction" | "simulating" | "sending" | "confirming" | "submitting_to_indexer" | "fetching_proof" | "complete";
/**
 * Options for deposit operation
 */
interface DepositOptions {
    /** Optional callback for progress updates with detailed status */
    onProgress?: (status: DepositStatus | string, details?: {
        message?: string;
        step?: number;
        totalSteps?: number;
        retryAttempt?: number;
    }) => void;
    /** Callback when transaction is sent (before confirmation) */
    onTransactionSent?: (signature: string) => void;
    /** Callback when transaction is confirmed */
    onConfirmed?: (signature: string, slot: number) => void;
    /** Skip simulation (default: false) */
    skipPreflight?: boolean;
    /** Compute units to request (default: auto) */
    computeUnits?: number;
    /** Priority fee in micro-lamports (default: 0) */
    priorityFee?: number;
    /**
     * Optional: Encrypt output for specific recipient's view key
     * If not provided, encrypts for the wallet's own view key (for self-scanning)
     */
    recipientViewKey?: string;
    /**
     * Skip privacy warning on testnet (default: false)
     * Warning: Only skip if you understand the privacy limitations
     */
    skipPrivacyWarning?: boolean;
}
/**
 * Options for private transfer/withdraw operation
 */
interface TransferOptions {
    /**
     * Optional callback for progress updates
     * Note: relayFeeBps is automatically calculated from protocol fees
     */
    onProgress?: (status: string) => void;
    /** Optional callback for proof generation progress (0-100) */
    onProofProgress?: (percent: number) => void;
}
/**
 * Options for withdrawal (convenience method with single recipient)
 */
interface WithdrawOptions extends TransferOptions {
    /** Whether to withdraw full amount minus fees (default: true) */
    withdrawAll?: boolean;
    /** Specific amount to withdraw in lamports (if not withdrawing all) */
    amount?: number;
}
/**
 * SP1 proof inputs for zero-knowledge proof generation
 */
interface SP1ProofInputs {
    privateInputs: {
        amount: number;
        r: string;
        sk_spend: string;
        leaf_index: number;
        merkle_path: {
            path_elements: string[];
            path_indices: number[];
        };
    };
    publicInputs: {
        root: string;
        nf: string;
        outputs_hash: string;
        amount: number;
    };
    outputs: Array<{
        address: string;
        amount: number;
    }>;
}
/**
 * Result from proof generation
 */
interface SP1ProofResult {
    success: boolean;
    proof?: string;
    publicInputs?: string;
    generationTimeMs: number;
    error?: string;
}
/**
 * Merkle root response from indexer
 */
interface MerkleRootResponse {
    root: string;
    next_index: number;
}
/**
 * Transaction status from relay service
 */
interface TxStatus {
    status: "pending" | "processing" | "completed" | "failed";
    txId?: string;
    error?: string;
}
/**
 * Note scanning options
 */
interface ScanNotesOptions {
    /** Start index for scanning (default: 0) */
    startIndex?: number;
    /** End index for scanning (default: latest) */
    endIndex?: number;
    /** Batch size for fetching notes (default: 100) */
    batchSize?: number;
    /** Progress callback */
    onProgress?: (current: number, total: number) => void;
}
/**
 * Scanned note result with metadata
 */
interface ScannedNote extends CloakNote {
    /** When this note was discovered */
    scannedAt: number;
    /** Whether this note has been spent (nullifier check) */
    isSpent?: boolean;
}

/**
 * Cloak Key Hierarchy (v2.0)
 *
 * Implements view/spend key separation for privacy-preserving note scanning:
 * - Master Seed → Spend Key → View Key → Public View Key
 * - Enables note discovery without exposing spending authority
 * - Compatible with v1.0 notes (backward compatible)
 */
interface MasterKey {
    seed: Uint8Array;
    seedHex: string;
}
interface SpendKey {
    sk_spend: Uint8Array;
    pk_spend: Uint8Array;
    sk_spend_hex: string;
    pk_spend_hex: string;
}
interface ViewKey {
    vk_secret: Uint8Array;
    pvk: Uint8Array;
    vk_secret_hex: string;
    pvk_hex: string;
}
interface CloakKeyPair {
    master: MasterKey;
    spend: SpendKey;
    view: ViewKey;
}
interface EncryptedNote {
    ephemeral_pk: string;
    ciphertext: string;
    nonce: string;
}
interface NoteData {
    amount: number;
    r: string;
    sk_spend: string;
    commitment: string;
}
/**
 * Generate a new master seed from secure randomness
 */
declare function generateMasterSeed(): MasterKey;
/**
 * Derive spend key from master seed
 */
declare function deriveSpendKey(masterSeed: Uint8Array): SpendKey;
/**
 * Derive view key from spend key
 */
declare function deriveViewKey(sk_spend: Uint8Array): ViewKey;
/**
 * Generate complete key hierarchy from master seed
 */
declare function generateCloakKeys(masterSeed?: Uint8Array): CloakKeyPair;
/**
 * Encrypt note data for a recipient using their public view key
 *
 * Uses X25519 ECDH + XSalsa20-Poly1305 authenticated encryption
 */
declare function encryptNoteForRecipient(noteData: NoteData, recipientPvk: Uint8Array): EncryptedNote;
/**
 * Attempt to decrypt an encrypted note using view key
 *
 * Returns null if decryption fails (note doesn't belong to this wallet)
 * Returns NoteData if successful
 */
declare function tryDecryptNote(encryptedNote: EncryptedNote, viewKey: ViewKey): NoteData | null;
/**
 * Scan a batch of encrypted outputs and return notes belonging to this wallet
 */
declare function scanNotesForWallet(encryptedOutputs: string[], // Base64 encoded encrypted note JSON
viewKey: ViewKey): NoteData[];
/**
 * Export keys for backup (WARNING: contains secrets!)
 */
declare function exportKeys(keys: CloakKeyPair): string;
/**
 * Import keys from backup
 */
declare function importKeys(exported: string): CloakKeyPair;

/**
 * Storage Interface
 *
 * Defines a pluggable storage interface for notes and keys.
 * Applications can implement their own storage (localStorage, IndexedDB, file system, etc.)
 */

/**
 * Storage adapter interface
 *
 * Implement this interface to provide custom storage for notes and keys.
 * The SDK will use this adapter for all persistence operations.
 */
interface StorageAdapter {
    /**
     * Save a note
     */
    saveNote(note: CloakNote): Promise<void> | void;
    /**
     * Load all notes
     */
    loadAllNotes(): Promise<CloakNote[]> | CloakNote[];
    /**
     * Update a note
     */
    updateNote(commitment: string, updates: Partial<CloakNote>): Promise<void> | void;
    /**
     * Delete a note
     */
    deleteNote(commitment: string): Promise<void> | void;
    /**
     * Clear all notes
     */
    clearAllNotes(): Promise<void> | void;
    /**
     * Save wallet keys
     */
    saveKeys(keys: CloakKeyPair): Promise<void> | void;
    /**
     * Load wallet keys
     */
    loadKeys(): Promise<CloakKeyPair | null> | CloakKeyPair | null;
    /**
     * Delete wallet keys
     */
    deleteKeys(): Promise<void> | void;
}
/**
 * In-memory storage adapter (default, no persistence)
 *
 * Useful for testing or when storage is handled externally
 */
declare class MemoryStorageAdapter implements StorageAdapter {
    private notes;
    private keys;
    saveNote(note: CloakNote): void;
    loadAllNotes(): CloakNote[];
    updateNote(commitment: string, updates: Partial<CloakNote>): void;
    deleteNote(commitment: string): void;
    clearAllNotes(): void;
    saveKeys(keys: CloakKeyPair): void;
    loadKeys(): CloakKeyPair | null;
    deleteKeys(): void;
}
/**
 * Browser localStorage adapter (optional, for browser environments)
 *
 * Only use this if you're in a browser environment and want localStorage persistence.
 * Import from a separate browser-specific module.
 */
declare class LocalStorageAdapter implements StorageAdapter {
    private notesKey;
    private keysKey;
    constructor(notesKey?: string, keysKey?: string);
    private getStorage;
    saveNote(note: CloakNote): void;
    loadAllNotes(): CloakNote[];
    updateNote(commitment: string, updates: Partial<CloakNote>): void;
    deleteNote(commitment: string): void;
    clearAllNotes(): void;
    saveKeys(keys: CloakKeyPair): void;
    loadKeys(): CloakKeyPair | null;
    deleteKeys(): void;
}

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
declare class CloakSDK {
    private config;
    private keypair;
    private cloakKeys?;
    private indexer;
    private prover;
    private relay;
    private depositRecovery;
    private storage;
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
        apiUrl?: string;
        storage?: StorageAdapter;
    });
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
    deposit(connection: Connection, amountOrNote: number | CloakNote, options?: DepositOptions): Promise<DepositResult>;
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
    privateTransfer(connection: Connection, note: CloakNote, recipients: MaxLengthArray<Transfer, 5>, options?: TransferOptions): Promise<TransferResult>;
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
    withdraw(connection: Connection, note: CloakNote, recipient: PublicKey, options?: WithdrawOptions): Promise<TransferResult>;
    /**
     * Generate a new note without depositing
     *
     * @param amountLamports - Amount for the note
     * @param useWalletKeys - Whether to use wallet keys (v2.0 recommended)
     * @returns New note (not yet deposited)
     */
    generateNote(amountLamports: number, useWalletKeys?: boolean): CloakNote;
    /**
     * Parse a note from JSON string
     *
     * @param jsonString - JSON representation
     * @returns Parsed note
     */
    parseNote(jsonString: string): CloakNote;
    /**
     * Export a note to JSON string
     *
     * @param note - Note to export
     * @param pretty - Format with indentation
     * @returns JSON string
     */
    exportNote(note: CloakNote, pretty?: boolean): string;
    /**
     * Check if a note is ready for withdrawal
     *
     * @param note - Note to check
     * @returns True if withdrawable
     */
    isWithdrawable(note: CloakNote): boolean;
    /**
     * Get Merkle proof for a leaf index
     *
     * @param leafIndex - Leaf index in tree
     * @returns Merkle proof
     */
    getMerkleProof(leafIndex: number): Promise<MerkleProof>;
    /**
     * Get current Merkle root
     *
     * @returns Current root hash
     */
    getCurrentRoot(): Promise<string>;
    /**
     * Get transaction status from relay service
     *
     * @param requestId - Request ID from previous submission
     * @returns Current status
     */
    getTransactionStatus(requestId: string): Promise<TxStatus>;
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
    recoverDeposit(options: {
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
    }>;
    /**
     * Load all notes from storage
     *
     * @returns Array of saved notes
     */
    loadNotes(): Promise<CloakNote[]>;
    /**
     * Save a note to storage
     *
     * @param note - Note to save
     */
    saveNote(note: CloakNote): Promise<void>;
    /**
     * Find a note by its commitment
     *
     * @param commitment - Commitment hash
     * @returns Note if found
     */
    findNote(commitment: string): Promise<CloakNote | undefined>;
    /**
     * Import wallet keys from JSON
     *
     * @param keysJson - JSON string containing keys
     */
    importWalletKeys(keysJson: string): Promise<void>;
    /**
     * Export wallet keys to JSON
     *
     * WARNING: This exports secret keys! Store securely.
     *
     * @returns JSON string with keys
     */
    exportWalletKeys(): string;
    /**
     * Get the configuration
     */
    getConfig(): CloakConfig;
    /**
     * Encode note data for indexer storage
     *
     * Enhanced version that supports encrypted outputs for v2.0 scanning
     */
    private encodeNote;
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
    scanNotes(options?: ScanNotesOptions): Promise<ScannedNote[]>;
    /**
     * Wrap errors with better categorization and user-friendly messages
     *
     * @private
     */
    private wrapError;
}

/**
 * Serialize a note to JSON string
 *
 * @param note - Note to serialize
 * @param pretty - Whether to format with indentation (default: false)
 * @returns JSON string
 *
 * @example
 * ```typescript
 * const json = serializeNote(note, true);
 * console.log(json);
 * // Or save to file, copy to clipboard, etc.
 * ```
 */
declare function serializeNote(note: CloakNote, pretty?: boolean): string;
/**
 * Export note as downloadable JSON (browser only)
 *
 * @param note - Note to export
 * @param filename - Optional custom filename
 */
declare function downloadNote(note: CloakNote, filename?: string): void;
/**
 * Copy note to clipboard as JSON (browser only)
 *
 * @param note - Note to copy
 * @returns Promise that resolves when copied
 */
declare function copyNoteToClipboard(note: CloakNote): Promise<void>;

/**
 * Note Manager
 *
 * Standalone note management - no browser dependencies.
 * Storage is handled externally via StorageAdapter.
 *
 * Core functionality:
 * - Generate notes (v1.0 and v2.0)
 * - Parse and validate notes
 * - Note utilities (formatting, fees, etc.)
 * - Key management (without storage)
 */

/**
 * Generate a new note without wallet keys (legacy v1.0)
 * @deprecated Use generateNoteFromWallet instead for enhanced security
 */
declare function generateNote(amountLamports: number, network?: Network): CloakNote;
/**
 * Generate a note using wallet's spend key (v2.0 recommended)
 */
declare function generateNoteFromWallet(amountLamports: number, keys: CloakKeyPair, network?: Network): CloakNote;
/**
 * Parse and validate a note from JSON string
 */
declare function parseNote(jsonString: string): CloakNote;
/**
 * Export note to JSON string
 */
declare function exportNote(note: CloakNote, pretty?: boolean): string;
/**
 * Check if a note is withdrawable (has been deposited)
 */
declare function isWithdrawable(note: CloakNote): boolean;
/**
 * Update note with deposit information
 * Returns a new note object with deposit info added
 */
declare function updateNoteWithDeposit(note: CloakNote, depositInfo: {
    signature: string;
    slot: number;
    leafIndex: number;
    root: string;
    merkleProof: {
        pathElements: string[];
        pathIndices: number[];
    };
}): CloakNote;
/**
 * Find note by commitment from an array
 */
declare function findNoteByCommitment(notes: CloakNote[], commitment: string): CloakNote | undefined;
/**
 * Filter notes by network
 */
declare function filterNotesByNetwork(notes: CloakNote[], network: Network): CloakNote[];
/**
 * Filter notes that can be withdrawn
 */
declare function filterWithdrawableNotes(notes: CloakNote[]): CloakNote[];
/**
 * Export keys to JSON string
 * WARNING: This exports secret keys! Store securely.
 */
declare function exportWalletKeys(keys: CloakKeyPair): string;
/**
 * Import keys from JSON string
 */
declare function importWalletKeys(keysJson: string): CloakKeyPair;
/**
 * Get public view key from keys
 */
declare function getPublicViewKey(keys: CloakKeyPair): string;
/**
 * Get view key from keys
 */
declare function getViewKey(keys: CloakKeyPair): ViewKey;
/**
 * Get recipient amount after fees
 */
declare function getRecipientAmount(amountLamports: number): number;

/**
 * Generate a Blake3 commitment for a note
 *
 * Formula: Blake3(amount || r || pk_spend)
 * where pk_spend = Blake3(sk_spend)
 *
 * @param amountLamports - Amount in lamports
 * @param r - Randomness bytes (32 bytes)
 * @param skSpend - Spending secret key bytes (32 bytes)
 * @returns Commitment hash (32 bytes)
 */
declare function generateCommitment(amountLamports: number, r: Uint8Array, skSpend: Uint8Array): Uint8Array;
/**
 * Compute nullifier from spending key and leaf index
 *
 * Formula: Blake3(sk_spend || leaf_index)
 *
 * The nullifier prevents double-spending by proving knowledge of sk_spend
 * without revealing it.
 *
 * @param skSpend - Spending secret key bytes (32 bytes)
 * @param leafIndex - Index in the Merkle tree
 * @returns Nullifier hash (32 bytes)
 */
declare function computeNullifier(skSpend: Uint8Array, leafIndex: number): Uint8Array;
/**
 * Compute outputs hash from recipients and amounts
 *
 * Formula: Blake3(recipient1 || amount1 || recipient2 || amount2 || ...)
 *
 * @param outputs - Array of {recipient: PublicKey, amount: number}
 * @returns Outputs hash (32 bytes)
 */
declare function computeOutputsHash(outputs: Array<{
    recipient: PublicKey;
    amount: number;
}>): Uint8Array;
/**
 * Convert hex string to Uint8Array
 *
 * @param hex - Hex string (with or without 0x prefix)
 * @returns Decoded bytes
 */
declare function hexToBytes(hex: string): Uint8Array;
/**
 * Convert Uint8Array to hex string
 *
 * @param bytes - Bytes to encode
 * @param prefix - Whether to include 0x prefix (default: false)
 * @returns Hex-encoded string
 */
declare function bytesToHex(bytes: Uint8Array, prefix?: boolean): string;
/**
 * Generate random bytes using Web Crypto API
 *
 * @param length - Number of bytes to generate
 * @returns Random bytes
 */
declare function randomBytes(length: number): Uint8Array;
/**
 * Validate a hex string format
 *
 * @param hex - Hex string to validate
 * @param expectedLength - Expected length in bytes (optional)
 * @returns True if valid hex string
 */
declare function isValidHex(hex: string, expectedLength?: number): boolean;

/**
 * Fee calculation utilities for Cloak Protocol
 *
 * The protocol charges a fixed fee plus a variable percentage fee
 * to prevent sybil attacks and cover operational costs.
 */
/** Fixed fee: 0.0025 SOL (2.5M lamports) */
declare const FIXED_FEE_LAMPORTS = 2500000;
/** Variable fee rate: 0.5% (5 basis points per 1000) */
declare const VARIABLE_FEE_RATE: number;
/** Lamports per SOL */
declare const LAMPORTS_PER_SOL = 1000000000;
/**
 * Calculate the total protocol fee for a given amount
 *
 * Formula: FIXED_FEE + floor((amount * 5) / 1000)
 *
 * @param amountLamports - Amount in lamports
 * @returns Total fee in lamports
 *
 * @example
 * ```typescript
 * const fee = calculateFee(1_000_000_000); // 1 SOL
 * // Returns: 2_500_000 (fixed) + 5_000_000 (0.5%) = 7_500_000 lamports
 * ```
 */
declare function calculateFee(amountLamports: number): number;
/**
 * Calculate the distributable amount after protocol fees
 *
 * This is the amount available to send to recipients.
 *
 * @param amountLamports - Total note amount in lamports
 * @returns Amount available for recipients in lamports
 *
 * @example
 * ```typescript
 * const distributable = getDistributableAmount(1_000_000_000);
 * // Returns: 1_000_000_000 - 7_500_000 = 992_500_000 lamports
 * ```
 */
declare function getDistributableAmount(amountLamports: number): number;
/**
 * Format lamports as SOL string
 *
 * @param lamports - Amount in lamports
 * @param decimals - Number of decimal places (default: 9)
 * @returns Formatted string (e.g., "1.000000000")
 *
 * @example
 * ```typescript
 * formatAmount(1_000_000_000); // "1.000000000"
 * formatAmount(1_500_000_000); // "1.500000000"
 * formatAmount(123_456_789, 4); // "0.1235"
 * ```
 */
declare function formatAmount(lamports: number, decimals?: number): string;
/**
 * Parse SOL string to lamports
 *
 * @param sol - SOL amount as string (e.g., "1.5")
 * @returns Amount in lamports
 * @throws Error if invalid format
 *
 * @example
 * ```typescript
 * parseAmount("1.5"); // 1_500_000_000
 * parseAmount("0.001"); // 1_000_000
 * ```
 */
declare function parseAmount(sol: string): number;
/**
 * Validate that outputs sum equals expected amount
 *
 * @param outputs - Array of output amounts
 * @param expectedTotal - Expected total amount
 * @returns True if amounts match
 */
declare function validateOutputsSum(outputs: Array<{
    amount: number;
}>, expectedTotal: number): boolean;
/**
 * Calculate relay fee from basis points
 *
 * @param amountLamports - Amount in lamports
 * @param feeBps - Fee in basis points (100 bps = 1%)
 * @returns Relay fee in lamports
 *
 * @example
 * ```typescript
 * calculateRelayFee(1_000_000_000, 50); // 0.5% = 5_000_000 lamports
 * ```
 */
declare function calculateRelayFee(amountLamports: number, feeBps: number): number;

/**
 * Validate a Solana public key
 *
 * @param address - Address string to validate
 * @returns True if valid Solana address
 */
declare function isValidSolanaAddress(address: string): boolean;
/**
 * Validate a Cloak note structure
 *
 * @param note - Note object to validate
 * @throws Error if invalid
 */
declare function validateNote(note: any): asserts note is CloakNote;
/**
 * Validate that a note is ready for withdrawal
 *
 * @param note - Note to validate
 * @throws Error if note cannot be used for withdrawal
 */
declare function validateWithdrawableNote(note: CloakNote): void;
/**
 * Validate transfer recipients
 *
 * @param recipients - Array of transfers to validate
 * @param totalAmount - Total amount available
 * @throws Error if invalid
 */
declare function validateTransfers(recipients: Array<{
    recipient: PublicKey;
    amount: number;
}>, totalAmount: number): void;

/**
 * Network Utilities
 *
 * Helper functions for network detection and configuration
 */

/**
 * Detect network from RPC URL
 *
 * Attempts to detect the Solana network from common RPC URL patterns.
 * Falls back to devnet if unable to detect.
 */
declare function detectNetworkFromRpcUrl(rpcUrl?: string): Network;
/**
 * Get the standard RPC URL for a network
 */
declare function getRpcUrlForNetwork(network: Network): string;
/**
 * Validate RPC URL format
 */
declare function isValidRpcUrl(url: string): boolean;
/**
 * Get explorer URL for a transaction
 */
declare function getExplorerUrl(signature: string, network?: Network): string;
/**
 * Get explorer URL for an address
 */
declare function getAddressExplorerUrl(address: string, network?: Network): string;

/**
 * Error Utilities
 *
 * Helper functions for parsing and presenting user-friendly error messages
 */

/**
 * Parse transaction error and return user-friendly message
 *
 * Attempts to extract meaningful error information from various
 * error formats (program errors, RPC errors, custom errors)
 */
declare function parseTransactionError(error: any): string;
/**
 * Create a CloakError with appropriate categorization
 */
declare function createCloakError(error: unknown, _context: string): CloakError;
/**
 * Format error for logging
 */
declare function formatErrorForLogging(error: unknown): string;

/**
 * Response from notes range query
 */
interface NotesRangeResponse {
    notes: string[];
    has_more: boolean;
    total: number;
    start: number;
    end: number;
}
/**
 * Indexer Service Client
 *
 * Provides access to the Cloak Indexer API for querying the Merkle tree
 * and registering deposits.
 */
declare class IndexerService {
    private baseUrl;
    /**
     * Create a new Indexer Service client
     *
     * @param baseUrl - Indexer API base URL
     */
    constructor(baseUrl: string);
    /**
     * Get current Merkle root and next available index
     *
     * @returns Current root and next index
     *
     * @example
     * ```typescript
     * const { root, next_index } = await indexer.getMerkleRoot();
     * console.log(`Current root: ${root}, Next index: ${next_index}`);
     * ```
     */
    getMerkleRoot(): Promise<MerkleRootResponse>;
    /**
     * Get Merkle proof for a specific leaf
     *
     * @param leafIndex - Index of the leaf in the tree
     * @returns Merkle proof with path elements and indices
     *
     * @example
     * ```typescript
     * const proof = await indexer.getMerkleProof(42);
     * console.log(`Proof has ${proof.pathElements.length} siblings`);
     * ```
     */
    getMerkleProof(leafIndex: number): Promise<MerkleProof>;
    /**
     * Get notes in a specific range
     *
     * Useful for scanning the tree or fetching notes in batches.
     *
     * @param start - Start index (inclusive)
     * @param end - End index (inclusive)
     * @param limit - Maximum number of notes to return (default: 100)
     * @returns Notes in the range
     *
     * @example
     * ```typescript
     * const { notes, has_more } = await indexer.getNotesRange(0, 99, 100);
     * console.log(`Fetched ${notes.length} notes`);
     * ```
     */
    getNotesRange(start: number, end: number, limit?: number): Promise<NotesRangeResponse>;
    /**
     * Get all notes from the tree
     *
     * Fetches all notes in batches. Use with caution for large trees.
     *
     * @param batchSize - Size of each batch (default: 100)
     * @returns All encrypted notes
     *
     * @example
     * ```typescript
     * const allNotes = await indexer.getAllNotes();
     * console.log(`Total notes: ${allNotes.length}`);
     * ```
     */
    getAllNotes(batchSize?: number): Promise<string[]>;
    /**
     * Submit a deposit to the indexer
     *
     * Registers a new deposit transaction with the indexer, which will
     * return the leaf index and current root.
     *
     * @param params - Deposit parameters
     * @returns Success response with leaf index and root
     *
     * @example
     * ```typescript
     * const result = await indexer.submitDeposit({
     *   leafCommit: note.commitment,
     *   encryptedOutput: btoa(JSON.stringify(noteData)),
     *   txSignature: signature,
     *   slot: txSlot
     * });
     * console.log(`Leaf index: ${result.leafIndex}`);
     * ```
     */
    submitDeposit(params: {
        leafCommit: string;
        encryptedOutput: string;
        txSignature: string;
        slot: number;
    }): Promise<{
        success: boolean;
        leafIndex?: number;
        root?: string;
    }>;
    /**
     * Check indexer health
     *
     * @returns Health status
     */
    healthCheck(): Promise<{
        status: string;
    }>;
}

/**
 * Options for proof generation
 */
interface ProofGenerationOptions {
    /** Progress callback - called with percentage (0-100) */
    onProgress?: (progress: number) => void;
    /** Called when proof generation starts */
    onStart?: () => void;
    /** Called on successful proof generation */
    onSuccess?: (result: SP1ProofResult) => void;
    /** Called on error */
    onError?: (error: string) => void;
    /** Custom timeout in milliseconds */
    timeout?: number;
}
/**
 * Prover Service Client
 *
 * Handles zero-knowledge proof generation via the backend prover service.
 *
 * ⚠️ PRIVACY WARNING: This implementation sends private inputs to a backend service.
 * For production use with full privacy, consider client-side proof generation.
 */
declare class ProverService {
    private indexerUrl;
    private timeout;
    /**
     * Create a new Prover Service client
     *
     * @param indexerUrl - Indexer/Prover service base URL
     * @param timeout - Proof generation timeout in ms (default: 5 minutes)
     */
    constructor(indexerUrl: string, timeout?: number);
    /**
     * Generate a zero-knowledge proof for withdrawal
     *
     * This process typically takes 30-180 seconds depending on the backend.
     *
     * @param inputs - Circuit inputs (private + public + outputs)
     * @param options - Optional progress tracking and callbacks
     * @returns Proof result with hex-encoded proof and public inputs
     *
     * @example
     * ```typescript
     * const result = await prover.generateProof(inputs);
     * if (result.success) {
     *   console.log(`Proof: ${result.proof}`);
     * }
     * ```
     *
     * @example
     * ```typescript
     * // With progress tracking
     * const result = await prover.generateProof(inputs, {
     *   onProgress: (progress) => console.log(`Progress: ${progress}%`),
     *   onStart: () => console.log("Starting proof generation..."),
     *   onSuccess: (result) => console.log("Proof generated!"),
     *   onError: (error) => console.error("Failed:", error)
     * });
     * ```
     */
    generateProof(inputs: SP1ProofInputs, options?: ProofGenerationOptions): Promise<SP1ProofResult>;
    /**
     * Check if the prover service is available
     *
     * @returns True if service is healthy
     */
    healthCheck(): Promise<boolean>;
    /**
     * Get the configured timeout
     */
    getTimeout(): number;
    /**
     * Set a new timeout
     */
    setTimeout(timeout: number): void;
}

/**
 * Relay Service Client
 *
 * Handles submission of withdrawal transactions through a relay service
 * that pays for transaction fees and submits the transaction on-chain.
 */
declare class RelayService {
    private baseUrl;
    /**
     * Create a new Relay Service client
     *
     * @param baseUrl - Relay service base URL
     */
    constructor(baseUrl: string);
    /**
     * Submit a withdrawal transaction via relay
     *
     * The relay service will validate the proof, pay for transaction fees,
     * and submit the transaction on-chain.
     *
     * @param params - Withdrawal parameters
     * @param onStatusUpdate - Optional callback for status updates
     * @returns Transaction signature when completed
     *
     * @example
     * ```typescript
     * const signature = await relay.submitWithdraw({
     *   proof: proofHex,
     *   publicInputs: { root, nf, outputs_hash, amount },
     *   outputs: [{ recipient: addr, amount: lamports }],
     *   feeBps: 50
     * }, (status) => console.log(`Status: ${status}`));
     * console.log(`Transaction: ${signature}`);
     * ```
     */
    submitWithdraw(params: {
        proof: string;
        publicInputs: {
            root: string;
            nf: string;
            outputs_hash: string;
            amount: number;
        };
        outputs: Array<{
            recipient: string;
            amount: number;
        }>;
        feeBps: number;
    }, onStatusUpdate?: (status: string) => void): Promise<string>;
    /**
     * Poll for withdrawal completion
     *
     * @param requestId - Request ID from relay service
     * @param onStatusUpdate - Optional callback for status updates
     * @returns Transaction signature when completed
     */
    private pollForCompletion;
    /**
     * Get transaction status
     *
     * @param requestId - Request ID from previous submission
     * @returns Current status
     *
     * @example
     * ```typescript
     * const status = await relay.getStatus(requestId);
     * console.log(`Status: ${status.status}`);
     * if (status.status === 'completed') {
     *   console.log(`TX: ${status.txId}`);
     * }
     * ```
     */
    getStatus(requestId: string): Promise<TxStatus>;
    /**
     * Convert bytes to base64 string
     */
    private bytesToBase64;
    /**
     * Sleep utility
     */
    private sleep;
}

/**
 * Deposit Recovery Service
 *
 * Handles recovery of deposits that completed on-chain but failed
 * to finalize with the indexer (e.g., browser crash, network failure)
 */

interface RecoveryOptions {
    /** Transaction signature to recover */
    signature: string;
    /** Note commitment hash */
    commitment: string;
    /** Optional: The full note if available */
    note?: CloakNote;
    /** Callback for progress updates */
    onProgress?: (status: string) => void;
}
interface RecoveryResult {
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
}
/**
 * Service for recovering incomplete deposits
 */
declare class DepositRecoveryService {
    private indexer;
    private apiUrl;
    constructor(indexer: IndexerService, apiUrl: string);
    /**
     * Recover a deposit that completed on-chain but failed to register
     *
     * @param options Recovery options
     * @returns Recovery result with updated note
     */
    recoverDeposit(options: RecoveryOptions): Promise<RecoveryResult>;
    /**
     * Check if a deposit already exists in the indexer
     *
     * @private
     */
    private checkExistingDeposit;
    /**
     * Finalize a deposit via server API (alternative recovery method)
     *
     * This method calls a server-side endpoint that can handle
     * the recovery process with elevated permissions.
     */
    finalizeDepositViaServer(signature: string, commitment: string, encryptedOutput?: string): Promise<RecoveryResult>;
}

/**
 * Encrypted Output Helpers
 *
 * Functions for creating encrypted outputs that enable note scanning
 */

/**
 * Prepare encrypted output for scanning by wallet owner
 *
 * @param note - Note to encrypt
 * @param cloakKeys - Wallet's Cloak keys (for self-encryption)
 * @returns Base64-encoded encrypted output
 */
declare function prepareEncryptedOutput(note: CloakNote, cloakKeys: CloakKeyPair): string;
/**
 * Prepare encrypted output for a specific recipient
 *
 * @param note - Note to encrypt
 * @param recipientPvkHex - Recipient's public view key (hex)
 * @returns Base64-encoded encrypted output
 */
declare function prepareEncryptedOutputForRecipient(note: CloakNote, recipientPvkHex: string): string;
/**
 * Simple base64 encoding for v1.0 notes (no encryption)
 *
 * @param note - Note to encode
 * @returns Base64-encoded note data
 */
declare function encodeNoteSimple(note: CloakNote): string;

/**
 * Wallet Integration Helpers
 *
 * Helper functions for working with Solana Wallet Adapters
 */

/**
 * Validate wallet is connected and has public key
 */
declare function validateWalletConnected(wallet: WalletAdapter | Keypair): void;
/**
 * Get public key from wallet or keypair
 */
declare function getPublicKey(wallet: WalletAdapter | Keypair): PublicKey;
/**
 * Send transaction using wallet adapter or keypair
 */
declare function sendTransaction(transaction: Transaction, wallet: WalletAdapter | Keypair, connection: Connection, options?: SendOptions): Promise<string>;
/**
 * Sign transaction using wallet adapter or keypair
 */
declare function signTransaction<T extends Transaction>(transaction: T, wallet: WalletAdapter | Keypair): Promise<T>;
/**
 * Create a keypair adapter for testing
 */
declare function keypairToAdapter(keypair: Keypair): WalletAdapter;

/**
 * Create a deposit instruction
 *
 * Deposits SOL into the Cloak protocol by creating a commitment.
 *
 * Instruction format:
 * - Byte 0: Discriminant (0x00 for deposit)
 * - Bytes 1-8: Amount (u64, little-endian)
 * - Bytes 9-40: Commitment (32 bytes)
 *
 * @param params - Deposit parameters
 * @returns Transaction instruction
 *
 * @example
 * ```typescript
 * const instruction = createDepositInstruction({
 *   programId: CLOAK_PROGRAM_ID,
 *   payer: wallet.publicKey,
 *   pool: POOL_ADDRESS,
 *   commitments: COMMITMENTS_ADDRESS,
 *   amount: 1_000_000_000, // 1 SOL
 *   commitment: commitmentBytes
 * });
 * ```
 */
declare function createDepositInstruction(params: {
    programId: PublicKey;
    payer: PublicKey;
    pool: PublicKey;
    commitments: PublicKey;
    amount: number;
    commitment: Uint8Array;
}): TransactionInstruction;
/**
 * Deposit instruction parameters for type safety
 */
interface DepositInstructionParams {
    programId: PublicKey;
    payer: PublicKey;
    pool: PublicKey;
    commitments: PublicKey;
    amount: number;
    commitment: Uint8Array;
}
/**
 * Validate deposit instruction parameters
 *
 * @param params - Parameters to validate
 * @throws Error if invalid
 */
declare function validateDepositParams(params: DepositInstructionParams): void;

/**
 * Cloak SDK - TypeScript SDK for Private Transactions on Solana
 *
 * @packageDocumentation
 */

declare const VERSION = "0.1.0";

export { type CloakConfig, CloakError, type CloakKeyPair, type CloakNote, CloakSDK, type DepositInstructionParams, type DepositOptions, DepositRecoveryService, type DepositResult, type DepositStatus, type EncryptedNote, FIXED_FEE_LAMPORTS, IndexerService, LAMPORTS_PER_SOL, LocalStorageAdapter, type MasterKey, type MaxLengthArray, MemoryStorageAdapter, type MerkleProof, type MerkleRootResponse, type Network, type NoteData, type NotesRangeResponse, type ProofGenerationOptions, ProverService, type RecoveryOptions, type RecoveryResult, RelayService, type SP1ProofInputs, type SP1ProofResult, type ScanNotesOptions, type ScannedNote, type SpendKey, type StorageAdapter, type Transfer, type TransferOptions, type TransferResult, type TxStatus, VARIABLE_FEE_RATE, VERSION, type ViewKey, type WalletAdapter, type WithdrawOptions, bytesToHex, calculateFee, calculateRelayFee, computeNullifier, computeOutputsHash, copyNoteToClipboard, createCloakError, createDepositInstruction, deriveSpendKey, deriveViewKey, detectNetworkFromRpcUrl, downloadNote, encodeNoteSimple, encryptNoteForRecipient, exportKeys, exportNote, exportWalletKeys, filterNotesByNetwork, filterWithdrawableNotes, findNoteByCommitment, formatAmount, formatErrorForLogging, generateCloakKeys, generateCommitment, generateMasterSeed, generateNote, generateNoteFromWallet, getAddressExplorerUrl, getDistributableAmount, getExplorerUrl, getPublicKey, getPublicViewKey, getRecipientAmount, getRpcUrlForNetwork, getViewKey, hexToBytes, importKeys, importWalletKeys, isValidHex, isValidRpcUrl, isValidSolanaAddress, isWithdrawable, keypairToAdapter, parseAmount, parseNote, parseTransactionError, prepareEncryptedOutput, prepareEncryptedOutputForRecipient, randomBytes, scanNotesForWallet, sendTransaction, serializeNote, signTransaction, tryDecryptNote, updateNoteWithDeposit, validateDepositParams, validateNote, validateOutputsSum, validateTransfers, validateWalletConnected, validateWithdrawableNote };
