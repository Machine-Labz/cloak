# ZK Guest SP1 - Cloak Withdraw Circuit

This package implements Cloak's withdraw circuit as an SP1 guest program with a host CLI for proof generation and verification.

## Overview

The withdraw circuit enforces the following constraints:
1. `pk_spend = H(sk_spend)` - Spending key validation
2. `C = H(amount || r || pk_spend)` - Commitment computation
3. `MerkleVerify(C, merkle_path) == root` - Merkle tree membership
4. `nf == H(sk_spend || leaf_index)` - Nullifier computation
5. `sum(outputs) + fee(amount, fee_bps) == amount` - Amount conservation
6. `H(serialize(outputs)) == outputs_hash` - Outputs binding

## Architecture

```
packages/zk-guest-sp1/
├── Cargo.toml                    # Workspace configuration
├── guest/                        # SP1 guest program (no_std)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # Guest entrypoint with circuit logic
│       └── encoding.rs          # BLAKE3 hashing & serialization
├── host/                        # Host CLI and library
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # CLI: prove/verify commands
│       ├── lib.rs               # Library exports
│       ├── encoding.rs          # Shared encoding functions
│       └── bin/
│           └── generate_examples.rs  # Example generator
├── examples/                    # Test data
│   ├── private.example.json     # Private witness inputs
│   ├── public.example.json      # Public inputs
│   └── outputs.example.json     # Outputs array
├── tests/
│   └── golden.rs               # Integration tests
└── README.md
```

## Usage

### Prerequisites

- Rust with SP1 toolchain
- SP1 environment setup (see [SP1 docs](https://docs.succinct.xyz/))

### Build

```bash
cargo build -p zk-guest-sp1-host
```

### Generate Proof

```bash
cargo run -p zk-guest-sp1-host --bin cloak-zk -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/proof.bin \
  --pubout out/public.json
```

### Verify Proof

```bash
cargo run -p zk-guest-sp1-host --bin cloak-zk -- verify \
  --proof out/proof.bin \
  --public out/public.json
```

### Run Tests

```bash
# Unit tests (encoding functions)
cargo test -p zk-guest-sp1-host

# Integration tests (full prove/verify cycle)
cargo test -p zk-guest-sp1 golden
```

## Data Formats

### Private Inputs (`private.json`)

```json
{
  "amount": 1000000,
  "r": "hex32_bytes",
  "sk_spend": "hex32_bytes", 
  "leaf_index": 42,
  "merkle_path": {
    "path_elements": ["hex32_bytes", ...],
    "path_indices": [0, 1, ...]
  }
}
```

### Public Inputs (`public.json`)

```json
{
  "root": "hex32_bytes",
  "nf": "hex32_bytes",
  "fee_bps": 60,
  "outputs_hash": "hex32_bytes",
  "amount": 1000000
}
```

### Outputs (`outputs.json`)

```json
[
  {
    "address": "base58_or_hex32",
    "amount": 400000
  },
  {
    "address": "base58_or_hex32", 
    "amount": 594000
  }
]
```

## Encoding Specification

All hashing uses **BLAKE3-256** with **little-endian** integer serialization:

- `u64` → 8 bytes LE
- `u32` → 4 bytes LE  
- `u16` → 2 bytes LE
- Addresses: 32 bytes (base58 or hex)
- Concatenation: no separators, fixed order

### Hash Computations

- **Commitment**: `C = H(amount:u64 || r:32 || pk_spend:32)`
- **Spend Key**: `pk_spend = H(sk_spend:32)`
- **Nullifier**: `nf = H(sk_spend:32 || leaf_index:u32)`
- **Outputs Hash**: `H(address₀:32 || amount₀:u64 || ... || addressₙ:32 || amountₙ:u64)`
- **Merkle**: `parent = H(left:32 || right:32)` (index 0=left, 1=right)

### Fee Calculation

```
fee = (amount * fee_bps) / 10_000
```

Where `fee_bps` is basis points (e.g., 60 = 0.6%).

## Example Generation

Generate consistent test data:

```bash
cargo run -p zk-guest-sp1-host --bin generate_examples
```

This creates `examples/*.json` files with matching hashes and valid constraints.

## Security Notes

- Guest program enforces all circuit constraints
- Proof generation fails if any constraint is violated
- Public inputs are committed to the proof and verified
- All cryptographic operations use BLAKE3-256
- Integer serialization is deterministic (little-endian)

## Development

The encoding module is shared between guest and host to ensure consistency. Unit tests verify hash computations match the specification, and integration tests ensure the full prove/verify cycle works correctly.

For debugging, the guest program will panic with descriptive messages if constraints fail, helping identify issues during development.