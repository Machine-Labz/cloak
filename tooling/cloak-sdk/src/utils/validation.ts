import { PublicKey } from "@solana/web3.js";
import { CloakNote } from "../core/types";
import { isValidHex } from "./crypto";

/**
 * Validate a Solana public key
 *
 * @param address - Address string to validate
 * @returns True if valid Solana address
 */
export function isValidSolanaAddress(address: string): boolean {
  try {
    new PublicKey(address);
    return true;
  } catch {
    return false;
  }
}

/**
 * Validate a Cloak note structure
 *
 * @param note - Note object to validate
 * @throws Error if invalid
 */
export function validateNote(note: any): asserts note is CloakNote {
  if (!note || typeof note !== "object") {
    throw new Error("Note must be an object");
  }

  // Check required fields
  const requiredFields = ["version", "amount", "commitment", "sk_spend", "r", "timestamp", "network"];
  for (const field of requiredFields) {
    if (!(field in note)) {
      throw new Error(`Missing required field: ${field}`);
    }
  }

  // Validate version
  if (typeof note.version !== "string") {
    throw new Error("Version must be a string");
  }

  // Validate amount
  if (typeof note.amount !== "number" || note.amount <= 0) {
    throw new Error("Amount must be a positive number");
  }

  // Validate commitment (64 hex chars = 32 bytes)
  if (!isValidHex(note.commitment, 32)) {
    throw new Error("Invalid commitment format (expected 64 hex characters)");
  }

  // Validate sk_spend (64 hex chars = 32 bytes)
  if (!isValidHex(note.sk_spend, 32)) {
    throw new Error("Invalid sk_spend format (expected 64 hex characters)");
  }

  // Validate r (64 hex chars = 32 bytes)
  if (!isValidHex(note.r, 32)) {
    throw new Error("Invalid r format (expected 64 hex characters)");
  }

  // Validate timestamp
  if (typeof note.timestamp !== "number" || note.timestamp <= 0) {
    throw new Error("Timestamp must be a positive number");
  }

  // Validate network
  if (!["localnet", "devnet", "testnet", "mainnet"].includes(note.network)) {
    throw new Error("Network must be localnet, devnet, testnet, or mainnet");
  }

  // If deposited, validate optional fields
  if (note.depositSignature !== undefined && typeof note.depositSignature !== "string") {
    throw new Error("Deposit signature must be a string");
  }

  if (note.depositSlot !== undefined && typeof note.depositSlot !== "number") {
    throw new Error("Deposit slot must be a number");
  }

  if (note.leafIndex !== undefined) {
    if (typeof note.leafIndex !== "number" || note.leafIndex < 0) {
      throw new Error("Leaf index must be a non-negative number");
    }
  }

  if (note.root !== undefined && !isValidHex(note.root, 32)) {
    throw new Error("Invalid root format (expected 64 hex characters)");
  }

  if (note.merkleProof !== undefined) {
    if (!Array.isArray(note.merkleProof.pathElements)) {
      throw new Error("Merkle proof pathElements must be an array");
    }
    if (!Array.isArray(note.merkleProof.pathIndices)) {
      throw new Error("Merkle proof pathIndices must be an array");
    }
    if (note.merkleProof.pathElements.length !== note.merkleProof.pathIndices.length) {
      throw new Error("Merkle proof pathElements and pathIndices must have same length");
    }
  }
}

/**
 * Parse and validate a note from JSON string
 *
 * @param jsonString - JSON string representation of note
 * @returns Parsed and validated note
 * @throws Error if invalid
 */
export function parseNote(jsonString: string): CloakNote {
  let parsed: any;
  try {
    parsed = JSON.parse(jsonString);
  } catch (error) {
    throw new Error("Invalid JSON format");
  }

  validateNote(parsed);
  return parsed;
}

/**
 * Validate that a note is ready for withdrawal
 *
 * @param note - Note to validate
 * @throws Error if note cannot be used for withdrawal
 */
export function validateWithdrawableNote(note: CloakNote): void {
  if (!note.depositSignature) {
    throw new Error("Note must be deposited before withdrawal (missing depositSignature)");
  }

  if (note.leafIndex === undefined) {
    throw new Error("Note must be deposited before withdrawal (missing leafIndex)");
  }

  if (!note.root) {
    throw new Error("Note must have historical root for withdrawal");
  }

  if (!note.merkleProof) {
    throw new Error("Note must have Merkle proof for withdrawal");
  }

  if (note.merkleProof.pathElements.length === 0) {
    throw new Error("Merkle proof is empty");
  }
}

/**
 * Validate transfer recipients
 *
 * @param recipients - Array of transfers to validate
 * @param totalAmount - Total amount available
 * @throws Error if invalid
 */
export function validateTransfers(
  recipients: Array<{ recipient: PublicKey; amount: number }>,
  totalAmount: number
): void {
  if (recipients.length === 0) {
    throw new Error("At least one recipient is required");
  }

  if (recipients.length > 5) {
    throw new Error("Maximum 5 recipients allowed");
  }

  // Validate each recipient
  for (let i = 0; i < recipients.length; i++) {
    const transfer = recipients[i];

    if (!transfer.recipient || !(transfer.recipient instanceof PublicKey)) {
      throw new Error(`Recipient ${i} must be a PublicKey`);
    }

    if (typeof transfer.amount !== "number" || transfer.amount <= 0) {
      throw new Error(`Recipient ${i} amount must be a positive number`);
    }
  }

  // Validate total
  const sum = recipients.reduce((acc, t) => acc + t.amount, 0);
  if (sum !== totalAmount) {
    throw new Error(
      `Recipients sum (${sum}) does not match expected total (${totalAmount})`
    );
  }
}
