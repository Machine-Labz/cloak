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

- **Zero-Knowledge Layer** – SP1 guest and host programs, verification key generation, proof extraction utilities.
- **On-Chain Programs** – Shield Pool (privacy pool), Scramble Registry (wildcard claims).
- **Off-Chain Services** – Indexer (Merkle tree management), Relay (transaction submission).
- **Wildcard Miner** – Proof-of-work claim generation for prioritized withdrawals.
- **Tooling & Tests** – Integration test suite for localnet and testnet validation.

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

1. **Privacy-Preserving Withdrawals** – Zero-knowledge proofs enable unlinkable withdrawals to arbitrary recipients with cryptographically enforced fee policies.
2. **Wildcard Mining System** – Economic incentives through BLAKE3 proof-of-work claims enable prioritized transaction processing.
3. **On-Chain ZK Verification** – SP1 Groth16 proofs verified on-chain via `sp1-solana`, ensuring trustless privacy guarantees.
4. **Production Infrastructure** – Comprehensive testing, metrics, and operational tooling for reliable deployment.

## Next Steps

- Continue with the [Quickstart](./quickstart.md) to set up a local environment.
- Review the [System Architecture](./system-architecture.md) for a component-level map.
- Dive into the [Zero-Knowledge Layer](../zk/README.md) for protocol internals.
