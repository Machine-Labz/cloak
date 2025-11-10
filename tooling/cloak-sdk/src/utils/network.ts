/**
 * Network Utilities
 * 
 * Helper functions for network detection and configuration
 */

import { Network } from "../core/types";

/**
 * Detect network from RPC URL
 * 
 * Attempts to detect the Solana network from common RPC URL patterns.
 * Falls back to devnet if unable to detect.
 */
export function detectNetworkFromRpcUrl(rpcUrl?: string): Network {
  // Use environment variable if no URL provided
  const url = rpcUrl || process.env.NEXT_PUBLIC_SOLANA_RPC_URL || "";
  
  const lowerUrl = url.toLowerCase();
  
  // Check for mainnet patterns
  if (lowerUrl.includes("mainnet") || 
      lowerUrl.includes("api.mainnet-beta") ||
      lowerUrl.includes("mainnet-beta")) {
    return "mainnet";
  }
  
  // Check for testnet patterns
  if (lowerUrl.includes("testnet") || 
      lowerUrl.includes("api.testnet")) {
    return "testnet";
  }
  
  // Check for devnet patterns
  if (lowerUrl.includes("devnet") || 
      lowerUrl.includes("api.devnet")) {
    return "devnet";
  }
  
  // Check for localnet patterns
  if (lowerUrl.includes("localhost") || 
      lowerUrl.includes("127.0.0.1") ||
      lowerUrl.includes("local")) {
    return "localnet";
  }
  
  // Default to devnet for safety
  return "devnet";
}

/**
 * Get the standard RPC URL for a network
 */
export function getRpcUrlForNetwork(network: Network): string {
  switch (network) {
    case "mainnet":
      return "https://api.mainnet-beta.solana.com";
    case "testnet":
      return "https://api.testnet.solana.com";
    case "devnet":
      return "https://api.devnet.solana.com";
    case "localnet":
      return "http://localhost:8899";
    default:
      return "https://api.devnet.solana.com";
  }
}

/**
 * Validate RPC URL format
 */
export function isValidRpcUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

/**
 * Get explorer URL for a transaction
 */
export function getExplorerUrl(
  signature: string,
  network: Network = "devnet"
): string {
  const cluster = network === "mainnet" ? "" : `?cluster=${network}`;
  return `https://explorer.solana.com/tx/${signature}${cluster}`;
}

/**
 * Get explorer URL for an address
 */
export function getAddressExplorerUrl(
  address: string,
  network: Network = "devnet"
): string {
  const cluster = network === "mainnet" ? "" : `?cluster=${network}`;
  return `https://explorer.solana.com/address/${address}${cluster}`;
}
