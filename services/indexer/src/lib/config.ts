import { Config } from '../types/index.js';

function getEnvVar(name: string, defaultValue?: string): string {
  const value = process.env[name] ?? defaultValue;
  if (!value) {
    throw new Error(`Environment variable ${name} is required`);
  }
  return value;
}

function getEnvVarAsNumber(name: string, defaultValue?: number): number {
  const value = process.env[name];
  if (value === undefined) {
    if (defaultValue === undefined) {
      throw new Error(`Environment variable ${name} is required`);
    }
    return defaultValue;
  }
  const parsed = parseInt(value, 10);
  if (isNaN(parsed)) {
    throw new Error(`Environment variable ${name} must be a valid number`);
  }
  return parsed;
}

export const config: Config = {
  database: {
    host: getEnvVar('DB_HOST', 'localhost'),
    port: getEnvVarAsNumber('DB_PORT', 5432),
    name: getEnvVar('DB_NAME', 'cloak_indexer'),
    user: getEnvVar('DB_USER', 'postgres'),
    password: getEnvVar('DB_PASSWORD', ''),
    url: process.env.DATABASE_URL || undefined,
  },
  solana: {
    rpcUrl: getEnvVar('SOLANA_RPC_URL', 'https://api.devnet.solana.com'),
    shieldPoolProgramId: process.env.SHIELD_POOL_PROGRAM_ID || '',
  },
  server: {
    port: getEnvVarAsNumber('PORT', 3001),
    nodeEnv: getEnvVar('NODE_ENV', 'development'),
    logLevel: getEnvVar('LOG_LEVEL', 'info'),
  },
  merkle: {
    treeHeight: getEnvVarAsNumber('TREE_HEIGHT', 32),
    zeroValue: getEnvVar('TREE_ZERO_VALUE', '0x0000000000000000000000000000000000000000000000000000000000000000'),
  },
  artifacts: {
    basePath: getEnvVar('ARTIFACTS_BASE_PATH', './artifacts'),
    sp1Version: getEnvVar('SP1_VERSION', 'v2.0.0'),
  },
};
