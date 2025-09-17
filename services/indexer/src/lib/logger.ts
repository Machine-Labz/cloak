import winston from 'winston';
import { config } from './config.js';

const { combine, timestamp, printf, colorize, errors } = winston.format;

// Custom log format
const logFormat = printf(({ level, message, timestamp, stack, ...meta }) => {
  let log = `${timestamp} [${level}]: ${message}`;
  
  // Add stack trace for errors
  if (stack) {
    log += `\n${stack}`;
  }
  
  // Add metadata if present
  const metaKeys = Object.keys(meta);
  if (metaKeys.length > 0) {
    log += ` ${JSON.stringify(meta)}`;
  }
  
  return log;
});

// Create logger instance
export const logger = winston.createLogger({
  level: config.server.logLevel,
  format: combine(
    errors({ stack: true }),
    timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
    logFormat
  ),
  defaultMeta: { service: 'cloak-indexer' },
  transports: [
    new winston.transports.File({
      filename: 'logs/error.log',
      level: 'error',
      handleExceptions: true,
    }),
    new winston.transports.File({
      filename: 'logs/combined.log',
      handleExceptions: true,
    }),
  ],
});

// Add console transport for non-production environments
if (config.server.nodeEnv !== 'production') {
  logger.add(new winston.transports.Console({
    format: combine(
      colorize(),
      errors({ stack: true }),
      timestamp({ format: 'HH:mm:ss' }),
      printf(({ level, message, timestamp, stack, ...meta }) => {
        let log = `${timestamp} ${level}: ${message}`;
        
        if (stack) {
          log += `\n${stack}`;
        }
        
        const metaKeys = Object.keys(meta);
        if (metaKeys.length > 0) {
          log += ` ${JSON.stringify(meta, null, 2)}`;
        }
        
        return log;
      })
    ),
    handleExceptions: true,
  }));
}

// Handle unhandled exceptions and rejections
process.on('unhandledRejection', (reason, promise) => {
  logger.error('Unhandled Rejection at:', { promise, reason });
});

process.on('uncaughtException', (error) => {
  logger.error('Uncaught Exception:', error);
  process.exit(1);
});
