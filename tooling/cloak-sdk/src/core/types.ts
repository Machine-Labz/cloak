import { PublicKey, Transaction } from "@solana/web3.js";

/**
 * Supported Solana networks
 */
export type Network = "localnet" | "devnet" | "mainnet" | "testnet";

/**
 * Minimal wallet adapter interface
 * Compatible with @solana/wallet-adapter-base
 */
export interface WalletAdapter {
  publicKey: PublicKey | null;
  signTransaction?<T extends Transaction>(transaction: T): Promise<T>;
  signAllTransactions?<T extends Transaction>(transactions: T[]): Promise<T[]>;
  sendTransaction?(
    transaction: Transaction,
    connection: any,
    options?: any
  ): Promise<string>;
}

/**
 * Cloak-specific error with categorization
 */
export class CloakError extends Error {
  constructor(
    message: string,
    public category: "network" | "indexer" | "prover" | "relay" | "validation" | "wallet" | "environment",
    public retryable: boolean = false,
    public originalError?: Error
  ) {
    super(message);
    this.name = "CloakError";
  }
}

/**
 * Cloak Note - Represents a private transaction commitment
 *
 * A note contains all the information needed to withdraw funds from the Cloak protocol.
 * Keep this safe and secret - anyone with access to this note can withdraw the funds!
 */
export interface CloakNote {
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
export interface MerkleProof {
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
export interface Transfer {
  /** Recipient's Solana public key */
  recipient: PublicKey;
  /** Amount to send in lamports */
  amount: number;
}

/**
 * Type-safe array with maximum length constraint
 * Used to enforce 1-5 recipients in privateTransfer
 */
export type MaxLengthArray<T, Max extends number, A extends T[] = []> =
  A['length'] extends Max
    ? A
    : A | MaxLengthArray<T, Max, [T, ...A]>;

/**
 * Result from a private transfer
 */
export interface TransferResult {
  /** Solana transaction signature */
  signature: string;
  /** Recipients and amounts that were sent */
  outputs: Array<{ recipient: string; amount: number }>;
  /** Nullifier used (prevents double-spending) */
  nullifier: string;
  /** Merkle root that was proven against */
  root: string;
}

/**
 * Result from a deposit operation
 */
export interface DepositResult {
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
export interface CloakConfig {
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
  cloakKeys?: any; // Will be CloakKeyPair from keys module

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
export type DepositStatus =
  | "generating_note"
  | "creating_transaction"
  | "simulating"
  | "sending"
  | "confirming"
  | "submitting_to_indexer"
  | "fetching_proof"
  | "complete";

/**
 * Options for deposit operation
 */
export interface DepositOptions {
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
export interface TransferOptions {
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
export interface WithdrawOptions extends TransferOptions {
  /** Whether to withdraw full amount minus fees (default: true) */
  withdrawAll?: boolean;
  /** Specific amount to withdraw in lamports (if not withdrawing all) */
  amount?: number;
}

/**
 * Staking configuration for private staking
 */
export interface StakeConfig {
  /** Stake account address (where SOL will be staked) */
  stakeAccount: PublicKey;
  /** Stake authority (who controls the stake account) */
  stakeAuthority: PublicKey;
  /** Validator vote account to delegate to */
  validatorVoteAccount: PublicKey;
}

/**
 * Options for private staking operation
 */
export interface StakeOptions extends TransferOptions {
  /** Optional: Stake configuration (if not provided, will be derived) */
  stakeConfig?: StakeConfig;
}

/**
 * Result from a private staking operation
 */
export interface StakeResult {
  /** Solana transaction signature */
  signature: string;
  /** Stake account that received the funds */
  stakeAccount: string;
  /** Validator vote account that was delegated to */
  validatorVoteAccount: string;
  /** Amount staked (after fees) */
  stakeAmount: number;
  /** Nullifier used (prevents double-spending) */
  nullifier: string;
  /** Merkle root that was proven against */
  root: string;
}

/**
 * SP1 proof inputs for zero-knowledge proof generation
 */
export interface SP1ProofInputs {
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
  /** Optional swap parameters for swap-mode withdrawals */
  swap_params?: {
    output_mint: string;
    recipient_ata: string;
    min_output_amount: number;
  };
  /** Optional stake parameters for stake-mode withdrawals */
  stake_params?: {
    stake_account: string;
  };
}

/**
 * Result from proof generation
 */
export interface SP1ProofResult {
  success: boolean;
  proof?: string; // Hex-encoded proof bytes (260 bytes for Groth16)
  publicInputs?: string; // Hex-encoded public inputs (104 bytes)
  generationTimeMs: number;
  error?: string;
}

/**
 * Merkle root response from indexer
 */
export interface MerkleRootResponse {
  root: string;
  next_index: number;
}

/**
 * Transaction status from relay service
 */
export interface TxStatus {
  status: "pending" | "processing" | "completed" | "failed";
  txId?: string;
  error?: string;
}

/**
 * Note scanning options
 */
export interface ScanNotesOptions {
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
export interface ScannedNote extends CloakNote {
  /** When this note was discovered */
  scannedAt: number;
  
  /** Whether this note has been spent (nullifier check) */
  isSpent?: boolean;
}
