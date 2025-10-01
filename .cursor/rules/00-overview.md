# Cloak – Cursor Rules: Overview

**Product:** Cloak (privacy-preserving exit router on Solana)  
**Status:** 🎉 **PRODUCTION READY** - Complete end-to-end flow working  
**Focus in this ruleset:** Zero-Knowledge layer (notes, Merkle, SP1 circuit, on-chain verification, indexer/relay APIs).  
**Non-ZK** pieces exist under `docs/nonzk/*` and are referenced but not expanded here.

## High-level flow ✅ COMPLETE
- **Deposit (Top Up):** User transfers SOL into Pool and submits `encrypted_output` + `leaf_commit = C`. Indexer appends `C` to the Merkle tree and serves new `root`.
- **Withdraw:** Client locally scans notes, builds `publicInputs`, generates SP1 proof (Groth16), submits `shield-pool::withdraw`. Program verifies proof, checks root/nullifier, pays outputs, fees to Treasury.

## Key references in this repo
- `docs/COMPLETE_FLOW_STATUS.md` — **Current production status and capabilities**
- `docs/zk/*` — source of truth for ZK design, encoding, Merkle, circuit, verifier, APIs, tests, threats.
- `programs/shield-pool/` — **Pinocchio program (IMPLEMENTED & WORKING)**.
- `packages/zk-guest-sp1/` — **SP1 guest (IMPLEMENTED & WORKING)**.
- `services/indexer/` — **Indexer service (IMPLEMENTED & WORKING)**.
- `test_complete_flow_rust/` — **End-to-end test suite (WORKING)**.

## Current Working State
- ✅ **Solana Program:** `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp` deployed and functional
- ✅ **SP1 Guest Program:** Zero-knowledge proof generation working
- ✅ **Indexer Service:** Merkle tree management with PostgreSQL
- ✅ **Complete Flow:** Real SOL deposits and withdrawals with ZK proofs
- ✅ **Real Addresses:** Withdrawals to actual Solana addresses
- ✅ **BLAKE3-256:** Consistent hashing across all components

## What Cursor should optimize for
- Keep FE/guest/on-chain **byte encoding** identical (see `docs/zk/encoding.md`).
- Prefer clear module boundaries and tests first.
- Avoid adding Jito/bundling in MVP; keep it simple and reliable.
- **Maintain working state** - all core functionality is complete and tested.

