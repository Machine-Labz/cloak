/**
 * Deposit Recovery Service
 * 
 * Handles recovery of deposits that completed on-chain but failed
 * to finalize with the indexer (e.g., browser crash, network failure)
 */

import { Connection } from "@solana/web3.js";
import { CloakNote, CloakError } from "../core/types";
import { IndexerService } from "./IndexerService";
import { updateNoteWithDeposit } from "../core/note-manager";
import { encodeNoteSimple } from "../helpers/encrypted-output";

export interface RecoveryOptions {
  /** Transaction signature to recover */
  signature: string;
  /** Note commitment hash */
  commitment: string;
  /** Optional: The full note if available */
  note?: CloakNote;
  /** Callback for progress updates */
  onProgress?: (status: string) => void;
}

export interface RecoveryResult {
  success: boolean;
  leafIndex?: number;
  root?: string;
  slot?: number;
  merkleProof?: {
    pathElements: string[];
    pathIndices: number[];
  };
  note?: CloakNote;
  error?: string;
}

/**
 * Service for recovering incomplete deposits
 */
export class DepositRecoveryService {
  constructor(
    private indexer: IndexerService,
    private apiUrl: string
  ) {}

  /**
   * Recover a deposit that completed on-chain but failed to register
   * 
   * @param options Recovery options
   * @returns Recovery result with updated note
   */
  async recoverDeposit(options: RecoveryOptions): Promise<RecoveryResult> {
    const { signature, commitment, note, onProgress } = options;

    try {
      onProgress?.("Validating inputs...");
      
      // Validate signature format
      if (!/^[1-9A-HJ-NP-Za-km-z]{87,88}$/.test(signature)) {
        throw new CloakError(
          "Invalid transaction signature format",
          "validation",
          false
        );
      }

      // Validate commitment format
      if (!/^[0-9a-f]{64}$/i.test(commitment)) {
        throw new CloakError(
          "Invalid commitment format",
          "validation",
          false
        );
      }

      onProgress?.("Checking if deposit is already registered...");

      // Try to get existing deposit info from indexer
      try {
        const existingInfo = await this.checkExistingDeposit(commitment);
        if (existingInfo) {
          onProgress?.("Deposit already registered!");
          return {
            success: true,
            ...existingInfo,
            note: note ? updateNoteWithDeposit(note, {
              signature,
              slot: existingInfo.slot!,
              leafIndex: existingInfo.leafIndex!,
              root: existingInfo.root!,
              merkleProof: existingInfo.merkleProof!,
            }) : undefined,
          };
        }
      } catch (e) {
        // Deposit not found, continue with recovery
      }

      onProgress?.("Fetching transaction details...");

      // Get transaction details from blockchain
      const connection = new Connection(
        process.env.NEXT_PUBLIC_SOLANA_RPC_URL || "https://api.devnet.solana.com"
      );
      
      const txDetails = await connection.getTransaction(signature, {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0,
      });

      if (!txDetails) {
        throw new CloakError(
          "Transaction not found on blockchain",
          "validation",
          false
        );
      }

      const slot = txDetails.slot;

      onProgress?.("Preparing encrypted output...");

      // Prepare encrypted output
      let encryptedOutput = "";
      if (note) {
        encryptedOutput = encodeNoteSimple(note);
      } else {
        // Without the full note, we can only create a placeholder
        encryptedOutput = btoa(JSON.stringify({ commitment }));
      }

      onProgress?.("Registering deposit with indexer...");

      // Submit to indexer
      const depositResponse = await this.indexer.submitDeposit({
        leafCommit: commitment,
        encryptedOutput,
        txSignature: signature,
        slot,
      });

      if (!depositResponse.success) {
        throw new CloakError(
          "Failed to register deposit with indexer",
          "indexer",
          true
        );
      }

      const leafIndex = depositResponse.leafIndex!;
      const root = depositResponse.root!;

      onProgress?.("Fetching Merkle proof...");

      // Get Merkle proof
      const merkleProof = await this.indexer.getMerkleProof(leafIndex);

      onProgress?.("Recovery complete!");

      // Update note if provided
      const updatedNote = note ? updateNoteWithDeposit(note, {
        signature,
        slot,
        leafIndex,
        root,
        merkleProof: {
          pathElements: merkleProof.pathElements,
          pathIndices: merkleProof.pathIndices,
        },
      }) : undefined;

      return {
        success: true,
        leafIndex,
        root,
        slot,
        merkleProof: {
          pathElements: merkleProof.pathElements,
          pathIndices: merkleProof.pathIndices,
        },
        note: updatedNote,
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      // Check for specific error cases
      if (errorMessage.includes("duplicate key") || 
          errorMessage.includes("already exists")) {
        // Deposit was already registered, try to fetch its info
        try {
          const existingInfo = await this.checkExistingDeposit(commitment);
          if (existingInfo) {
            return {
              success: true,
              ...existingInfo,
              note: note ? updateNoteWithDeposit(note, {
                signature,
                slot: existingInfo.slot!,
                leafIndex: existingInfo.leafIndex!,
                root: existingInfo.root!,
                merkleProof: existingInfo.merkleProof!,
              }) : undefined,
            };
          }
        } catch (e) {
          // Fall through to error
        }
      }

      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  /**
   * Check if a deposit already exists in the indexer
   * 
   * @private
   */
  private async checkExistingDeposit(_commitment: string): Promise<{
    leafIndex: number;
    root: string;
    slot: number;
    merkleProof: {
      pathElements: string[];
      pathIndices: number[];
    };
  } | null> {
    try {
      // Try to find the deposit by searching through notes
      // This is a simplified approach - in production you might want
      // a dedicated endpoint for this
      const { next_index } = await this.indexer.getMerkleRoot();
      
      // Search in batches (this could be optimized with a server-side endpoint)
      const batchSize = 100;
      for (let i = 0; i < next_index; i += batchSize) {
        const end = Math.min(i + batchSize - 1, next_index - 1);
        const { notes } = await this.indexer.getNotesRange(i, end, batchSize);
        
        // Check if any note matches our commitment
        // Note: This is simplified - actual implementation would need
        // to decrypt and check commitments
        for (let j = 0; j < notes.length; j++) {
          // const leafIndex = i + j;
          try {
            // Get merkle proof for this index
            // const merkleProof = await this.indexer.getMerkleProof(leafIndex);
            
            // For now, we can't definitively match without decryption
            // In production, you'd want a server endpoint that can check
            // commitments directly
            
            // This is a placeholder - actual implementation would need
            // proper commitment matching
            return null;
          } catch (e) {
            continue;
          }
        }
      }
      
      return null;
    } catch (error) {
      return null;
    }
  }

  /**
   * Finalize a deposit via server API (alternative recovery method)
   * 
   * This method calls a server-side endpoint that can handle
   * the recovery process with elevated permissions.
   */
  async finalizeDepositViaServer(
    signature: string,
    commitment: string,
    encryptedOutput?: string
  ): Promise<RecoveryResult> {
    try {
      const response = await fetch(`${this.apiUrl}/api/deposit/finalize`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tx_signature: signature,
          commitment,
          encrypted_output: encryptedOutput || btoa(JSON.stringify({ commitment })),
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Recovery failed: ${errorText}`);
      }

      const data = await response.json() as {
        success: boolean;
        error?: string;
        leaf_index?: number;
        root?: string;
        slot?: number;
        merkle_proof?: {
          path_elements: string[];
          path_indices: number[];
        };
      };

      if (!data.success) {
        throw new Error(data.error || "Recovery failed");
      }

      if (!data.leaf_index || !data.root || data.slot === undefined || !data.merkle_proof) {
        throw new Error("Recovery response missing required fields");
      }

      return {
        success: true,
        leafIndex: data.leaf_index,
        root: data.root,
        slot: data.slot,
        merkleProof: {
          pathElements: data.merkle_proof.path_elements,
          pathIndices: data.merkle_proof.path_indices,
        },
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}
