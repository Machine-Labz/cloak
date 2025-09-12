# Shield Pool - Cloak On-Chain Program

A minimal Pinocchio-based Solana program that verifies SP1 withdraw proofs and executes private withdrawals from a shielded pool.

## Overview

This program implements the on-chain component of Cloak's privacy protocol, handling:
- SP1 Groth16 proof verification for withdraw circuits
- Merkle root management (recent roots ring buffer)
- Nullifier tracking to prevent double-spending
- Lamports transfers to recipients and treasury

## Program ID

```
99999999999999999999999999999999999999999999 (placeholder)
```

## Instructions

### 1. Deposit (`0x01`)

**Purpose**: Accept deposits and log commitments for indexer consumption.

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

### 2. Admin Push Root (`0x02`)

**Purpose**: Add a new Merkle root to the ring buffer.

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

### 3. Withdraw (`0x03`)

**Purpose**: Verify SP1 proof and execute private withdrawal.

**Accounts**: `[Pool (writable), Treasury (writable), RootsRing, NullifierShard (writable), Recipients... (writable), System]`

**Data Layout**:
```
[tag: u8 = 0x03]
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

**Effects**:
1. **SP1 Verification**: Verifies Groth16 proof using `sp1-solana` crate
2. **Root Check**: Ensures `public_root` exists in `RootsRing`
3. **Double-Spend**: Checks `public_nf` not in `NullifierShard`
4. **Outputs Hash**: Recomputes using BLAKE3 and validates match
5. **Conservation**: Verifies `sum(outputs) + fee == amount`
6. **Transfers**: Debits Pool, credits recipients + treasury
7. **Record**: Adds `public_nf` to `NullifierShard`
8. **Event**: Logs `withdraw_event:nf:{hex},root:{hex},outputs_hash:{hex}`

## Account Layouts

### RootsRing (2,056 bytes)

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

## Hash Functions

**BLAKE3-256** used throughout for consistency with guest circuit:

- **Outputs Hash**: `H(recipient₀:32 || amount₀:u64_LE || ... || recipientₙ:32 || amountₙ:u64_LE)`
- **Fee Calculation**: `fee = (amount * fee_bps) / 10_000`

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

## SP1 Integration

**VKey Hash**: `0x0064c7b959bfd20407b69859a8126b8efaa6df25191373b91cb78eb03a0bd12f`

The program uses `sp1-solana::verify_proof()` with:
- 256-byte Groth16 proof
- 64-byte public inputs (guest commitment format)
- Hardcoded VKey hash from withdraw circuit
- `sp1_solana::GROTH16_VK_5_0_0_BYTES` verification key

## Build & Test

### Build Program
```bash
cargo build-sbf
```

### Run Tests
```bash
cargo test
```

### Unit Tests
- `test_withdraw_ix_parsing`: Instruction data parsing
- `test_outputs_hash_computation`: BLAKE3 hash consistency
- `test_roots_ring`: Ring buffer operations
- `test_nullifier_shard`: Nullifier storage operations

### Integration Tests
- `test_instruction_parsing`: End-to-end instruction building
- `test_double_spend_prevention`: Structure for nullifier reuse tests

## Usage Example

```rust
// 1. Push root to enable withdrawals
let push_root_ix = Instruction {
    program_id: SHIELD_POOL_PROGRAM_ID,
    accounts: vec![AccountMeta::new(roots_ring_pubkey, false)],
    data: [&[0x02], root.as_ref()].concat(),
};

// 2. Execute withdraw with SP1 proof
let withdraw_data = [
    &[0x03],                    // tag
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
    program_id: SHIELD_POOL_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(pool_pubkey, false),
        AccountMeta::new(treasury_pubkey, false),
        AccountMeta::new(roots_ring_pubkey, false),
        AccountMeta::new(nullifier_shard_pubkey, false),
        // ... recipient accounts (writable)
        AccountMeta::new_readonly(system_program::id(), false),
    ],
    data: withdraw_data,
};
```

## Security Notes

- **SP1 Proof**: All circuit constraints verified cryptographically
- **Root Validation**: Only recent roots accepted (64-slot window)
- **Double-Spend**: Nullifiers tracked to prevent reuse
- **Conservation**: Exact amount accounting enforced
- **Hash Consistency**: BLAKE3 used consistently with guest circuit
- **Memory Safety**: Unsafe operations used only for performance with bounds checks

## Dependencies

- **pinocchio**: Efficient Solana program framework
- **sp1-solana**: SP1 Groth16 proof verification
- **blake3**: Hash function (matching guest circuit)
- **five8_const**: Constant pubkey encoding
- **hex**: Hex encoding for logs

## Deployment

The program compiles to a BPF bytecode that can be deployed to Solana. Account initialization and funding must be handled separately by client applications.