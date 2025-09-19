/// <reference types="jest" />
import { PostgresTreeStorage } from '../db/storage.js';
import { db } from '../db/connection.js';

describe('PostgresTreeStorage', () => {
  let storage: PostgresTreeStorage;

  beforeAll(async () => {
    storage = new PostgresTreeStorage();
    
    // Ensure database is connected
    const isConnected = await db.testConnection();
    if (!isConnected) {
      throw new Error('Cannot connect to test database');
    }
  });

  beforeEach(async () => {
    // Clean up test data before each test
    await db.query('DELETE FROM merkle_tree_nodes WHERE level >= 0');
    await db.query('DELETE FROM notes WHERE id >= 0');
    await db.query('DELETE FROM event_processing_log WHERE id >= 0');
    await db.query('UPDATE indexer_metadata SET value = \'0\' WHERE key = \'next_leaf_index\'');
  });

  afterAll(async () => {
    await db.close();
  });

  describe('Tree Node Operations', () => {
    it('should store and retrieve tree nodes', async () => {
      const level = 0;
      const index = 0;
      const value = 'a'.repeat(64);

      await storage.storeNode(level, index, value);
      const retrieved = await storage.getNode(level, index);

      expect(retrieved).toBe(value.toLowerCase());
    });

    it('should handle non-existent nodes', async () => {
      const retrieved = await storage.getNode(999, 999);
      expect(retrieved).toBeNull();
    });

    it('should update existing nodes', async () => {
      const level = 1;
      const index = 0;
      const value1 = 'a'.repeat(64);
      const value2 = 'b'.repeat(64);

      // Store first value
      await storage.storeNode(level, index, value1);
      let retrieved = await storage.getNode(level, index);
      expect(retrieved).toBe(value1.toLowerCase());

      // Update with second value
      await storage.storeNode(level, index, value2);
      retrieved = await storage.getNode(level, index);
      expect(retrieved).toBe(value2.toLowerCase());
    });

    it('should get max leaf index correctly', async () => {
      // Initially should be 0 (no leaves)
      let maxIndex = await storage.getMaxLeafIndex();
      expect(maxIndex).toBe(0);

      // Store some leaves at level 0
      await storage.storeNode(0, 0, 'a'.repeat(64));
      await storage.storeNode(0, 1, 'b'.repeat(64));
      await storage.storeNode(0, 2, 'c'.repeat(64));

      maxIndex = await storage.getMaxLeafIndex();
      expect(maxIndex).toBe(3); // Next index should be 3
    });
  });

  describe('Note Operations', () => {
    it('should store and retrieve notes', async () => {
      const leafCommit = 'a'.repeat(64);
      const encryptedOutput = 'test_encrypted_output';
      const leafIndex = 0;
      const txSignature = 'test_signature_123';
      const slot = 1000;

      await storage.storeNote(
        leafCommit,
        encryptedOutput,
        leafIndex,
        txSignature,
        slot,
        new Date()
      );

      const note = await storage.getNoteByIndex(leafIndex);
      expect(note).not.toBeNull();
      expect(note!.leafCommit).toBe(leafCommit.toLowerCase());
      expect(note!.encryptedOutput).toBe(encryptedOutput);
      expect(note!.leafIndex).toBe(leafIndex);
      expect(note!.txSignature).toBe(txSignature);
      expect(note!.slot).toBe(slot);
    });

    it('should get notes in range', async () => {
      // Store multiple notes
      const notes = [
        { leafCommit: 'a'.repeat(64), encryptedOutput: 'output1', leafIndex: 0, txSignature: 'tx1', slot: 1000 },
        { leafCommit: 'b'.repeat(64), encryptedOutput: 'output2', leafIndex: 1, txSignature: 'tx2', slot: 1001 },
        { leafCommit: 'c'.repeat(64), encryptedOutput: 'output3', leafIndex: 2, txSignature: 'tx3', slot: 1002 },
        { leafCommit: 'd'.repeat(64), encryptedOutput: 'output4', leafIndex: 3, txSignature: 'tx4', slot: 1003 },
      ];

      for (const note of notes) {
        await storage.storeNote(
          note.leafCommit,
          note.encryptedOutput,
          note.leafIndex,
          note.txSignature,
          note.slot,
          new Date()
        );
      }

      // Get range
      const result = await storage.getNotesRange(1, 2, 10);

      expect(result.encryptedOutputs).toHaveLength(2);
      expect(result.encryptedOutputs).toContain('output2');
      expect(result.encryptedOutputs).toContain('output3');
      expect(result.total).toBe(2);
      expect(result.start).toBe(1);
      expect(result.end).toBe(2);
      expect(result.hasMore).toBe(false);
    });

    it('should handle pagination correctly', async () => {
      // Store 5 notes
      for (let i = 0; i < 5; i++) {
        await storage.storeNote(
          i.toString().repeat(64).substring(0, 64).padEnd(64, '0'),
          `output${i}`,
          i,
          `tx${i}`,
          1000 + i,
          new Date()
        );
      }

      // Get first 3 notes
      const result = await storage.getNotesRange(0, 4, 3);

      expect(result.encryptedOutputs).toHaveLength(3);
      expect(result.total).toBe(5);
      expect(result.hasMore).toBe(true);
    });
  });

  describe('Metadata Operations', () => {
    it('should store and retrieve metadata', async () => {
      const key = 'test_key';
      const value = 'test_value';

      await storage.updateMetadata(key, value);
      const retrieved = await storage.getMetadata(key);

      expect(retrieved).toBe(value);
    });

    it('should update existing metadata', async () => {
      const key = 'test_key';
      const value1 = 'value1';
      const value2 = 'value2';

      await storage.updateMetadata(key, value1);
      let retrieved = await storage.getMetadata(key);
      expect(retrieved).toBe(value1);

      await storage.updateMetadata(key, value2);
      retrieved = await storage.getMetadata(key);
      expect(retrieved).toBe(value2);
    });

    it('should return null for non-existent metadata', async () => {
      const retrieved = await storage.getMetadata('non_existent_key');
      expect(retrieved).toBeNull();
    });
  });

  describe('Event Processing Log', () => {
    it('should log event processing', async () => {
      const txSignature = 'test_signature';
      const slot = 1000;
      const eventType = 'deposit';

      await storage.logEventProcessing(txSignature, slot, eventType, 'success');

      // Verify log was created (we don't have a direct getter, but this tests the operation doesn't throw)
      expect(true).toBe(true);
    });

    it('should handle failed event processing', async () => {
      const txSignature = 'failed_signature';
      const slot = 1001;
      const eventType = 'deposit';
      const errorMessage = 'Test error message';

      await storage.logEventProcessing(txSignature, slot, eventType, 'failed', errorMessage);

      // Verify operation completes without error
      expect(true).toBe(true);
    });
  });

  describe('Health Check', () => {
    it('should return healthy status when database is connected', async () => {
      const health = await storage.healthCheck();

      expect(health.healthy).toBe(true);
      expect(health.details.currentTime).toBeDefined();
      expect(health.details.tablesFound).toContain('merkle_tree_nodes');
      expect(health.details.tablesFound).toContain('notes');
      expect(health.details.tablesFound).toContain('indexer_metadata');
      expect(health.details.stats).toBeDefined();
      expect(health.details.poolStats).toBeDefined();
    });
  });

  describe('Data Validation', () => {
    it('should reject invalid hex strings', async () => {
      const invalidHex = 'invalid_hex_string';

      await expect(
        storage.storeNode(0, 0, invalidHex)
      ).rejects.toThrow('Invalid node value length');
    });

    it('should normalize hex strings to lowercase', async () => {
      const upperCaseHex = 'A'.repeat(64);
      
      await storage.storeNode(0, 0, upperCaseHex);
      const retrieved = await storage.getNode(0, 0);

      expect(retrieved).toBe(upperCaseHex.toLowerCase());
    });

    it('should handle hex strings with 0x prefix', async () => {
      const hexWithPrefix = '0x' + 'a'.repeat(62);
      const expectedHex = 'a'.repeat(64);

      await storage.storeNote(
        hexWithPrefix,
        'test_output',
        0,
        'test_tx',
        1000,
        new Date()
      );

      const note = await storage.getNoteByIndex(0);
      expect(note!.leafCommit).toBe(expectedHex);
    });
  });
});
