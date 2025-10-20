---
title: SP1 Withdraw Circuit
description: Guest program and host CLI that generate Groth16 proofs for Cloak withdrawals.
---

# SP1 Withdraw Circuit

`packages/zk-guest-sp1` houses the SP1 guest program, host CLI, integration tests, and example data for Cloak's withdraw circuit.

## Circuit Constraints

1. `pk_spend = H(sk_spend)` – Spend key binding.
2. `C = H(amount ∥ r ∥ pk_spend)` – Commitment recomputation.
3. `MerkleVerify(C, path) == root` – Inclusion proof.
4. `nf = H(sk_spend ∥ leaf_index)` – Nullifier uniqueness.
5. `Σ(outputs) + fee(amount, fee_bps) == amount` – Conservation.
6. `outputs_hash = H(serialize(outputs))` – Bind outputs to proof.

All hashes use BLAKE3-256 with little-endian integer encoding.

## Directory Layout

```
packages/zk-guest-sp1/
├── guest/        # no_std SP1 guest program
├── host/         # CLI binary + library helpers
├── examples/     # Sample private/public inputs and outputs
├── tests/        # Golden prove/verify integration tests
└── README.md
```

## Host CLI

```bash
# Build host CLI
cargo build -p zk-guest-sp1-host

# Generate proof
cargo run -p zk-guest-sp1-host -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/proof.bin \
  --pubout out/public.json

# Verify proof
cargo run -p zk-guest-sp1-host -- verify \
  --proof out/proof.bin \
  --public out/public.json
```

The CLI uses SP1's `ProverClient` to drive proof generation and verification. Example JSON files illustrate expected serialization.

## Encoding Utilities

- Shared between guest and host to avoid divergence.
- Hashing helpers: commitments, outputs hash, spend key derivation, nullifier.
- Fee computation uses basis points (e.g., `60 = 0.6%`).

## Testing

- `cargo test -p zk-guest-sp1-host` – Unit tests for encoding + helper functions.
- `cargo test -p zk-guest-sp1 golden` – Integration test verifying full prove/verify cycle against golden artifacts.

## Cycle Counting

See `CYCLE_COUNTING.md` for profiling results and optimisation notes.

## Related Tools

- [`packages/vkey-generator`](./vkey-generator.md) – Extract verification key hash from the compiled guest ELF.
- [`packages/cloak-proof-extract`](./cloak-proof-extract.md) – Parse SP1 bundles to isolate Groth16 proof bytes for on-chain verification.
