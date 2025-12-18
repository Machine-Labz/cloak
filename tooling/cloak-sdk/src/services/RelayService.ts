import { TxStatus } from "../core/types";
import { hexToBytes } from "../utils/crypto";

/**
 * Relay Service Client
 *
 * Handles submission of withdrawal transactions through a relay service
 * that pays for transaction fees and submits the transaction on-chain.
 */
export class RelayService {
  private baseUrl: string;

  /**
   * Create a new Relay Service client
   *
   * @param baseUrl - Relay service base URL
   */
  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash
  }

  /**
   * Submit a withdrawal transaction via relay
   *
   * The relay service will validate the proof, pay for transaction fees,
   * and submit the transaction on-chain.
   *
   * @param params - Withdrawal parameters
   * @param onStatusUpdate - Optional callback for status updates
   * @returns Transaction signature when completed
   *
   * @example
   * ```typescript
   * const signature = await relay.submitWithdraw({
   *   proof: proofHex,
   *   publicInputs: { root, nf, outputs_hash, amount },
   *   outputs: [{ recipient: addr, amount: lamports }],
   *   feeBps: 50
   * }, (status) => console.log(`Status: ${status}`));
   * console.log(`Transaction: ${signature}`);
   * ```
   */
  async submitWithdraw(
    params: {
      proof: string;
      publicInputs: {
        root: string;
        nf: string;
        outputs_hash: string;
        amount: number;
      };
      outputs: Array<{ recipient: string; amount: number }>;
      feeBps: number;
    },
    onStatusUpdate?: (status: string) => void
  ): Promise<string> {
    // Convert proof from hex to base64
    const proofBytes = hexToBytes(params.proof);
    const proofBase64 = this.bytesToBase64(proofBytes);

    // Prepare request body
    const requestBody = {
      outputs: params.outputs,
      policy: {
        fee_bps: params.feeBps,
      },
      public_inputs: {
        root: params.publicInputs.root,
        nf: params.publicInputs.nf,
        amount: params.publicInputs.amount,
        fee_bps: params.feeBps,
        outputs_hash: params.publicInputs.outputs_hash,
      },
      proof_bytes: proofBase64,
    };

    // Submit withdrawal
    const response = await fetch(`${this.baseUrl}/withdraw`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      let errorMessage = `${response.status} ${response.statusText}`;
      try {
        const errorText = await response.text();
        errorMessage = errorText || errorMessage;
      } catch {
        // Ignore parse errors
      }
      throw new Error(`Relay withdraw failed: ${errorMessage}`);
    }

    const json = (await response.json()) as any;

    if (!json.success) {
      throw new Error(json.error || "Relay withdraw failed");
    }

    const requestId: string | undefined = json.data?.request_id;

    if (!requestId) {
      throw new Error("Relay response missing request_id");
    }

    // Poll for completion
    return this.pollForCompletion(requestId, onStatusUpdate);
  }

  /**
   * Poll for withdrawal completion
   *
   * @param requestId - Request ID from relay service
   * @param onStatusUpdate - Optional callback for status updates
   * @returns Transaction signature when completed
   */
  private async pollForCompletion(
    requestId: string,
    onStatusUpdate?: (status: string) => void
  ): Promise<string> {
    let attempts = 0;
    const maxAttempts = 120; // 10 minutes (120 * 5s = 600s)
    const pollInterval = 5000; // 5 seconds

    while (attempts < maxAttempts) {
      await this.sleep(pollInterval);
      attempts++;

      try {
        const statusResp = await fetch(`${this.baseUrl}/status/${requestId}`);

        if (!statusResp.ok) {
          // Continue polling on error
          continue;
        }

        const statusJson = (await statusResp.json()) as any;
        const statusData = statusJson.data;
        const status: string | undefined = statusData?.status;

        if (onStatusUpdate && status) {
          onStatusUpdate(status);
        }

        if (status === "completed") {
          const txId: string | undefined = statusData?.tx_id;
          if (!txId) {
            throw new Error("Relay completed without tx_id");
          }
          return txId;
        }

        if (status === "failed") {
          throw new Error(statusData?.error || "Relay job failed");
        }

        // Status is pending/processing, continue polling
      } catch (error) {
        // If this is a final error (not network), throw it
        if (error instanceof Error && error.message.includes("failed")) {
          throw error;
        }
        // Otherwise continue polling
      }
    }

    throw new Error(
      `Withdrawal polling timed out after ${maxAttempts * pollInterval}ms`
    );
  }

  /**
   * Submit a swap transaction via relay
   *
   * Similar to submitWithdraw but includes swap parameters for token swaps.
   * The relay service will validate the proof, execute the swap, pay for fees,
   * and submit the transaction on-chain.
   *
   * @param params - Swap parameters
   * @param onStatusUpdate - Optional callback for status updates
   * @returns Transaction signature when completed
   *
   * @example
   * ```typescript
   * const signature = await relay.submitSwap({
   *   proof: proofHex,
   *   publicInputs: { root, nf, outputs_hash, amount },
   *   outputs: [{ recipient: addr, amount: lamports }],
   *   feeBps: 50,
   *   swap: {
   *     output_mint: tokenMint.toBase58(),
   *     slippage_bps: 100,
   *     min_output_amount: minAmount
   *   }
   * }, (status) => console.log(`Status: ${status}`));
   * console.log(`Transaction: ${signature}`);
   * ```
   */
  async submitSwap(
    params: {
      proof: string;
      publicInputs: {
        root: string;
        nf: string;
        outputs_hash: string;
        amount: number;
      };
      outputs: Array<{ recipient: string; amount: number }>;
      feeBps: number;
      swap: {
        output_mint: string;
        slippage_bps: number;
        min_output_amount: number;
      };
    },
    onStatusUpdate?: (status: string) => void
  ): Promise<string> {
    // Convert proof from hex to base64
    const proofBytes = hexToBytes(params.proof);
    const proofBase64 = this.bytesToBase64(proofBytes);

    // Prepare request body
    const requestBody = {
      outputs: params.outputs,
      swap: params.swap,
      policy: {
        fee_bps: params.feeBps,
      },
      public_inputs: {
        root: params.publicInputs.root,
        nf: params.publicInputs.nf,
        amount: params.publicInputs.amount,
        fee_bps: params.feeBps,
        outputs_hash: params.publicInputs.outputs_hash,
      },
      proof_bytes: proofBase64,
    };

    // Submit swap
    const response = await fetch(`${this.baseUrl}/withdraw`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      let errorMessage = `${response.status} ${response.statusText}`;
      try {
        const errorText = await response.text();
        errorMessage = errorText || errorMessage;
      } catch {
        // Ignore parse errors
      }
      throw new Error(`Relay swap failed: ${errorMessage}`);
    }

    const json = (await response.json()) as any;

    if (!json.success) {
      throw new Error(json.error || "Relay swap failed");
    }

    const requestId: string | undefined = json.data?.request_id;

    if (!requestId) {
      throw new Error("Relay response missing request_id");
    }

    // Poll for completion
    return this.pollForCompletion(requestId, onStatusUpdate);
  }

  /**
   * Get transaction status
   *
   * @param requestId - Request ID from previous submission
   * @returns Current status
   *
   * @example
   * ```typescript
   * const status = await relay.getStatus(requestId);
   * console.log(`Status: ${status.status}`);
   * if (status.status === 'completed') {
   *   console.log(`TX: ${status.txId}`);
   * }
   * ```
   */
  async getStatus(requestId: string): Promise<TxStatus> {
    const response = await fetch(`${this.baseUrl}/status/${requestId}`);

    if (!response.ok) {
      throw new Error(
        `Failed to get status: ${response.status} ${response.statusText}`
      );
    }

    const json = (await response.json()) as any;
    const data = json.data;

    return {
      status: data?.status || "pending",
      txId: data?.tx_id,
      error: data?.error,
    };
  }

  /**
   * Convert bytes to base64 string
   */
  private bytesToBase64(bytes: Uint8Array): string {
    // Check if we're in Node.js or browser
    if (typeof Buffer !== "undefined") {
      return Buffer.from(bytes).toString("base64");
    } else if (typeof btoa !== "undefined") {
      // Browser environment
      const binary = Array.from(bytes)
        .map((b) => String.fromCharCode(b))
        .join("");
      return btoa(binary);
    } else {
      throw new Error("No base64 encoding method available");
    }
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
