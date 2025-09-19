// Core types for the indexer service

export interface MerkleProof {
  pathElements: string[];
  pathIndices: number[];
}

export interface MerkleTreeState {
  root: string;
  nextIndex: number;
}

export interface DepositEvent {
  leafCommit: string;
  encryptedOutput: string;
  txSignature: string;
  slot: number;
  timestamp: number;
}

export interface StoredNote {
  id: number;
  leafCommit: string;
  encryptedOutput: string;
  leafIndex: number;
  txSignature: string;
  slot: number;
  timestamp: number;
  createdAt: Date;
}

export interface NotesRangeResponse {
  encryptedOutputs: string[];
  hasMore: boolean;
  total: number;
  start: number;
  end: number;
}

export interface WithdrawArtifacts {
  guestElfUrl: string;
  vk: string;
  sha256: {
    elf: string;
    vk: string;
  };
  sp1Version: string;
}

export interface Config {
  database: {
    host: string;
    port: number;
    name: string;
    user: string;
    password: string;
    url?: string | undefined;
  };
  solana: {
    rpcUrl: string;
    shieldPoolProgramId: string;
  };
  server: {
    port: number;
    nodeEnv: string;
    logLevel: string;
  };
  merkle: {
    treeHeight: number;
    zeroValue: string;
  };
  artifacts: {
    basePath: string;
    sp1Version: string;
  };
}

export interface TreeNode {
  level: number;
  index: number;
  value: string;
}

export interface MerkleTreeRow {
  level: number;
  index: number;
  value: string;
  createdAt: Date;
  updatedAt: Date;
}
