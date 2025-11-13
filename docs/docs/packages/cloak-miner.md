---
title: Cloak Miner CLI
description: Standalone proof-of-work miner that discovers wildcard claims for the scramble registry.
---

# Cloak Miner CLI

`packages/cloak-miner` is a Rust CLI inspired by Ore's mining workflow. Operators run it to mine wildcard claims that the relay consumes during withdraw processing.

## Capabilities

- Fetch registry difficulty and slot hashes from Solana RPC.
- Mine BLAKE3 preimages with wildcard batch hash (`[0u8; 32]`).
- Submit `mine_claim` and `reveal_claim` transactions.
- Track claim status, expiry slots, and consumption counts.
- Provide status dashboards and metrics-friendly logging.

## Installation

```bash
cargo build --release -p cloak-miner
# Binary at target/release/cloak-miner
```

## Key Commands

```bash
# Register a miner PDA (one-time)
cloak-miner --keypair ~/.config/solana/miner.json register

# Start mining wildcard claims
cloak-miner --network mainnet --keypair ~/.config/solana/miner.json mine

# Query miner + difficulty status
cloak-miner --network devnet --keypair ~/.config/solana/miner.json status
```

Flags:

- `--network` (`mainnet`, `devnet`, `localnet`).
- `--rpc-url` for custom RPC endpoints.
- `--timeout`, `--interval`, and `--target-claims` for mining control.

Environment variables mirror the flags (`CLOAK_NETWORK`, `SOLANA_RPC_URL`, `SCRAMBLE_PROGRAM_ID`, etc.).

## Mining Loop

1. Derive miner PDA and ensure it is registered.
2. Fetch difficulty + latest slot hash.
3. Iterate nonces until `BLAKE3(preimage) < target` with wildcard batch hash.
4. Submit `mine_claim`, wait for reveal window, then submit `reveal_claim`.
5. Monitor claim consumption via RPC (relay logs indicate usage).

## Economics

- Miners pay for `mine_claim` and `reveal_claim` (~10k lamports combined per claim).
- Revenue comes from withdraw jobs that consume the claim (fee distribution configurable).
- Difficulty adjustments tune claim supply relative to demand.

## Internals

- `engine/` implements the mining loop with parallel workers.
- `manager/` orchestrates RPC calls, difficulty fetch, and batching.
- `instructions/` encodes scramble registry instructions.
- `rpc/` module abstracts Solana RPC queries for slot hashes and account data.

For protocol-level context, refer to the [Wildcard Mining Overview](../pow/overview.md) and [Integration Guide](../POW_INTEGRATION_GUIDE.md).
