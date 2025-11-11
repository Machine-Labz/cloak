import { describe, it, expect, beforeEach } from "@jest/globals";
import {
  generateMasterSeed,
  deriveSpendKey,
  deriveViewKey,
  generateCloakKeys,
  encryptNoteForRecipient,
  tryDecryptNote,
  scanNotesForWallet,
  exportKeys,
  importKeys,
  type CloakKeyPair,
  type NoteData,
} from "../keys";
import { hexToBytes, bytesToHex } from "../../utils/crypto";

describe("Key Management (v2.0)", () => {
  describe("generateMasterSeed", () => {
    it("should generate a 32-byte seed", () => {
      const master = generateMasterSeed();
      expect(master.seed).toBeInstanceOf(Uint8Array);
      expect(master.seed.length).toBe(32);
      expect(master.seedHex).toHaveLength(64);
    });

    it("should generate unique seeds", () => {
      const master1 = generateMasterSeed();
      const master2 = generateMasterSeed();

      expect(master1.seedHex).not.toBe(master2.seedHex);
    });

    it("should have matching hex representation", () => {
      const master = generateMasterSeed();
      expect(bytesToHex(master.seed)).toBe(master.seedHex);
    });
  });

  describe("deriveSpendKey", () => {
    it("should derive 32-byte spend keys", () => {
      const masterSeed = new Uint8Array(32).fill(1);
      const spend = deriveSpendKey(masterSeed);

      expect(spend.sk_spend).toBeInstanceOf(Uint8Array);
      expect(spend.sk_spend.length).toBe(32);
      expect(spend.pk_spend).toBeInstanceOf(Uint8Array);
      expect(spend.pk_spend.length).toBe(32);
    });

    it("should be deterministic", () => {
      const masterSeed = new Uint8Array(32).fill(1);
      const spend1 = deriveSpendKey(masterSeed);
      const spend2 = deriveSpendKey(masterSeed);

      expect(spend1.sk_spend_hex).toBe(spend2.sk_spend_hex);
      expect(spend1.pk_spend_hex).toBe(spend2.pk_spend_hex);
    });

    it("should generate different keys for different seeds", () => {
      const seed1 = new Uint8Array(32).fill(1);
      const seed2 = new Uint8Array(32).fill(2);

      const spend1 = deriveSpendKey(seed1);
      const spend2 = deriveSpendKey(seed2);

      expect(spend1.sk_spend_hex).not.toBe(spend2.sk_spend_hex);
      expect(spend1.pk_spend_hex).not.toBe(spend2.pk_spend_hex);
    });

    it("should have matching hex representations", () => {
      const masterSeed = new Uint8Array(32).fill(1);
      const spend = deriveSpendKey(masterSeed);

      expect(bytesToHex(spend.sk_spend)).toBe(spend.sk_spend_hex);
      expect(bytesToHex(spend.pk_spend)).toBe(spend.pk_spend_hex);
    });
  });

  describe("deriveViewKey", () => {
    it("should derive view keys", () => {
      const sk_spend = new Uint8Array(32).fill(1);
      const view = deriveViewKey(sk_spend);

      expect(view.vk_secret).toBeInstanceOf(Uint8Array);
      expect(view.vk_secret.length).toBe(32);
      expect(view.pvk).toBeInstanceOf(Uint8Array);
      expect(view.pvk.length).toBe(32);
    });

    it("should be deterministic", () => {
      const sk_spend = new Uint8Array(32).fill(1);
      const view1 = deriveViewKey(sk_spend);
      const view2 = deriveViewKey(sk_spend);

      expect(view1.vk_secret_hex).toBe(view2.vk_secret_hex);
      expect(view1.pvk_hex).toBe(view2.pvk_hex);
    });

    it("should generate different keys for different spend keys", () => {
      const sk_spend1 = new Uint8Array(32).fill(1);
      const sk_spend2 = new Uint8Array(32).fill(2);

      const view1 = deriveViewKey(sk_spend1);
      const view2 = deriveViewKey(sk_spend2);

      expect(view1.vk_secret_hex).not.toBe(view2.vk_secret_hex);
      expect(view1.pvk_hex).not.toBe(view2.pvk_hex);
    });
  });

  describe("generateCloakKeys", () => {
    it("should generate complete key hierarchy", () => {
      const keys = generateCloakKeys();

      expect(keys.master.seed.length).toBe(32);
      expect(keys.spend.sk_spend.length).toBe(32);
      expect(keys.spend.pk_spend.length).toBe(32);
      expect(keys.view.vk_secret.length).toBe(32);
      expect(keys.view.pvk.length).toBe(32);
    });

    it("should be deterministic with provided seed", () => {
      const masterSeed = new Uint8Array(32).fill(42);
      
      const keys1 = generateCloakKeys(masterSeed);
      const keys2 = generateCloakKeys(masterSeed);

      expect(keys1.master.seedHex).toBe(keys2.master.seedHex);
      expect(keys1.spend.sk_spend_hex).toBe(keys2.spend.sk_spend_hex);
      expect(keys1.view.pvk_hex).toBe(keys2.view.pvk_hex);
    });

    it("should generate random keys when no seed provided", () => {
      const keys1 = generateCloakKeys();
      const keys2 = generateCloakKeys();

      expect(keys1.master.seedHex).not.toBe(keys2.master.seedHex);
      expect(keys1.spend.sk_spend_hex).not.toBe(keys2.spend.sk_spend_hex);
    });

    it("should derive consistent hierarchy", () => {
      const masterSeed = new Uint8Array(32).fill(1);
      const keys = generateCloakKeys(masterSeed);

      // Re-derive spend and view keys manually
      const spend = deriveSpendKey(masterSeed);
      const view = deriveViewKey(spend.sk_spend);

      expect(keys.spend.sk_spend_hex).toBe(spend.sk_spend_hex);
      expect(keys.view.pvk_hex).toBe(view.pvk_hex);
    });
  });

  describe("encryptNoteForRecipient and tryDecryptNote", () => {
    let senderKeys: CloakKeyPair;
    let recipientKeys: CloakKeyPair;
    let noteData: NoteData;

    beforeEach(() => {
      senderKeys = generateCloakKeys();
      recipientKeys = generateCloakKeys();
      noteData = {
        amount: 1_000_000_000,
        r: "a".repeat(64),
        sk_spend: senderKeys.spend.sk_spend_hex,
        commitment: "b".repeat(64),
      };
    });

    it("should encrypt and decrypt note successfully", () => {
      const encrypted = encryptNoteForRecipient(noteData, recipientKeys.view.pvk);
      const decrypted = tryDecryptNote(encrypted, recipientKeys.view);

      expect(decrypted).not.toBeNull();
      expect(decrypted?.amount).toBe(noteData.amount);
      expect(decrypted?.r).toBe(noteData.r);
      expect(decrypted?.sk_spend).toBe(noteData.sk_spend);
      expect(decrypted?.commitment).toBe(noteData.commitment);
    });

    it("should return null when decrypting with wrong key", () => {
      const encrypted = encryptNoteForRecipient(noteData, recipientKeys.view.pvk);
      const wrongKeys = generateCloakKeys();
      const decrypted = tryDecryptNote(encrypted, wrongKeys.view);

      expect(decrypted).toBeNull();
    });

    it("should generate unique ephemeral keys", () => {
      const encrypted1 = encryptNoteForRecipient(noteData, recipientKeys.view.pvk);
      const encrypted2 = encryptNoteForRecipient(noteData, recipientKeys.view.pvk);

      expect(encrypted1.ephemeral_pk).not.toBe(encrypted2.ephemeral_pk);
      expect(encrypted1.nonce).not.toBe(encrypted2.nonce);
      expect(encrypted1.ciphertext).not.toBe(encrypted2.ciphertext);
    });

    it("should have valid encrypted note structure", () => {
      const encrypted = encryptNoteForRecipient(noteData, recipientKeys.view.pvk);

      expect(encrypted.ephemeral_pk).toHaveLength(64); // 32 bytes in hex
      expect(encrypted.nonce).toHaveLength(48); // 24 bytes in hex
      expect(encrypted.ciphertext.length).toBeGreaterThan(0);
    });

    it("should preserve all note data fields", () => {
      const complexNoteData: NoteData = {
        amount: 123_456_789,
        r: "1234567890abcdef".repeat(4),
        sk_spend: "fedcba0987654321".repeat(4),
        commitment: "abcdef1234567890".repeat(4),
      };

      const encrypted = encryptNoteForRecipient(complexNoteData, recipientKeys.view.pvk);
      const decrypted = tryDecryptNote(encrypted, recipientKeys.view);

      expect(decrypted).toEqual(complexNoteData);
    });

    it("should handle malformed encrypted data", () => {
      const malformed = {
        ephemeral_pk: "invalid",
        ciphertext: "invalid",
        nonce: "invalid",
      };

      const decrypted = tryDecryptNote(malformed, recipientKeys.view);
      expect(decrypted).toBeNull();
    });
  });

  describe("scanNotesForWallet", () => {
    let recipientKeys: CloakKeyPair;
    let otherKeys: CloakKeyPair;

    beforeEach(() => {
      recipientKeys = generateCloakKeys();
      otherKeys = generateCloakKeys();
    });

    it("should find notes encrypted to recipient", () => {
      const noteData1: NoteData = {
        amount: 1_000_000_000,
        r: "a".repeat(64),
        sk_spend: recipientKeys.spend.sk_spend_hex,
        commitment: "b".repeat(64),
      };
      const noteData2: NoteData = {
        amount: 2_000_000_000,
        r: "c".repeat(64),
        sk_spend: recipientKeys.spend.sk_spend_hex,
        commitment: "d".repeat(64),
      };

      const encrypted1 = encryptNoteForRecipient(noteData1, recipientKeys.view.pvk);
      const encrypted2 = encryptNoteForRecipient(noteData2, recipientKeys.view.pvk);

      const encryptedOutputs = [
        btoa(JSON.stringify(encrypted1)),
        btoa(JSON.stringify(encrypted2)),
      ];

      const foundNotes = scanNotesForWallet(encryptedOutputs, recipientKeys.view);

      expect(foundNotes).toHaveLength(2);
      expect(foundNotes[0].amount).toBe(1_000_000_000);
      expect(foundNotes[1].amount).toBe(2_000_000_000);
    });

    it("should ignore notes encrypted to other recipients", () => {
      const noteData: NoteData = {
        amount: 1_000_000_000,
        r: "a".repeat(64),
        sk_spend: otherKeys.spend.sk_spend_hex,
        commitment: "b".repeat(64),
      };

      const encrypted = encryptNoteForRecipient(noteData, otherKeys.view.pvk);
      const encryptedOutputs = [btoa(JSON.stringify(encrypted))];

      const foundNotes = scanNotesForWallet(encryptedOutputs, recipientKeys.view);
      expect(foundNotes).toHaveLength(0);
    });

    it("should handle mixed encrypted outputs", () => {
      const myNote: NoteData = {
        amount: 1_000_000_000,
        r: "a".repeat(64),
        sk_spend: recipientKeys.spend.sk_spend_hex,
        commitment: "b".repeat(64),
      };
      const theirNote: NoteData = {
        amount: 2_000_000_000,
        r: "c".repeat(64),
        sk_spend: otherKeys.spend.sk_spend_hex,
        commitment: "d".repeat(64),
      };

      const encryptedMine = encryptNoteForRecipient(myNote, recipientKeys.view.pvk);
      const encryptedTheirs = encryptNoteForRecipient(theirNote, otherKeys.view.pvk);

      const encryptedOutputs = [
        btoa(JSON.stringify(encryptedMine)),
        btoa(JSON.stringify(encryptedTheirs)),
      ];

      const foundNotes = scanNotesForWallet(encryptedOutputs, recipientKeys.view);

      expect(foundNotes).toHaveLength(1);
      expect(foundNotes[0].amount).toBe(1_000_000_000);
    });

    it("should handle malformed outputs gracefully", () => {
      const validNote: NoteData = {
        amount: 1_000_000_000,
        r: "a".repeat(64),
        sk_spend: recipientKeys.spend.sk_spend_hex,
        commitment: "b".repeat(64),
      };

      const encrypted = encryptNoteForRecipient(validNote, recipientKeys.view.pvk);

      const encryptedOutputs = [
        "invalid base64",
        btoa("invalid json"),
        btoa(JSON.stringify(encrypted)),
      ];

      const foundNotes = scanNotesForWallet(encryptedOutputs, recipientKeys.view);

      expect(foundNotes).toHaveLength(1);
      expect(foundNotes[0].amount).toBe(1_000_000_000);
    });

    it("should handle empty outputs", () => {
      const foundNotes = scanNotesForWallet([], recipientKeys.view);
      expect(foundNotes).toHaveLength(0);
    });
  });

  describe("exportKeys and importKeys", () => {
    it("should export keys as JSON", () => {
      const keys = generateCloakKeys();
      const exported = exportKeys(keys);

      const parsed = JSON.parse(exported);
      expect(parsed.version).toBe("2.0");
      expect(parsed.master_seed).toBe(keys.master.seedHex);
      expect(parsed.sk_spend).toBe(keys.spend.sk_spend_hex);
      expect(parsed.pk_spend).toBe(keys.spend.pk_spend_hex);
      expect(parsed.vk_secret).toBe(keys.view.vk_secret_hex);
      expect(parsed.pvk).toBe(keys.view.pvk_hex);
    });

    it("should import exported keys", () => {
      const original = generateCloakKeys();
      const exported = exportKeys(original);
      const imported = importKeys(exported);

      expect(imported.master.seedHex).toBe(original.master.seedHex);
      expect(imported.spend.sk_spend_hex).toBe(original.spend.sk_spend_hex);
      expect(imported.spend.pk_spend_hex).toBe(original.spend.pk_spend_hex);
      expect(imported.view.vk_secret_hex).toBe(original.view.vk_secret_hex);
      expect(imported.view.pvk_hex).toBe(original.view.pvk_hex);
    });

    it("should re-derive keys from master seed on import", () => {
      const masterSeed = new Uint8Array(32).fill(42);
      const original = generateCloakKeys(masterSeed);
      const exported = exportKeys(original);
      const imported = importKeys(exported);

      // Keys should be re-derived, not just copied
      const rederived = generateCloakKeys(masterSeed);
      expect(imported.spend.sk_spend_hex).toBe(rederived.spend.sk_spend_hex);
      expect(imported.view.pvk_hex).toBe(rederived.view.pvk_hex);
    });

    it("should maintain key hierarchy consistency", () => {
      const keys = generateCloakKeys();
      const exported = exportKeys(keys);
      const imported = importKeys(exported);

      // Verify hierarchy by re-deriving
      const spendFromMaster = deriveSpendKey(imported.master.seed);
      const viewFromSpend = deriveViewKey(spendFromMaster.sk_spend);

      expect(imported.spend.sk_spend_hex).toBe(spendFromMaster.sk_spend_hex);
      expect(imported.view.pvk_hex).toBe(viewFromSpend.pvk_hex);
    });

    it("should handle round-trip for multiple exports/imports", () => {
      const original = generateCloakKeys();
      
      const exported1 = exportKeys(original);
      const imported1 = importKeys(exported1);
      
      const exported2 = exportKeys(imported1);
      const imported2 = importKeys(exported2);

      expect(imported2.master.seedHex).toBe(original.master.seedHex);
      expect(imported2.spend.sk_spend_hex).toBe(original.spend.sk_spend_hex);
      expect(imported2.view.pvk_hex).toBe(original.view.pvk_hex);
    });
  });
});

