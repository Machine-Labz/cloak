/**
 * Jest setup file for global test configuration
 */

// Increase timeout for database operations
jest.setTimeout(30000);

// Mock environment variables for testing
process.env.NODE_ENV = 'test';
process.env.LOG_LEVEL = 'silent'; // Reduce log noise during tests
process.env.DB_HOST = process.env.DB_HOST || 'localhost';
process.env.DB_PORT = process.env.DB_PORT || '5432';
process.env.DB_NAME = process.env.DB_NAME || 'cloak_indexer_test';
process.env.DB_USER = process.env.DB_USER || 'cloak';
process.env.DB_PASSWORD = process.env.DB_PASSWORD || 'test_password';
process.env.PORT = '3001';
process.env.TREE_HEIGHT = '32';
