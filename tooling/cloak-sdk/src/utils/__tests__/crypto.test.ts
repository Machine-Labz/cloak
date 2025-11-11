import { describe, it, expect } from "@jest/globals";
import {
  generateCommitment,
  computeNullifier,
  computeOutputsHash,
  hexToBytes,
  bytesToHex,
  randomBytes,
  isValidHex,
} from "../crypto";
import { PublicKey } from "@solana/web3.js";
import { blake3 } from "@noble/hashes/blake3.js";

describe("Crypto Utils", () => {
  describe("hexToBytes and bytesToHex", () => {
    it("should convert hex to bytes correctly", () => {
      const hex = "0123456789abcdef";
      const bytes = hexToBytes(hex);
      expect(bytes).toBeInstanceOf(Uint8Array);
      expect(bytes.length).toBe(8);
      expect(bytesToHex(bytes)).toBe(hex);
    });

    it("should handle 0x prefix", () => {
      const hex = "0x0123456789abcdef";
      const bytes = hexToBytes(hex);
      expect(bytesToHex(bytes)).toBe("0123456789abcdef");
    });

    it("should convert bytes to hex with prefix option", () => {
      const bytes = new Uint8Array([1, 2, 3, 4]);
      expect(bytesToHex(bytes, true)).toBe("0x01020304");
      expect(bytesToHex(bytes, false)).toBe("01020304");
    });

    it("should handle empty arrays", () => {
      const bytes = new Uint8Array([]);
      expect(bytesToHex(bytes)).toBe("");
      expect(hexToBytes("").length).toBe(0);
    });

    it("should handle single byte", () => {
      const hex = "ff";
      const bytes = hexToBytes(hex);
      expect(bytes.length).toBe(1);
      expect(bytes[0]).toBe(255);
      expect(bytesToHex(bytes)).toBe(hex);
    });
  });

  describe("randomBytes", () => {
    it("should generate random bytes of specified length", () => {
      const bytes = randomBytes(32);
      expect(bytes).toBeInstanceOf(Uint8Array);
      expect(bytes.length).toBe(32);
    });

    it("should generate different values on subsequent calls", () => {
      const bytes1 = randomBytes(32);
      const bytes2 = randomBytes(32);
      expect(bytesToHex(bytes1)).not.toBe(bytesToHex(bytes2));
    });

    it("should work with different lengths", () => {
      expect(randomBytes(1).length).toBe(1);
      expect(randomBytes(16).length).toBe(16);
      expect(randomBytes(64).length).toBe(64);
    });
  });

  describe("isValidHex", () => {
    it("should validate correct hex strings", () => {
      expect(isValidHex("0123456789abcdef")).toBe(true);
      expect(isValidHex("0x0123456789abcdef")).toBe(true);
      expect(isValidHex("ABCDEF")).toBe(true);
    });

    it("should reject invalid hex strings", () => {
      expect(isValidHex("xyz")).toBe(false);
      expect(isValidHex("12g")).toBe(false);
      expect(isValidHex("1")).toBe(false); // Odd length
    });

    it("should validate expected length", () => {
      expect(isValidHex("0123456789abcdef", 8)).toBe(true);
      expect(isValidHex("0123456789abcdef", 16)).toBe(false);
      expect(isValidHex("0x0123456789abcdef", 8)).toBe(true);
    });

    it("should handle empty string", () => {
      expect(isValidHex("")).toBe(true);
      expect(isValidHex("", 0)).toBe(true);
      expect(isValidHex("", 1)).toBe(false);
    });
  });

  describe("generateCommitment", () => {
    it("should generate a 32-byte commitment", () => {
      const amount = 1_000_000_000;
      const r = randomBytes(32);
      const skSpend = randomBytes(32);

      const commitment = generateCommitment(amount, r, skSpend);
      expect(commitment).toBeInstanceOf(Uint8Array);
      expect(commitment.length).toBe(32);
    });

    it("should generate different commitments for different amounts", () => {
      const r = randomBytes(32);
      const skSpend = randomBytes(32);

      const commitment1 = generateCommitment(1_000_000_000, r, skSpend);
      const commitment2 = generateCommitment(2_000_000_000, r, skSpend);

      expect(bytesToHex(commitment1)).not.toBe(bytesToHex(commitment2));
    });

    it("should generate different commitments for different randomness", () => {
      const amount = 1_000_000_000;
      const skSpend = randomBytes(32);

      const commitment1 = generateCommitment(amount, randomBytes(32), skSpend);
      const commitment2 = generateCommitment(amount, randomBytes(32), skSpend);

      expect(bytesToHex(commitment1)).not.toBe(bytesToHex(commitment2));
    });

    it("should generate different commitments for different spend keys", () => {
      const amount = 1_000_000_000;
      const r = randomBytes(32);

      const commitment1 = generateCommitment(amount, r, randomBytes(32));
      const commitment2 = generateCommitment(amount, r, randomBytes(32));

      expect(bytesToHex(commitment1)).not.toBe(bytesToHex(commitment2));
    });

    it("should be deterministic", () => {
      const amount = 1_000_000_000;
      const r = new Uint8Array(32).fill(1);
      const skSpend = new Uint8Array(32).fill(2);

      const commitment1 = generateCommitment(amount, r, skSpend);
      const commitment2 = generateCommitment(amount, r, skSpend);

      expect(bytesToHex(commitment1)).toBe(bytesToHex(commitment2));
    });

    it("should match the expected format: Blake3(amount || r || pk_spend)", () => {
      const amount = 1_000_000_000;
      const r = new Uint8Array(32).fill(1);
      const skSpend = new Uint8Array(32).fill(2);

      const commitment = generateCommitment(amount, r, skSpend);

      // Manually compute expected result
      const pkSpend = blake3(skSpend);
      const amountBytes = new Uint8Array(8);
      new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amount), true);
      const input = new Uint8Array(72);
      input.set(amountBytes, 0);
      input.set(r, 8);
      input.set(pkSpend, 40);
      const expected = blake3(input);

      expect(bytesToHex(commitment)).toBe(bytesToHex(expected));
    });
  });

  describe("computeNullifier", () => {
    it("should generate a 32-byte nullifier", () => {
      const skSpend = randomBytes(32);
      const leafIndex = 42;

      const nullifier = computeNullifier(skSpend, leafIndex);
      expect(nullifier).toBeInstanceOf(Uint8Array);
      expect(nullifier.length).toBe(32);
    });

    it("should generate different nullifiers for different leaf indices", () => {
      const skSpend = randomBytes(32);

      const nullifier1 = computeNullifier(skSpend, 0);
      const nullifier2 = computeNullifier(skSpend, 1);

      expect(bytesToHex(nullifier1)).not.toBe(bytesToHex(nullifier2));
    });

    it("should generate different nullifiers for different spend keys", () => {
      const leafIndex = 42;

      const nullifier1 = computeNullifier(randomBytes(32), leafIndex);
      const nullifier2 = computeNullifier(randomBytes(32), leafIndex);

      expect(bytesToHex(nullifier1)).not.toBe(bytesToHex(nullifier2));
    });

    it("should be deterministic", () => {
      const skSpend = new Uint8Array(32).fill(1);
      const leafIndex = 42;

      const nullifier1 = computeNullifier(skSpend, leafIndex);
      const nullifier2 = computeNullifier(skSpend, leafIndex);

      expect(bytesToHex(nullifier1)).toBe(bytesToHex(nullifier2));
    });

    it("should work with large leaf indices", () => {
      const skSpend = randomBytes(32);
      const largeIndex = 999999;

      const nullifier = computeNullifier(skSpend, largeIndex);
      expect(nullifier.length).toBe(32);
    });
  });

  describe("computeOutputsHash", () => {
    it("should generate a 32-byte hash", () => {
      const outputs = [
        { recipient: new PublicKey("11111111111111111111111111111111"), amount: 500_000_000 },
        { recipient: new PublicKey("11111111111111111111111111111112"), amount: 500_000_000 },
      ];

      const hash = computeOutputsHash(outputs);
      expect(hash).toBeInstanceOf(Uint8Array);
      expect(hash.length).toBe(32);
    });

    it("should generate different hashes for different recipients", () => {
      const outputs1 = [
        { recipient: new PublicKey("11111111111111111111111111111111"), amount: 500_000_000 },
      ];
      const outputs2 = [
        { recipient: new PublicKey("11111111111111111111111111111112"), amount: 500_000_000 },
      ];

      const hash1 = computeOutputsHash(outputs1);
      const hash2 = computeOutputsHash(outputs2);

      expect(bytesToHex(hash1)).not.toBe(bytesToHex(hash2));
    });

    it("should generate different hashes for different amounts", () => {
      const recipient = new PublicKey("11111111111111111111111111111111");
      const outputs1 = [{ recipient, amount: 500_000_000 }];
      const outputs2 = [{ recipient, amount: 600_000_000 }];

      const hash1 = computeOutputsHash(outputs1);
      const hash2 = computeOutputsHash(outputs2);

      expect(bytesToHex(hash1)).not.toBe(bytesToHex(hash2));
    });

    it("should handle multiple outputs", () => {
      const outputs = [
        { recipient: new PublicKey("11111111111111111111111111111111"), amount: 100_000_000 },
        { recipient: new PublicKey("11111111111111111111111111111112"), amount: 200_000_000 },
        { recipient: new PublicKey("11111111111111111111111111111113"), amount: 300_000_000 },
        { recipient: new PublicKey("11111111111111111111111111111114"), amount: 400_000_000 },
        { recipient: new PublicKey("11111111111111111111111111111115"), amount: 500_000_000 },
      ];

      const hash = computeOutputsHash(outputs);
      expect(hash.length).toBe(32);
    });

    it("should be deterministic", () => {
      const outputs = [
        { recipient: new PublicKey("11111111111111111111111111111111"), amount: 500_000_000 },
        { recipient: new PublicKey("11111111111111111111111111111112"), amount: 500_000_000 },
      ];

      const hash1 = computeOutputsHash(outputs);
      const hash2 = computeOutputsHash(outputs);

      expect(bytesToHex(hash1)).toBe(bytesToHex(hash2));
    });

    it("should handle single output", () => {
      const outputs = [
        { recipient: new PublicKey("11111111111111111111111111111111"), amount: 1_000_000_000 },
      ];

      const hash = computeOutputsHash(outputs);
      expect(hash.length).toBe(32);
    });
  });
});

