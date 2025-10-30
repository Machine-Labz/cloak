import { PublicKey } from "@solana/web3.js";

/**
 * Supported Solana networks
 */
export type Network = "localnet" | "devnet" | "mainnet";

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
  network: Network;
  /** Cloak program ID */
  programId: PublicKey;
  /** Pool account address (PDA) */
  poolAddress: PublicKey;
  /** Commitments account address (PDA) */
  commitmentsAddress: PublicKey;
  /**
   * Single API base URL for both Indexer and Relay services.
   * If provided, it will be used for both services and overrides
   * any `indexerUrl` or `relayUrl` values.
   */
  apiUrl?: string;
  /** Indexer service URL (required if `apiUrl` is not provided) */
  indexerUrl?: string;
  /** Relay service URL (required if `apiUrl` is not provided) */
  relayUrl?: string;
  /** Optional: Roots ring account address (for root verification) */
  rootsRingAddress?: PublicKey;
  /** Optional: Treasury address (for fee collection) */
  treasuryAddress?: PublicKey;
  /** Optional: Nullifier shard address (for double-spend prevention) */
  nullifierShardAddress?: PublicKey;
  /** Optional: Proof generation timeout in milliseconds (default: 5 minutes) */
  proofTimeout?: number;
}

/**
 * Options for deposit operation
 */
export interface DepositOptions {
  /** Optional callback for progress updates */
  onProgress?: (status: string) => void;
  /** Skip simulation (default: false) */
  skipPreflight?: boolean;
}

/**
 * Options for private transfer/withdraw operation
 */
export interface TransferOptions {
  /** Relay fee in basis points (default: 0, max: 1000 = 10%) */
  relayFeeBps?: number;
  /** Optional callback for progress updates */
  onProgress?: (status: string) => void;
  /** Optional callback for proof generation progress (0-100) */
  onProofProgress?: (progress: number) => void;
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
