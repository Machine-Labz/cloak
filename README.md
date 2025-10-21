# Cloak

Cloak is a **privacy-preserving exit router** on Solana. This repo hosts the whole system, with a strong focus on the **Zero-Knowledge layer**:

- **ZK Notes (UTXO-style):** commitments, nullifiers, Merkle proofs
- **Withdraw circuit (SP1):** inclusion + nullifier + conservation
- **On-chain verification:** Pinocchio program `shield-pool`
- **Indexer API:** append-only Merkle tree & proofs
- **Relay:** submits withdraw txs (no Jito in MVP)

## 🚀 Getting Started

**New to Cloak?** Start here:
- 📖 **[SETUP_COMPLETE.md](./SETUP_COMPLETE.md)** - Setup summary & current status
- ⚡ **[QUICK_START.md](./QUICK_START.md)** - 5-minute quick start guide
- 📋 **[SETUP.md](./SETUP.md)** - Comprehensive setup instructions
- 🔧 **[ENV_SETUP.md](./ENV_SETUP.md)** - Environment variables guide
- 🐳 **[DOCKER_SETUP.md](./DOCKER_SETUP.md)** - Docker deployment guide

## Quick links

- ZK overview: `docs/zk/README.md`
- Circuit spec: `docs/zk/circuit-withdraw.md`
- Encoding contract: `docs/zk/encoding.md`
- Merkle tree & proofs: `docs/zk/merkle.md`
- SP1 prover details: `docs/zk/prover-sp1.md`
- On-chain verifier & program: `docs/zk/onchain-verifier.md`
- API contracts (Indexer/Relay): `docs/zk/api-contracts.md`
- Threat model: `docs/zk/threat-model.md`
- Roadmap: `docs/roadmap.md`

> Build order (MVP): Merkle+Indexer → Deposit tx/event → SP1 withdraw circuit → On-chain verifier → Relay → Web wiring.

## 📚 Documentation Site

- Run `yarn` inside `docs-site/` and `yarn start` to launch the Docusaurus portal backed by the markdown docs in `docs/`.
- Generated site covers architecture, workflows, on-chain programs, services, PoW, tooling, and operations.
