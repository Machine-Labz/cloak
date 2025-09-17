#!/usr/bin/env node

/**
 * Cloak Indexer Service
 * 
 * Main entry point for the indexer microservice.
 * Provides HTTP API for Merkle tree operations and encrypted note storage.
 */

import { startServer } from './api/server.js';
import { logger } from './lib/logger.js';
import { config } from './lib/config.js';

async function main() {
  try {
    logger.info('Starting Cloak Indexer Service', {
      nodeEnv: config.server.nodeEnv,
      port: config.server.port,
      treeHeight: config.merkle.treeHeight,
      solanaRpc: config.solana.rpcUrl,
      logLevel: config.server.logLevel
    });

    // Start the HTTP server
    await startServer();
    
  } catch (error) {
    logger.error('Failed to start Cloak Indexer Service', { error });
    process.exit(1);
  }
}

// Run the application
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}
