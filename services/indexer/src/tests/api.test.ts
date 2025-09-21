/// <reference types="jest" />
import request from 'supertest';
import { Application } from 'express';
import { createServer } from '../api/server.js';
import { db } from '../db/connection.js';

describe('Indexer API', () => {
  let app: Application;

  beforeAll(async () => {
    app = createServer();
    
    // Ensure database is connected
    const isConnected = await db.testConnection();
    if (!isConnected) {
      throw new Error('Cannot connect to test database');
    }

    // Clean up any existing test data
    await db.query('DELETE FROM merkle_tree_nodes WHERE level >= 0');
    await db.query('DELETE FROM notes WHERE id >= 0');
    await db.query('UPDATE indexer_metadata SET value = \'0\' WHERE key = \'next_leaf_index\'');
  });

  afterAll(async () => {
    await db.close();
  });

  describe('Health Checks', () => {
    it('should return API info at root', async () => {
      const response = await request(app).get('/');
      
      expect(response.status).toBe(200);
      expect(response.body.name).toBe('Cloak Indexer API');
      expect(response.body.endpoints).toHaveProperty('deposit');
      expect(response.body.endpoints).toHaveProperty('merkleRoot');
    });

    it('should return health status', async () => {
      const response = await request(app).get('/health');
      
      expect(response.status).toBe(200);
      expect(response.body.status).toBe('healthy');
      expect(response.body.database.healthy).toBe(true);
    });
  });

  describe('Deposit Flow', () => {
    const testLeafCommit = 'a'.repeat(64); // Valid 64-char hex string
    const testEncryptedOutput = 'encrypted_test_output_base64_encoded';

    it('should accept valid deposit', async () => {
      const response = await request(app)
        .post('/api/v1/deposit')
        .send({
          leafCommit: testLeafCommit,
          encryptedOutput: testEncryptedOutput,
          txSignature: 'test_signature_123',
          slot: 1000
        });

      expect(response.status).toBe(201);
      expect(response.body.success).toBe(true);
      expect(response.body.leafIndex).toBe(0); // First leaf
      expect(response.body.nextIndex).toBe(1);
      expect(response.body.root).toBeDefined();
      expect(response.body.leafCommit).toBe(testLeafCommit);
    });

    it('should reject invalid leaf commit', async () => {
      const response = await request(app)
        .post('/api/v1/deposit')
        .send({
          leafCommit: 'invalid_hex', // Too short and invalid chars
          encryptedOutput: testEncryptedOutput
        });

      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Validation error');
    });

    it('should reject missing encrypted output', async () => {
      const response = await request(app)
        .post('/api/v1/deposit')
        .send({
          leafCommit: testLeafCommit
          // Missing encryptedOutput
        });

      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Validation error');
    });
  });

  describe('Merkle Tree Operations', () => {
    beforeAll(async () => {
      // Insert a test deposit first
      await request(app)
        .post('/api/v1/deposit')
        .send({
          leafCommit: 'b'.repeat(64),
          encryptedOutput: 'test_encrypted_output_2'
        });
    });

    it('should return current tree root', async () => {
      const response = await request(app).get('/api/v1/merkle/root');
      
      expect(response.status).toBe(200);
      expect(response.body.root).toBeDefined();
      expect(response.body.nextIndex).toBeGreaterThan(0);
      expect(typeof response.body.root).toBe('string');
      expect(response.body.root.length).toBe(64); // 32 bytes = 64 hex chars
    });

    it('should generate merkle proof for valid index', async () => {
      const response = await request(app).get('/api/v1/merkle/proof/0');
      
      expect(response.status).toBe(200);
      expect(response.body.pathElements).toBeDefined();
      expect(response.body.pathIndices).toBeDefined();
      expect(Array.isArray(response.body.pathElements)).toBe(true);
      expect(Array.isArray(response.body.pathIndices)).toBe(true);
      expect(response.body.pathElements.length).toBe(31); // Height 32 - 1
    });

    it('should reject proof for invalid index', async () => {
      const response = await request(app).get('/api/v1/merkle/proof/999');
      
      expect(response.status).toBe(400);
      expect(response.body.error).toBeDefined();
    });
  });

  describe('Notes Range Query', () => {
    it('should return notes in valid range', async () => {
      const response = await request(app).get('/api/v1/notes/range?start=0&end=10');
      
      expect(response.status).toBe(200);
      expect(response.body.encryptedOutputs).toBeDefined();
      expect(Array.isArray(response.body.encryptedOutputs)).toBe(true);
      expect(response.body.hasMore).toBeDefined();
      expect(response.body.total).toBeDefined();
      expect(response.body.start).toBe(0);
      expect(response.body.end).toBe(10);
    });

    it('should validate range parameters', async () => {
      const response = await request(app).get('/api/v1/notes/range?start=invalid&end=10');
      
      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Validation error');
    });

    it('should reject end < start', async () => {
      const response = await request(app).get('/api/v1/notes/range?start=10&end=5');
      
      expect(response.status).toBe(400);
      expect(response.body.error).toContain('End index must be greater than or equal to start');
    });
  });

  describe('Error Handling', () => {
    it('should return 404 for unknown routes', async () => {
      const response = await request(app).get('/api/v1/nonexistent');
      
      expect(response.status).toBe(404);
      expect(response.body.error).toBe('Not found');
      expect(response.body.availableEndpoints).toBeDefined();
    });
  });
});
