# Shield Pool Upstream

This is an alternative implementation of the Shield Pool program using the upstream Pinocchio patterns from `.context/upstream-pinocchio-escrow`.

## Overview

Shield Pool Upstream maintains the same functionality as the original `shield-pool` program but follows a different architectural pattern inspired by the upstream Pinocchio escrow example.

### Key Differences from Original Shield Pool

1. **Architecture Pattern**:
   - Uses `Context` struct to pass accounts and instruction data
   - Implements `TryFrom<Context>` for each instruction
   - Separates account validation into dedicated structs
   - Implements `execute()` method pattern on instruction structs

2. **Dependencies**:
   - Uses local upstream Pinocchio crates from `.context/upstream-pinocchio-escrow/crates/`
   - `no_allocator!()` and `nostd_panic_handler!()` macros (instead of `default_*` variants)
   - Same SP1 verifier and cryptographic dependencies as original

3. **Instruction Handling**:
   - Discriminator-based routing (0=Deposit, 1=AdminPushRoot, 2=Withdraw)
   - Structured account validation with dedicated account structs
   - Explicit error handling with `sol_log` messages

## Project Structure

```
src/
├── lib.rs                  # Program entrypoint with upstream pattern
├── constants.rs            # Constants (same as original)
├── error.rs                # Error types (same as original)
├── groth16/                # Groth16 verifier (same as original)
│   └── mod.rs
├── state/                  # State modules
│   ├── mod.rs              # Context struct + re-exports
│   ├── roots_ring.rs       # Merkle root ring buffer
│   └── nullifier_shard.rs  # Nullifier storage
├── instructions/           # Instruction handlers
│   ├── mod.rs
│   ├── deposit.rs          # Deposit instruction (upstream pattern)
│   ├── admin_push_root.rs  # Admin root push (upstream pattern)
│   └── withdraw.rs         # Withdraw instruction (upstream pattern)
└── tests/                  # Test modules
    ├── mod.rs
    ├── deposit.rs
    ├── withdraw.rs
    └── admin_push_root.rs
```

## Instructions

### Deposit (Discriminator: 0)
Accepts SOL deposits and stores commitment for private transactions.

**Accounts**: `[user (signer), pool (writable), system_program]`
**Data**: `amount (u64) + commitment (32 bytes)`

### AdminPushRoot (Discriminator: 1)
Admin-only instruction to push new Merkle root to the ring buffer.

**Accounts**: `[admin (signer), roots_ring (writable)]`
**Data**: `root (32 bytes)`

### Withdraw (Discriminator: 2)
Verifies SP1 proof and processes private withdrawal.

**Accounts**: `[pool (writable), treasury (writable), roots_ring, nullifier_shard (writable), recipient (writable), system_program]`
**Data**: `proof (260) + public_inputs (104) + nullifier (32) + num_outputs (1) + recipient (32) + amount (8)`

## Building

```bash
# From workspace root
cargo build -p shield-pool-upstream

# Or with Solana BPF
cargo build-sbf --manifest-path programs/shield-pool-upstream/Cargo.toml
```

## Testing

```bash
cargo test -p shield-pool-upstream
```

## Comparison with Original

| Aspect | Original Shield Pool | Shield Pool Upstream |
|--------|---------------------|---------------------|
| Pattern | Direct function calls | Context + TryFrom + execute() |
| Allocator | `default_allocator!()` | `no_allocator!()` |
| Panic Handler | `default_panic_handler!()` | `nostd_panic_handler!()` |
| Account Parsing | Inline destructuring | Dedicated account structs |
| Pinocchio Deps | Workspace dependencies | Local upstream crates |
| Functionality | ✅ Full | ✅ Full (same) |

## Notes

- Both programs share the same program ID: `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`
- SP1 verification logic is identical
- Fee structure and cryptographic operations are unchanged
- Test suite needs to be ported from original program

## Future Work

- [ ] Port complete test suite from original shield-pool
- [ ] Add benchmarks comparing performance
- [ ] Document any behavioral differences
- [ ] Consider merging patterns back into main program
