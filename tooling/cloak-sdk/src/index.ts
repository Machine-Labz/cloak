/**
 * Cloak SDK - TypeScript SDK for Private Transactions on Solana
 *
 * @packageDocumentation
 */

// Main client
export { CloakSDK } from "./core/CloakSDK";

// Core types
export type {
  Network,
  CloakNote,
  MerkleProof,
  Transfer,
  MaxLengthArray,
  TransferResult,
  DepositResult,
  CloakConfig,
  DepositOptions,
  TransferOptions,
  WithdrawOptions,
  SP1ProofInputs,
  SP1ProofResult,
  MerkleRootResponse,
  TxStatus,
  WalletAdapter,
  DepositStatus,
  ScanNotesOptions,
  ScannedNote,
  SwapOptions,
  SwapParams,
  SwapResult,
} from "./core/types";

// Export CloakError
export { CloakError } from "./core/types";

// Note management (from note.ts)
export {
  serializeNote,
  downloadNote,
  copyNoteToClipboard,
} from "./core/note";

// Note management (from note-manager.ts)
export {
  generateNote,
  generateNoteFromWallet,
  parseNote,
  exportNote,
  findNoteByCommitment,
  filterNotesByNetwork,
  filterWithdrawableNotes,
  isWithdrawable,
  updateNoteWithDeposit,
  // Wallet key management (re-exported from note-manager for convenience)
  exportWalletKeys,
  importWalletKeys,
  getPublicViewKey,
  getViewKey,
  getRecipientAmount,
} from "./core/note-manager";

// Storage adapters
export type { StorageAdapter } from "./core/storage";
export {
  MemoryStorageAdapter,
  LocalStorageAdapter,
} from "./core/storage";

// Key management (v2.0 - view/spend keys)
export type {
  MasterKey,
  SpendKey,
  ViewKey,
  CloakKeyPair,
  EncryptedNote,
  NoteData,
} from "./core/keys";
export {
  generateMasterSeed,
  deriveSpendKey,
  deriveViewKey,
  generateCloakKeys,
  encryptNoteForRecipient,
  tryDecryptNote,
  scanNotesForWallet,
  exportKeys,
  importKeys,
} from "./core/keys";

// Crypto utilities
export {
  generateCommitment,
  computeNullifier,
  computeOutputsHash,
  computeSwapOutputsHash,
  hexToBytes,
  bytesToHex,
  randomBytes,
  isValidHex,
} from "./utils/crypto";

// Fee utilities
export {
  FIXED_FEE_LAMPORTS,
  VARIABLE_FEE_RATE,
  LAMPORTS_PER_SOL,
  calculateFee,
  getDistributableAmount,
  formatAmount,
  parseAmount,
  validateOutputsSum,
  calculateRelayFee,
} from "./utils/fees";

// Validation utilities
export {
  isValidSolanaAddress,
  validateNote,
  validateWithdrawableNote,
  validateTransfers,
} from "./utils/validation";

// Network utilities
export {
  detectNetworkFromRpcUrl,
  getRpcUrlForNetwork,
  isValidRpcUrl,
  getExplorerUrl,
  getAddressExplorerUrl,
} from "./utils/network";

// Error utilities
export {
  parseTransactionError,
  createCloakError,
  formatErrorForLogging,
} from "./utils/errors";

// Service clients
export { IndexerService } from "./services/IndexerService";
export { ProverService } from "./services/ProverService";
export type { ProofGenerationOptions } from "./services/ProverService";
export { RelayService } from "./services/RelayService";
export { DepositRecoveryService } from "./services/DepositRecoveryService";

// Export service types
export type { NotesRangeResponse } from "./services/IndexerService";
export type { RecoveryOptions, RecoveryResult } from "./services/DepositRecoveryService";

// Helpers
export {
  prepareEncryptedOutput,
  prepareEncryptedOutputForRecipient,
  encodeNoteSimple,
} from "./helpers/encrypted-output";
export {
  validateWalletConnected,
  getPublicKey,
  sendTransaction,
  signTransaction,
  keypairToAdapter,
} from "./helpers/wallet-integration";

// Solana instructions
export {
  createDepositInstruction,
  validateDepositParams,
} from "./solana/instructions";
export type { DepositInstructionParams } from "./solana/instructions";

// PDA utilities
export { getShieldPoolPDAs } from "./utils/pda";
export type { ShieldPoolPDAs } from "./utils/pda";

// Version
export const VERSION = "0.1.0";
