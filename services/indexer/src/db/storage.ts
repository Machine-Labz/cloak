import { TreeStorageInterface } from '../lib/merkle.js';
import { StoredNote, NotesRangeResponse, MerkleTreeRow } from '../types/index.js';
import { db } from './connection.js';
import { logger } from '../lib/logger.js';

/**
 * PostgreSQL implementation of TreeStorageInterface
 * Provides storage operations for Merkle tree nodes and notes
 */
export class PostgresTreeStorage implements TreeStorageInterface {
  
  /**
   * Store a tree node at the specified level and index
   */
  async storeNode(level: number, index: number, value: string): Promise<void> {
    const cleanValue = value.replace('0x', '').toLowerCase();
    
    if (cleanValue.length !== 64) {
      throw new Error(`Invalid node value length: ${cleanValue.length} (expected 64 hex chars)`);
    }

    try {
      await db.query(
        `INSERT INTO merkle_tree_nodes (level, index_at_level, value) 
         VALUES ($1, $2, $3)
         ON CONFLICT (level, index_at_level) 
         DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()`,
        [level, index, cleanValue]
      );

      logger.debug('Stored tree node', { level, index, value: cleanValue });
    } catch (error) {
      logger.error('Failed to store tree node', { level, index, value: cleanValue, error });
      throw error;
    }
  }

  /**
   * Retrieve a tree node by level and index
   */
  async getNode(level: number, index: number): Promise<string | null> {
    try {
      const result = await db.query<{ value: string }>(
        'SELECT value FROM merkle_tree_nodes WHERE level = $1 AND index_at_level = $2',
        [level, index]
      );

      if (result.rows.length === 0) {
        return null;
      }

      const value = result.rows[0]!.value;
      logger.debug('Retrieved tree node', { level, index, value });
      return value;
    } catch (error) {
      logger.error('Failed to retrieve tree node', { level, index, error });
      throw error;
    }
  }

  /**
   * Get the maximum leaf index (used to initialize nextIndex on startup)
   * Returns the next consecutive index, not just max + 1
   */
  async getMaxLeafIndex(): Promise<number> {
    try {
      // Get all leaf indices in order
      const result = await db.query<{ index_at_level: number }>(
        'SELECT index_at_level FROM merkle_tree_nodes WHERE level = 0 ORDER BY index_at_level'
      );

      const indices = result.rows.map(row => row.index_at_level);
      
      if (indices.length === 0) {
        logger.info('No leaves found, starting from index 0');
        return 0;
      }

      // Find the first gap or return the next consecutive index
      let nextIndex = indices.length; // Start with the length (next available index)
      for (let i = 0; i < indices.length; i++) {
        if (indices[i] !== i) {
          nextIndex = i; // Found a gap, use that index
          break;
        }
      }

      logger.info('Retrieved max leaf index', { 
        totalLeaves: indices.length, 
        maxIndex: Math.max(...indices), 
        nextIndex,
        indices: indices.slice(0, 10) // Show first 10 indices for debugging
      });
      
      return nextIndex;
    } catch (error) {
      logger.error('Failed to get max leaf index', { error });
      throw error;
    }
  }

  /**
   * Store a note (deposit event) with encrypted output
   */
  async storeNote(
    leafCommit: string,
    encryptedOutput: string,
    leafIndex: number,
    txSignature: string,
    slot: number,
    blockTime?: Date
  ): Promise<void> {
    const cleanCommit = leafCommit.replace('0x', '').toLowerCase();
    
    try {
      await db.query(
        `INSERT INTO notes (leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time) 
         VALUES ($1, $2, $3, $4, $5, $6)`,
        [cleanCommit, encryptedOutput, leafIndex, txSignature, slot, blockTime || new Date()]
      );

      logger.info('Stored note', { 
        leafCommit: cleanCommit, 
        leafIndex, 
        txSignature, 
        slot,
        encryptedOutputLength: encryptedOutput.length
      });
    } catch (error) {
      logger.error('Failed to store note', { 
        leafCommit: cleanCommit, 
        leafIndex, 
        txSignature, 
        slot, 
        error 
      });
      throw error;
    }
  }

  /**
   * Get notes in a range with pagination
   */
  async getNotesRange(start: number, end: number, limit: number = 100): Promise<NotesRangeResponse> {
    try {
      // Validate range
      if (start < 0 || end < start) {
        throw new Error('Invalid range: start must be >= 0 and end must be >= start');
      }

      if (limit > 1000) {
        limit = 1000; // Cap limit to prevent excessive memory usage
      }

      // Get total count in range
      const countResult = await db.query<{ count: string }>(
        'SELECT COUNT(*) as count FROM notes WHERE leaf_index >= $1 AND leaf_index <= $2',
        [start, end]
      );
      const total = parseInt(countResult.rows[0]!.count);

      // Get the actual notes
      const notesResult = await db.query<StoredNote>(
        `SELECT id, leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time as timestamp, created_at 
         FROM notes 
         WHERE leaf_index >= $1 AND leaf_index <= $2
         ORDER BY leaf_index ASC 
         LIMIT $3`,
        [start, end, limit]
      );

      const notes = notesResult.rows;
      const encryptedOutputs = notes.map(note => note.encryptedOutput);
      const hasMore = total > limit;

      logger.debug('Retrieved notes range', { 
        start, 
        end, 
        limit, 
        total, 
        returned: notes.length, 
        hasMore 
      });

      return {
        encryptedOutputs,
        hasMore,
        total,
        start,
        end
      };
    } catch (error) {
      logger.error('Failed to get notes range', { start, end, limit, error });
      throw error;
    }
  }

  /**
   * Get a specific note by leaf index
   */
  async getNoteByIndex(leafIndex: number): Promise<StoredNote | null> {
    try {
      const result = await db.query<StoredNote>(
        `SELECT id, leaf_commit, encrypted_output, leaf_index, tx_signature, slot, block_time as timestamp, created_at 
         FROM notes 
         WHERE leaf_index = $1`,
        [leafIndex]
      );

      if (result.rows.length === 0) {
        return null;
      }

      return result.rows[0]!;
    } catch (error) {
      logger.error('Failed to get note by index', { leafIndex, error });
      throw error;
    }
  }

  /**
   * Update indexer metadata
   */
  async updateMetadata(key: string, value: string): Promise<void> {
    try {
      await db.query(
        `INSERT INTO indexer_metadata (key, value) 
         VALUES ($1, $2)
         ON CONFLICT (key) 
         DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()`,
        [key, value]
      );

      logger.debug('Updated metadata', { key, value });
    } catch (error) {
      logger.error('Failed to update metadata', { key, value, error });
      throw error;
    }
  }

  /**
   * Get indexer metadata value
   */
  async getMetadata(key: string): Promise<string | null> {
    try {
      const result = await db.query<{ value: string }>(
        'SELECT value FROM indexer_metadata WHERE key = $1',
        [key]
      );

      if (result.rows.length === 0) {
        return null;
      }

      return result.rows[0]!.value;
    } catch (error) {
      logger.error('Failed to get metadata', { key, error });
      throw error;
    }
  }

  /**
   * Log event processing
   */
  async logEventProcessing(
    txSignature: string,
    slot: number,
    eventType: string,
    status: 'success' | 'failed' | 'skipped' = 'success',
    errorMessage?: string
  ): Promise<void> {
    try {
      await db.query(
        `INSERT INTO event_processing_log (tx_signature, slot, event_type, processing_status, error_message) 
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (tx_signature, event_type) 
         DO UPDATE SET 
           processing_status = EXCLUDED.processing_status,
           error_message = EXCLUDED.error_message,
           processed_at = NOW()`,
        [txSignature, slot, eventType, status, errorMessage || null]
      );

      logger.debug('Logged event processing', { txSignature, slot, eventType, status });
    } catch (error) {
      logger.error('Failed to log event processing', { txSignature, slot, eventType, status, error });
      throw error;
    }
  }

  /**
   * Get all tree nodes for a specific level (useful for debugging/verification)
   */
  async getNodesAtLevel(level: number): Promise<MerkleTreeRow[]> {
    try {
      const result = await db.query<MerkleTreeRow>(
        `SELECT level, index_at_level as index, value, created_at, updated_at 
         FROM merkle_tree_nodes 
         WHERE level = $1 
         ORDER BY index_at_level`,
        [level]
      );

      return result.rows;
    } catch (error) {
      logger.error('Failed to get nodes at level', { level, error });
      throw error;
    }
  }

  /**
   * Health check - verify database connectivity and basic operations
   */
  async healthCheck(): Promise<{ healthy: boolean; details: Record<string, any> }> {
    try {
      // Test basic connectivity
      const timeResult = await db.query('SELECT NOW() as current_time');
      
      // Test table accessibility
      const tableResult = await db.query(`
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name IN ('merkle_tree_nodes', 'notes', 'indexer_metadata')
        ORDER BY table_name
      `);

      // Get some basic stats
      const statsResult = await db.query(`
        SELECT 
          (SELECT COUNT(*) FROM merkle_tree_nodes) as tree_nodes,
          (SELECT COUNT(*) FROM notes) as notes_count,
          (SELECT value FROM indexer_metadata WHERE key = 'next_leaf_index') as next_index
      `);

      return {
        healthy: true,
        details: {
          currentTime: timeResult.rows[0]?.current_time,
          tablesFound: tableResult.rows.map(r => r.table_name),
          stats: statsResult.rows[0] || {},
          poolStats: db.getPoolStats()
        }
      };
    } catch (error) {
      logger.error('Storage health check failed', { error });
      return {
        healthy: false,
        details: {
          error: error instanceof Error ? error.message : String(error)
        }
      };
    }
  }
}

// Export singleton instance
export const treeStorage = new PostgresTreeStorage();
