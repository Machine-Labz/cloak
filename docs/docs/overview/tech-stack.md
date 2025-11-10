---
title: Technology Stack
description: Summary of languages, frameworks, and external dependencies powering Cloak.
---

# Technology Stack

Cloak blends Rust, Pinocchio, SP1, and modern web tooling to deliver privacy-preserving withdrawals. This page provides a comprehensive overview of the technical stack and implementation details.

## Languages & Frameworks

- **Rust** – Primary language for on-chain programs, miners, services, and tooling.
- **Pinocchio** – Lightweight Solana framework used by on-chain programs.
- **Succinct SP1** – ZK VM powering the withdraw circuit and proof generation.
- **TypeScript / Next.js** – Web client and supporting scripts.

## Cryptography

- **BLAKE3-256** – Hash function for commitments, nullifiers, and Merkle nodes.
- **Groth16** – zkSNARK proving system produced by SP1 and verified on-chain.

## On-Chain

- `shield-pool`, `shield-pool-upstream` – Proof verification and withdrawal settlement.
- `scramble-registry` – PoW claim registry managing miners and wildcard claims.

## Off-Chain Services

- **Indexer** – Rust + Axum + SQLx + PostgreSQL + Tokio.
- **Relay** – Rust + Axum + Redis + PostgreSQL + Jito integration.
- **Web App** – Next.js 14, Tailwind CSS, shadcn/ui, wallet adapter.

## Tooling & DevOps

- **Docker Compose** – Local Postgres and Redis.
- **just** – Task runner for workspace commands.
- **solana-cli** – Program deployment & key management.
- **cargo-watch**, **sqlx-cli** – Optional developer utilities.

For component-specific dependencies, refer to package `README.md` files and the `Cargo.toml` workspace manifest.
