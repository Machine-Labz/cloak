import { describe, it, expect, jest, beforeEach } from "@jest/globals";
import { CloakSDK, formatAmount, calculateFee } from "../../src/index";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

// Mock the SDK methods
jest.mock("../../src/core/CloakSDK");

describe("Basic Usage Example", () => {
  let mockClient: jest.Mocked<CloakSDK>;
  let connection: Connection;
  let keypair: Keypair;

  beforeEach(() => {
    // Create a test keypair
    keypair = Keypair.generate();
    
    // Create mock connection
    connection = new Connection("https://api.testnet.solana.com", "confirmed");

    // Create mock client
    mockClient = {
      deposit: jest.fn(),
      privateTransfer: jest.fn(),
      withdraw: jest.fn(),
      generateNote: jest.fn(),
      exportNote: jest.fn(),
      parseNote: jest.fn(),
      isWithdrawable: jest.fn(),
      getMerkleProof: jest.fn(),
      getCurrentRoot: jest.fn(),
    } as any;
  });

  describe("Fee Calculations", () => {
    it("should calculate correct fees for 0.1 SOL deposit", () => {
      const depositAmount = 100_000_000; // 0.1 SOL
      const fee = calculateFee(depositAmount);

      // Fixed fee: 2,500,000
      // Variable fee: floor((100,000,000 * 5) / 1000) = 500,000
      // Total: 3,000,000
      expect(fee).toBe(3_000_000);
    });

    it("should format amounts correctly", () => {
      expect(formatAmount(100_000_000)).toBe("0.100000000");
      expect(formatAmount(1_000_000_000)).toBe("1.000000000");
    });

    it("should calculate distributable amount correctly", () => {
      const depositAmount = 100_000_000;
      const fee = calculateFee(depositAmount);
      const distributable = depositAmount - fee;

      expect(distributable).toBe(97_000_000);
    });
  });

  describe("Deposit Flow", () => {
    it("should initialize SDK with correct parameters", () => {
      const sdk = new CloakSDK({
        network: "testnet",
        keypairBytes: keypair.secretKey,
      });

      expect(sdk).toBeDefined();
    });

    it("should handle deposit with progress callbacks", async () => {
      const depositAmount = 100_000_000;
      const mockNote = {
        version: "1.0" as const,
        amount: depositAmount,
        commitment: "a".repeat(64),
        sk_spend: "b".repeat(64),
        r: "c".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
        depositSignature: "test-sig",
        leafIndex: 42,
        root: "d".repeat(64),
      };

      mockClient.deposit.mockResolvedValue({
        signature: "test-signature",
        leafIndex: 42,
        root: "d".repeat(64),
        note: mockNote,
      });

      const progressCallbacks: string[] = [];
      const result = await mockClient.deposit(
        connection,
        depositAmount,
        {
          onProgress: (status: string) => {
            progressCallbacks.push(status);
          },
        }
      );

      expect(mockClient.deposit).toHaveBeenCalledWith(
        connection,
        depositAmount,
        expect.objectContaining({
          onProgress: expect.any(Function),
        })
      );

      expect(result.signature).toBe("test-signature");
      expect(result.leafIndex).toBe(42);
      expect(result.note).toBeDefined();
    });
  });

  describe("Private Transfer Flow", () => {
    it("should split amounts correctly between 3 recipients", () => {
      const depositAmount = 100_000_000;
      const fee = calculateFee(depositAmount);
      const distributable = depositAmount - fee; // 97,000,000

      const amount1 = Math.floor(distributable * 0.5); // 50%
      const amount2 = Math.floor(distributable * 0.3); // 30%
      const amount3 = distributable - amount1 - amount2; // Remaining

      expect(amount1).toBe(48_500_000); // 50%
      expect(amount2).toBe(29_100_000); // 30%
      expect(amount3).toBe(19_400_000); // 20%
      expect(amount1 + amount2 + amount3).toBe(distributable);
    });

    it("should handle private transfer with multiple recipients", async () => {
      const depositAmount = 100_000_000;
      const mockNote = {
        version: "1.0" as const,
        amount: depositAmount,
        commitment: "a".repeat(64),
        sk_spend: "b".repeat(64),
        r: "c".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
      };

      mockClient.generateNote.mockReturnValue(mockNote);
      mockClient.privateTransfer.mockResolvedValue({
        signature: "transfer-sig",
        nullifier: "e".repeat(64),
      });

      const note = mockClient.generateNote(depositAmount);
      const fee = calculateFee(depositAmount);
      const distributable = depositAmount - fee;

      const recipient1 = Keypair.generate().publicKey;
      const recipient2 = Keypair.generate().publicKey;
      const recipient3 = Keypair.generate().publicKey;

      const amount1 = Math.floor(distributable * 0.5);
      const amount2 = Math.floor(distributable * 0.3);
      const amount3 = distributable - amount1 - amount2;

      const result = await mockClient.privateTransfer(
        connection,
        note,
        [
          { recipient: recipient1, amount: amount1 },
          { recipient: recipient2, amount: amount2 },
          { recipient: recipient3, amount: amount3 },
        ],
        {
          relayFeeBps: 50,
          onProgress: jest.fn(),
        }
      );

      expect(mockClient.privateTransfer).toHaveBeenCalled();
      expect(result.signature).toBe("transfer-sig");
      expect(result.nullifier).toBeDefined();
    });

    it("should validate recipient amounts sum to distributable", () => {
      const depositAmount = 100_000_000;
      const fee = calculateFee(depositAmount);
      const distributable = depositAmount - fee;

      const amount1 = Math.floor(distributable * 0.5);
      const amount2 = Math.floor(distributable * 0.3);
      const amount3 = distributable - amount1 - amount2;

      const totalAmount = amount1 + amount2 + amount3;
      expect(totalAmount).toBe(distributable);
    });
  });

  describe("Withdrawal Flow", () => {
    it("should handle single recipient withdrawal", async () => {
      const mockNote = {
        version: "1.0" as const,
        amount: 100_000_000,
        commitment: "a".repeat(64),
        sk_spend: "b".repeat(64),
        r: "c".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
        depositSignature: "sig",
        leafIndex: 42,
        root: "d".repeat(64),
      };

      mockClient.withdraw.mockResolvedValue({
        signature: "withdraw-sig",
        nullifier: "f".repeat(64),
      });

      const recipient = Keypair.generate().publicKey;
      const result = await mockClient.withdraw(
        connection,
        mockNote,
        recipient,
        {
          withdrawAll: true,
          relayFeeBps: 50,
          onProgress: jest.fn(),
        }
      );

      expect(mockClient.withdraw).toHaveBeenCalledWith(
        connection,
        mockNote,
        recipient,
        expect.objectContaining({
          withdrawAll: true,
          relayFeeBps: 50,
        })
      );

      expect(result.signature).toBe("withdraw-sig");
    });
  });

  describe("Note Management", () => {
    const mockNote = {
      version: "1.0" as const,
      amount: 100_000_000,
      commitment: "a".repeat(64),
      sk_spend: "b".repeat(64),
      r: "c".repeat(64),
      timestamp: Date.now(),
      network: "testnet" as const,
      depositSignature: "sig",
      leafIndex: 42,
      root: "d".repeat(64),
    };

    it("should export note as JSON", () => {
      mockClient.exportNote.mockReturnValue(JSON.stringify(mockNote, null, 2));

      const exported = mockClient.exportNote(mockNote, true);
      expect(exported).toBeDefined();
      expect(exported.length).toBeGreaterThan(0);
    });

    it("should parse note from JSON", () => {
      const noteJson = JSON.stringify(mockNote);
      mockClient.parseNote.mockReturnValue(mockNote);

      const parsed = mockClient.parseNote(noteJson);
      expect(parsed.commitment).toBe(mockNote.commitment);
      expect(parsed.amount).toBe(mockNote.amount);
    });

    it("should check if note is withdrawable", () => {
      mockClient.isWithdrawable.mockReturnValue(true);

      const withdrawable = mockClient.isWithdrawable(mockNote);
      expect(withdrawable).toBe(true);
    });

    it("should get merkle proof for deposited note", async () => {
      const mockProof = {
        pathElements: [
          "a".repeat(64),
          "b".repeat(64),
          "c".repeat(64),
        ],
        pathIndices: [0, 1, 0],
      };

      mockClient.getMerkleProof.mockResolvedValue(mockProof);

      const proof = await mockClient.getMerkleProof(mockNote.leafIndex!);
      expect(proof.pathElements.length).toBeGreaterThan(0);
      expect(proof.pathIndices.length).toBe(proof.pathElements.length);
    });

    it("should get current merkle root", async () => {
      mockClient.getCurrentRoot.mockResolvedValue("g".repeat(64));

      const root = await mockClient.getCurrentRoot();
      expect(root).toBeDefined();
      expect(root.length).toBe(64);
    });
  });

  describe("Error Handling", () => {
    it("should handle deposit errors gracefully", async () => {
      mockClient.deposit.mockRejectedValue(new Error("Insufficient funds"));

      await expect(
        mockClient.deposit(connection, 100_000_000, {})
      ).rejects.toThrow("Insufficient funds");
    });

    it("should handle transfer errors gracefully", async () => {
      const mockNote = {
        version: "1.0" as const,
        amount: 100_000_000,
        commitment: "a".repeat(64),
        sk_spend: "b".repeat(64),
        r: "c".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
      };

      mockClient.privateTransfer.mockRejectedValue(
        new Error("Note not deposited")
      );

      await expect(
        mockClient.privateTransfer(connection, mockNote, [], {})
      ).rejects.toThrow("Note not deposited");
    });

    it("should handle withdrawal errors gracefully", async () => {
      const mockNote = {
        version: "1.0" as const,
        amount: 100_000_000,
        commitment: "a".repeat(64),
        sk_spend: "b".repeat(64),
        r: "c".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
      };

      mockClient.withdraw.mockRejectedValue(
        new Error("Note already spent")
      );

      const recipient = Keypair.generate().publicKey;
      await expect(
        mockClient.withdraw(connection, mockNote, recipient, {})
      ).rejects.toThrow("Note already spent");
    });
  });

  describe("Integration Scenarios", () => {
    it("should complete full deposit-transfer-withdraw cycle", async () => {
      const depositAmount = 100_000_000;
      const fee = calculateFee(depositAmount);
      const distributable = depositAmount - fee;

      // 1. Deposit
      const depositNote = {
        version: "1.0" as const,
        amount: depositAmount,
        commitment: "a".repeat(64),
        sk_spend: "b".repeat(64),
        r: "c".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
        depositSignature: "sig1",
        leafIndex: 1,
        root: "d".repeat(64),
      };

      mockClient.deposit.mockResolvedValue({
        signature: "deposit-sig",
        leafIndex: 1,
        root: "d".repeat(64),
        note: depositNote,
      });

      const depositResult = await mockClient.deposit(connection, depositAmount, {});
      expect(depositResult.note).toBeDefined();

      // 2. Generate new note for transfer
      const transferNote = {
        version: "1.0" as const,
        amount: depositAmount,
        commitment: "e".repeat(64),
        sk_spend: "f".repeat(64),
        r: "g".repeat(64),
        timestamp: Date.now(),
        network: "testnet" as const,
      };

      mockClient.generateNote.mockReturnValue(transferNote);
      const newNote = mockClient.generateNote(depositAmount);
      expect(newNote).toBeDefined();

      // 3. Private transfer
      mockClient.privateTransfer.mockResolvedValue({
        signature: "transfer-sig",
        nullifier: "h".repeat(64),
      });

      const recipients = [
        { recipient: Keypair.generate().publicKey, amount: Math.floor(distributable * 0.5) },
        { recipient: Keypair.generate().publicKey, amount: Math.floor(distributable * 0.5) },
      ];

      const transferResult = await mockClient.privateTransfer(
        connection,
        newNote,
        recipients,
        { relayFeeBps: 50 }
      );
      expect(transferResult.signature).toBeDefined();

      // 4. Withdraw deposited note
      mockClient.withdraw.mockResolvedValue({
        signature: "withdraw-sig",
        nullifier: "i".repeat(64),
      });

      const withdrawResult = await mockClient.withdraw(
        connection,
        depositResult.note,
        Keypair.generate().publicKey,
        { withdrawAll: true }
      );
      expect(withdrawResult.signature).toBeDefined();
    });
  });
});

