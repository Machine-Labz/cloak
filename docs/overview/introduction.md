---
title: Introduction to Cloak
description: High-level overview of the Cloak privacy-preserving exit router on Solana and its core building blocks.
---

# What is Cloak?

Cloak is a privacy-preserving exit router for Solana. It combines zero-knowledge proofs, on-chain verification, and off-chain services to let users deposit SOL privately and withdraw later without linking the two actions.

## System Goals

- **Privacy-first withdrawals:** Users deposit into a shared shield pool and later withdraw without revealing which deposit they own.
- **Programmable guarantees:** The withdraw circuit enforces nullifier uniqueness, Merkle inclusion, and amount conservation.
- **Economic sustainability:** Miners earn fees by supplying proof-of-work (PoW) wildcard claims that unblock congested withdraws.
- **Operational clarity:** Indexer, relay, web app, tooling, and runbooks are all contained in this repository with a consistent Rust + TypeScript stack.

## Major Components

- **Zero-Knowledge Layer** – `packages/zk-guest-sp1` (guest and host), `packages/vkey-generator`, `packages/cloak-proof-extract`.
- **On-Chain Programs** – `programs/shield-pool`, `programs/shield-pool-upstream`, `programs/scramble-registry`.
- **Off-Chain Services** – `services/indexer`, `services/relay`, `services/web`.
- **Proof-of-Work Miner** – `packages/cloak-miner` mines wildcard claims that the relay consumes.
- **Tooling & Tests** – `tooling/test` for integration helpers, root-level runbooks, metrics, and operational guides.

## Repository Layout

| Path | Description |
| --- | --- |
| `docs/` | Design notes, architecture diagrams, ZK specs, and this Docusaurus content. |
| `packages/` | Rust crates for miners, ZK tooling, and SP1 guest/host code. |
| `programs/` | Solana BPF programs (Pinocchio) for shield pool and scramble registry. |
| `services/` | Rust indexer, Rust relay, and Next.js web interface. |
| `tooling/` | Shared testing utilities for localnet/testnet validation. |
| `compose.yml` | Docker services for Postgres, Redis, and local dependencies. |

## Feature Pillars

1. **Shield Pool Withdrawals** – Users prove they own a commitment in the Merkle tree and withdraw to arbitrary recipients with enforced fee policy.
2. **Wildcard PoW Pipeline** – Miners submit 256-bit BLAKE3 preimages registered in the scramble registry. Relay workers locate wildcard claims to prioritize exits.
3. **SP1 Groth16 Proofs** – Succinct Labs' SP1 guest program produces Groth16 proofs verified on-chain via `sp1-solana`.
4. **Auditable Infrastructure** – Structured logging, metrics, and runbooks cover the full validator/relayer operations story.

## Next Steps

- Continue with the [Quickstart](./quickstart.md) to set up a local environment.
- Review the [System Architecture](./system-architecture.md) for a component-level map.
- Dive into the [Zero-Knowledge Layer](../zk/README.md) for protocol internals.
