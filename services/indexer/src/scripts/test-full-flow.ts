#!/usr/bin/env node

/**
 * Complete integration test for the Cloak Indexer
 * Tests the full deposit -> proof -> artifacts flow
 */

import { config } from '../lib/config.js';
import { logger } from '../lib/logger.js';

interface TestResults {
  deposit: boolean;
  merkleRoot: boolean;
  merkleProof: boolean;
  notesRange: boolean;
  artifacts: boolean;
  artifactFiles: boolean;
}

class IndexerTester {
  private baseUrl: string;
  private results: TestResults = {
    deposit: false,
    merkleRoot: false,
    merkleProof: false,
    notesRange: false,
    artifacts: false,
    artifactFiles: false,
  };

  constructor() {
    this.baseUrl = `http://localhost:${config.server.port}`;
  }

  async runAllTests(): Promise<void> {
    logger.info('🚀 Starting complete indexer integration test');

    try {
      // Clean database first
      await this.cleanupDatabase();
      
      // Test health check first
      await this.testHealthCheck();

      // Test core deposit flow
    const depositResult = await this.testDeposit();
    this.results.deposit = depositResult.success;

    if (depositResult.success && depositResult.leafIndex !== undefined) {
      // Test merkle operations
      this.results.merkleRoot = await this.testMerkleRoot();
      this.results.merkleProof = await this.testMerkleProof(depositResult.leafIndex);
        this.results.notesRange = await this.testNotesRange();
      }

      // Test artifact hosting (independent of deposit)
      this.results.artifacts = await this.testArtifacts();
      this.results.artifactFiles = await this.testArtifactFiles();

      // Print results
      this.printResults();

    } catch (error) {
      logger.error('Integration test failed', { error });
      throw error;
    }
  }

  private async testHealthCheck(): Promise<void> {
    logger.info('🔍 Testing health check...');
    
    const response = await fetch(`${this.baseUrl}/health`);
    const data: any = await response.json();
    
    if (response.status !== 200 || data.status !== 'healthy') {
      throw new Error(`Health check failed: ${JSON.stringify(data)}`);
    }
    
    logger.info('✅ Health check passed');
  }

  private async cleanupDatabase(): Promise<void> {
    logger.info('🧹 Cleaning up test data...');
    
    try {
      // Clean up test data
      await fetch(`${this.baseUrl}/api/v1/admin/cleanup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      }).catch(() => {
        // Ignore cleanup errors - database might be clean already
        logger.info('Database cleanup skipped (no cleanup endpoint)');
      });
    } catch (error) {
      logger.info('Database cleanup skipped', { error });
    }
  }

  private async testDeposit(): Promise<{ success: boolean; leafIndex?: number }> {
    logger.info('🔍 Testing deposit endpoint...');
    
    // Use unique data to avoid conflicts
    const timestamp = Date.now();
    const testDeposit = {
      leafCommit: timestamp.toString(16).padStart(64, '0'), // Unique based on timestamp
      encryptedOutput: Buffer.from(`Integration test ${timestamp}`).toString('base64'),
      txSignature: `integration_test_${timestamp}`,
      slot: 2000 + Math.floor(Math.random() * 1000), // Random slot to avoid conflicts
    };

    try {
      const response = await fetch(`${this.baseUrl}/api/v1/deposit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(testDeposit),
      });

      const data: any = await response.json();

      if (response.status === 201 && data.success) {
        logger.info('✅ Deposit test passed', {
          leafIndex: data.leafIndex,
          root: data.root.substring(0, 16) + '...',
          nextIndex: data.nextIndex,
        });
        return { success: true, leafIndex: data.leafIndex };
      } else {
        logger.error('❌ Deposit test failed', { status: response.status, data });
        return { success: false };
      }
    } catch (error) {
      logger.error('❌ Deposit test error', { error });
      return { success: false };
    }
  }

  private async testMerkleRoot(): Promise<boolean> {
    logger.info('🔍 Testing merkle root endpoint...');
    
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/merkle/root`);
      const data = await response.json() as any;

      if (response.status === 200 && data.root && typeof data.nextIndex === 'number') {
        logger.info('✅ Merkle root test passed', {
          root: data.root.substring(0, 16) + '...',
          nextIndex: data.nextIndex,
        });
        return true;
      } else {
        logger.error('❌ Merkle root test failed', { status: response.status, data });
        return false;
      }
    } catch (error) {
      logger.error('❌ Merkle root test error', { error });
      return false;
    }
  }

  private async testMerkleProof(leafIndex: number): Promise<boolean> {
    logger.info('🔍 Testing merkle proof endpoint...', { leafIndex });
    
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/merkle/proof/${leafIndex}`);
      const data = await response.json() as any;

      if (response.status === 200 && 
          Array.isArray(data.pathElements) && 
          Array.isArray(data.pathIndices) &&
          data.pathElements.length === 31) { // Height 32 - 1
        
        logger.info('✅ Merkle proof test passed', {
          leafIndex,
          pathElementsCount: data.pathElements.length,
          pathIndicesCount: data.pathIndices.length,
        });
        return true;
      } else {
        logger.error('❌ Merkle proof test failed', { status: response.status, data });
        return false;
      }
    } catch (error) {
      logger.error('❌ Merkle proof test error', { error, leafIndex });
      return false;
    }
  }

  private async testNotesRange(): Promise<boolean> {
    logger.info('🔍 Testing notes range endpoint...');
    
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/notes/range?start=0&end=10&limit=5`);
      const data = await response.json() as any;

      if (response.status === 200 && 
          Array.isArray(data.encryptedOutputs) &&
          typeof data.hasMore === 'boolean' &&
          typeof data.total === 'number') {
        
        logger.info('✅ Notes range test passed', {
          encryptedOutputsCount: data.encryptedOutputs.length,
          hasMore: data.hasMore,
          total: data.total,
        });
        return true;
      } else {
        logger.error('❌ Notes range test failed', { status: response.status, data });
        return false;
      }
    } catch (error) {
      logger.error('❌ Notes range test error', { error });
      return false;
    }
  }

  private async testArtifacts(): Promise<boolean> {
    logger.info('🔍 Testing artifacts endpoint...');
    
    const version = 'v2.0.0';
    
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/artifacts/withdraw/${version}`);
      const data = await response.json() as any;

      if (response.status === 200 && 
          data.guestElfUrl &&
          data.vk &&
          data.sha256?.elf &&
          data.sha256?.vk &&
          data.sp1Version) {
        
        logger.info('✅ Artifacts test passed', {
          version,
          guestElfUrl: data.guestElfUrl,
          vkLength: data.vk.length,
          elfHash: data.sha256.elf.substring(0, 16) + '...',
          vkHash: data.sha256.vk.substring(0, 16) + '...',
          sp1Version: data.sp1Version,
        });
        return true;
      } else {
        logger.error('❌ Artifacts test failed', { status: response.status, data });
        return false;
      }
    } catch (error) {
      logger.error('❌ Artifacts test error', { error });
      return false;
    }
  }

  private async testArtifactFiles(): Promise<boolean> {
    logger.info('🔍 Testing artifact file serving...');
    
    const version = 'v2.0.0';
    const filename = 'guest.elf';
    
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/artifacts/files/${version}/${filename}`);

      if (response.status === 200) {
        const contentType = response.headers.get('content-type');
        const contentLength = response.headers.get('content-length');
        const sha256 = response.headers.get('x-sha256');
        const etag = response.headers.get('etag');

        logger.info('✅ Artifact file serving test passed', {
          version,
          filename,
          contentType,
          contentLength,
          sha256: sha256?.substring(0, 16) + '...',
          etag: etag?.substring(0, 20) + '...',
        });
        return true;
      } else {
        const data = await response.json();
        logger.error('❌ Artifact file serving test failed', { status: response.status, data });
        return false;
      }
    } catch (error) {
      logger.error('❌ Artifact file serving test error', { error });
      return false;
    }
  }

  private printResults(): void {
    logger.info('📊 Integration Test Results:');
    
    const allPassed = Object.values(this.results).every(result => result === true);
    
    Object.entries(this.results).forEach(([test, passed]) => {
      const icon = passed ? '✅' : '❌';
      const status = passed ? 'PASSED' : 'FAILED';
      logger.info(`${icon} ${test.toUpperCase()}: ${status}`);
    });

    if (allPassed) {
      logger.info('🎉 ALL TESTS PASSED! Indexer is working correctly.');
    } else {
      logger.error('❌ Some tests failed. Check the logs above.');
      process.exit(1);
    }
  }
}

// Run tests if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const tester = new IndexerTester();
  
  tester.runAllTests()
    .then(() => {
      logger.info('Integration test completed successfully');
      process.exit(0);
    })
    .catch((error) => {
      logger.error('Integration test failed', { error });
      process.exit(1);
    });
}

export { IndexerTester };
