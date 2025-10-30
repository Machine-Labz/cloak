import { CloakNote, Network } from "./types";
import { generateCommitment, randomBytes, bytesToHex } from "../utils/crypto";
import { validateNote, parseNote as parseNoteUtil } from "../utils/validation";

/**
 * Generate a new Cloak note
 *
 * Creates a cryptographic commitment and generates the secret values needed
 * for future withdrawal. Keep this note safe and secret!
 *
 * @param amountLamports - Amount to deposit in lamports
 * @param network - Solana network (default: "localnet")
 * @returns New Cloak note
 *
 * @example
 * ```typescript
 * const note = generateNote(1_000_000_000, "devnet"); // 1 SOL
 * console.log(note.commitment);
 * // Save this note securely!
 * ```
 */
export function generateNote(
  amountLamports: number,
  network: Network = "localnet"
): CloakNote {
  if (amountLamports <= 0) {
    throw new Error("Amount must be positive");
  }

  // Generate random secret values (32 bytes each)
  const skSpend = randomBytes(32);
  const r = randomBytes(32);

  // Compute commitment
  const commitmentBytes = generateCommitment(amountLamports, r, skSpend);

  // Convert to hex strings
  const skSpendHex = bytesToHex(skSpend);
  const rHex = bytesToHex(r);
  const commitmentHex = bytesToHex(commitmentBytes);

  return {
    version: "1.0",
    amount: amountLamports,
    commitment: commitmentHex,
    sk_spend: skSpendHex,
    r: rHex,
    timestamp: Date.now(),
    network,
  };
}

/**
 * Parse a note from JSON string
 *
 * @param jsonString - JSON representation of the note
 * @returns Parsed note object
 * @throws Error if invalid format
 *
 * @example
 * ```typescript
 * const noteJson = '{"version":"1.0","amount":1000000000,...}';
 * const note = parseNote(noteJson);
 * ```
 */
export function parseNote(jsonString: string): CloakNote {
  return parseNoteUtil(jsonString);
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
export function serializeNote(note: CloakNote, pretty: boolean = false): string {
  validateNote(note);
  return pretty ? JSON.stringify(note, null, 2) : JSON.stringify(note);
}

/**
 * Check if a note is ready for withdrawal
 *
 * @param note - Note to check
 * @returns True if note can be used for withdrawal
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
 * Update a note with deposit information
 *
 * This is typically called after a successful deposit to store the
 * on-chain information needed for future withdrawal.
 *
 * @param note - Original note
 * @param depositInfo - Deposit information to add
 * @returns Updated note
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
 * Export note as downloadable JSON (browser only)
 *
 * @param note - Note to export
 * @param filename - Optional custom filename
 */
export function downloadNote(note: CloakNote, filename?: string): void {
  const g: any = globalThis as any;
  const doc = g?.document;
  const URL_ = g?.URL;
  const Blob_ = g?.Blob;

  if (!doc || !URL_ || !Blob_) {
    throw new Error("downloadNote is only available in browser environments");
  }

  const json = serializeNote(note, true);
  const blob = new Blob_([json], { type: "application/json" });
  const url = URL_.createObjectURL(blob);

  const defaultFilename = `cloak-note-${note.commitment.slice(0, 8)}.json`;
  const link = doc.createElement("a");
  link.href = url;
  link.download = filename || defaultFilename;
  link.click();

  URL_.revokeObjectURL(url);
}

/**
 * Copy note to clipboard as JSON (browser only)
 *
 * @param note - Note to copy
 * @returns Promise that resolves when copied
 */
export async function copyNoteToClipboard(note: CloakNote): Promise<void> {
  const g: any = globalThis as any;
  const nav = g?.navigator;
  if (!nav || !nav.clipboard) {
    throw new Error("Clipboard API not available");
  }

  const json = serializeNote(note, true);
  await nav.clipboard.writeText(json);
}
