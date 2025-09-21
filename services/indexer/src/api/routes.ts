import { Router, Request, Response, NextFunction } from 'express';
import { z } from 'zod';
import { MerkleTree } from '../lib/merkle.js';
import { treeStorage } from '../db/storage.js';
import { logger } from '../lib/logger.js';
import { config } from '../lib/config.js';
import { artifactManager } from '../lib/artifacts.js';

// Initialize Merkle tree
const merkleTree = new MerkleTree(config.merkle.treeHeight, config.merkle.zeroValue);

// Validation schemas
const indexParamSchema = z.object({
  index: z.string().transform((val) => {
    const num = parseInt(val, 10);
    if (isNaN(num) || num < 0) {
      throw new Error('Index must be a non-negative integer');
    }
    return num;
  })
});

const rangeQuerySchema = z.object({
  start: z.string().transform((val) => {
    const num = parseInt(val, 10);
    if (isNaN(num) || num < 0) {
      throw new Error('Start must be a non-negative integer');
    }
    return num;
  }),
  end: z.string().transform((val) => {
    const num = parseInt(val, 10);
    if (isNaN(num) || num < 0) {
      throw new Error('End must be a non-negative integer');
    }
    return num;
  }),
  limit: z.string().optional().transform((val) => {
    if (!val) return 100;
    const num = parseInt(val, 10);
    if (isNaN(num) || num < 1 || num > 1000) {
      throw new Error('Limit must be between 1 and 1000');
    }
    return num;
  })
});

const versionParamSchema = z.object({
  version: z.string().regex(/^v\d+\.\d+\.\d+$/, 'Version must be in format vX.Y.Z')
});

// Error handler middleware
const asyncHandler = (fn: (req: Request, res: Response, next: NextFunction) => Promise<any>) => {
  return (req: Request, res: Response, next: NextFunction) => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
};

// Initialize tree on startup
let treeInitialized = false;

async function initializeMerkleTree(): Promise<void> {
  if (treeInitialized) return;

  try {
    const nextIndex = await treeStorage.getMaxLeafIndex();
    merkleTree.setNextIndex(nextIndex);
    treeInitialized = true;
    logger.info('Merkle tree initialized', { nextIndex });
  } catch (error) {
    logger.error('Failed to initialize Merkle tree', { error });
    throw error;
  }
}

export function createRouter(): Router {
  const router = Router();

  // Note: Health endpoint is defined in server.ts outside the router

  // GET /merkle/root - Get current Merkle tree root and next index
  router.get('/merkle/root', asyncHandler(async (req: Request, res: Response) => {
    await initializeMerkleTree();
    
    const treeState = await merkleTree.getTreeState(treeStorage);
    
    logger.debug('Merkle root requested', treeState);
    
    res.json({
      root: treeState.root,
      nextIndex: treeState.nextIndex
    });
  }));

  // GET /merkle/proof/:index - Get Merkle proof for a specific leaf index
  router.get('/merkle/proof/:index', asyncHandler(async (req: Request, res: Response) => {
    await initializeMerkleTree();
    
    const { index } = indexParamSchema.parse(req.params);
    
    logger.debug('Merkle proof requested', { index });
    
    const proof = await merkleTree.generateProof(index, treeStorage);
    
    res.json({
      pathElements: proof.pathElements,
      pathIndices: proof.pathIndices
    });
  }));

  // GET /notes/range?start=<n>&end=<n>&limit=<n> - Get encrypted outputs in range
  router.get('/notes/range', asyncHandler(async (req: Request, res: Response) => {
    const { start, end, limit } = rangeQuerySchema.parse(req.query);
    
    if (end < start) {
      res.status(400).json({
        error: 'End index must be greater than or equal to start index'
      });
      return;
    }
    
    logger.debug('Notes range requested', { start, end, limit });
    
    const result = await treeStorage.getNotesRange(start, end, limit);
    
    res.json(result);
    return;
  }));

  // GET /artifacts/withdraw/:version - Get SP1 guest ELF and verification key artifacts
  router.get('/artifacts/withdraw/:version', asyncHandler(async (req: Request, res: Response) => {
    const { version } = versionParamSchema.parse(req.params);
    
    logger.info('Withdraw artifacts requested', { version });
    
    try {
      // In development, create placeholder artifacts if they don't exist
      if (config.server.nodeEnv === 'development') {
        await artifactManager.createPlaceholderArtifacts(version);
      }

      const artifacts = await artifactManager.getWithdrawArtifacts(version);
      
      res.json(artifacts);
    } catch (error) {
      logger.error('Failed to get withdraw artifacts', { version, error });
      
      if (error instanceof Error && error.message.includes('not found')) {
        res.status(404).json({
          error: 'Artifacts not found',
          message: `No artifacts available for version ${version}`,
          availableVersions: await artifactManager.listAvailableVersions()
        });
      } else {
        res.status(500).json({
          error: 'Artifact retrieval failed',
          message: error instanceof Error ? error.message : 'Unknown error'
        });
      }
    }
  }));

  // GET /artifacts/files/:version/:filename - Serve individual artifact files
  router.get('/artifacts/files/:version/:filename', asyncHandler(async (req: Request, res: Response) => {
    const { version } = versionParamSchema.parse(req.params);
    const { filename } = z.object({
      filename: z.string().regex(/^[a-zA-Z0-9._-]+$/, 'Invalid filename')
    }).parse(req.params);

    logger.info('Artifact file requested', { version, filename });

    try {
      const fileData = await artifactManager.serveArtifactFile(version, filename);
      
      // Set appropriate headers
      res.set({
        'Content-Type': fileData.contentType,
        'Content-Length': fileData.size.toString(),
        'Cache-Control': 'public, max-age=3600', // Cache for 1 hour
        'ETag': `"${fileData.sha256}"`,
        'X-SHA256': fileData.sha256,
      });

      // Check if client already has the file (ETag matching)
      if (req.get('If-None-Match') === `"${fileData.sha256}"`) {
        res.status(304).end();
        return;
      }

      res.send(fileData.data);
      return;
    } catch (error) {
      logger.error('Failed to serve artifact file', { version, filename, error });
      
      if (error instanceof Error && error.message.includes('not found')) {
        res.status(404).json({
          error: 'File not found',
          message: `Artifact file not found: ${filename} for version ${version}`
        });
      } else {
        res.status(500).json({
          error: 'File serving failed',
          message: error instanceof Error ? error.message : 'Unknown error'
        });
      }
    }
  }));

  // POST /deposit - Main deposit ingestion endpoint
  router.post('/deposit', asyncHandler(async (req: Request, res: Response) => {
    await initializeMerkleTree();
    
    const depositSchema = z.object({
      leafCommit: z.string().regex(/^[0-9a-fA-F]{64}$/, 'Leaf commit must be a 64-character hex string'),
      encryptedOutput: z.string().min(1, 'Encrypted output cannot be empty'),
      txSignature: z.string().optional(), // Optional for manual testing
      slot: z.number().int().nonnegative().optional(), // Optional for manual testing
    });

    const { leafCommit, encryptedOutput, txSignature, slot } = depositSchema.parse(req.body);
    
    logger.info('Deposit received', { 
      leafCommit, 
      encryptedOutputLength: encryptedOutput.length,
      txSignature,
      slot
    });
    
    // Insert leaf into tree and get new root
    const { root, leafIndex } = await merkleTree.insertLeaf(leafCommit, treeStorage);
    
    // Store note data with metadata
    await treeStorage.storeNote(
      leafCommit,
      encryptedOutput,
      leafIndex,
      txSignature || `deposit_${Date.now()}`, // fallback tx signature for testing
      slot || 0, // fallback slot for testing
      new Date()
    );

    // Update metadata with new tree state
    await treeStorage.updateMetadata('next_leaf_index', (leafIndex + 1).toString());
    
    logger.info('Deposit processed successfully', { 
      leafIndex, 
      root, 
      leafCommit,
      nextIndex: leafIndex + 1
    });

    res.status(201).json({
      success: true,
      leafIndex,
      root,
      nextIndex: leafIndex + 1,
      leafCommit: leafCommit.toLowerCase(),
      message: 'Deposit processed successfully'
    });
  }));

  // POST /admin/push-root - Admin endpoint to push new root (for testing)
  router.post('/admin/push-root', asyncHandler(async (req: Request, res: Response) => {
    // TODO: Add authentication middleware
    const { root } = z.object({
      root: z.string().regex(/^[0-9a-fA-F]{64}$/, 'Root must be a 64-character hex string')
    }).parse(req.body);
    
    logger.info('Admin root push requested', { root });
    
    // Store the root at max level, index 0 (there's only one root)
    await treeStorage.storeNode(config.merkle.treeHeight - 1, 0, root);
    
    res.json({
      success: true,
      root: root.toLowerCase(),
      message: 'Root pushed successfully'
    });
  }));

  // POST /admin/insert-leaf - Admin endpoint to insert leaf (for testing)
  router.post('/admin/insert-leaf', asyncHandler(async (req: Request, res: Response) => {
    await initializeMerkleTree();
    
    // TODO: Add authentication middleware
    const { leafCommit, encryptedOutput } = z.object({
      leafCommit: z.string().regex(/^[0-9a-fA-F]{64}$/, 'Leaf commit must be a 64-character hex string'),
      encryptedOutput: z.string().min(1, 'Encrypted output cannot be empty')
    }).parse(req.body);
    
    logger.info('Admin leaf insertion requested', { 
      leafCommit, 
      encryptedOutputLength: encryptedOutput.length 
    });
    
    // Insert leaf into tree
    const { root, leafIndex } = await merkleTree.insertLeaf(leafCommit, treeStorage);
    
    // Store note data
    await treeStorage.storeNote(
      leafCommit,
      encryptedOutput,
      leafIndex,
      'admin_insert', // placeholder tx signature
      0, // placeholder slot
      new Date()
    );
    
    res.json({
      success: true,
      leafIndex,
      root,
      leafCommit: leafCommit.toLowerCase(),
      message: 'Leaf inserted successfully'
    });
  }));

  // Error handling middleware
  router.use((error: any, req: Request, res: Response, next: NextFunction) => {
    logger.error('API error', { 
      method: req.method,
      url: req.url,
      error: error.message,
      stack: error.stack
    });

    // Handle Zod validation errors
    if (error instanceof z.ZodError) {
      return res.status(400).json({
        error: 'Validation error',
        details: error.errors.map(e => ({
          field: e.path.join('.'),
          message: e.message
        }))
      });
    }

    // Handle custom application errors
    if (error.message) {
      return res.status(400).json({
        error: error.message
      });
    }

    // Generic server error
    res.status(500).json({
      error: 'Internal server error',
      message: config.server.nodeEnv === 'development' ? error.message : 'Something went wrong'
    });
    return;
  });

  return router;
}
