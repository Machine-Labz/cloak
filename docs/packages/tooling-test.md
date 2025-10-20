---
title: Testing Toolkit
description: Shared Rust helpers for localnet/testnet end-to-end testing of the shield pool and relay stack.
---

# Testing Toolkit

`tooling/test` contains reusable Rust helpers for integration tests, smoke tests, and manual QA scenarios.

## Capabilities

- Load Solana keypairs from JSON or binary files.
- Query cluster health via `RpcClient`.
- Fund user accounts from admin keypairs (with send-and-confirm handling).
- Construct PDA addresses for the shield pool program (pool, roots ring, nullifier shard, commitments, treasury).
- Build deposit and withdraw instructions using `shield_pool::instructions` constants.
- Manage Merkle proofs and test fixtures for indexer/relay interactions.

## Key Structures

- `TestConfig` – Describes RPC URLs, program IDs, lamport amounts, keypair paths, and indexer URLs. Provides `localnet()` and `testnet()` constructors.
- `MerkleProof` & `DepositRequest` – Serde-friendly structs matching API contracts.

## Helper Functions

- `load_keypair(path)` – Reads JSON or binary keypair files.
- `check_cluster_health(rpc_url)` – Validates RPC availability.
- `ensure_user_funding()` – Transfers lamports from admin to user if below threshold.
- `get_pda_addresses(program_id)` – Returns tuple of PDAs derived from seeds.
- Instruction builders for deposits and withdrawals (see module for specifics).

## Usage Example

```rust
use tooling::shared::{TestConfig, check_cluster_health, ensure_user_funding};

let config = TestConfig::localnet();
check_cluster_health(&config.rpc_url)?;
let user = load_keypair(&config.user_keypair_path)?;
let admin = load_keypair("admin-keypair.json")?;
ensure_user_funding(&config.rpc_url, &user, &admin)?;
```

## Running Tests

The crate is part of the workspace. Run:

```bash
cargo test -p tooling-test
```

Use it as a foundation when writing integration tests for new flows or when automating smoke tests for deployments.
