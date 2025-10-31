import { MerkleProof, MerkleRootResponse } from "../core/types";

/**
 * Response from notes range query
 */
export interface NotesRangeResponse {
  notes: string[];
  has_more: boolean;
  total: number;
  start: number;
  end: number;
}

/**
 * Indexer Service Client
 *
 * Provides access to the Cloak Indexer API for querying the Merkle tree
 * and registering deposits.
 */
export class IndexerService {
  private baseUrl: string;

  /**
   * Create a new Indexer Service client
   *
   * @param baseUrl - Indexer API base URL
   */
  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash
  }

  /**
   * Get current Merkle root and next available index
   *
   * @returns Current root and next index
   *
   * @example
   * ```typescript
   * const { root, next_index } = await indexer.getMerkleRoot();
   * console.log(`Current root: ${root}, Next index: ${next_index}`);
   * ```
   */
  async getMerkleRoot(): Promise<MerkleRootResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/merkle/root`);

    if (!response.ok) {
      throw new Error(
        `Failed to get Merkle root: ${response.status} ${response.statusText}`
      );
    }

    const json = (await response.json()) as unknown as MerkleRootResponse;
    return json;
  }

  /**
   * Get Merkle proof for a specific leaf
   *
   * @param leafIndex - Index of the leaf in the tree
   * @returns Merkle proof with path elements and indices
   *
   * @example
   * ```typescript
   * const proof = await indexer.getMerkleProof(42);
   * console.log(`Proof has ${proof.pathElements.length} siblings`);
   * ```
   */
  async getMerkleProof(leafIndex: number): Promise<MerkleProof> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/merkle/proof/${leafIndex}`
    );

    if (!response.ok) {
      throw new Error(
        `Failed to get Merkle proof: ${response.status} ${response.statusText}`
      );
    }

    const data = (await response.json()) as any;

    // Normalize field names (handle both snake_case and camelCase)
    return {
      pathElements: data.pathElements ?? data.path_elements,
      pathIndices: data.pathIndices ?? data.path_indices,
      root: data.root,
    };
  }

  /**
   * Get notes in a specific range
   *
   * Useful for scanning the tree or fetching notes in batches.
   *
   * @param start - Start index (inclusive)
   * @param end - End index (inclusive)
   * @param limit - Maximum number of notes to return (default: 100)
   * @returns Notes in the range
   *
   * @example
   * ```typescript
   * const { notes, has_more } = await indexer.getNotesRange(0, 99, 100);
   * console.log(`Fetched ${notes.length} notes`);
   * ```
   */
  async getNotesRange(
    start: number,
    end: number,
    limit: number = 100
  ): Promise<NotesRangeResponse> {
    const url = new URL(`${this.baseUrl}/api/v1/notes/range`);
    url.searchParams.set("start", start.toString());
    url.searchParams.set("end", end.toString());
    url.searchParams.set("limit", limit.toString());

    const response = await fetch(url.toString());

    if (!response.ok) {
      throw new Error(
        `Failed to get notes range: ${response.status} ${response.statusText}`
      );
    }

    const json = (await response.json()) as unknown as NotesRangeResponse;
    return json;
  }

  /**
   * Get all notes from the tree
   *
   * Fetches all notes in batches. Use with caution for large trees.
   *
   * @param batchSize - Size of each batch (default: 100)
   * @returns All encrypted notes
   *
   * @example
   * ```typescript
   * const allNotes = await indexer.getAllNotes();
   * console.log(`Total notes: ${allNotes.length}`);
   * ```
   */
  async getAllNotes(batchSize: number = 100): Promise<string[]> {
    const rootResponse = await this.getMerkleRoot();
    const totalNotes = rootResponse.next_index;

    if (totalNotes === 0) {
      return [];
    }

    const allNotes: string[] = [];

    for (let start = 0; start < totalNotes; start += batchSize) {
      const end = Math.min(start + batchSize - 1, totalNotes - 1);
      const response = await this.getNotesRange(start, end, batchSize);
      allNotes.push(...response.notes);
    }

    return allNotes;
  }

  /**
   * Submit a deposit to the indexer
   *
   * Registers a new deposit transaction with the indexer, which will
   * return the leaf index and current root.
   *
   * @param params - Deposit parameters
   * @returns Success response with leaf index and root
   *
   * @example
   * ```typescript
   * const result = await indexer.submitDeposit({
   *   leafCommit: note.commitment,
   *   encryptedOutput: btoa(JSON.stringify(noteData)),
   *   txSignature: signature,
   *   slot: txSlot
   * });
   * console.log(`Leaf index: ${result.leafIndex}`);
   * ```
   */
  async submitDeposit(params: {
    leafCommit: string;
    encryptedOutput: string;
    txSignature: string;
    slot: number;
  }): Promise<{ success: boolean; leafIndex?: number; root?: string }> {
    const response = await fetch(`${this.baseUrl}/api/v1/deposit`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        leaf_commit: params.leafCommit,
        encrypted_output: params.encryptedOutput,
        tx_signature: params.txSignature,
        slot: params.slot,
      }),
    });

    // Read response body once
    let responseData: any;
    try {
      responseData = await response.json();
    } catch {
      // If JSON parse fails, try text
      try {
        const text = await response.text();
        responseData = text ? { error: text } : null;
      } catch {
        responseData = null;
      }
    }

    if (!response.ok) {
      let errorMessage = `${response.status} ${response.statusText}`;
      if (responseData) {
        if (typeof responseData === 'string') {
          errorMessage = responseData;
        } else {
          errorMessage = responseData?.error || responseData?.message || errorMessage;
          // Include additional context if available
          if (responseData?.details) {
            errorMessage += ` (${JSON.stringify(responseData.details)})`;
          }
        }
      }
      throw new Error(`Failed to submit deposit: ${errorMessage}`);
    }

    // Response was successful
    const data = responseData;

    // Normalize field names
    return {
      success: data.success ?? true,
      leafIndex: data.leafIndex ?? data.leaf_index,
      root: data.root,
    };
  }

  /**
   * Check indexer health
   *
   * @returns Health status
   */
  async healthCheck(): Promise<{ status: string }> {
    const response = await fetch(`${this.baseUrl}/health`);

    if (!response.ok) {
      throw new Error(
        `Health check failed: ${response.status} ${response.statusText}`
      );
    }

    const json = (await response.json()) as unknown as { status: string };
    return json;
  }
}
