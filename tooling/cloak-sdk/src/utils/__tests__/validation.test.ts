import { describe, it, expect } from "@jest/globals";
import { PublicKey } from "@solana/web3.js";
import {
  isValidSolanaAddress,
  validateNote,
  validateWithdrawableNote,
  validateTransfers,
} from "../validation";
import { CloakNote } from "../../core/types";
import { randomBytes, bytesToHex } from "../crypto";

describe("Validation Utils", () => {
  describe("isValidSolanaAddress", () => {
    it("should validate correct Solana addresses", () => {
      expect(isValidSolanaAddress("11111111111111111111111111111111")).toBe(true);
      expect(isValidSolanaAddress("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")).toBe(true);
    });

    it("should reject invalid addresses", () => {
      expect(isValidSolanaAddress("invalid")).toBe(false);
      expect(isValidSolanaAddress("")).toBe(false);
      expect(isValidSolanaAddress("0x1234")).toBe(false);
    });

    it("should reject addresses with invalid base58 characters", () => {
      expect(isValidSolanaAddress("000000000000000000000000000000O0")).toBe(false); // Contains 'O'
      expect(isValidSolanaAddress("000000000000000000000000000000I0")).toBe(false); // Contains 'I'
    });
  });

  describe("validateNote", () => {
    const createValidNote = (): CloakNote => ({
      version: "1.0",
      amount: 1_000_000_000,
      commitment: bytesToHex(randomBytes(32)),
      sk_spend: bytesToHex(randomBytes(32)),
      r: bytesToHex(randomBytes(32)),
      timestamp: Date.now(),
      network: "devnet",
    });

    it("should validate a correct note", () => {
      const note = createValidNote();
      expect(() => validateNote(note)).not.toThrow();
    });

    it("should reject non-object", () => {
      expect(() => validateNote(null)).toThrow("Note must be an object");
      expect(() => validateNote(undefined)).toThrow("Note must be an object");
      expect(() => validateNote("string")).toThrow("Note must be an object");
    });

    it("should reject missing required fields", () => {
      const note = createValidNote();
      
      delete (note as any).version;
      expect(() => validateNote(note)).toThrow("Missing required field: version");

      const note2 = createValidNote();
      delete (note2 as any).amount;
      expect(() => validateNote(note2)).toThrow("Missing required field: amount");

      const note3 = createValidNote();
      delete (note3 as any).commitment;
      expect(() => validateNote(note3)).toThrow("Missing required field: commitment");
    });

    it("should reject invalid version type", () => {
      const note = createValidNote();
      (note as any).version = 123;
      expect(() => validateNote(note)).toThrow("Version must be a string");
    });

    it("should reject invalid amount", () => {
      const note1 = createValidNote();
      note1.amount = 0;
      expect(() => validateNote(note1)).toThrow("Amount must be a positive number");

      const note2 = createValidNote();
      note2.amount = -100;
      expect(() => validateNote(note2)).toThrow("Amount must be a positive number");

      const note3 = createValidNote();
      (note3 as any).amount = "1000000000";
      expect(() => validateNote(note3)).toThrow("Amount must be a positive number");
    });

    it("should reject invalid commitment format", () => {
      const note = createValidNote();
      note.commitment = "invalid";
      expect(() => validateNote(note)).toThrow("Invalid commitment format");

      const note2 = createValidNote();
      note2.commitment = bytesToHex(randomBytes(16)); // Wrong length
      expect(() => validateNote(note2)).toThrow("Invalid commitment format");
    });

    it("should reject invalid sk_spend format", () => {
      const note = createValidNote();
      note.sk_spend = "xyz";
      expect(() => validateNote(note)).toThrow("Invalid sk_spend format");
    });

    it("should reject invalid r format", () => {
      const note = createValidNote();
      note.r = bytesToHex(randomBytes(16)); // Wrong length
      expect(() => validateNote(note)).toThrow("Invalid r format");
    });

    it("should reject invalid timestamp", () => {
      const note1 = createValidNote();
      note1.timestamp = 0;
      expect(() => validateNote(note1)).toThrow("Timestamp must be a positive number");

      const note2 = createValidNote();
      (note2 as any).timestamp = "123";
      expect(() => validateNote(note2)).toThrow("Timestamp must be a positive number");
    });

    it("should reject invalid network", () => {
      const note = createValidNote();
      (note as any).network = "testnet";
      expect(() => validateNote(note)).toThrow("Network must be localnet, devnet, or mainnet");
    });

    it("should validate optional deposit fields", () => {
      const note = createValidNote();
      note.depositSignature = "abc123";
      note.depositSlot = 100;
      note.leafIndex = 42;
      note.root = bytesToHex(randomBytes(32));
      note.merkleProof = {
        pathElements: [bytesToHex(randomBytes(32))],
        pathIndices: [0],
      };

      expect(() => validateNote(note)).not.toThrow();
    });

    it("should reject invalid merkle proof", () => {
      const note = createValidNote();
      note.merkleProof = {
        pathElements: [bytesToHex(randomBytes(32)), bytesToHex(randomBytes(32))],
        pathIndices: [0], // Mismatched length
      };

      expect(() => validateNote(note)).toThrow(
        "Merkle proof pathElements and pathIndices must have same length"
      );
    });
  });

  describe("validateWithdrawableNote", () => {
    const createWithdrawableNote = (): CloakNote => ({
      version: "1.0",
      amount: 1_000_000_000,
      commitment: bytesToHex(randomBytes(32)),
      sk_spend: bytesToHex(randomBytes(32)),
      r: bytesToHex(randomBytes(32)),
      timestamp: Date.now(),
      network: "devnet",
      depositSignature: "abc123",
      depositSlot: 100,
      leafIndex: 42,
      root: bytesToHex(randomBytes(32)),
      merkleProof: {
        pathElements: [bytesToHex(randomBytes(32))],
        pathIndices: [0],
      },
    });

    it("should validate a withdrawable note", () => {
      const note = createWithdrawableNote();
      expect(() => validateWithdrawableNote(note)).not.toThrow();
    });

    it("should reject note without deposit signature", () => {
      const note = createWithdrawableNote();
      delete note.depositSignature;
      expect(() => validateWithdrawableNote(note)).toThrow(
        "Note must be deposited before withdrawal (missing depositSignature)"
      );
    });

    it("should reject note without leaf index", () => {
      const note = createWithdrawableNote();
      delete note.leafIndex;
      expect(() => validateWithdrawableNote(note)).toThrow(
        "Note must be deposited before withdrawal (missing leafIndex)"
      );
    });

    it("should reject note without root", () => {
      const note = createWithdrawableNote();
      delete note.root;
      expect(() => validateWithdrawableNote(note)).toThrow(
        "Note must have historical root for withdrawal"
      );
    });

    it("should reject note without merkle proof", () => {
      const note = createWithdrawableNote();
      delete note.merkleProof;
      expect(() => validateWithdrawableNote(note)).toThrow(
        "Note must have Merkle proof for withdrawal"
      );
    });

    it("should reject note with empty merkle proof", () => {
      const note = createWithdrawableNote();
      note.merkleProof = { pathElements: [], pathIndices: [] };
      expect(() => validateWithdrawableNote(note)).toThrow("Merkle proof is empty");
    });
  });

  describe("validateTransfers", () => {
    const recipient1 = new PublicKey("11111111111111111111111111111111");
    const recipient2 = new PublicKey("11111111111111111111111111111112");

    it("should validate correct transfers", () => {
      const transfers = [
        { recipient: recipient1, amount: 500_000_000 },
        { recipient: recipient2, amount: 500_000_000 },
      ];

      expect(() => validateTransfers(transfers, 1_000_000_000)).not.toThrow();
    });

    it("should validate single recipient", () => {
      const transfers = [{ recipient: recipient1, amount: 1_000_000_000 }];
      expect(() => validateTransfers(transfers, 1_000_000_000)).not.toThrow();
    });

    it("should reject empty recipients", () => {
      expect(() => validateTransfers([], 1_000_000_000)).toThrow(
        "At least one recipient is required"
      );
    });

    it("should reject more than 5 recipients", () => {
      const transfers = Array(6)
        .fill(null)
        .map(() => ({ recipient: recipient1, amount: 100_000_000 }));

      expect(() => validateTransfers(transfers, 600_000_000)).toThrow(
        "Maximum 5 recipients allowed"
      );
    });

    it("should reject invalid recipient type", () => {
      const transfers = [{ recipient: "invalid" as any, amount: 1_000_000_000 }];
      expect(() => validateTransfers(transfers, 1_000_000_000)).toThrow(
        "Recipient 0 must be a PublicKey"
      );
    });

    it("should reject invalid amount", () => {
      const transfers = [{ recipient: recipient1, amount: 0 }];
      expect(() => validateTransfers(transfers, 0)).toThrow(
        "Recipient 0 amount must be a positive number"
      );

      const transfers2 = [{ recipient: recipient1, amount: -100 }];
      expect(() => validateTransfers(transfers2, -100)).toThrow(
        "Recipient 0 amount must be a positive number"
      );
    });

    it("should reject mismatched sum", () => {
      const transfers = [
        { recipient: recipient1, amount: 500_000_000 },
        { recipient: recipient2, amount: 400_000_000 }, // Only 900M total
      ];

      expect(() => validateTransfers(transfers, 1_000_000_000)).toThrow(
        "Recipients sum (900000000) does not match expected total (1000000000)"
      );
    });

    it("should validate up to 5 recipients", () => {
      const transfers = [
        { recipient: recipient1, amount: 200_000_000 },
        { recipient: recipient2, amount: 200_000_000 },
        { recipient: recipient1, amount: 200_000_000 },
        { recipient: recipient2, amount: 200_000_000 },
        { recipient: recipient1, amount: 200_000_000 },
      ];

      expect(() => validateTransfers(transfers, 1_000_000_000)).not.toThrow();
    });
  });
});

