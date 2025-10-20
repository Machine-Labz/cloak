---
title: System Architecture
description: A component-level map of Cloak covering on-chain programs, services, miners, and data stores.
---

# System Architecture

At a high level, Cloak consists of four cooperating domains:

1. **Users** interacting via the web app or APIs.
2. **Off-chain services** (indexer and relay) that coordinate state, proofs, and withdrawals.
3. **Zero-knowledge tooling** (SP1 guest/host and proof utilities).
4. **On-chain programs** (shield pool and scramble registry) that enforce protocol rules.

```text
┌────────────┐     ┌──────────────┐     ┌────────────────┐     ┌───────────────────┐
│ Web / CLI  │ --> │ Relay (Axum) │ --> │ Solana Programs │ --> │ Users / Recipients │
│ Clients    │     │ + Queue      │     │ (Shield + PoW)  │     │ (Receive funds)   │
└────────────┘     └────┬─────────┘     └────┬────────────┘     └───────────────────┘
                         │                    │
                         │ Proofs + txs       │ Events
                         ▼                    ▼
                    ┌──────────────┐    ┌──────────────┐
                    │ SP1 Prover   │    │ Indexer API  │
                    │ / Artifacts  │    │ + Database   │
                    └──────────────┘    └──────────────┘
```

## Data Flow Overview

1. **Deposits:**
   - Users submit deposit transactions to the `shield-pool` program.
   - The program emits commitment events; the **Indexer** ingests them and updates the Merkle tree.
2. **Merkle Roots:**
   - Indexer exposes the current root via `GET /api/v1/merkle/root`.
   - Operators push accepted roots on-chain via the `AdminPushRoot` instruction.
3. **Proof Generation:**
   - Withdrawals use the `zk-guest-sp1` guest to produce Groth16 proofs (SP1 host CLI or remote service).
   - `vkey-generator` outputs the verification key hash stored in `shield-pool`.
4. **Relay Workflow:**
   - Clients submit withdraw jobs to the relay.
   - The relay validates inputs, checks nullifiers via PostgreSQL, and enqueues the job in Redis.
   - Workers look up wildcard claims using `ClaimFinder` (scramble registry PoW) and assemble Solana transactions.
   - Transactions are simulated, broadcast, optionally via Jito, and confirmed.
5. **PoW Miners:**
   - `cloak-miner` continuously mines Wildcard claims against the scramble registry difficulty.
   - Claims become consumable by relay workers to satisfy withdraw PoW requirements.

## Component Responsibilities

| Domain | Component | Responsibilities |
| --- | --- | --- |
| On-chain | `shield-pool` | Verify SP1 proofs, manage nullifiers, ring buffer of Merkle roots, distribute lamports. |
| On-chain | `scramble-registry` | Manage miner registrations, claim lifecycle, wildcard validation for PoW. |
| Off-chain | `services/indexer` | Axum service backed by Postgres. Maintains Merkle tree, persists encrypted outputs, serves proofs/artifacts. |
| Off-chain | `services/relay` | Axum API + background workers. Validates requests, queues jobs, submits withdraw transactions, integrates PoW claims. |
| Off-chain | `services/web` | Next.js application offering deposit/withdraw UI, note management, and admin tooling. |
| ZK | `packages/zk-guest-sp1` | SP1 guest circuit and host CLI for proof generation/verification. |
| ZK | `packages/vkey-generator` | Extract verification key hash from the guest ELF. |
| ZK | `packages/cloak-proof-extract` | Parse SP1 proof bundles to extract Groth16 proof/public inputs. |
| Tooling | `tooling/test` | Shared Rust helpers to exercise programs on localnet/testnet. |

## Key Storages

- **PostgreSQL (`services/indexer`, `services/relay`)** – Merkle nodes, encrypted notes, job state, nullifiers.
- **Redis** – High throughput queue for withdraw jobs with retry/backoff configuration.
- **Solana Accounts** – Pool PDA (funds), roots ring buffer, nullifier shards, scramble registry data.
- **Filesystem (`packages/zk-guest-sp1/out/`)** – Proof artifacts generated during development/testing.

## Communication Interfaces

- **HTTP APIs** – Indexer (`/api/v1/*`) and Relay (`/withdraw`, `/jobs`, `/status`).
- **Admin CLI** – `cloak-miner` for PoW and `zk-guest-sp1-host` for proving.
- **Solana RPC** – Both services use RPC for fetching blocks, submitting transactions, and verifying cluster health.

Refer to the [Visual Flow](./visual-flow.md) diagrams for end-to-end sequence charts and to the [Runbook](../operations/runbook.md) for operational procedures.
