# ZK Layer Overview

**Goal:** Spend notes privately by proving:
1) Leaf inclusion for a commitment `C`
2) Correct nullifier `nf` (never seen before)
3) Conservation: `sum(outputs) + fee == amount`

**Stack**
- Prover: **SP1 zkVM** guest (Rust)
- Hash: **BLAKE3-256** (MVP)
- On-chain: Anchor program `shield-pool` + SP1 verifier CPI
- Indexer: append-only Merkle tree and note scans

**Key docs**
- Circuit: `circuit-withdraw.md`
- Encoding: `encoding.md`
- Merkle: `merkle.md`
- Prover (SP1): `prover-sp1.md`
- On-chain verifier: `onchain-verifier.md`
- API contracts: `api-contracts.md`
- Tests: `testing.md`
- Threat model: `threat-model.md`