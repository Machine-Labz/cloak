import { SP1ProofInputs, SP1ProofResult } from "../core/types";

/**
 * Options for artifact-based proof generation
 */
export interface ArtifactProofGenerationOptions {
  /** Progress callback - called with percentage (0-100) */
  onProgress?: (progress: number) => void;
  /** Called when proof generation starts */
  onStart?: () => void;
  /** Called on successful proof generation */
  onSuccess?: (result: SP1ProofResult) => void;
  /** Called on error */
  onError?: (error: string) => void;
  /** Custom timeout in milliseconds */
  timeout?: number;
  /** Polling interval for proof status (default: 2000ms) */
  pollInterval?: number;
}

/**
 * Artifact-based Prover Service
 *
 * Implements the artifact-based proof generation flow where private inputs
 * are uploaded directly to the TEE, never passing through the backend in plaintext.
 *
 * Flow:
 * 1. Create artifact → get artifact_id and upload_url
 * 2. Upload stdin directly to TEE (bypassing backend)
 * 3. Request proof generation (backend only gets artifact_id)
 * 4. Poll for proof status until ready
 *
 * ✅ PRIVACY: Private inputs never pass through backend in plaintext
 */
export class ArtifactProverService {
  private indexerUrl: string;
  private timeout: number;
  private pollInterval: number;

  /**
   * Create a new Artifact Prover Service client
   *
   * @param indexerUrl - Indexer service base URL
   * @param timeout - Proof generation timeout in ms (default: 5 minutes)
   * @param pollInterval - Polling interval for status checks (default: 2 seconds)
   */
  constructor(
    indexerUrl: string,
    timeout: number = 5 * 60 * 1000,
    pollInterval: number = 2000
  ) {
    this.indexerUrl = indexerUrl.replace(/\/$/, ""); // Remove trailing slash
    this.timeout = timeout;
    this.pollInterval = pollInterval;
  }

  /**
   * Generate a zero-knowledge proof using artifact-based flow
   *
   * This process typically takes 30-180 seconds depending on the TEE.
   * Private inputs are uploaded directly to TEE, never passing through backend.
   *
   * @param inputs - Circuit inputs (private + public + outputs)
   * @param options - Optional progress tracking and callbacks
   * @returns Proof result with hex-encoded proof and public inputs
   *
   * @example
   * ```typescript
   * const result = await prover.generateProof(inputs);
   * if (result.success) {
   *   console.log(`Proof: ${result.proof}`);
   * }
   * ```
   */
  async generateProof(
    inputs: SP1ProofInputs,
    options?: ArtifactProofGenerationOptions
  ): Promise<SP1ProofResult> {
    const startTime = Date.now();
    const actualTimeout = options?.timeout || this.timeout;
    const pollInterval = options?.pollInterval || this.pollInterval;

    // Call onStart callback
    options?.onStart?.();
    options?.onProgress?.(5);

    try {
      // Step 1: Create artifact
      options?.onProgress?.(10);
      const artifactResponse = await fetch(`${this.indexerUrl}/api/v1/tee/artifact`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          program_id: null, // Optional, can be null
        }),
      });

      if (!artifactResponse.ok) {
        const errorText = await artifactResponse.text();
        const errorMessage = `Failed to create artifact: ${errorText}`;
        options?.onError?.(errorMessage);
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage,
        };
      }

      const artifactData = (await artifactResponse.json()) as {
        artifact_id: string;
        upload_url: string;
        expires_at?: string;
      };
      const { artifact_id, upload_url } = artifactData;

      if (!artifact_id || !upload_url) {
        const errorMessage = "Invalid artifact response: missing artifact_id or upload_url";
        options?.onError?.(errorMessage);
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage,
        };
      }

      // Resolve upload URL (might be relative or absolute)
      const fullUploadUrl = upload_url.startsWith("http")
        ? upload_url
        : `${this.indexerUrl}${upload_url}`;

      // Step 2: Upload stdin directly to TEE
      options?.onProgress?.(20);
      const stdinPayload = JSON.stringify({
        private: inputs.privateInputs,
        public: inputs.publicInputs,
        outputs: inputs.outputs,
        // Include swap_params if present (for swap transactions)
        ...(inputs.swapParams && { swap_params: inputs.swapParams }),
      });

      const uploadResponse = await fetch(fullUploadUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: stdinPayload,
      });

      if (!uploadResponse.ok) {
        const errorText = await uploadResponse.text();
        const errorMessage = `Failed to upload stdin to TEE: ${errorText}`;
        options?.onError?.(errorMessage);
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage,
        };
      }

      // Step 3: Request proof generation
      options?.onProgress?.(30);
      const requestProofBody: any = {
        artifact_id,
        program_id: null,
        public_inputs: JSON.stringify(inputs.publicInputs),
      };

      const requestProofResponse = await fetch(
        `${this.indexerUrl}/api/v1/tee/request-proof`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(requestProofBody),
        }
      );

      if (!requestProofResponse.ok) {
        const errorText = await requestProofResponse.text();
        const errorMessage = `Failed to request proof: ${errorText}`;
        options?.onError?.(errorMessage);
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage,
        };
      }

      const requestProofData = (await requestProofResponse.json()) as {
        request_id: string;
        status: string;
      };
      const { request_id } = requestProofData;

      if (!request_id) {
        const errorMessage = "Invalid proof request response: missing request_id";
        options?.onError?.(errorMessage);
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage,
        };
      }

      // Step 4: Poll for proof status
      options?.onProgress?.(40);
      const pollStartTime = Date.now();
      let lastProgress = 40;

      while (Date.now() - pollStartTime < actualTimeout) {
        const statusResponse = await fetch(
          `${this.indexerUrl}/api/v1/tee/proof-status?request_id=${request_id}`,
          {
            method: "GET",
          }
        );

        if (!statusResponse.ok) {
          const errorText = await statusResponse.text();
          const errorMessage = `Failed to check proof status: ${errorText}`;
          options?.onError?.(errorMessage);
          return {
            success: false,
            generationTimeMs: Date.now() - startTime,
            error: errorMessage,
          };
        }

        const statusData = (await statusResponse.json()) as {
          request_id: string;
          status: string;
          proof?: string;
          public_inputs?: string;
          generation_time_ms?: number;
          error?: string;
        };
        const { status, proof, public_inputs, generation_time_ms, error } = statusData;

        if (status === "ready") {
          // Proof is ready!
          options?.onProgress?.(100);

          if (!proof || !public_inputs) {
            const errorMessage = "Proof status is 'ready' but proof or public_inputs is missing";
            options?.onError?.(errorMessage);
            return {
              success: false,
              generationTimeMs: Date.now() - startTime,
              error: errorMessage,
            };
          }

          const result: SP1ProofResult = {
            success: true,
            proof,
            publicInputs: public_inputs,
            generationTimeMs: generation_time_ms || Date.now() - startTime,
          };

          options?.onSuccess?.(result);
          return result;
        }

        if (status === "failed") {
          const errorMessage = error || "Proof generation failed";
          options?.onError?.(errorMessage);
          return {
            success: false,
            generationTimeMs: Date.now() - startTime,
            error: errorMessage,
          };
        }

        // Status is "pending" or "processing" - continue polling
        // Simulate progress (40% to 90%)
        const elapsed = Date.now() - pollStartTime;
        const progress = Math.min(90, 40 + Math.floor((elapsed / actualTimeout) * 50));
        if (progress > lastProgress) {
          lastProgress = progress;
          options?.onProgress?.(progress);
        }

        // Wait before next poll
        await new Promise((resolve) => setTimeout(resolve, pollInterval));
      }

      // Timeout
      const errorMessage = `Proof generation timed out after ${actualTimeout}ms`;
      options?.onError?.(errorMessage);
      return {
        success: false,
        generationTimeMs: Date.now() - startTime,
        error: errorMessage,
      };
    } catch (error) {
      const totalTime = Date.now() - startTime;
      let errorMessage: string;

      if (error instanceof Error && error.name === "AbortError") {
        errorMessage = `Proof generation timed out after ${actualTimeout}ms`;
      } else {
        errorMessage = error instanceof Error ? error.message : "Unknown error occurred";
      }

      options?.onError?.(errorMessage);

      return {
        success: false,
        generationTimeMs: totalTime,
        error: errorMessage,
      };
    }
  }

  /**
   * Check if the artifact prover service is available
   *
   * @returns True if service is healthy
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.indexerUrl}/health`, {
        method: "GET",
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Get the configured timeout
   */
  getTimeout(): number {
    return this.timeout;
  }

  /**
   * Set a new timeout
   */
  setTimeout(timeout: number): void {
    if (timeout <= 0) {
      throw new Error("Timeout must be positive");
    }
    this.timeout = timeout;
  }
}

