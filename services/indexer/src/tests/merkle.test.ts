/// <reference types="jest" />
import { MerkleTree, TreeStorageInterface } from '../lib/merkle.js';

// Mock storage implementation for testing
class MockTreeStorage implements TreeStorageInterface {
  private nodes: Map<string, string> = new Map();

  async storeNode(level: number, index: number, value: string): Promise<void> {
    const key = `${level}:${index}`;
    this.nodes.set(key, value.toLowerCase());
  }

  async getNode(level: number, index: number): Promise<string | null> {
    const key = `${level}:${index}`;
    return this.nodes.get(key) || null;
  }

  async getMaxLeafIndex(): Promise<number> {
    let maxIndex = -1;
    for (const [key, _] of this.nodes) {
      const [levelStr, indexStr] = key.split(':');
      const level = parseInt(levelStr!);
      const index = parseInt(indexStr!);
      
      if (level === 0 && index > maxIndex) {
        maxIndex = index;
      }
    }
    return maxIndex + 1;
  }

  clear(): void {
    this.nodes.clear();
  }

  getStoredNodes(): Map<string, string> {
    return new Map(this.nodes);
  }
}

describe('Merkle Tree', () => {
  let merkleTree: MerkleTree;
  let storage: MockTreeStorage;

  beforeEach(() => {
    storage = new MockTreeStorage();
    merkleTree = new MerkleTree(4, '0000000000000000000000000000000000000000000000000000000000000000'); // Height 4 for testing
  });

  describe('Tree Operations', () => {
    it('should initialize with height and zero values', async () => {
      const treeState = await merkleTree.getTreeState(storage);
      
      expect(treeState.nextIndex).toBe(0);
      expect(treeState.root).toBeDefined();
      expect(treeState.root.length).toBe(64); // 32 bytes as hex string
    });

    it('should insert leaves and update root', async () => {
      const leaf1 = 'a'.repeat(64);
      const leaf2 = 'b'.repeat(64);

      // Insert first leaf
      const result1 = await merkleTree.insertLeaf(leaf1, storage);
      expect(result1.leafIndex).toBe(0);
      expect(result1.root).toBeDefined();

      // Insert second leaf
      const result2 = await merkleTree.insertLeaf(leaf2, storage);
      expect(result2.leafIndex).toBe(1);
      expect(result2.root).toBeDefined();
      expect(result2.root).not.toBe(result1.root); // Root should change

      // Verify tree state
      const treeState = await merkleTree.getTreeState(storage);
      expect(treeState.nextIndex).toBe(2);
      expect(treeState.root).toBe(result2.root);
    });

    it('should store all intermediate nodes', async () => {
      const leaf = 'c'.repeat(64);
      
      await merkleTree.insertLeaf(leaf, storage);
      
      const storedNodes = storage.getStoredNodes();
      
      // Should have leaf at level 0
      expect(storedNodes.has('0:0')).toBe(true);
      expect(storedNodes.get('0:0')).toBe(leaf);
      
      // Should have internal nodes up to root
      expect(storedNodes.has('1:0')).toBe(true); // Parent of leaf
      expect(storedNodes.has('2:0')).toBe(true); // Grandparent
      expect(storedNodes.has('3:0')).toBe(true); // Root
    });
  });

  describe('Proof Generation and Verification', () => {
    beforeEach(async () => {
      // Insert test leaves
      const leaves = ['a'.repeat(64), 'b'.repeat(64), 'c'.repeat(64)];
      for (const leaf of leaves) {
        await merkleTree.insertLeaf(leaf, storage);
      }
    });

    it('should generate valid proof for existing leaf', async () => {
      const leafIndex = 0;
      const leafValue = 'a'.repeat(64);
      
      const proof = await merkleTree.generateProof(leafIndex, storage);
      
      expect(proof.pathElements).toBeDefined();
      expect(proof.pathIndices).toBeDefined();
      expect(proof.pathElements.length).toBe(3); // Height 4 - 1
      expect(proof.pathIndices.length).toBe(3);
    });

    it('should verify generated proofs', async () => {
      const leafIndex = 0;
      const leafValue = 'a'.repeat(64);
      
      const proof = await merkleTree.generateProof(leafIndex, storage);
      const treeState = await merkleTree.getTreeState(storage);
      
      const isValid = merkleTree.verifyProof(
        leafValue,
        leafIndex,
        proof,
        treeState.root
      );
      
      expect(isValid).toBe(true);
    });

    it('should reject invalid proofs', async () => {
      const leafIndex = 0;
      const wrongLeafValue = 'z'.repeat(64); // Wrong leaf value
      
      const proof = await merkleTree.generateProof(leafIndex, storage);
      const treeState = await merkleTree.getTreeState(storage);
      
      const isValid = merkleTree.verifyProof(
        wrongLeafValue,
        leafIndex,
        proof,
        treeState.root
      );
      
      expect(isValid).toBe(false);
    });

    it('should throw error for non-existent leaf proof', async () => {
      const nonExistentIndex = 999;
      
      await expect(
        merkleTree.generateProof(nonExistentIndex, storage)
      ).rejects.toThrow('Leaf index 999 does not exist');
    });
  });

  describe('Deterministic Behavior', () => {
    it('should produce same root for same sequence of leaves', async () => {
      const storage1 = new MockTreeStorage();
      const storage2 = new MockTreeStorage();
      const tree1 = new MerkleTree(4, '0000000000000000000000000000000000000000000000000000000000000000');
      const tree2 = new MerkleTree(4, '0000000000000000000000000000000000000000000000000000000000000000');
      
      const leaves = ['a'.repeat(64), 'b'.repeat(64), 'c'.repeat(64), 'd'.repeat(64)];
      
      // Insert same leaves in both trees
      for (const leaf of leaves) {
        await tree1.insertLeaf(leaf, storage1);
        await tree2.insertLeaf(leaf, storage2);
      }
      
      const state1 = await tree1.getTreeState(storage1);
      const state2 = await tree2.getTreeState(storage2);
      
      expect(state1.root).toBe(state2.root);
      expect(state1.nextIndex).toBe(state2.nextIndex);
    });

    it('should produce same proofs for same leaf positions', async () => {
      const storage1 = new MockTreeStorage();
      const storage2 = new MockTreeStorage();
      const tree1 = new MerkleTree(4, '0000000000000000000000000000000000000000000000000000000000000000');
      const tree2 = new MerkleTree(4, '0000000000000000000000000000000000000000000000000000000000000000');
      
      const leaves = ['a'.repeat(64), 'b'.repeat(64), 'c'.repeat(64)];
      
      // Insert same leaves in both trees
      for (const leaf of leaves) {
        await tree1.insertLeaf(leaf, storage1);
        await tree2.insertLeaf(leaf, storage2);
      }
      
      const proof1 = await tree1.generateProof(1, storage1);
      const proof2 = await tree2.generateProof(1, storage2);
      
      expect(proof1.pathElements).toEqual(proof2.pathElements);
      expect(proof1.pathIndices).toEqual(proof2.pathIndices);
    });
  });
});
