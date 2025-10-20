---
title: Shield Pool (Upstream Pattern)
description: Alternative implementation of the shield pool program using the upstream Pinocchio context pattern.
---

# Shield Pool – Upstream Pattern

`programs/shield-pool-upstream` mirrors the functionality of the main shield pool program but adopts the upstream Pinocchio reference architecture. Use it to evaluate alternative account parsing and instruction handling patterns.

## Motivation

- Align with the upstream escrow example (`.context/upstream-pinocchio-escrow`).
- Showcase `Context` structs, `TryFrom<Context>` conversions, and explicit `execute()` implementations per instruction.
- Experiment with `no_allocator!()` / `nostd_panic_handler!()` macros instead of the defaults.

## Project Structure

```
src/
├── lib.rs              # Entrypoint, discriminator routing
├── constants.rs
├── error.rs
├── groth16/
├── state/
│   ├── mod.rs          # Shared Context struct
│   ├── roots_ring.rs
│   └── nullifier_shard.rs
└── instructions/
    ├── deposit.rs
    ├── admin_push_root.rs
    └── withdraw.rs
```

## Instruction Differences

- Each instruction implements `TryFrom<Context>` to validate and borrow accounts.
- An `execute()` method performs business logic, returning `ProgramResult`.
- Logging uses upstream helper macros for clarity.

## Build & Test

```bash
cargo build -p shield-pool-upstream
cargo test -p shield-pool-upstream
```

To compile for BPF:

```bash
cargo build-sbf --manifest-path programs/shield-pool-upstream/Cargo.toml
```

## Comparison with Mainline

| Aspect | Original | Upstream Variant |
| --- | --- | --- |
| Account validation | Inline pattern matching | `Context` + `TryFrom` guards |
| Allocator/panic | `default_*` macros | `no_allocator!`, `nostd_panic_handler!` |
| Dependencies | Workspace crates | Local copies of upstream Pinocchio crates |
| Functionality | ✅ Full | ✅ Full (parity maintained) |

The upstream variant is useful for benchmarking, auditing new patterns, or gradually backporting improvements to the main program.
