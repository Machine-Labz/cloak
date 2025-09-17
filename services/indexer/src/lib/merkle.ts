import { createHash } from 'blake3';
import { MerkleProof, MerkleTreeState, TreeNode } from '../types/index.js';
import { logger } from './logger.js';

/**
 * Merkle Tree implementation for Cloak privacy protocol
 * Uses BLAKE3 hashing with fixed height of 32
 * 
 * Tree structure:
 * - Binary tree with height 32 (supports up to 2^32 leaves)
 * - Uses BLAKE3-256 for all hashing operations  
 * - Zero values are all zeros (32 bytes)
 * - Leaves are commitment values (32 bytes hex)
 */
export class MerkleTree {
  private readonly height: number;
  private readonly zeroValues: string[];
  private nextIndex: number = 0;

  constructor(height: number = 32, zeroValue: string = '0'.repeat(64)) {
    this.height = height;
    this.zeroValues = this.computeZeroValues(zeroValue);
    
    logger.info('Initialized Merkle Tree', {
      height: this.height,
      zeroValue: zeroValue,
      zeroValuesComputed: this.zeroValues.length
    });
  }

  /**
   * Compute zero values for each level of the tree
   * Level 0 = leaf level, Level height-1 = root level
   */
  private computeZeroValues(zeroValue: string): string[] {
    const zeros: string[] = [zeroValue];
    
    for (let i = 1; i < this.height; i++) {
      const prevZero = zeros[i - 1]!;
      const hash = this.hashPair(prevZero, prevZero);
      zeros.push(hash);
    }
    
    return zeros;
  }

  /**
   * Hash two values using BLAKE3-256
   * Inputs should be hex strings without 0x prefix
   * Returns hex string without 0x prefix
   */
  private hashPair(left: string, right: string): string {
    // Remove 0x prefix if present and ensure lowercase
    const leftClean = left.replace('0x', '').toLowerCase();
    const rightClean = right.replace('0x', '').toLowerCase();
    
    // Convert hex to bytes
    const leftBytes = Buffer.from(leftClean, 'hex');
    const rightBytes = Buffer.from(rightClean, 'hex');
    
    if (leftBytes.length !== 32 || rightBytes.length !== 32) {
      throw new Error(`Invalid hash input length: left=${leftBytes.length}, right=${rightBytes.length}`);
    }
    
    // Concatenate and hash
    const combined = Buffer.concat([leftBytes, rightBytes]);
    const hash = createHash();
    hash.update(combined);
    
    return hash.digest('hex');
  }

  /**
   * Insert a new leaf into the tree and return the new root
   * @param leafValue - Hex string of the leaf commitment (32 bytes)
   * @param storageInterface - Interface to persist tree nodes
   */
  async insertLeaf(
    leafValue: string, 
    storageInterface: TreeStorageInterface
  ): Promise<{ root: string; leafIndex: number }> {
    const leafIndex = this.nextIndex;
    logger.info('Inserting leaf', { leafValue, leafIndex });

    // Store the leaf at level 0
    await storageInterface.storeNode(0, leafIndex, leafValue);

    // Compute and store internal nodes bottom-up
    let currentIndex = leafIndex;
    let currentValue = leafValue;

    for (let level = 0; level < this.height - 1; level++) {
      const isLeftChild = currentIndex % 2 === 0;
      const parentIndex = Math.floor(currentIndex / 2);
      
      let leftChild: string;
      let rightChild: string;

      if (isLeftChild) {
        leftChild = currentValue;
        // Try to get right sibling from storage, otherwise use zero value
        const rightSibling = await storageInterface.getNode(level, currentIndex + 1);
        rightChild = rightSibling ?? this.zeroValues[level]!;
      } else {
        // Left sibling must exist since we're processing leaves in order
        const leftSibling = await storageInterface.getNode(level, currentIndex - 1);
        if (!leftSibling) {
          throw new Error(`Missing left sibling at level ${level}, index ${currentIndex - 1}`);
        }
        leftChild = leftSibling;
        rightChild = currentValue;
      }

      // Compute parent hash
      const parentValue = this.hashPair(leftChild, rightChild);
      
      // Store parent node
      await storageInterface.storeNode(level + 1, parentIndex, parentValue);
      
      // Move up the tree
      currentIndex = parentIndex;
      currentValue = parentValue;
    }

    this.nextIndex++;
    const rootValue = currentValue;
    
    logger.info('Leaf inserted successfully', { 
      leafIndex, 
      rootValue, 
      nextIndex: this.nextIndex 
    });

    return { root: rootValue, leafIndex };
  }

  /**
   * Generate a Merkle proof for a given leaf index
   * @param leafIndex - Index of the leaf to prove
   * @param storageInterface - Interface to retrieve tree nodes
   */
  async generateProof(
    leafIndex: number, 
    storageInterface: TreeStorageInterface
  ): Promise<MerkleProof> {
    if (leafIndex >= this.nextIndex) {
      throw new Error(`Leaf index ${leafIndex} does not exist (nextIndex: ${this.nextIndex})`);
    }

    const pathElements: string[] = [];
    const pathIndices: number[] = [];

    let currentIndex = leafIndex;

    for (let level = 0; level < this.height - 1; level++) {
      const isLeftChild = currentIndex % 2 === 0;
      pathIndices.push(isLeftChild ? 0 : 1);

      const siblingIndex = isLeftChild ? currentIndex + 1 : currentIndex - 1;
      
      // Get sibling from storage or use zero value
      let siblingValue: string;
      const storedSibling = await storageInterface.getNode(level, siblingIndex);
      if (storedSibling) {
        siblingValue = storedSibling;
      } else {
        // Use zero value for non-existent siblings
        siblingValue = this.zeroValues[level]!;
      }

      pathElements.push(siblingValue);
      currentIndex = Math.floor(currentIndex / 2);
    }

    logger.debug('Generated Merkle proof', { 
      leafIndex, 
      pathElements: pathElements.length, 
      pathIndices 
    });

    return { pathElements, pathIndices };
  }

  /**
   * Verify a Merkle proof
   * @param leafValue - The leaf value being proved
   * @param leafIndex - Index of the leaf
   * @param proof - The Merkle proof
   * @param expectedRoot - Expected root to verify against
   */
  verifyProof(
    leafValue: string, 
    leafIndex: number, 
    proof: MerkleProof, 
    expectedRoot: string
  ): boolean {
    if (proof.pathElements.length !== this.height - 1) {
      logger.error('Invalid proof length', { 
        expected: this.height - 1, 
        actual: proof.pathElements.length 
      });
      return false;
    }

    let currentValue = leafValue;
    let currentIndex = leafIndex;

    for (let i = 0; i < proof.pathElements.length; i++) {
      const pathElement = proof.pathElements[i]!;
      const isLeftChild = proof.pathIndices[i] === 0;

      if (isLeftChild) {
        // Current value is left child
        currentValue = this.hashPair(currentValue, pathElement);
      } else {
        // Current value is right child  
        currentValue = this.hashPair(pathElement, currentValue);
      }

      currentIndex = Math.floor(currentIndex / 2);
    }

    const isValid = currentValue === expectedRoot.replace('0x', '').toLowerCase();
    
    logger.debug('Proof verification result', { 
      leafIndex, 
      computedRoot: currentValue, 
      expectedRoot, 
      isValid 
    });

    return isValid;
  }

  /**
   * Get current tree state
   */
  async getTreeState(storageInterface: TreeStorageInterface): Promise<MerkleTreeState> {
    let root: string;
    
    if (this.nextIndex === 0) {
      // Empty tree - root is zero value at max level
      root = this.zeroValues[this.height - 1]!;
    } else {
      // Get current root from storage
      const storedRoot = await storageInterface.getNode(this.height - 1, 0);
      if (!storedRoot) {
        throw new Error('Root not found in storage');
      }
      root = storedRoot;
    }

    return { root, nextIndex: this.nextIndex };
  }

  /**
   * Set the next index (used during initialization from storage)
   */
  setNextIndex(index: number): void {
    this.nextIndex = index;
    logger.info('Set next index', { nextIndex: this.nextIndex });
  }
}

/**
 * Interface for tree storage operations
 * Allows different storage backends (PostgreSQL, RocksDB, etc.)
 */
export interface TreeStorageInterface {
  /**
   * Store a tree node
   * @param level - Tree level (0 = leaves, height-1 = root)
   * @param index - Index at that level
   * @param value - Hex string value (32 bytes)
   */
  storeNode(level: number, index: number, value: string): Promise<void>;

  /**
   * Retrieve a tree node
   * @param level - Tree level
   * @param index - Index at that level
   * @returns Hex string value or null if not found
   */
  getNode(level: number, index: number): Promise<string | null>;

  /**
   * Get the maximum index used at level 0 (leaves)
   * Used to initialize nextIndex on startup
   */
  getMaxLeafIndex(): Promise<number>;
}
