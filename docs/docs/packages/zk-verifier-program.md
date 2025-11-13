---
title: SP1 Verifier Program
description: Planned Solana CPI verifier for SP1 proofs shared across programs.
---

# SP1 Verifier Program

`packages/zk-verifier-program` is a placeholder crate intended to expose a reusable Solana CPI entrypoint for SP1 Groth16 verification. While the primary shield pool program currently embeds the verifier, this crate documents the intended abstraction.

## Goals

- Provide a `verify(proof_bytes, public_inputs)` interface callable via CPI.
- Share verifier logic across multiple programs (shield pool, registry, future circuits).
- Encapsulate verification key hashing and byte layout to reduce duplication.

## Current Status

- README outlines tasks; implementation pending.
- Aligns with `sp1-solana` integration used by `programs/shield-pool`.
- Keep in sync with verification key produced by `vkey-generator`.

## Next Steps

- Implement program skeleton mirroring shield pool's verifier logic.
- Define account layout for verification key storage (if not hardcoded).
- Build integration tests ensuring parity with direct verifier usage.
- Update shield pool to depend on this crate once stable.

Track progress in the roadmap and status reports found in the documentation sidebar.
