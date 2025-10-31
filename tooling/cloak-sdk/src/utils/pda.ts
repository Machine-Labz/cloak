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
 */
export function getShieldPoolPDAs(): ShieldPoolPDAs {
  const [pool] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool")],
    CLOAK_PROGRAM_ID
  );

  const [commitments] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitments")],
    CLOAK_PROGRAM_ID
  );

  const [rootsRing] = PublicKey.findProgramAddressSync(
    [Buffer.from("roots_ring")],
    CLOAK_PROGRAM_ID
  );

  const [nullifierShard] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_shard")],
    CLOAK_PROGRAM_ID
  );

  const [treasury] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    CLOAK_PROGRAM_ID
  );

  return {
    pool,
    commitments,
    rootsRing,
    nullifierShard,
    treasury,
  };
}

