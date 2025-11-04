import { describe, it, expect } from "@jest/globals";
import {
  FIXED_FEE_LAMPORTS,
  VARIABLE_FEE_RATE,
  LAMPORTS_PER_SOL,
  calculateFee,
  getDistributableAmount,
  formatAmount,
  parseAmount,
  validateOutputsSum,
  calculateRelayFee,
} from "../fees";

describe("Fee Utils", () => {
  describe("constants", () => {
    it("should have correct constant values", () => {
      expect(FIXED_FEE_LAMPORTS).toBe(2_500_000);
      expect(VARIABLE_FEE_RATE).toBe(0.005);
      expect(LAMPORTS_PER_SOL).toBe(1_000_000_000);
    });
  });

  describe("calculateFee", () => {
    it("should calculate fee for 1 SOL", () => {
      const amount = 1_000_000_000; // 1 SOL
      const fee = calculateFee(amount);
      
      // Fixed: 2,500,000
      // Variable: floor((1,000,000,000 * 5) / 1000) = 5,000,000
      // Total: 7,500,000
      expect(fee).toBe(7_500_000);
    });

    it("should calculate fee for 0.1 SOL", () => {
      const amount = 100_000_000; // 0.1 SOL
      const fee = calculateFee(amount);
      
      // Fixed: 2,500,000
      // Variable: floor((100,000,000 * 5) / 1000) = 500,000
      // Total: 3,000,000
      expect(fee).toBe(3_000_000);
    });

    it("should calculate fee for 10 SOL", () => {
      const amount = 10_000_000_000; // 10 SOL
      const fee = calculateFee(amount);
      
      // Fixed: 2,500,000
      // Variable: floor((10,000,000,000 * 5) / 1000) = 50,000,000
      // Total: 52,500,000
      expect(fee).toBe(52_500_000);
    });

    it("should handle very small amounts", () => {
      const amount = 1_000; // 0.000001 SOL
      const fee = calculateFee(amount);
      
      // Fixed: 2,500,000
      // Variable: floor((1000 * 5) / 1000) = 5
      // Total: 2,500,005
      expect(fee).toBe(2_500_005);
    });

    it("should use floor for variable fee calculation", () => {
      const amount = 999; // Should floor to 4
      const fee = calculateFee(amount);
      expect(fee).toBe(2_500_004); // 2,500,000 + 4
    });
  });

  describe("getDistributableAmount", () => {
    it("should calculate distributable amount for 1 SOL", () => {
      const amount = 1_000_000_000;
      const distributable = getDistributableAmount(amount);
      
      // 1 SOL - 7.5M fee = 992.5M
      expect(distributable).toBe(992_500_000);
    });

    it("should calculate distributable amount for 0.1 SOL", () => {
      const amount = 100_000_000;
      const distributable = getDistributableAmount(amount);
      
      // 0.1 SOL - 3M fee = 97M
      expect(distributable).toBe(97_000_000);
    });

    it("should handle edge case where fee equals amount", () => {
      const amount = 2_500_000; // Exactly the fixed fee
      const distributable = getDistributableAmount(amount);
      
      // Fee = 2,500,000 + floor((2,500,000 * 5) / 1000) = 2,500,000 + 12,500 = 2,512,500
      // Distributable = 2,500,000 - 2,512,500 = -12,500 (negative!)
      expect(distributable).toBeLessThan(0);
    });
  });

  describe("formatAmount", () => {
    it("should format 1 SOL correctly", () => {
      expect(formatAmount(1_000_000_000)).toBe("1.000000000");
    });

    it("should format 0.5 SOL correctly", () => {
      expect(formatAmount(500_000_000)).toBe("0.500000000");
    });

    it("should format with custom decimals", () => {
      expect(formatAmount(1_000_000_000, 2)).toBe("1.00");
      expect(formatAmount(1_500_000_000, 2)).toBe("1.50");
    });

    it("should handle zero", () => {
      expect(formatAmount(0)).toBe("0.000000000");
      expect(formatAmount(0, 2)).toBe("0.00");
    });

    it("should handle very large amounts", () => {
      expect(formatAmount(1_000_000_000_000_000)).toBe("1000000.000000000");
    });

    it("should handle very small amounts", () => {
      expect(formatAmount(1)).toBe("0.000000001");
      expect(formatAmount(100)).toBe("0.000000100");
    });
  });

  describe("parseAmount", () => {
    it("should parse 1 SOL correctly", () => {
      expect(parseAmount("1")).toBe(1_000_000_000);
      expect(parseAmount("1.0")).toBe(1_000_000_000);
    });

    it("should parse 0.5 SOL correctly", () => {
      expect(parseAmount("0.5")).toBe(500_000_000);
    });

    it("should parse very small amounts", () => {
      expect(parseAmount("0.001")).toBe(1_000_000);
      expect(parseAmount("0.0001")).toBe(100_000);
    });

    it("should handle zero", () => {
      expect(parseAmount("0")).toBe(0);
      expect(parseAmount("0.0")).toBe(0);
    });

    it("should throw on negative amounts", () => {
      expect(() => parseAmount("-1")).toThrow("Invalid SOL amount");
    });

    it("should throw on invalid input", () => {
      expect(() => parseAmount("abc")).toThrow("Invalid SOL amount");
      expect(() => parseAmount("")).toThrow("Invalid SOL amount");
    });

    it("should floor fractional lamports", () => {
      // 1.5000000005 SOL = 1,500,000,000.5 lamports -> should floor to 1,500,000,000
      expect(parseAmount("1.5000000005")).toBe(1_500_000_000);
    });

    it("should be inverse of formatAmount", () => {
      const lamports = 1_234_567_890;
      const formatted = formatAmount(lamports);
      const parsed = parseAmount(formatted);
      expect(parsed).toBe(lamports);
    });
  });

  describe("validateOutputsSum", () => {
    it("should validate correct sum", () => {
      const outputs = [
        { amount: 500_000_000 },
        { amount: 300_000_000 },
        { amount: 200_000_000 },
      ];
      expect(validateOutputsSum(outputs, 1_000_000_000)).toBe(true);
    });

    it("should reject incorrect sum", () => {
      const outputs = [
        { amount: 500_000_000 },
        { amount: 300_000_000 },
      ];
      expect(validateOutputsSum(outputs, 1_000_000_000)).toBe(false);
    });

    it("should handle single output", () => {
      const outputs = [{ amount: 1_000_000_000 }];
      expect(validateOutputsSum(outputs, 1_000_000_000)).toBe(true);
    });

    it("should handle empty outputs", () => {
      expect(validateOutputsSum([], 0)).toBe(true);
      expect(validateOutputsSum([], 100)).toBe(false);
    });

    it("should be strict about equality", () => {
      const outputs = [{ amount: 999_999_999 }];
      expect(validateOutputsSum(outputs, 1_000_000_000)).toBe(false);
    });
  });

  describe("calculateRelayFee", () => {
    it("should calculate relay fee correctly", () => {
      const amount = 1_000_000_000; // 1 SOL
      const feeBps = 50; // 0.5%
      
      // (1,000,000,000 * 50) / 10000 = 5,000,000
      expect(calculateRelayFee(amount, feeBps)).toBe(5_000_000);
    });

    it("should handle 1% fee", () => {
      const amount = 1_000_000_000;
      const feeBps = 100; // 1%
      expect(calculateRelayFee(amount, feeBps)).toBe(10_000_000);
    });

    it("should handle 0% fee", () => {
      const amount = 1_000_000_000;
      const feeBps = 0;
      expect(calculateRelayFee(amount, feeBps)).toBe(0);
    });

    it("should handle 100% fee", () => {
      const amount = 1_000_000_000;
      const feeBps = 10000; // 100%
      expect(calculateRelayFee(amount, feeBps)).toBe(1_000_000_000);
    });

    it("should throw on negative fee", () => {
      expect(() => calculateRelayFee(1_000_000_000, -1)).toThrow(
        "Fee basis points must be between 0 and 10000"
      );
    });

    it("should throw on fee > 10000", () => {
      expect(() => calculateRelayFee(1_000_000_000, 10001)).toThrow(
        "Fee basis points must be between 0 and 10000"
      );
    });

    it("should floor fractional results", () => {
      const amount = 999;
      const feeBps = 50;
      
      // (999 * 50) / 10000 = 4.995 -> floor to 4
      expect(calculateRelayFee(amount, feeBps)).toBe(4);
    });

    it("should handle very small fees", () => {
      const amount = 1_000_000_000;
      const feeBps = 1; // 0.01%
      expect(calculateRelayFee(amount, feeBps)).toBe(100_000);
    });
  });
});

