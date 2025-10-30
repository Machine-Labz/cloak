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
} from "./core/types";

// Note management
export {
  generateNote,
  parseNote,
  serializeNote,
  isWithdrawable,
  updateNoteWithDeposit,
  downloadNote,
  copyNoteToClipboard,
} from "./core/note";

// Crypto utilities
export {
  generateCommitment,
  computeNullifier,
  computeOutputsHash,
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

// Service clients
export { IndexerService } from "./services/IndexerService";
export { ProverService } from "./services/ProverService";
export { RelayService } from "./services/RelayService";

// Solana instructions
export {
  createDepositInstruction,
  validateDepositParams,
} from "./solana/instructions";
export type { DepositInstructionParams } from "./solana/instructions";

// Version
export const VERSION = "0.1.0";
