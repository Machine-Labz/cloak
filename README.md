# Cloak

Cloak is a **privacy-preserving exit router** on Solana. This repo hosts the whole system, with a strong focus on the **Zero-Knowledge layer**:

- **ZK Notes (UTXO-style):** commitments, nullifiers, Merkle proofs
- **Withdraw circuit (SP1):** inclusion + nullifier + conservation
- **On-chain verification:** Pinocchio program `shield-pool`
- **Indexer API:** append-only Merkle tree & proofs
- **Relay:** submits withdraw txs (no Jito in MVP)

## Quick links

- ZK overview: `docs/docs/zk/README.md`
- Circuit spec: `docs/docs/zk/circuit-withdraw.md`
- Encoding contract: `docs/docs/zk/encoding.md`
- Merkle tree & proofs: `docs/docs/zk/merkle.md`
- SP1 prover details: `docs/docs/zk/prover-sp1.md`
- On-chain verifier & program: `docs/docs/zk/onchain-verifier.md`
- API contracts (Indexer/Relay): `docs/docs/zk/api-contracts.md`
- Threat model: `docs/docs/zk/threat-model.md`
- Roadmap: `docs/docs/roadmap.md`

> Build order (MVP): Merkle+Indexer → Deposit tx/event → SP1 withdraw circuit → On-chain verifier → Relay → Web wiring.
