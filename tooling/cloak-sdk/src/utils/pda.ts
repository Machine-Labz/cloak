/**
 * Program Derived Address (PDA) utilities for Shield Pool
 *
 * These functions derive deterministic addresses from the program ID and seeds.
 * This matches the behavior in tooling/test/src/shared.rs::get_pda_addresses()
 */

import { PublicKey } from "@solana/web3.js";
import { CLOAK_PROGRAM_ID } from "../core/CloakSDK";

export interface ShieldPoolPDAs {
  pool: PublicKey;
  commitments: PublicKey;
  rootsRing: PublicKey;
  nullifierShard: PublicKey;
  treasury: PublicKey;
}

/**
 * Derive all Shield Pool PDAs from the program ID
 *
 * Seeds must match the on-chain program and Rust helper:
 * - pool: b"pool", mint
 * - commitments: b"commitments", mint
 * - roots_ring: b"roots_ring", mint
 * - nullifier_shard: b"nullifier_shard", mint
 * - treasury: b"treasury", mint
 *
 * For native SOL, the Rust tests use `Pubkey::default()` (32 zero bytes) as the
 * mint. We mirror that here by accepting an optional mint and defaulting to a
 * zeroed public key to stay in sync with `tooling/test/src/shared.rs`.
 * 
 * @param programId - Optional program ID (defaults to CLOAK_PROGRAM_ID)
 * @param mint - Optional mint address (defaults to 32 zero bytes for native SOL)
 */
export function getShieldPoolPDAs(programId?: PublicKey, mint?: PublicKey): ShieldPoolPDAs {
  const pid = programId || CLOAK_PROGRAM_ID;
  const mintKey =
    mint ??
    new PublicKey(
      // 32 zero bytes, matching Rust `Pubkey::default()`
      new Uint8Array(32)
    );
  
  const [pool] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), mintKey.toBytes()],
    pid
  );

  const [commitments] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitments"), mintKey.toBytes()],
    pid
  );

  const [rootsRing] = PublicKey.findProgramAddressSync(
    [Buffer.from("roots_ring"), mintKey.toBytes()],
    pid
  );

  const [nullifierShard] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_shard"), mintKey.toBytes()],
    pid
  );

  const [treasury] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury"), mintKey.toBytes()],
    pid
  );

  return {
    pool,
    commitments,
    rootsRing,
    nullifierShard,
    treasury,
  };
}

