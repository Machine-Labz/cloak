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

import { blake3 } from "@noble/hashes/blake3.js";
import { Buffer } from "buffer";
import {
  CloakNote,
  Network,
  CloakError,
} from "./types";
import {
  generateCloakKeys as generateKeys,
  type CloakKeyPair,
  type ViewKey,
  scanNotesForWallet as scanNotes,
  exportKeys,
  importKeys,
  type NoteData,
} from "./keys";
import { detectNetworkFromRpcUrl } from "../utils/network";

/**
 * Generate a new note without wallet keys (legacy v1.0)
 * @deprecated Use generateNoteFromWallet instead for enhanced security
 */
export function generateNote(amountLamports: number, network?: Network): CloakNote {
  const actualNetwork = network || detectNetworkFromRpcUrl();
  const skSpend = new Uint8Array(32);
  const rBytes = new Uint8Array(32);
  crypto.getRandomValues(skSpend);
  crypto.getRandomValues(rBytes);

  const pkSpend = blake3(skSpend);
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amountLamports), true);

  const commitmentInput = new Uint8Array(8 + 32 + 32);
  commitmentInput.set(amountBytes, 0);
  commitmentInput.set(rBytes, 8);
  commitmentInput.set(pkSpend, 40);
  const commitmentBytes = blake3(commitmentInput);

  const skSpendHex = Buffer.from(skSpend).toString("hex");
  const rHex = Buffer.from(rBytes).toString("hex");
  const commitmentHex = Buffer.from(commitmentBytes).toString("hex");

  return {
    version: "1.0",
    amount: amountLamports,
    commitment: commitmentHex,
    sk_spend: skSpendHex,
    r: rHex,
    timestamp: Date.now(),
    network: actualNetwork,
  };
}

/**
 * Generate a note using wallet's spend key (v2.0 recommended)
 */
export function generateNoteFromWallet(
  amountLamports: number,
  keys: CloakKeyPair,
  network?: Network
): CloakNote {
  const actualNetwork = network || detectNetworkFromRpcUrl();
  const rBytes = new Uint8Array(32);
  crypto.getRandomValues(rBytes);

  const pk_spend = Buffer.from(keys.spend.pk_spend_hex, "hex");
  
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amountLamports), true);

  const commitmentInput = new Uint8Array(8 + 32 + 32);
  commitmentInput.set(amountBytes, 0);
  commitmentInput.set(rBytes, 8);
  commitmentInput.set(pk_spend, 40);
  const commitmentBytes = blake3(commitmentInput);

  return {
    version: "2.0",
    amount: amountLamports,
    commitment: Buffer.from(commitmentBytes).toString("hex"),
    sk_spend: keys.spend.sk_spend_hex,
    r: Buffer.from(rBytes).toString("hex"),
    timestamp: Date.now(),
    network: actualNetwork,
  };
}

/**
 * Parse and validate a note from JSON string
 */
export function parseNote(jsonString: string): CloakNote {
  const note = JSON.parse(jsonString);

  // Validate required fields
  if (!note.version || !note.amount || !note.commitment || !note.sk_spend || !note.r) {
    throw new CloakError(
      "Invalid note format: missing required fields",
      "validation",
      false
    );
  }

  // Validate hex strings
  if (!/^[0-9a-f]{64}$/i.test(note.commitment)) {
    throw new CloakError("Invalid commitment format", "validation", false);
  }
  if (!/^[0-9a-f]{64}$/i.test(note.sk_spend)) {
    throw new CloakError("Invalid sk_spend format", "validation", false);
  }
  if (!/^[0-9a-f]{64}$/i.test(note.r)) {
    throw new CloakError("Invalid r format", "validation", false);
  }

  return note as CloakNote;
}

/**
 * Export note to JSON string
 */
export function exportNote(note: CloakNote, pretty: boolean = false): string {
  return pretty ? JSON.stringify(note, null, 2) : JSON.stringify(note);
}

/**
 * Check if a note is withdrawable (has been deposited)
 */
export function isWithdrawable(note: CloakNote): boolean {
  return !!(
    note.depositSignature &&
    note.leafIndex !== undefined &&
    note.root &&
    note.merkleProof
  );
}

/**
 * Update note with deposit information
 * Returns a new note object with deposit info added
 */
export function updateNoteWithDeposit(
  note: CloakNote,
  depositInfo: {
    signature: string;
    slot: number;
    leafIndex: number;
    root: string;
    merkleProof: {
      pathElements: string[];
      pathIndices: number[];
    };
  }
): CloakNote {
  return {
    ...note,
    depositSignature: depositInfo.signature,
    depositSlot: depositInfo.slot,
    leafIndex: depositInfo.leafIndex,
    root: depositInfo.root,
    merkleProof: depositInfo.merkleProof,
  };
}

/**
 * Find note by commitment from an array
 */
export function findNoteByCommitment(
  notes: CloakNote[],
  commitment: string
): CloakNote | undefined {
  return notes.find((n) => n.commitment === commitment);
}

/**
 * Filter notes by network
 */
export function filterNotesByNetwork(notes: CloakNote[], network: Network): CloakNote[] {
  return notes.filter((n) => n.network === network);
}

/**
 * Filter notes that can be withdrawn
 */
export function filterWithdrawableNotes(notes: CloakNote[]): CloakNote[] {
  return notes.filter(isWithdrawable);
}

// ============================================================================
// Key Management Functions (v2.0) - No storage, just key operations
// ============================================================================

/**
 * Generate new Cloak keys
 */
export function generateCloakKeys(masterSeed?: Uint8Array): CloakKeyPair {
  return generateKeys(masterSeed);
}

/**
 * Export keys to JSON string
 * WARNING: This exports secret keys! Store securely.
 */
export function exportWalletKeys(keys: CloakKeyPair): string {
  return exportKeys(keys);
}

/**
 * Import keys from JSON string
 */
export function importWalletKeys(keysJson: string): CloakKeyPair {
  return importKeys(keysJson);
}

/**
 * Get public view key from keys
 */
export function getPublicViewKey(keys: CloakKeyPair): string {
  return keys.view.pvk_hex;
}

/**
 * Get view key from keys
 */
export function getViewKey(keys: CloakKeyPair): ViewKey {
  return keys.view;
}

/**
 * Scan encrypted outputs and return notes belonging to this wallet
 * 
 * @param encryptedOutputs - Array of base64-encoded encrypted notes
 * @param viewKey - View key for decryption
 * @returns Array of discovered notes
 */
export function scanNotesForWallet(
  encryptedOutputs: string[],
  viewKey: ViewKey
): CloakNote[] {
  const discoveredNotes: NoteData[] = scanNotes(encryptedOutputs, viewKey);
  const network = detectNetworkFromRpcUrl();
  
  return discoveredNotes.map((noteData) => ({
    version: "2.0",
    amount: noteData.amount,
    commitment: noteData.commitment,
    sk_spend: noteData.sk_spend,
    r: noteData.r,
    timestamp: Date.now(),
    network,
  }));
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Format amount for display (lamports to SOL)
 */
export function formatAmount(lamports: number): string {
  return (lamports / 1_000_000_000).toFixed(9);
}

/**
 * Calculate fee for an amount (must match circuit)
 */
export function calculateFee(amountLamports: number): number {
  const FIXED_FEE_LAMPORTS = 2_500_000;
  const variableFee = Math.floor((amountLamports * 5) / 1_000);
  return FIXED_FEE_LAMPORTS + variableFee;
}

/**
 * Amount remaining for recipient outputs after fees
 */
export function getDistributableAmount(amountLamports: number): number {
  return amountLamports - calculateFee(amountLamports);
}

/**
 * Get recipient amount after fees
 */
export function getRecipientAmount(amountLamports: number): number {
  return getDistributableAmount(amountLamports);
}