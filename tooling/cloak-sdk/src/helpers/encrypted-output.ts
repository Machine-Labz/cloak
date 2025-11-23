/**
 * Encrypted Output Helpers
 * 
 * Functions for creating encrypted outputs that enable note scanning
 */

import { CloakNote } from "../core/types";
import { encryptNoteForRecipient, type CloakKeyPair, type NoteData } from "../core/keys";
import { hexToBytes } from "../utils/crypto";

/**
 * Prepare encrypted output for scanning by wallet owner
 * 
 * @param note - Note to encrypt
 * @param cloakKeys - Wallet's Cloak keys (for self-encryption)
 * @returns Base64-encoded encrypted output
 */
export function prepareEncryptedOutput(
  note: CloakNote,
  cloakKeys: CloakKeyPair
): string {
  const noteData: NoteData = {
    amount: note.amount,
    r: note.r,
    sk_spend: note.sk_spend,
    commitment: note.commitment,
  };
  
  // Encrypt for wallet's own public view key
  const encrypted = encryptNoteForRecipient(noteData, cloakKeys.view.pvk);
  
  // Encode as base64 JSON
  return btoa(JSON.stringify(encrypted));
}

/**
 * Prepare encrypted output for a specific recipient
 * 
 * @param note - Note to encrypt
 * @param recipientPvkHex - Recipient's public view key (hex)
 * @returns Base64-encoded encrypted output
 */
export function prepareEncryptedOutputForRecipient(
  note: CloakNote,
  recipientPvkHex: string
): string {
  const noteData: NoteData = {
    amount: note.amount,
    r: note.r,
    sk_spend: note.sk_spend,
    commitment: note.commitment,
  };
  
  const recipientPvk = hexToBytes(recipientPvkHex);
  const encrypted = encryptNoteForRecipient(noteData, recipientPvk);
  
  return btoa(JSON.stringify(encrypted));
}

/**
 * Simple base64 encoding for v1.0 notes (no encryption)
 * 
 * @param note - Note to encode
 * @returns Base64-encoded note data
 */
export function encodeNoteSimple(note: CloakNote): string {
  const data = {
    amount: note.amount,
    r: note.r,
    sk_spend: note.sk_spend,
    commitment: note.commitment,
  };
  
  const json = JSON.stringify(data);
  
  if (typeof Buffer !== "undefined") {
    return Buffer.from(json).toString("base64");
  } else if (typeof btoa !== "undefined") {
    return btoa(json);
  } else {
    throw new Error("No base64 encoding method available");
  }
}

