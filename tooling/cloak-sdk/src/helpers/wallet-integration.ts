/**
 * Wallet Integration Helpers
 * 
 * Helper functions for working with Solana Wallet Adapters
 */

import {
  Connection,
  Transaction,
  PublicKey,
  SendOptions,
  Keypair,
} from "@solana/web3.js";
import { WalletAdapter, CloakError } from "../core/types";

/**
 * Validate wallet is connected and has public key
 */
export function validateWalletConnected(wallet: WalletAdapter | Keypair): void {
  if (wallet instanceof Keypair) {
    return; // Keypair is always "connected"
  }
  
  if (!wallet.publicKey) {
    throw new CloakError(
      "Wallet not connected. Please connect your wallet first.",
      "wallet",
      false
    );
  }
}

/**
 * Get public key from wallet or keypair
 */
export function getPublicKey(wallet: WalletAdapter | Keypair): PublicKey {
  if (wallet instanceof Keypair) {
    return wallet.publicKey;
  }
  
  if (!wallet.publicKey) {
    throw new CloakError(
      "Wallet not connected",
      "wallet",
      false
    );
  }
  
  return wallet.publicKey;
}

/**
 * Send transaction using wallet adapter or keypair
 */
export async function sendTransaction(
  transaction: Transaction,
  wallet: WalletAdapter | Keypair,
  connection: Connection,
  options?: SendOptions
): Promise<string> {
  if (wallet instanceof Keypair) {
    // Use Keypair to sign and send
    return await connection.sendTransaction(transaction, [wallet], options);
  }
  
  // Use wallet adapter
  if (wallet.sendTransaction) {
    // Wallet has sendTransaction method
    return await wallet.sendTransaction(transaction, connection, options);
  } else if (wallet.signTransaction) {
    // Wallet only has signTransaction
    const signed = await wallet.signTransaction(transaction);
    return await connection.sendRawTransaction(signed.serialize(), options);
  } else {
    throw new CloakError(
      "Wallet does not support transaction signing",
      "wallet",
      false
    );
  }
}

/**
 * Sign transaction using wallet adapter or keypair
 */
export async function signTransaction<T extends Transaction>(
  transaction: T,
  wallet: WalletAdapter | Keypair
): Promise<T> {
  if (wallet instanceof Keypair) {
    transaction.sign(wallet);
    return transaction;
  }
  
  if (!wallet.signTransaction) {
    throw new CloakError(
      "Wallet does not support transaction signing",
      "wallet",
      false
    );
  }
  
  return await wallet.signTransaction(transaction);
}

/**
 * Create a keypair adapter for testing
 */
export function keypairToAdapter(keypair: Keypair): WalletAdapter {
  return {
    publicKey: keypair.publicKey,
    signTransaction: async <T extends Transaction>(tx: T): Promise<T> => {
      tx.sign(keypair);
      return tx;
    },
    signAllTransactions: async <T extends Transaction>(txs: T[]): Promise<T[]> => {
      txs.forEach(tx => tx.sign(keypair));
      return txs;
    },
  };
}

