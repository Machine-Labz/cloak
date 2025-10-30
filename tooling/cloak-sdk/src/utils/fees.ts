/**
 * Fee calculation utilities for Cloak Protocol
 *
 * The protocol charges a fixed fee plus a variable percentage fee
 * to prevent sybil attacks and cover operational costs.
 */

/** Fixed fee: 0.0025 SOL (2.5M lamports) */
export const FIXED_FEE_LAMPORTS = 2_500_000;

/** Variable fee rate: 0.5% (5 basis points per 1000) */
export const VARIABLE_FEE_RATE = 5 / 1000;

/** Lamports per SOL */
export const LAMPORTS_PER_SOL = 1_000_000_000;

/**
 * Calculate the total protocol fee for a given amount
 *
 * Formula: FIXED_FEE + floor((amount * 5) / 1000)
 *
 * @param amountLamports - Amount in lamports
 * @returns Total fee in lamports
 *
 * @example
 * ```typescript
 * const fee = calculateFee(1_000_000_000); // 1 SOL
 * // Returns: 2_500_000 (fixed) + 5_000_000 (0.5%) = 7_500_000 lamports
 * ```
 */
export function calculateFee(amountLamports: number): number {
  const variableFee = Math.floor((amountLamports * 5) / 1_000);
  return FIXED_FEE_LAMPORTS + variableFee;
}

/**
 * Calculate the distributable amount after protocol fees
 *
 * This is the amount available to send to recipients.
 *
 * @param amountLamports - Total note amount in lamports
 * @returns Amount available for recipients in lamports
 *
 * @example
 * ```typescript
 * const distributable = getDistributableAmount(1_000_000_000);
 * // Returns: 1_000_000_000 - 7_500_000 = 992_500_000 lamports
 * ```
 */
export function getDistributableAmount(amountLamports: number): number {
  return amountLamports - calculateFee(amountLamports);
}

/**
 * Format lamports as SOL string
 *
 * @param lamports - Amount in lamports
 * @param decimals - Number of decimal places (default: 9)
 * @returns Formatted string (e.g., "1.000000000")
 *
 * @example
 * ```typescript
 * formatAmount(1_000_000_000); // "1.000000000"
 * formatAmount(1_500_000_000); // "1.500000000"
 * formatAmount(123_456_789, 4); // "0.1235"
 * ```
 */
export function formatAmount(lamports: number, decimals: number = 9): string {
  return (lamports / LAMPORTS_PER_SOL).toFixed(decimals);
}

/**
 * Parse SOL string to lamports
 *
 * @param sol - SOL amount as string (e.g., "1.5")
 * @returns Amount in lamports
 * @throws Error if invalid format
 *
 * @example
 * ```typescript
 * parseAmount("1.5"); // 1_500_000_000
 * parseAmount("0.001"); // 1_000_000
 * ```
 */
export function parseAmount(sol: string): number {
  const num = parseFloat(sol);
  if (isNaN(num) || num < 0) {
    throw new Error(`Invalid SOL amount: ${sol}`);
  }
  return Math.floor(num * LAMPORTS_PER_SOL);
}

/**
 * Validate that outputs sum equals expected amount
 *
 * @param outputs - Array of output amounts
 * @param expectedTotal - Expected total amount
 * @returns True if amounts match
 */
export function validateOutputsSum(
  outputs: Array<{ amount: number }>,
  expectedTotal: number
): boolean {
  const sum = outputs.reduce((acc, out) => acc + out.amount, 0);
  return sum === expectedTotal;
}

/**
 * Calculate relay fee from basis points
 *
 * @param amountLamports - Amount in lamports
 * @param feeBps - Fee in basis points (100 bps = 1%)
 * @returns Relay fee in lamports
 *
 * @example
 * ```typescript
 * calculateRelayFee(1_000_000_000, 50); // 0.5% = 5_000_000 lamports
 * ```
 */
export function calculateRelayFee(
  amountLamports: number,
  feeBps: number
): number {
  if (feeBps < 0 || feeBps > 10000) {
    throw new Error("Fee basis points must be between 0 and 10000");
  }
  return Math.floor((amountLamports * feeBps) / 10000);
}
