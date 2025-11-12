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
 * Seeds must match the on-chain program:
 * - pool: b"pool"
 * - commitments: b"commitments"
 * - roots_ring: b"roots_ring"
 * - nullifier_shard: b"nullifier_shard"
 * - treasury: b"treasury"
 * 
 * @param programId - Optional program ID (defaults to CLOAK_PROGRAM_ID)
 */
export function getShieldPoolPDAs(programId?: PublicKey): ShieldPoolPDAs {
  const pid = programId || CLOAK_PROGRAM_ID;
  
  const [pool] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool")],
    pid
  );

  const [commitments] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitments")],
    pid
  );

  const [rootsRing] = PublicKey.findProgramAddressSync(
    [Buffer.from("roots_ring")],
    pid
  );

  const [nullifierShard] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_shard")],
    pid
  );

  const [treasury] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
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

