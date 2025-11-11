import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";

/**
 * Create a deposit instruction
 *
 * Deposits SOL into the Cloak protocol by creating a commitment.
 *
 * Instruction format:
 * - Byte 0: Discriminant (0x00 for deposit)
 * - Bytes 1-8: Amount (u64, little-endian)
 * - Bytes 9-40: Commitment (32 bytes)
 *
 * @param params - Deposit parameters
 * @returns Transaction instruction
 *
 * @example
 * ```typescript
 * const instruction = createDepositInstruction({
 *   programId: CLOAK_PROGRAM_ID,
 *   payer: wallet.publicKey,
 *   pool: POOL_ADDRESS,
 *   commitments: COMMITMENTS_ADDRESS,
 *   amount: 1_000_000_000, // 1 SOL
 *   commitment: commitmentBytes
 * });
 * ```
 */
export function createDepositInstruction(params: {
  programId: PublicKey;
  payer: PublicKey;
  pool: PublicKey;
  commitments: PublicKey;
  amount: number;
  commitment: Uint8Array;
}): TransactionInstruction {
  if (params.commitment.length !== 32) {
    throw new Error(
      `Invalid commitment length: ${params.commitment.length} (expected 32 bytes)`
    );
  }

  if (params.amount <= 0) {
    throw new Error("Amount must be positive");
  }

  // Deposit discriminant = 0x00
  const discriminant = new Uint8Array([0x00]);

  // Encode amount as little-endian u64
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(
    0,
    BigInt(params.amount),
    true // little-endian
  );

  // Concatenate: discriminant (1) + amount (8) + commitment (32) = 41 bytes
  const data = new Uint8Array(41);
  data.set(discriminant, 0);
  data.set(amountBytes, 1);
  data.set(params.commitment, 9);

  // Create instruction with accounts
  return new TransactionInstruction({
    programId: params.programId,
    keys: [
      // Account 0: Payer (signer, writable) - pays for transaction
      { pubkey: params.payer, isSigner: true, isWritable: true },
      // Account 1: Pool (writable) - receives SOL
      { pubkey: params.pool, isSigner: false, isWritable: true },
      // Account 2: System Program (readonly) - for transfers
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      // Account 3: Commitments (writable) - stores commitment
      { pubkey: params.commitments, isSigner: false, isWritable: true },
    ],
    data: Buffer.from(data),
  });
}

/**
 * Deposit instruction parameters for type safety
 */
export interface DepositInstructionParams {
  programId: PublicKey;
  payer: PublicKey;
  pool: PublicKey;
  commitments: PublicKey;
  amount: number;
  commitment: Uint8Array;
}

/**
 * Validate deposit instruction parameters
 *
 * @param params - Parameters to validate
 * @throws Error if invalid
 */
export function validateDepositParams(params: DepositInstructionParams): void {
  if (!(params.programId instanceof PublicKey)) {
    throw new Error("programId must be a PublicKey");
  }

  if (!(params.payer instanceof PublicKey)) {
    throw new Error("payer must be a PublicKey");
  }

  if (!(params.pool instanceof PublicKey)) {
    throw new Error("pool must be a PublicKey");
  }

  if (!(params.commitments instanceof PublicKey)) {
    throw new Error("commitments must be a PublicKey");
  }

  if (typeof params.amount !== "number" || params.amount <= 0) {
    throw new Error("amount must be a positive number");
  }

  if (!(params.commitment instanceof Uint8Array)) {
    throw new Error("commitment must be a Uint8Array");
  }

  if (params.commitment.length !== 32) {
    throw new Error(
      `commitment must be 32 bytes (got ${params.commitment.length})`
    );
  }
}
