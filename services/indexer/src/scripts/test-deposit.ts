#!/usr/bin/env node

/**
 * Test script for the deposit endpoint
 * 
 * Demonstrates how to make deposit requests to the indexer API
 * Useful for testing the integration without running the full Solana listener
 */

import { config } from '../lib/config.js';
import { logger } from '../lib/logger.js';

interface DepositRequest {
  leafCommit: string;
  encryptedOutput: string;
  txSignature?: string;
  slot?: number;
}

interface DepositResponse {
  success: boolean;
  leafIndex: number;
  root: string;
  nextIndex: number;
  leafCommit: string;
  message: string;
}

async function makeDepositRequest(deposit: DepositRequest): Promise<DepositResponse> {
  const url = `http://localhost:${config.server.port}/api/v1/deposit`;
  
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(deposit),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`);
    }

    const data = await response.json() as DepositResponse;
    return data;
  } catch (error) {
    logger.error('Failed to make deposit request', { error, deposit });
    throw error;
  }
}

async function getMerkleRoot(): Promise<{ root: string; nextIndex: number }> {
  const url = `http://localhost:${config.server.port}/api/v1/merkle/root`;
  
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    
    return await response.json() as { root: string; nextIndex: number };
  } catch (error) {
    logger.error('Failed to get merkle root', { error });
    throw error;
  }
}

async function getMerkleProof(index: number): Promise<{ pathElements: string[]; pathIndices: number[] }> {
  const url = `http://localhost:${config.server.port}/api/v1/merkle/proof/${index}`;
  
  try {
    const response = await fetch(url);
    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`);
    }
    
    return await response.json() as { pathElements: string[]; pathIndices: number[] };
  } catch (error) {
    logger.error('Failed to get merkle proof', { error, index });
    throw error;
  }
}

async function testDepositFlow(): Promise<void> {
  logger.info('Starting deposit flow test');
  
  try {
    // Generate some test data
    const testDeposits: DepositRequest[] = [
      {
        leafCommit: 'a'.repeat(64),
        encryptedOutput: Buffer.from('Test encrypted output 1').toString('base64'),
        txSignature: 'test_signature_1',
        slot: 1000
      },
      {
        leafCommit: 'b'.repeat(64),
        encryptedOutput: Buffer.from('Test encrypted output 2').toString('base64'),
        txSignature: 'test_signature_2',
        slot: 1001
      },
      {
        leafCommit: 'c'.repeat(64),
        encryptedOutput: Buffer.from('Test encrypted output 3').toString('base64'),
        txSignature: 'test_signature_3',
        slot: 1002
      }
    ];

    // Get initial state
    logger.info('Getting initial merkle root...');
    const initialState = await getMerkleRoot();
    logger.info('Initial state', initialState);

    // Make deposits
    const depositResults: DepositResponse[] = [];
    for (let i = 0; i < testDeposits.length; i++) {
      const deposit = testDeposits[i]!;
      logger.info(`Making deposit ${i + 1}/${testDeposits.length}`, {
        leafCommit: deposit.leafCommit.substring(0, 8) + '...',
        txSignature: deposit.txSignature
      });
      
      const result = await makeDepositRequest(deposit);
      depositResults.push(result);
      
      logger.info(`Deposit ${i + 1} successful`, {
        leafIndex: result.leafIndex,
        root: result.root.substring(0, 8) + '...',
        nextIndex: result.nextIndex
      });
    }

    // Get final state
    logger.info('Getting final merkle root...');
    const finalState = await getMerkleRoot();
    logger.info('Final state', finalState);

    // Test proof generation for each deposit
    logger.info('Testing proof generation...');
    for (let i = 0; i < depositResults.length; i++) {
      const result = depositResults[i]!;
      
      try {
        const proof = await getMerkleProof(result.leafIndex);
        logger.info(`Proof generated for leaf ${result.leafIndex}`, {
          pathElementsCount: proof.pathElements.length,
          pathIndicesCount: proof.pathIndices.length
        });
      } catch (error) {
        logger.error(`Failed to generate proof for leaf ${result.leafIndex}`, { error });
      }
    }

    logger.info('Deposit flow test completed successfully', {
      depositsProcessed: depositResults.length,
      initialNextIndex: initialState.nextIndex,
      finalNextIndex: finalState.nextIndex
    });

  } catch (error) {
    logger.error('Deposit flow test failed', { error });
    throw error;
  }
}

// Run the test if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  testDepositFlow()
    .then(() => {
      logger.info('Test completed successfully');
      process.exit(0);
    })
    .catch((error) => {
      logger.error('Test failed', { error });
      process.exit(1);
    });
}

export { testDepositFlow, makeDepositRequest, getMerkleRoot, getMerkleProof };
