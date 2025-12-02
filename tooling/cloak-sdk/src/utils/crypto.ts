import { blake3 } from "@noble/hashes/blake3.js";
import { PublicKey } from "@solana/web3.js";

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
export function generateCommitment(
  amountLamports: number,
  r: Uint8Array,
  skSpend: Uint8Array
): Uint8Array {
  // Compute pk_spend = Blake3(sk_spend)
  const pkSpend = blake3(skSpend);

  // Encode amount as little-endian u64
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amountLamports), true);

  // Concatenate: amount (8) + r (32) + pk_spend (32) = 72 bytes
  const commitmentInput = new Uint8Array(72);
  commitmentInput.set(amountBytes, 0);
  commitmentInput.set(r, 8);
  commitmentInput.set(pkSpend, 40);

  // Hash to get commitment
  return blake3(commitmentInput);
}

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
export function computeNullifier(
  skSpend: Uint8Array,
  leafIndex: number
): Uint8Array {
  // Encode leaf index as little-endian u32 (4 bytes) to match circuit
  // Circuit uses: serialize_u32_le(leaf_index)
  const leafIndexBytes = new Uint8Array(4);
  new DataView(leafIndexBytes.buffer).setUint32(0, leafIndex, true);

  // Concatenate: sk_spend (32) + leaf_index (4) = 36 bytes
  const nullifierInput = new Uint8Array(36);
  nullifierInput.set(skSpend, 0);
  nullifierInput.set(leafIndexBytes, 32);

  // Hash to get nullifier
  return blake3(nullifierInput);
}

/**
 * Compute outputs hash from recipients and amounts
 *
 * Formula: Blake3(recipient1 || amount1 || recipient2 || amount2 || ...)
 *
 * @param outputs - Array of {recipient: PublicKey, amount: number}
 * @returns Outputs hash (32 bytes)
 */
export function computeOutputsHash(
  outputs: Array<{ recipient: PublicKey; amount: number }>
): Uint8Array {
  // Each output is 32 bytes (pubkey) + 8 bytes (amount)
  const chunks: Uint8Array[] = [];

  for (const output of outputs) {
    // Recipient public key (32 bytes)
    chunks.push(output.recipient.toBytes());

    // Amount as little-endian u64 (8 bytes)
    const amountBytes = new Uint8Array(8);
    new DataView(amountBytes.buffer).setBigUint64(0, BigInt(output.amount), true);
    chunks.push(amountBytes);
  }

  // Concatenate all chunks
  const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    combined.set(chunk, offset);
    offset += chunk.length;
  }

  // Hash to get outputs hash
  return blake3(combined);
}

/**
 * Compute stake outputs hash for staking mode
 *
 * Formula: Blake3(stake_account || public_amount)
 *
 * @param stakeAccount - Stake account public key
 * @param publicAmount - Public amount (u64)
 * @returns Stake outputs hash (32 bytes)
 */
export function computeStakeOutputsHash(
  stakeAccount: PublicKey,
  publicAmount: number
): Uint8Array {
  // Stake account public key (32 bytes)
  const stakeAccountBytes = stakeAccount.toBytes();

  // Amount as little-endian u64 (8 bytes)
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(publicAmount), true);

  // Concatenate: stake_account (32) + amount (8) = 40 bytes
  const combined = new Uint8Array(40);
  combined.set(stakeAccountBytes, 0);
  combined.set(amountBytes, 32);

  // Hash to get stake outputs hash
  return blake3(combined);
}

/**
 * Convert hex string to Uint8Array
 *
 * @param hex - Hex string (with or without 0x prefix)
 * @returns Decoded bytes
 */
export function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
  }
  return bytes;
}

/**
 * Convert Uint8Array to hex string
 *
 * @param bytes - Bytes to encode
 * @param prefix - Whether to include 0x prefix (default: false)
 * @returns Hex-encoded string
 */
export function bytesToHex(bytes: Uint8Array, prefix: boolean = false): string {
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return prefix ? `0x${hex}` : hex;
}

/**
 * Generate random bytes using Web Crypto API
 *
 * @param length - Number of bytes to generate
 * @returns Random bytes
 */
export function randomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  const g: any = globalThis as any;

  try {
    const cryptoObj = g?.crypto || g?.window?.crypto || g?.self?.crypto;
    if (cryptoObj && typeof cryptoObj.getRandomValues === 'function') {
      cryptoObj.getRandomValues(bytes);
      return bytes;
    }
  } catch {}

  try {
    // Fallback to Node.js crypto
    const nodeCrypto = require('crypto');
    if (nodeCrypto?.randomBytes) {
      const buffer = nodeCrypto.randomBytes(length);
      bytes.set(buffer);
      return bytes;
    }
    if (nodeCrypto?.webcrypto?.getRandomValues) {
      nodeCrypto.webcrypto.getRandomValues(bytes);
      return bytes;
    }
  } catch {}

  // Last resort (non-cryptographic)
  for (let i = 0; i < length; i++) bytes[i] = Math.floor(Math.random() * 256);
  return bytes;
}

/**
 * Validate a hex string format
 *
 * @param hex - Hex string to validate
 * @param expectedLength - Expected length in bytes (optional)
 * @returns True if valid hex string
 */
export function isValidHex(hex: string, expectedLength?: number): boolean {
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;

  // Check if it's a valid hex string
  if (!/^[0-9a-f]*$/i.test(cleanHex)) {
    return false;
  }

  // Check if length is even (each byte is 2 hex chars)
  if (cleanHex.length % 2 !== 0) {
    return false;
  }

  // Check expected length if provided
  if (expectedLength !== undefined) {
    return cleanHex.length === expectedLength * 2;
  }

  return true;
}
