import { blake3 } from "@noble/hashes/blake3.js";
import nacl from "tweetnacl";
import { bytesToHex, hexToBytes } from "../utils/crypto";

/**
 * Cloak Key Hierarchy (v2.0)
 * 
 * Implements view/spend key separation for privacy-preserving note scanning:
 * - Master Seed → Spend Key → View Key → Public View Key
 * - Enables note discovery without exposing spending authority
 * - Compatible with v1.0 notes (backward compatible)
 */

export interface MasterKey {
  seed: Uint8Array; // 32 bytes - MUST be kept secret
  seedHex: string;
}

export interface SpendKey {
  sk_spend: Uint8Array; // 32 bytes - spend key
  pk_spend: Uint8Array; // 32 bytes - public spend key (used in commitments)
  sk_spend_hex: string;
  pk_spend_hex: string;
}

export interface ViewKey {
  vk_secret: Uint8Array; // 32 bytes - secret view key for decryption
  pvk: Uint8Array; // 32 bytes - public view key (for receiving encrypted notes)
  vk_secret_hex: string;
  pvk_hex: string;
}

export interface CloakKeyPair {
  master: MasterKey;
  spend: SpendKey;
  view: ViewKey;
}

export interface EncryptedNote {
  ephemeral_pk: string; // hex - ephemeral public key for ECDH
  ciphertext: string; // hex - encrypted note data
  nonce: string; // hex - nonce for secretbox
}

export interface NoteData {
  amount: number;
  r: string; // hex
  sk_spend: string; // hex
  commitment: string; // hex
}

/**
 * Generate a new master seed from secure randomness
 */
export function generateMasterSeed(): MasterKey {
  const seed = new Uint8Array(32);
  
  // Use crypto.getRandomValues (works in browser and Node.js 15+)
  const g: any = globalThis as any;
  const cryptoObj = g?.crypto || g?.window?.crypto || g?.self?.crypto;
  
  if (cryptoObj && typeof cryptoObj.getRandomValues === 'function') {
    cryptoObj.getRandomValues(seed);
  } else {
    // Fallback for older Node.js
    try {
      const nodeCrypto = require('crypto');
      const buffer = nodeCrypto.randomBytes(32);
      seed.set(buffer);
    } catch {
      throw new Error("No secure random number generator available");
    }
  }
  
  return {
    seed,
    seedHex: bytesToHex(seed),
  };
}

/**
 * Derive spend key from master seed
 */
export function deriveSpendKey(masterSeed: Uint8Array): SpendKey {
  // sk_spend = BLAKE3(master_seed || "cloak_spend_key")
  const context = new TextEncoder().encode("cloak_spend_key");
  const preimage = new Uint8Array(masterSeed.length + context.length);
  preimage.set(masterSeed, 0);
  preimage.set(context, masterSeed.length);
  
  const sk_spend = blake3(preimage);
  const pk_spend = blake3(sk_spend); // Same as current commitment scheme
  
  return {
    sk_spend,
    pk_spend,
    sk_spend_hex: bytesToHex(sk_spend),
    pk_spend_hex: bytesToHex(pk_spend),
  };
}

/**
 * Derive view key from spend key
 */
export function deriveViewKey(sk_spend: Uint8Array): ViewKey {
  // vk_secret = BLAKE3(sk_spend || "cloak_view_key_secret")
  const context = new TextEncoder().encode("cloak_view_key_secret");
  const preimage = new Uint8Array(sk_spend.length + context.length);
  preimage.set(sk_spend, 0);
  preimage.set(context, sk_spend.length);
  
  const vk_secret = blake3(preimage);
  
  // Use nacl to generate X25519 keypair from the secret
  // nacl.box.keyPair.fromSecretKey generates the public key from secret
  const x25519Keypair = nacl.box.keyPair.fromSecretKey(vk_secret);
  
  return {
    vk_secret,
    pvk: x25519Keypair.publicKey,
    vk_secret_hex: bytesToHex(vk_secret),
    pvk_hex: bytesToHex(x25519Keypair.publicKey),
  };
}

/**
 * Generate complete key hierarchy from master seed
 */
export function generateCloakKeys(masterSeed?: Uint8Array): CloakKeyPair {
  const master = masterSeed 
    ? { seed: masterSeed, seedHex: bytesToHex(masterSeed) }
    : generateMasterSeed();
  
  const spend = deriveSpendKey(master.seed);
  const view = deriveViewKey(spend.sk_spend);
  
  return {
    master,
    spend,
    view,
  };
}

/**
 * Encrypt note data for a recipient using their public view key
 * 
 * Uses X25519 ECDH + XSalsa20-Poly1305 authenticated encryption
 */
export function encryptNoteForRecipient(
  noteData: NoteData,
  recipientPvk: Uint8Array
): EncryptedNote {
  // Generate ephemeral X25519 keypair
  const ephemeralKeypair = nacl.box.keyPair();
  
  // Compute shared secret via ECDH
  const sharedSecret = nacl.box.before(recipientPvk, ephemeralKeypair.secretKey);
  
  // Serialize note data as JSON
  const plaintext = new TextEncoder().encode(JSON.stringify(noteData));
  
  // Generate random nonce
  const nonce = nacl.randomBytes(nacl.secretbox.nonceLength);
  
  // Encrypt using secretbox (XSalsa20-Poly1305)
  const ciphertext = nacl.secretbox(plaintext, nonce, sharedSecret);
  
  return {
    ephemeral_pk: bytesToHex(ephemeralKeypair.publicKey),
    ciphertext: bytesToHex(ciphertext),
    nonce: bytesToHex(nonce),
  };
}

/**
 * Attempt to decrypt an encrypted note using view key
 * 
 * Returns null if decryption fails (note doesn't belong to this wallet)
 * Returns NoteData if successful
 */
export function tryDecryptNote(
  encryptedNote: EncryptedNote,
  viewKey: ViewKey
): NoteData | null {
  try {
    // Parse hex strings
    const ephemeralPk = hexToBytes(encryptedNote.ephemeral_pk);
    const ciphertext = hexToBytes(encryptedNote.ciphertext);
    const nonce = hexToBytes(encryptedNote.nonce);
    
    // Use view key secret directly as X25519 secret (they're both Curve25519 scalars)
    const x25519Secret = viewKey.vk_secret;
    
    // Compute shared secret using ECDH
    const sharedSecret = nacl.box.before(ephemeralPk, x25519Secret);
    
    // Attempt decryption
    const plaintext = nacl.secretbox.open(ciphertext, nonce, sharedSecret);
    
    if (!plaintext) {
      return null; // Decryption failed - not our note
    }
    
    // Parse and return note data
    const noteData = JSON.parse(new TextDecoder().decode(plaintext));
    return noteData as NoteData;
  } catch (e) {
    // Decryption or parsing failed - not our note
    return null;
  }
}

/**
 * Scan a batch of encrypted outputs and return notes belonging to this wallet
 */
export function scanNotesForWallet(
  encryptedOutputs: string[], // Base64 encoded encrypted note JSON
  viewKey: ViewKey
): NoteData[] {
  const foundNotes: NoteData[] = [];
  
  for (const encryptedOutput of encryptedOutputs) {
    try {
      // Decode base64
      const decoded = atob(encryptedOutput);
      const encryptedNote = JSON.parse(decoded) as EncryptedNote;
      
      // Try to decrypt
      const noteData = tryDecryptNote(encryptedNote, viewKey);
      
      if (noteData) {
        foundNotes.push(noteData);
      }
    } catch (e) {
      // Skip malformed outputs
      continue;
    }
  }
  
  return foundNotes;
}

/**
 * Export keys for backup (WARNING: contains secrets!)
 */
export function exportKeys(keys: CloakKeyPair): string {
  return JSON.stringify({
    version: "2.0",
    master_seed: keys.master.seedHex,
    sk_spend: keys.spend.sk_spend_hex,
    pk_spend: keys.spend.pk_spend_hex,
    vk_secret: keys.view.vk_secret_hex,
    pvk: keys.view.pvk_hex,
  }, null, 2);
}

/**
 * Import keys from backup
 */
export function importKeys(exported: string): CloakKeyPair {
  const parsed = JSON.parse(exported);
  
  const masterSeed = hexToBytes(parsed.master_seed);
  
  // Re-derive keys to ensure consistency
  return generateCloakKeys(masterSeed);
}

