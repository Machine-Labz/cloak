---
title: SP1 Proof Extractor
description: Library for parsing SP1 proof bundles to extract Groth16 proof bytes and public inputs.
---

# SP1 Proof Extractor

`packages/cloak-proof-extract` provides utilities for working with SP1 proof bundles. It extracts the Groth16 proof fragment and public inputs needed by the shield pool program.

## Features

- `extract_groth16_260` – Scans the SP1 bundle for the 260-byte Groth16 proof (either using a known offset or length-prefixed search heuristic).
- `parse_public_inputs_104` – Parses the 104-byte public inputs block into `(root, nf, outputs_hash, amount)`.
- Optional feature gates:
  - `std` – Enable `Display`/`Error` implementations.
  - `sp1` – Deserialize bundles using `sp1_sdk::SP1ProofWithPublicValues`.
  - `hex` – Serialize structures as hex strings via Serde helpers.

## Usage

```rust
use cloak_proof_extract::{extract_groth16_260, parse_public_inputs_104};

let bundle = std::fs::read("proof.bin")?;
let proof = extract_groth16_260(&bundle)?;      // [u8; 260]
let public_inputs = parse_public_inputs_104(&bundle[OFFSET..OFFSET+104])?;
```

When the `sp1` feature is enabled you can bypass manual offsets:

```rust
use cloak_proof_extract::parse_public_inputs_104_sp1;

let bundle = std::fs::read("proof.bin")?;
let public_inputs = parse_public_inputs_104_sp1(&bundle)?;
assert_eq!(public_inputs.amount, 1_000_000);
```

## Error Handling

Errors are reported via the `Error::InvalidFormat` enum variant when byte lengths or offsets are unexpected. The heuristics reject all-zero slices to guard against false positives.

## Tests

- Integration tests load fixture bundles from disk (see `tests` directory) to verify both heuristic and SP1-backed paths.

## Integration Points

- Relay workers can use the library to check bundles submitted by clients before forwarding to Solana.
- Tooling/CLIs can convert SP1 proof bundles into the raw bytes required by `sp1-solana`.

This crate is `no_std` friendly (with optional `alloc`) for future embedding in constrained environments.
