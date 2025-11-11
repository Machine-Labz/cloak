import { describe, it, expect, beforeEach } from "@jest/globals";
import {
  generateNote,
  generateNoteFromWallet,
  parseNote,
  serializeNote,
  isWithdrawable,
  updateNoteWithDeposit,
} from "../note";
import { generateCloakKeys } from "../keys";
import { bytesToHex, hexToBytes, randomBytes } from "../../utils/crypto";
import { CloakNote } from "../types";

describe("Note Management", () => {
  describe("generateNote (v1.0)", () => {
    it("should generate a valid note", () => {
      const note = generateNote(1_000_000_000, "devnet");

      expect(note.version).toBe("1.0");
      expect(note.amount).toBe(1_000_000_000);
      expect(note.network).toBe("devnet");
      expect(note.commitment).toHaveLength(64); // 32 bytes in hex
      expect(note.sk_spend).toHaveLength(64);
      expect(note.r).toHaveLength(64);
      expect(note.timestamp).toBeGreaterThan(0);
    });

    it("should default to localnet", () => {
      const note = generateNote(1_000_000_000);
      expect(note.network).toBe("localnet");
    });

    it("should generate unique values for each note", () => {
      const note1 = generateNote(1_000_000_000);
      const note2 = generateNote(1_000_000_000);

      expect(note1.commitment).not.toBe(note2.commitment);
      expect(note1.sk_spend).not.toBe(note2.sk_spend);
      expect(note1.r).not.toBe(note2.r);
    });

    it("should throw on zero amount", () => {
      expect(() => generateNote(0)).toThrow("Amount must be positive");
    });

    it("should throw on negative amount", () => {
      expect(() => generateNote(-100)).toThrow("Amount must be positive");
    });

    it("should generate valid hex strings", () => {
      const note = generateNote(1_000_000_000);

      const hexPattern = /^[0-9a-f]{64}$/i;
      expect(hexPattern.test(note.commitment)).toBe(true);
      expect(hexPattern.test(note.sk_spend)).toBe(true);
      expect(hexPattern.test(note.r)).toBe(true);
    });

    it("should not have deposit information initially", () => {
      const note = generateNote(1_000_000_000);

      expect(note.depositSignature).toBeUndefined();
      expect(note.depositSlot).toBeUndefined();
      expect(note.leafIndex).toBeUndefined();
      expect(note.root).toBeUndefined();
      expect(note.merkleProof).toBeUndefined();
    });
  });

  describe("generateNoteFromWallet (v2.0)", () => {
    let keys: ReturnType<typeof generateCloakKeys>;

    beforeEach(() => {
      keys = generateCloakKeys();
    });

    it("should generate a v2.0 note", () => {
      const note = generateNoteFromWallet(keys, 1_000_000_000, "devnet");

      expect(note.version).toBe("2.0");
      expect(note.amount).toBe(1_000_000_000);
      expect(note.network).toBe("devnet");
      expect(note.commitment).toHaveLength(64);
      expect(note.sk_spend).toBe(keys.spend.sk_spend_hex);
      expect(note.r).toHaveLength(64);
    });

    it("should use deterministic spend key from wallet", () => {
      const note1 = generateNoteFromWallet(keys, 1_000_000_000);
      const note2 = generateNoteFromWallet(keys, 1_000_000_000);

      expect(note1.sk_spend).toBe(note2.sk_spend);
      expect(note1.sk_spend).toBe(keys.spend.sk_spend_hex);
    });

    it("should generate different randomness each time", () => {
      const note1 = generateNoteFromWallet(keys, 1_000_000_000);
      const note2 = generateNoteFromWallet(keys, 1_000_000_000);

      expect(note1.r).not.toBe(note2.r);
      expect(note1.commitment).not.toBe(note2.commitment);
    });

    it("should throw on invalid amount", () => {
      expect(() => generateNoteFromWallet(keys, 0)).toThrow("Amount must be positive");
      expect(() => generateNoteFromWallet(keys, -100)).toThrow("Amount must be positive");
    });
  });

  describe("parseNote and serializeNote", () => {
    it("should serialize and parse a note", () => {
      const original = generateNote(1_000_000_000, "devnet");
      const serialized = serializeNote(original);
      const parsed = parseNote(serialized);

      expect(parsed).toEqual(original);
    });

    it("should serialize with pretty formatting", () => {
      const note = generateNote(1_000_000_000);
      const pretty = serializeNote(note, true);
      const compact = serializeNote(note, false);

      expect(pretty.length).toBeGreaterThan(compact.length);
      expect(pretty).toContain("\n");
      expect(compact).not.toContain("\n");
    });

    it("should throw on invalid JSON", () => {
      expect(() => parseNote("not json")).toThrow("Invalid JSON format");
      expect(() => parseNote("")).toThrow("Invalid JSON format");
    });

    it("should throw on malformed note", () => {
      const invalidNote = JSON.stringify({ invalid: "note" });
      expect(() => parseNote(invalidNote)).toThrow();
    });

    it("should preserve all fields", () => {
      const note: CloakNote = {
        version: "1.0",
        amount: 1_000_000_000,
        commitment: bytesToHex(randomBytes(32)),
        sk_spend: bytesToHex(randomBytes(32)),
        r: bytesToHex(randomBytes(32)),
        timestamp: Date.now(),
        network: "devnet",
        depositSignature: "sig123",
        depositSlot: 100,
        leafIndex: 42,
        root: bytesToHex(randomBytes(32)),
        merkleProof: {
          pathElements: [bytesToHex(randomBytes(32))],
          pathIndices: [0],
        },
      };

      const serialized = serializeNote(note);
      const parsed = parseNote(serialized);

      expect(parsed).toEqual(note);
    });
  });

  describe("isWithdrawable", () => {
    it("should return false for undeposited note", () => {
      const note = generateNote(1_000_000_000);
      expect(isWithdrawable(note)).toBe(false);
    });

    it("should return true for fully deposited note", () => {
      const note = generateNote(1_000_000_000);
      note.depositSignature = "sig123";
      note.leafIndex = 42;
      note.root = bytesToHex(randomBytes(32));
      note.merkleProof = {
        pathElements: [bytesToHex(randomBytes(32))],
        pathIndices: [0],
      };

      expect(isWithdrawable(note)).toBe(true);
    });

    it("should return false if missing deposit signature", () => {
      const note = generateNote(1_000_000_000);
      note.leafIndex = 42;
      note.root = bytesToHex(randomBytes(32));
      note.merkleProof = {
        pathElements: [bytesToHex(randomBytes(32))],
        pathIndices: [0],
      };

      expect(isWithdrawable(note)).toBe(false);
    });

    it("should return false if missing leaf index", () => {
      const note = generateNote(1_000_000_000);
      note.depositSignature = "sig123";
      note.root = bytesToHex(randomBytes(32));
      note.merkleProof = {
        pathElements: [bytesToHex(randomBytes(32))],
        pathIndices: [0],
      };

      expect(isWithdrawable(note)).toBe(false);
    });

    it("should handle leafIndex = 0", () => {
      const note = generateNote(1_000_000_000);
      note.depositSignature = "sig123";
      note.leafIndex = 0; // Valid leaf index
      note.root = bytesToHex(randomBytes(32));
      note.merkleProof = {
        pathElements: [bytesToHex(randomBytes(32))],
        pathIndices: [0],
      };

      expect(isWithdrawable(note)).toBe(true);
    });
  });

  describe("updateNoteWithDeposit", () => {
    it("should add deposit information to note", () => {
      const note = generateNote(1_000_000_000);
      const depositInfo = {
        signature: "sig123",
        slot: 100,
        leafIndex: 42,
        root: bytesToHex(randomBytes(32)),
        merkleProof: {
          pathElements: [bytesToHex(randomBytes(32)), bytesToHex(randomBytes(32))],
          pathIndices: [0, 1],
        },
      };

      const updated = updateNoteWithDeposit(note, depositInfo);

      expect(updated.depositSignature).toBe("sig123");
      expect(updated.depositSlot).toBe(100);
      expect(updated.leafIndex).toBe(42);
      expect(updated.root).toBe(depositInfo.root);
      expect(updated.merkleProof).toEqual(depositInfo.merkleProof);
    });

    it("should not mutate original note", () => {
      const note = generateNote(1_000_000_000);
      const originalNote = { ...note };
      
      const depositInfo = {
        signature: "sig123",
        slot: 100,
        leafIndex: 42,
        root: bytesToHex(randomBytes(32)),
        merkleProof: {
          pathElements: [bytesToHex(randomBytes(32))],
          pathIndices: [0],
        },
      };

      updateNoteWithDeposit(note, depositInfo);

      expect(note).toEqual(originalNote);
      expect(note.depositSignature).toBeUndefined();
    });

    it("should make note withdrawable", () => {
      const note = generateNote(1_000_000_000);
      expect(isWithdrawable(note)).toBe(false);

      const depositInfo = {
        signature: "sig123",
        slot: 100,
        leafIndex: 42,
        root: bytesToHex(randomBytes(32)),
        merkleProof: {
          pathElements: [bytesToHex(randomBytes(32))],
          pathIndices: [0],
        },
      };

      const updated = updateNoteWithDeposit(note, depositInfo);
      expect(isWithdrawable(updated)).toBe(true);
    });

    it("should preserve original note fields", () => {
      const note = generateNote(1_000_000_000, "devnet");
      const depositInfo = {
        signature: "sig123",
        slot: 100,
        leafIndex: 42,
        root: bytesToHex(randomBytes(32)),
        merkleProof: {
          pathElements: [bytesToHex(randomBytes(32))],
          pathIndices: [0],
        },
      };

      const updated = updateNoteWithDeposit(note, depositInfo);

      expect(updated.version).toBe(note.version);
      expect(updated.amount).toBe(note.amount);
      expect(updated.commitment).toBe(note.commitment);
      expect(updated.sk_spend).toBe(note.sk_spend);
      expect(updated.r).toBe(note.r);
      expect(updated.timestamp).toBe(note.timestamp);
      expect(updated.network).toBe(note.network);
    });
  });
});

