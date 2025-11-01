---
title: Shield Pool Program
description: On-chain Solana program for privacy-preserving withdrawals with SP1 proof verification
---

# Shield Pool Program

The Shield Pool is the core on-chain program of the Cloak protocol, responsible for managing private deposits and withdrawals using zero-knowledge proofs.

## Overview

The Shield Pool program implements the privacy-preserving exit router functionality by:

- **Accepting deposits** and logging commitments for indexer consumption
- **Managing Merkle roots** in a ring buffer for proof verification
- **Verifying SP1 proofs** for private withdrawals
- **Preventing double-spending** through nullifier tracking
- **Executing transfers** to recipients and treasury

## Program Details

- **Program ID**: `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`
- **Framework**: Pinocchio (efficient Solana program framework)
- **Proof System**: SP1 Groth16 verification
- **Hash Function**: BLAKE3-256 (consistent with guest circuit)

## Instructions

### Initialize (`0x03`)

Initialize the shield pool with required accounts.

**Accounts**: `[Pool (writable), RootsRing (writable), Treasury (writable), Admin]`

**Purpose**: Set up the initial state of the shield pool program.

### Deposit (`0x01`)

Accept deposits and log commitments for indexer consumption.

**Accounts**: `[Pool, RootsRing, System]`

**Data Layout**:
```
[tag: u8 = 0x01]
[leaf_commit: 32 bytes]
[enc_output_len: u16 LE]
[enc_output: enc_output_len bytes]
```

**Effects**:
- Logs `deposit_commit:{hex}` for indexer
- No on-chain state changes (lamports accepted via separate system transfer)

### Admin Push Root (`0x02`)

Add a new Merkle root to the ring buffer.

**Accounts**: `[RootsRing (writable)]`

**Data Layout**:
```
[tag: u8 = 0x02]
[new_root: 32 bytes]
```

**Effects**:
- Advances ring buffer head: `head = (head + 1) % 64`
- Stores root at `roots[head] = new_root`
- Logs `pushed_root:{hex}`

### Withdraw (`0x04`)

Verify SP1 proof and execute private withdrawal.

**Accounts**: `[Pool (writable), Treasury (writable), RootsRing, NullifierShard (writable), Recipients... (writable), System]`

**Data Layout**:
```
[tag: u8 = 0x04]
[sp1_proof: 256 bytes]
[sp1_public_inputs: 64 bytes]
[public_root: 32 bytes]
[public_nf: 32 bytes]
[public_amount: u64 LE]
[public_fee_bps: u16 LE]
[public_outputs_hash: 32 bytes]
[num_outputs: u8]
then num_outputs * (
    [recipient_pubkey: 32 bytes]
    [amount: u64 LE]
)
```

**Verification Steps**:
1. **SP1 Verification**: Verifies Groth16 proof using `sp1-solana` crate
2. **Root Check**: Ensures `public_root` exists in `RootsRing`
3. **Double-Spend**: Checks `public_nf` not in `NullifierShard`
4. **Outputs Hash**: Recomputes using BLAKE3 and validates match
5. **Conservation**: Verifies `sum(outputs) + fee == amount`
6. **Transfers**: Debits Pool, credits recipients + treasury
7. **Record**: Adds `public_nf` to `NullifierShard`

## Account Structures

### RootsRing (2,056 bytes)

Ring buffer storing recent Merkle roots for proof verification.

```
Offset | Size | Field
-------|------|-------
0      | 1    | head: u8 (current position, 0-63)
1      | 7    | padding
8      | 2048 | roots: [32 bytes; 64] (ring buffer)
```

**Ring Buffer Logic**:
- `head` points to the most recently added root
- New roots stored at `(head + 1) % 64`
- Contains check scans all 64 slots

### NullifierShard (4 + 32*N bytes)

Tracks used nullifiers to prevent double-spending.

```
Offset | Size | Field
-------|------|-------
0      | 4    | count: u32 LE (number of nullifiers)
4      | 32*N | nullifiers: [32 bytes; N] (used nullifiers)
```

**Nullifier Logic**:
- Linear scan for contains check (O(N))
- Append-only (no realloc in program)
- Capacity limit: 1,000 nullifiers per shard

## SP1 Integration

**VKey Hash**: `0x0064c7b959bfd20407b69859a8126b8efaa6df25191373b91cb78eb03a0bd12f`

The program uses `sp1-solana::verify_proof()` with:
- 256-byte Groth16 proof
- 64-byte public inputs (guest commitment format)
- Hardcoded VKey hash from withdraw circuit

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 0x1000 | InvalidRoot | Root not found in RootsRing |
| 0x1001 | ProofInvalid | SP1 proof verification failed |
| 0x1002 | DoubleSpend | Nullifier already used |
| 0x1003 | OutputsMismatch | Computed outputs_hash ≠ provided |
| 0x1004 | Conservation | sum(outputs) + fee ≠ amount |
| 0x1005 | MathOverflow | Integer overflow in calculations |
| 0x1006 | BadAccounts | Wrong number/type of accounts |
| 0x1007 | BadIxLength | Invalid instruction data length |
| 0x1008 | NullifierCapacity | Too many nullifiers in shard |
| 0x1009 | InvalidTag | Unknown instruction tag |

## Security Features

- **Cryptographic Verification**: All circuit constraints verified via SP1
- **Root Validation**: Only recent roots accepted (64-slot window)
- **Double-Spend Prevention**: Nullifiers tracked to prevent reuse
- **Amount Conservation**: Exact accounting enforced
- **Hash Consistency**: BLAKE3 used consistently with guest circuit
- **Memory Safety**: Bounds checks on all unsafe operations

## Usage Example

```rust
// Initialize shield pool
let init_ix = Instruction {
    program_id: CLOAK_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(pool_pubkey, false),
        AccountMeta::new(roots_ring_pubkey, false),
        AccountMeta::new(treasury_pubkey, false),
        AccountMeta::new_readonly(admin_pubkey, true),
    ],
    data: vec![0x03], // Initialize tag
};

// Push Merkle root
let push_root_ix = Instruction {
    program_id: CLOAK_PROGRAM_ID,
    accounts: vec![AccountMeta::new(roots_ring_pubkey, false)],
    data: [&[0x02], root.as_ref()].concat(),
};

// Execute withdraw with SP1 proof
let withdraw_data = [
    &[0x04],                    // tag
    sp1_proof.as_ref(),         // 256 bytes
    sp1_public_inputs.as_ref(), // 64 bytes
    root.as_ref(),              // 32 bytes
    nf.as_ref(),                // 32 bytes
    &amount.to_le_bytes(),      // 8 bytes
    &fee_bps.to_le_bytes(),     // 2 bytes
    outputs_hash.as_ref(),      // 32 bytes
    &[num_outputs],             // 1 byte
    // then num_outputs * (recipient:32 + amount:8)
].concat();

let withdraw_ix = Instruction {
    program_id: CLOAK_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(pool_pubkey, false),
        AccountMeta::new(treasury_pubkey, false),
        AccountMeta::new_readonly(roots_ring_pubkey, false),
        AccountMeta::new(nullifier_shard_pubkey, false),
        // ... recipient accounts (writable)
        AccountMeta::new_readonly(system_program::id(), false),
    ],
    data: withdraw_data,
};
```

## Dependencies

- **pinocchio**: Efficient Solana program framework
- **sp1-solana**: SP1 Groth16 proof verification
- **blake3**: Hash function (matching guest circuit)
- **five8_const**: Constant pubkey encoding
- **hex**: Hex encoding for logs

## Testing

```bash
# Build program
cargo build-sbf

# Run tests
cargo test

# Unit tests include:
# - Instruction data parsing
# - BLAKE3 hash consistency
# - Ring buffer operations
# - Nullifier storage operations
# - End-to-end instruction building
# - Double-spend prevention
```