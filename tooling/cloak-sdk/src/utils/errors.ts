/**
 * Error Utilities
 * 
 * Helper functions for parsing and presenting user-friendly error messages
 */

import { CloakError } from "../core/types";

/**
 * Common program error codes and their user-friendly messages
 */
const PROGRAM_ERRORS: Record<string, string> = {
  // Nullifier errors
  "NullifierAlreadyUsed": "This note has already been withdrawn. Each note can only be spent once.",
  "0x1770": "This note has already been withdrawn.",
  
  // Proof verification errors
  "ProofVerificationFailed": "Zero-knowledge proof verification failed. Please try again.",
  "0x1771": "Invalid proof. Please regenerate and try again.",
  "InvalidProof": "The provided proof is invalid.",
  
  // Root errors
  "RootNotFound": "The Merkle root is outdated or invalid. Please refresh and try again.",
  "InvalidRoot": "Invalid Merkle root. The tree may have been updated.",
  "0x1772": "Merkle root not found in history.",
  
  // Amount errors
  "InvalidAmount": "Invalid amount. Please check your input.",
  "InsufficientFunds": "Insufficient funds for this transaction.",
  "AmountMismatch": "Output amounts don't match the note amount.",
  
  // Fee errors
  "InvalidFee": "Invalid fee calculation. Please try again.",
  "FeeTooHigh": "Fee exceeds maximum allowed.",
  
  // Output errors
  "InvalidOutputs": "Invalid recipient configuration.",
  "TooManyOutputs": "Maximum 5 recipients allowed per transaction.",
  "OutputHashMismatch": "Output hash doesn't match proof.",
  
  // General errors
  "Unauthorized": "You don't have permission to perform this action.",
  "AccountNotFound": "Required account not found.",
  "InvalidInstruction": "Invalid instruction data.",
};

/**
 * Parse transaction error and return user-friendly message
 * 
 * Attempts to extract meaningful error information from various
 * error formats (program errors, RPC errors, custom errors)
 */
export function parseTransactionError(error: any): string {
  if (!error) return "An unknown error occurred";

  const errorStr = typeof error === "string" ? error : error.message || error.toString();
  
  // Check for custom error codes (hex format like 0x1770)
  const hexMatch = errorStr.match(/0x[0-9a-f]{4}/i);
  if (hexMatch) {
    const errorCode = hexMatch[0];
    if (PROGRAM_ERRORS[errorCode]) {
      return PROGRAM_ERRORS[errorCode];
    }
  }
  
  // Check for named program errors
  for (const [key, message] of Object.entries(PROGRAM_ERRORS)) {
    if (errorStr.includes(key)) {
      return message;
    }
  }
  
  // Check for common Solana/RPC errors
  if (errorStr.includes("insufficient funds") || 
      errorStr.includes("insufficient lamports")) {
    return "Insufficient SOL balance. Please add funds to your wallet.";
  }
  
  if (errorStr.includes("blockhash not found")) {
    return "Transaction expired. Please try again.";
  }
  
  if (errorStr.includes("already been processed")) {
    return "This transaction has already been processed.";
  }
  
  if (errorStr.includes("signature verification failed")) {
    return "Transaction signature verification failed. Please try again.";
  }
  
  if (errorStr.includes("account does not exist")) {
    return "Required account not found. The program may need to be initialized.";
  }
  
  // Network errors
  if (errorStr.includes("fetch") || errorStr.includes("network")) {
    return "Network error. Please check your connection and try again.";
  }
  
  if (errorStr.includes("timeout")) {
    return "Request timed out. Please try again.";
  }
  
  // Relay specific errors
  if (errorStr.includes("relay") || errorStr.includes("withdraw")) {
    if (errorStr.includes("in progress")) {
      return "A withdrawal is already in progress. Please wait for it to complete.";
    }
    if (errorStr.includes("rate limit")) {
      return "Too many requests. Please wait a moment and try again.";
    }
    return "Relay service error. Please try again later.";
  }
  
  // Proof generation errors
  if (errorStr.includes("proof") && errorStr.includes("generation")) {
    return "Failed to generate zero-knowledge proof. Please try again.";
  }
  
  // Indexer errors
  if (errorStr.includes("indexer") || errorStr.includes("merkle")) {
    if (errorStr.includes("inconsistent")) {
      return "The indexer is temporarily unavailable. Please try again in a moment.";
    }
    if (errorStr.includes("not found")) {
      return "Note not found in the indexer. It may not be confirmed yet.";
    }
    return "Indexer service error. Please try again later.";
  }
  
  // If no specific match, try to clean up the error message
  let cleanError = errorStr
    .replace(/Error:\s*/gi, "")
    .replace(/\s+at\s+.*$/g, "") // Remove stack traces
    .replace(/\[.*?\]/g, "") // Remove bracketed content
    .trim();
  
  // Limit length for display
  if (cleanError.length > 200) {
    cleanError = cleanError.substring(0, 197) + "...";
  }
  
  return cleanError || "Transaction failed. Please try again.";
}

/**
 * Create a CloakError with appropriate categorization
 */
export function createCloakError(error: unknown, _context: string): CloakError {
  if (error instanceof CloakError) {
    return error;
  }
  
  const errorMessage = error instanceof Error ? error.message : String(error);
  const userMessage = parseTransactionError(error);
  
  // Determine error category and retryability
  let category: CloakError["category"] = "network";
  let retryable = false;
  
  if (errorMessage.includes("insufficient") || 
      errorMessage.includes("balance")) {
    category = "wallet";
    retryable = false;
  } else if (errorMessage.includes("proof")) {
    category = "prover";
    retryable = true;
  } else if (errorMessage.includes("indexer") || 
             errorMessage.includes("merkle")) {
    category = "indexer";
    retryable = errorMessage.includes("inconsistent") || 
                errorMessage.includes("temporary");
  } else if (errorMessage.includes("relay")) {
    category = "relay";
    retryable = true;
  } else if (errorMessage.includes("timeout") || 
             errorMessage.includes("network")) {
    category = "network";
    retryable = true;
  } else if (errorMessage.includes("validation") || 
             errorMessage.includes("invalid")) {
    category = "validation";
    retryable = false;
  }
  
  return new CloakError(
    userMessage,
    category,
    retryable,
    error instanceof Error ? error : undefined
  );
}

/**
 * Format error for logging
 */
export function formatErrorForLogging(error: unknown): string {
  if (error instanceof CloakError) {
    return `[${error.category}] ${error.message}${error.retryable ? " (retryable)" : ""}`;
  }
  
  if (error instanceof Error) {
    return `${error.name}: ${error.message}`;
  }
  
  return String(error);
}
