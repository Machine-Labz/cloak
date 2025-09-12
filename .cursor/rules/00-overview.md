# Cloak – Cursor Rules: Overview

**Product:** Cloak (privacy-preserving exit router on Solana)  
**Focus in this ruleset:** Zero-Knowledge layer (notes, Merkle, SP1 circuit, on-chain verification, indexer/relay APIs).  
**Non-ZK** pieces exist under `docs/nonzk/*` and are referenced but not expanded here.

## High-level flow
- **Deposit (Top Up):** User transfers SOL into Pool and submits `encrypted_output` + `leaf_commit = C`. Indexer appends `C` to the Merkle tree and serves new `root`.
- **Withdraw:** Client locally scans notes, builds `publicInputs`, generates SP1 proof (Groth16), relay submits `shield-pool::withdraw`. Program verifies proof, checks root/nullifier, pays outputs, fees to Treasury.

## Key references in this repo
- `docs/zk/*` — source of truth for ZK design, encoding, Merkle, circuit, verifier, APIs, tests, threats.
- `programs/shield-pool/` — Anchor program (to be implemented).
- `packages/zk-guest-sp1/` — SP1 guest (to be implemented).
- `packages/zk-verifier-program/` — SP1 verifier CPI (to be implemented).
- `services/indexer/`, `services/relay/`, `apps/web/` — service stubs.
- `./.context/sp1-solana` — SP1 + Solana examples/fork for reference.

## What Cursor should optimize for
- Keep FE/guest/on-chain **byte encoding** identical (see `docs/zk/encoding.md`).
- Prefer clear module boundaries and tests first.
- Avoid adding Jito/bundling in MVP; keep it simple and reliable.

