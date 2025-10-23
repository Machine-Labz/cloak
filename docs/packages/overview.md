---
title: Packages & Tooling Overview
description: Overview of Cloak's Rust packages for ZK proving, mining, and development utilities.
---

# Packages & Tooling Overview

Cloak includes several Rust packages that provide essential functionality for the privacy protocol. These packages range from zero-knowledge proof generation to mining utilities and testing tools.

## Package Directory

```
packages/
├── cloak-miner/          - Standalone PoW miner CLI
├── cloak-proof-extract/  - SP1 proof parsing utilities
├── vkey-generator/       - Verification key extraction tool
└── zk-guest-sp1/        - SP1 withdraw circuit (guest + host)
```

## Quick Reference

| Package | Purpose | When to Use |
|---------|---------|-------------|
| **[cloak-miner](./cloak-miner.md)** | PoW mining for scrambler claims | Run as a miner to earn fees |
| **[zk-guest-sp1](./zk-guest-sp1.md)** | SP1 withdraw circuit | Generate withdrawal proofs |
| **[vkey-generator](./vkey-generator.md)** | VKey hash extraction | Get verification key for programs |
| **[cloak-proof-extract](./cloak-proof-extract.md)** | Proof bundle parsing | Extract Groth16 proofs from SP1 bundles |

## Package Details

### cloak-miner

**Type:** Binary (CLI application)
**Language:** Rust
**Location:** `packages/cloak-miner/`

**Purpose:**
Standalone proof-of-work miner that competes to earn fees by producing scrambler claims for the Cloak protocol.

**Key Features:**
- BLAKE3-based PoW mining
- Automatic difficulty adjustment
- Claim lifecycle management (Mine → Reveal)
- Multi-network support (localnet, devnet, mainnet)
- Configurable mining parameters

**Usage:**
```bash
# Register as miner
cargo run --package cloak-miner -- --network devnet register

# Start mining
cargo run --package cloak-miner -- --network devnet mine
```

**See:** [Cloak Miner Documentation](./cloak-miner.md)

---

### zk-guest-sp1

**Type:** Library + Binary (workspace)
**Language:** Rust (guest: no_std, host: std)
**Location:** `packages/zk-guest-sp1/`

**Purpose:**
Implements the Cloak withdraw circuit as an SP1 guest program with a host CLI for proof generation and verification.

**Structure:**
```
zk-guest-sp1/
├── guest/        - SP1 guest program (no_std RISC-V)
├── host/         - Host CLI and library (std Rust)
├── examples/     - Example input files
└── out/          - Generated proof artifacts
```

**Key Features:**
- Complete withdraw circuit constraints
- Merkle inclusion proof verification
- Nullifier computation
- Amount conservation checks
- BLAKE3 hashing throughout
- Host CLI for prove/verify operations

**Usage:**
```bash
# Generate proof
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/proof.bin

# Verify proof
cargo run --package zk-guest-sp1-host --bin cloak-zk -- verify \
  --proof out/proof.bin \
  --public out/public.json
```

**See:** [ZK Guest SP1 Documentation](./zk-guest-sp1.md)

---

### vkey-generator

**Type:** Binary (utility tool)
**Language:** Rust
**Location:** `packages/vkey-generator/`

**Purpose:**
Extracts the SP1 verification key hash from the compiled guest ELF binary. This hash must match the value hardcoded in the shield-pool program.

**Key Features:**
- Automatic ELF file discovery
- VKey hash computation using SP1 SDK
- Output to console and file
- Multiple search paths for ELF location

**Usage:**
```bash
# Generate vkey hash
cargo run --package vkey-generator

# Output:
# SP1 Withdraw Circuit VKey Hash: 0x0000000000000000000000000000000000000000000000000000000000000000
# VKey hash written to: target/vkey_hash.txt
```

**Integration:**
The generated hash must be used in the shield-pool program's withdraw instruction to verify SP1 proofs on-chain.

**See:** [VKey Generator Documentation](./vkey-generator.md)

---

### cloak-proof-extract

**Type:** Library
**Language:** Rust (no_std compatible)
**Location:** `packages/cloak-proof-extract/`

**Purpose:**
Provides utilities to extract Groth16 proof bytes and public inputs from SP1 proof bundles.

**Key Features:**
- no_std compatible (works on-chain and off-chain)
- Multiple extraction strategies (known offset + heuristic scanning)
- Parses 104-byte public inputs structure
- SP1 feature for direct deserialization
- Hex serialization support

**Public API:**
```rust
// Extract 260-byte Groth16 proof from SP1 bundle
pub fn extract_groth16_260(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], Error>

// Parse 104-byte public inputs (root||nf||outputs_hash||amount)
pub fn parse_public_inputs_104(bytes: &[u8]) -> Result<PublicInputs, Error>

// With SP1 feature enabled:
pub fn extract_groth16_260_sp1(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], Error>
pub fn extract_public_inputs_104_sp1(sp1_proof_bundle: &[u8]) -> Result<[u8; 104], Error>
```

**Usage:**
```rust
use cloak_proof_extract::{extract_groth16_260_sp1, parse_public_inputs_104_sp1};

// Extract from SP1 bundle
let proof_bytes = extract_groth16_260_sp1(&bundle)?;
let public_inputs = parse_public_inputs_104_sp1(&bundle)?;

// Use in relay service or on-chain verification
```

**See:** [Cloak Proof Extract Documentation](./cloak-proof-extract.md)

---

## Development Prerequisites

### Required Software

**Rust Toolchain:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly (required for SP1)
rustup install nightly
rustup default nightly
```

**SP1 Toolchain:**
```bash
# Install SP1
curl -L https://sp1.succinct.xyz | bash
sp1up

# Verify installation
sp1 --version
```

**Solana CLI:**
```bash
# Install Solana
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Verify installation
solana --version
```

**Additional Tools:**
```bash
# For BPF program compilation
cargo install cargo-build-sbf

# For development
cargo install cargo-watch  # Hot reloading
cargo install cargo-nextest  # Better test runner
```

### Environment Setup

**Environment Variables:**
```bash
# For SP1 proving
export SP1_PROVER=network  # or 'local' for local proving
export SP1_NETWORK_RPC=https://rpc.succinct.xyz  # SP1 network endpoint

# For Solana
export SOLANA_RPC_URL=https://api.devnet.solana.com  # or localhost:8899

# For local development
export RUST_LOG=info  # Logging level
export RUST_BACKTRACE=1  # Full backtraces on panic
```

**Configuration Files:**
```bash
# Solana config
solana config set --url devnet

# Generate keypairs for testing
solana-keygen new -o ~/.config/solana/id.json
```

## Build Commands

### Build All Packages

```bash
# From repository root
cargo build --all

# Release build (optimized)
cargo build --all --release
```

### Build Specific Package

```bash
# Cloak miner
cargo build --package cloak-miner
cargo build --package cloak-miner --release

# ZK guest/host (workspace)
cargo build --package zk-guest-sp1-guest
cargo build --package zk-guest-sp1-host

# Utilities
cargo build --package vkey-generator
cargo build --package cloak-proof-extract
```

### Build Guest Program (SP1)

```bash
# Build SP1 guest program
cd packages/zk-guest-sp1
cargo build-sp1 --release

# Or use the build script
cd guest
cargo build --release --target riscv32im-succinct-zkvm-elf
```

## Testing

### Run All Tests

```bash
# Unit tests
cargo test --all

# With output
cargo test --all -- --nocapture

# Specific package
cargo test --package cloak-miner
cargo test --package zk-guest-sp1-host
```

### Integration Tests

```bash
# ZK circuit tests
cd packages/zk-guest-sp1
cargo test --package zk-guest-sp1 golden

# Proof extraction tests
cargo test --package cloak-proof-extract
```

### End-to-End Testing

```bash
# Using tooling/test package
cargo test --package tooling-test

# See tooling-test documentation for details
```

## Package Dependencies

### Internal Dependencies

```
cloak-proof-extract
    └── (no internal deps)

vkey-generator
    └── (no internal deps, uses zk-guest-sp1 ELF output)

zk-guest-sp1
    └── (no internal deps)

cloak-miner
    └── (no internal deps)

relay service
    └── cloak-proof-extract (for proof parsing)

tooling-test
    ├── cloak-proof-extract
    └── uses all programs and services
```

### External Dependencies

**Common:**
- `anyhow` - Error handling
- `serde` / `serde_json` - Serialization
- `hex` - Hex encoding/decoding
- `blake3` - Cryptographic hashing

**SP1 Specific:**
- `sp1-sdk` - SP1 prover and verifier
- `sp1-zkvm` - SP1 guest runtime (guest only)

**Solana Specific:**
- `solana-sdk` - Solana types and utilities
- `solana-client` - RPC client
- `anchor-lang` / `pinocchio` - Program frameworks

**Async Runtime:**
- `tokio` - Async runtime (services and miner)

## Common Workflows

### 1. Generate a Withdrawal Proof

```bash
# Step 1: Ensure guest is built
cd packages/zk-guest-sp1/guest
cargo build --release --target riscv32im-succinct-zkvm-elf

# Step 2: Create input files (or use examples)
cd ..
cp examples/private.example.json examples/private.json
cp examples/public.example.json examples/public.json
cp examples/outputs.example.json examples/outputs.json
# Edit files with actual values...

# Step 3: Generate proof
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove \
  --private examples/private.json \
  --public examples/public.json \
  --outputs examples/outputs.json \
  --proof out/proof.bin \
  --pubout out/public.bin

# Step 4: Extract proof for submission
# Proof is now in out/proof.bin (SP1 bundle)
# Can be submitted to relay service
```

### 2. Start Mining

```bash
# Step 1: Register miner (one-time)
cargo run --package cloak-miner -- \
  --network devnet \
  --keypair ~/.config/solana/miner.json \
  register

# Step 2: Start mining
cargo run --package cloak-miner -- \
  --network devnet \
  --keypair ~/.config/solana/miner.json \
  mine

# Miner will continuously:
# - Mine PoW solutions
# - Submit claims to scramble-registry
# - Reveal claims for consumption
```

### 3. Update Verification Key

```bash
# Step 1: Build guest program
cd packages/zk-guest-sp1/guest
cargo build --release --target riscv32im-succinct-zkvm-elf

# Step 2: Generate vkey hash
cd ../..
cargo run --package vkey-generator

# Step 3: Copy hash to shield-pool program
# Update programs/shield-pool/src/constants.rs with new hash

# Step 4: Rebuild and redeploy shield-pool program
cargo build-sbf
solana program deploy ...
```

## Troubleshooting

### SP1 Build Errors

**Problem:** `error: failed to run cargo build`

**Solution:**
```bash
# Ensure SP1 toolchain is installed
sp1up

# Use nightly Rust
rustup default nightly

# Clean and rebuild
cargo clean
cargo build
```

### ELF Not Found (vkey-generator)

**Problem:** `Could not find guest ELF in any expected location`

**Solution:**
```bash
# Build the guest program first
cd packages/zk-guest-sp1/guest
cargo build --release --target riscv32im-succinct-zkvm-elf

# Then run vkey-generator
cd ../..
cargo run --package vkey-generator
```

### Proof Extraction Fails

**Problem:** `InvalidFormat` error when extracting proof

**Solution:**
- Ensure proof.bin is a valid SP1 bundle (not raw Groth16 bytes)
- Check proof was generated with compatible SP1 version
- Verify proof.bin is not corrupted
- Try using `extract_groth16_260_sp1` with SP1 feature

### Mining Connection Issues

**Problem:** Miner cannot connect to RPC or program not found

**Solution:**
```bash
# Check RPC URL
solana config get

# Verify program is deployed
solana program show <PROGRAM_ID>

# Check keypair has funds
solana balance ~/.config/solana/miner.json

# Use correct network flag
cargo run --package cloak-miner -- --network devnet ...
```

## Related Documentation

- **[Cloak Miner](./cloak-miner.md)** - Detailed mining documentation
- **[ZK Guest SP1](./zk-guest-sp1.md)** - Circuit implementation details
- **[VKey Generator](./vkey-generator.md)** - Verification key extraction
- **[Proof Extract](./cloak-proof-extract.md)** - Proof parsing utilities
- **[Tooling Test](./tooling-test.md)** - Testing utilities
- **[PoW Overview](../pow/overview.md)** - Mining system architecture
- **[ZK Design](../zk/design.md)** - Zero-knowledge protocol design
