import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { createRouter } from './routes.js';
import { logger } from '../lib/logger.js';
import { config } from '../lib/config.js';
import { db } from '../db/connection.js';

/**
 * Express server setup for the Cloak Indexer API
 */
export function createServer(): express.Application {
  const app = express();

  // Security middleware
  app.use(helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        styleSrc: ["'self'", "'unsafe-inline'"],
        scriptSrc: ["'self'"],
        imgSrc: ["'self'", "data:", "https:"],
      },
    },
    hsts: config.server.nodeEnv === 'production',
  }));

  // CORS configuration
  app.use(cors({
    origin: config.server.nodeEnv === 'production' 
      ? ['https://cloak.network', 'https://app.cloak.network'] // Update with actual domains
      : true, // Allow all origins in development
    credentials: true,
    methods: ['GET', 'POST', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization'],
    maxAge: 86400, // 24 hours
  }));

  // Request parsing middleware
  app.use(express.json({ 
    limit: '1mb',
    type: 'application/json'
  }));
  app.use(express.urlencoded({ 
    extended: true, 
    limit: '1mb' 
  }));

  // Request logging middleware
  app.use((req, res, next) => {
    const start = Date.now();
    
    res.on('finish', () => {
      const duration = Date.now() - start;
      logger.info('Request processed', {
        method: req.method,
        url: req.url,
        statusCode: res.statusCode,
        duration: `${duration}ms`,
        userAgent: req.get('User-Agent'),
        ip: req.ip || req.connection.remoteAddress
      });
    });
    
    next();
  });

  // Health endpoint (outside API versioning)
  app.get('/health', async (req, res) => {
    try {
      const { treeStorage } = await import('../db/storage.js');
      const dbHealth = await treeStorage.healthCheck();
      const isHealthy = dbHealth.healthy;

      res.status(isHealthy ? 200 : 503).json({
        status: isHealthy ? 'healthy' : 'unhealthy',
        timestamp: new Date().toISOString(),
        database: dbHealth,
        merkleTree: {
          initialized: true,
          height: config.server.nodeEnv === 'test' ? 32 : config.merkle?.treeHeight || 32
        },
        version: process.env.npm_package_version || 'unknown'
      });
    } catch (error) {
      res.status(503).json({
        status: 'unhealthy',
        error: error instanceof Error ? error.message : 'Unknown error',
        timestamp: new Date().toISOString()
      });
    }
  });

  // API routes
  app.use('/api/v1', createRouter());
  
  // Root endpoint - API info
  app.get('/', (req, res) => {
    res.json({
      name: 'Cloak Indexer API',
      version: process.env.npm_package_version || 'unknown',
      description: 'Merkle tree indexer for Cloak privacy protocol',
      endpoints: {
        health: '/health',
        deposit: '/api/v1/deposit',
        merkleRoot: '/api/v1/merkle/root',
        merkleProof: '/api/v1/merkle/proof/:index',
        notesRange: '/api/v1/notes/range?start=<n>&end=<n>&limit=<n>',
        artifacts: '/api/v1/artifacts/withdraw/:version',
        artifactFiles: '/api/v1/artifacts/files/:version/:filename'
      },
      documentation: 'https://docs.cloak.network/indexer',
      timestamp: new Date().toISOString()
    });
  });

  // 404 handler
  app.use('*', (req, res) => {
    res.status(404).json({
      error: 'Not found',
      message: `Route ${req.method} ${req.originalUrl} does not exist`,
      availableEndpoints: [
        'GET /',
        'GET /health',
        'POST /api/v1/deposit',
        'GET /api/v1/merkle/root',
        'GET /api/v1/merkle/proof/:index',
        'GET /api/v1/notes/range',
        'GET /api/v1/artifacts/withdraw/:version',
        'GET /api/v1/artifacts/files/:version/:filename'
      ]
    });
  });

  // Global error handler
  app.use((error: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
    logger.error('Unhandled server error', {
      method: req.method,
      url: req.url,
      error: error.message,
      stack: error.stack
    });

    res.status(500).json({
      error: 'Internal server error',
      message: config.server.nodeEnv === 'development' ? error.message : 'Something went wrong',
      timestamp: new Date().toISOString()
    });
  });

  return app;
}

/**
 * Start the server and handle graceful shutdown
 */
export async function startServer(): Promise<void> {
  const app = createServer();
  
  // Test database connection on startup
  logger.info('Testing database connection...');
  const isConnected = await db.testConnection();
  if (!isConnected) {
    logger.error('Failed to connect to database. Exiting...');
    process.exit(1);
  }

  // Start HTTP server
  const server = app.listen(config.server.port, () => {
    logger.info('Cloak Indexer API started', {
      port: config.server.port,
      nodeEnv: config.server.nodeEnv,
      processId: process.pid,
      nodeVersion: process.version,
      memoryUsage: process.memoryUsage()
    });
  });

  // Graceful shutdown handler
  const gracefulShutdown = async (signal: string) => {
    logger.info(`Received ${signal}. Starting graceful shutdown...`);
    
    // Stop accepting new connections
    server.close(async () => {
      logger.info('HTTP server closed');
      
      try {
        // Close database connections
        await db.close();
        logger.info('Database connections closed');
        
        logger.info('Graceful shutdown completed');
        process.exit(0);
      } catch (error) {
        logger.error('Error during shutdown:', error);
        process.exit(1);
      }
    });

    // Force shutdown after timeout
    setTimeout(() => {
      logger.error('Forced shutdown due to timeout');
      process.exit(1);
    }, 10000);
  };

  // Handle shutdown signals
  process.on('SIGTERM', () => gracefulShutdown('SIGTERM'));
  process.on('SIGINT', () => gracefulShutdown('SIGINT'));
  
  // Handle uncaught exceptions
  process.on('uncaughtException', (error) => {
    logger.error('Uncaught exception:', error);
    process.exit(1);
  });

  process.on('unhandledRejection', (reason, promise) => {
    logger.error('Unhandled promise rejection:', { reason, promise });
    process.exit(1);
  });
}
