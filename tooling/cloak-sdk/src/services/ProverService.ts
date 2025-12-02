import { SP1ProofInputs, SP1ProofResult } from "../core/types";

/**
 * Options for proof generation
 */
export interface ProofGenerationOptions {
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
}

/**
 * Prover Service Client
 *
 * Handles zero-knowledge proof generation via the backend prover service.
 *
 * ⚠️ PRIVACY WARNING: This implementation sends private inputs to a backend service.
 * For production use with full privacy, consider client-side proof generation.
 */
export class ProverService {
  private indexerUrl: string;
  private timeout: number;

  /**
   * Create a new Prover Service client
   *
   * @param indexerUrl - Indexer/Prover service base URL
   * @param timeout - Proof generation timeout in ms (default: 5 minutes)
   */
  constructor(indexerUrl: string, timeout: number = 5 * 60 * 1000) {
    this.indexerUrl = indexerUrl.replace(/\/$/, ""); // Remove trailing slash
    this.timeout = timeout;
  }

  /**
   * Generate a zero-knowledge proof for withdrawal
   *
   * This process typically takes 30-180 seconds depending on the backend.
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
   * 
   * @example
   * ```typescript
   * // With progress tracking
   * const result = await prover.generateProof(inputs, {
   *   onProgress: (progress) => console.log(`Progress: ${progress}%`),
   *   onStart: () => console.log("Starting proof generation..."),
   *   onSuccess: (result) => console.log("Proof generated!"),
   *   onError: (error) => console.error("Failed:", error)
   * });
   * ```
   */
  async generateProof(
    inputs: SP1ProofInputs,
    options?: ProofGenerationOptions
  ): Promise<SP1ProofResult> {
    const startTime = Date.now();
    const actualTimeout = options?.timeout || this.timeout;
    
    // Call onStart callback if provided
    options?.onStart?.();
    
    // Track progress interval for cleanup
    let progressInterval: NodeJS.Timeout | undefined;
    
    try {
      // Prepare request body with snake_case field names for backend
      const requestBody: any = {
        private_inputs: JSON.stringify(inputs.privateInputs),
        public_inputs: JSON.stringify(inputs.publicInputs),
        outputs: JSON.stringify(inputs.outputs),
      };

      // Add optional swap_params if present (as JSON Value, not stringified)
      if (inputs.swap_params) {
        requestBody.swap_params = inputs.swap_params;
      }

      // Add optional stake_params if present (as JSON Value, not stringified)
      if (inputs.stake_params) {
        requestBody.stake_params = inputs.stake_params;
      }

      // Create abort controller for timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), actualTimeout);
      
      // Simulate progress updates (since we don't have real progress from the backend)
      if (options?.onProgress) {
        let progress = 0;
        progressInterval = setInterval(() => {
          // Simulate gradual progress up to 90%
          progress = Math.min(90, progress + Math.random() * 10);
          options.onProgress!(Math.floor(progress));
        }, 2000);
      }

      // Make API request
      const response = await fetch(`${this.indexerUrl}/api/v1/prove`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(requestBody),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      if (progressInterval) clearInterval(progressInterval);

      if (!response.ok) {
        let errorMessage = `${response.status} ${response.statusText}`;
        try {
          const errorText = await response.text();
          // Try to parse as JSON to extract error message
          try {
            const errorJson = JSON.parse(errorText);
            errorMessage = errorJson.error || errorJson.message || errorText;
          } catch {
            errorMessage = errorText || errorMessage;
          }
        } catch {
          // Ignore parse errors
        }
        
        // Call onError callback
        options?.onError?.(errorMessage);
        
        // Return error result instead of throwing
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage,
        };
      }
      
      // Update progress to 100% before returning success
      options?.onProgress?.(100);

      const rawData = (await response.json()) as any;

      // Map API response fields to our interface (snake_case -> camelCase)
      const result: SP1ProofResult = {
        success: rawData.success,
        proof: rawData.proof,
        publicInputs: rawData.public_inputs, // Map snake_case
        generationTimeMs: rawData.generation_time_ms || (Date.now() - startTime),
        error: rawData.error,
      };
      
      // Store additional debug info in the error if available
      if (!result.success && rawData.execution_report) {
        // execution_report will be included when parsing the error below
      }
      
      // If the response indicates failure, try to extract better error message
      if (!result.success && result.error) {
        try {
          // If error is a JSON string, parse it
          const errorObj = typeof result.error === "string" ? JSON.parse(result.error) : result.error;
          if (errorObj?.error && typeof errorObj.error === "string") {
            result.error = errorObj.error;
          } else if (typeof errorObj === "string") {
            result.error = errorObj;
          }
          // Include execution_report if available for debugging
          if (errorObj?.execution_report && typeof errorObj.execution_report === "string") {
            result.error += `\nExecution report: ${errorObj.execution_report}`;
          }
          // Include total_cycles and total_syscalls if available
          if (errorObj?.total_cycles !== undefined) {
            result.error += `\nTotal cycles: ${errorObj.total_cycles}`;
          }
          if (errorObj?.total_syscalls !== undefined) {
            result.error += `\nTotal syscalls: ${errorObj.total_syscalls}`;
          }
        } catch {
          // If not JSON, keep the error as-is
        }
      }
  
  // Call appropriate callback based on result
  if (result.success) {
    options?.onSuccess?.(result);
  } else if (result.error) {
    options?.onError?.(result.error);
  }

  return result;
    } catch (error) {
      const totalTime = Date.now() - startTime;
      
      // Clean up progress interval if it exists
      if (progressInterval) clearInterval(progressInterval);
      
      let errorMessage: string;
      if (error instanceof Error && error.name === "AbortError") {
        errorMessage = `Proof generation timed out after ${actualTimeout}ms`;
      } else {
        errorMessage = error instanceof Error ? error.message : "Unknown error occurred";
      }
      
      // Call error callback
      options?.onError?.(errorMessage);

      return {
        success: false,
        generationTimeMs: totalTime,
        error: errorMessage,
      };
    }
  }

  /**
   * Check if the prover service is available
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
