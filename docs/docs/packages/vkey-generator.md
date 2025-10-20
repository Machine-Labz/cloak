---
title: Verification Key Generator
description: Utility that computes the SP1 verification key hash consumed by the shield pool program.
---

# Verification Key Generator

The `vkey-generator` crate is a small CLI that loads the compiled SP1 guest ELF, runs `ProverClient::setup`, and prints the resulting verification key hash.

## Why It Matters

- The shield pool program hardcodes the verification key hash. Any circuit update requires regenerating the hash and updating the on-chain constant.
- The relay/indexer may also need the hash to validate artifact versions.

## Usage

```bash
cargo run -p vkey-generator
```

The tool searches for the guest ELF in several locations:

1. `packages/zk-guest-sp1/target/elf-compilation/...`
2. `packages/zk-guest-sp1/guest/target/...`
3. `target/elf-compilation/...`

On success it prints:

```
SP1 Withdraw Circuit VKey Hash: 0x0064c7...
VKey hash written to: target/vkey_hash.txt
```

## Integrating the Hash

- Update the constant in `programs/shield-pool/src/constants.rs` (or whichever module stores it).
- Rebuild and redeploy the program to ensure on-chain verification matches the circuit version.

## Troubleshooting

- Ensure the SP1 guest has been built (`cargo build -p zk-guest-sp1-guest --release`).
- If the tool cannot find the ELF, pass the path manually by extending the `paths` array in `src/main.rs`.
- Confirm `SP1_PROVER` related environment variables are set if using remote prover clients.
