---
title: Verification Key Generator
description: Utility tool to extract SP1 verification key hash from compiled guest ELF for on-chain verification.
---

# Verification Key Generator

`vkey-generator` is a utility CLI tool that extracts the SP1 verification key hash from the compiled withdraw circuit guest ELF. This hash is critical for on-chain proof verification in the shield-pool program.

**Source:** `packages/vkey-generator/`

## Overview

The verification key (vkey) is a cryptographic artifact that uniquely identifies a specific SP1 circuit. The vkey hash is used by the on-chain verifier to ensure that submitted proofs were generated using the correct circuit version.

### Why This Matters

**Circuit Versioning:**
- Each version of the guest program has a unique verification key
- Changing even one line of circuit code generates a new vkey
- The shield-pool program hardcodes the expected vkey hash
- Proofs generated with mismatched circuits will fail verification

**Deployment Workflow:**
1. Modify withdraw circuit guest program
2. Rebuild guest ELF
3. Run `vkey-generator` to extract new hash
4. Update hardcoded hash in shield-pool program
5. Redeploy shield-pool program
6. New circuit is now active on-chain

### Architecture

```
┌──────────────────────┐
│ Guest ELF Binary     │  ← Compiled circuit (RISC-V)
│ (zk-guest-sp1-guest) │
└──────────┬───────────┘
           │
           │ Read ELF
           ▼
┌──────────────────────┐
│ vkey-generator       │
│ • Load ELF           │
│ • ProverClient       │
│ • setup() → vkey     │
│ • Compute bytes32    │
└──────────┬───────────┘
           │
           │ Output
           ▼
┌──────────────────────┐
│ VKey Hash            │
│ 0x0064c7b959bfd...   │  ← 32-byte hex string
└──────────┬───────────┘
           │
           │ Copy to
           ▼
┌──────────────────────┐
│ Shield Pool Program  │
│ constants.rs:        │
│ SP1_VKEY_HASH =      │
│   "0x0064c7..."      │
└──────────────────────┘
```

## How SP1 Verification Works

### Verification Key Generation

1. **Setup Phase:**
   - SP1's `ProverClient::setup(elf)` analyzes the guest program
   - Generates a proving key (pk) and verification key (vk)
   - Verification key is deterministic based on circuit code

2. **VKey Hash Computation:**
   - `vk.bytes32()` computes a 32-byte hash of the verification key
   - This hash uniquely identifies the circuit version
   - Used for efficient on-chain verification

3. **On-Chain Verification:**
   - Shield-pool program stores expected vkey hash as constant
   - When verifying a proof, SP1 verifier checks vkey hash matches
   - If mismatch: proof rejected (circuit version mismatch)
   - If match: proof verified using embedded verification key

### What Changes the VKey?

**Changes that generate a new vkey:**
- Modifying circuit constraints in `guest/src/main.rs`
- Updating encoding functions in `guest/src/encoding.rs`
- Changing guest dependencies or compilation flags
- Upgrading SP1 SDK version (sometimes)

**Changes that DON'T affect vkey:**
- Comments or documentation
- Host code changes
- Input data changes
- Proof generation settings

## Installation

### Prerequisites

```bash
# SP1 SDK required
curl -L https://sp1.succinct.xyz | bash
sp1up

# Verify installation
sp1 --version
```

### Build

```bash
# Navigate to repository root
cd cloak

# Build vkey-generator
cargo build --release --package vkey-generator

# Binary location
# target/release/vkey-generator
```

## Usage

### Basic Usage

```bash
# From repository root
cargo run --package vkey-generator

# Output:
# SP1 Withdraw Circuit VKey Hash: 0x0000000000000000000000000000000000000000000000000000000000000000
# VKey hash written to: target/vkey_hash.txt
```

### Using Release Binary

```bash
# Build release binary
cargo build --release --package vkey-generator

# Run directly
./target/release/vkey-generator
```

## ELF Search Paths

The tool searches for the guest ELF in multiple locations:

**Search Order:**
1. `packages/zk-guest-sp1/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest`
2. `packages/zk-guest-sp1/guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest`
3. `target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest`
4. `target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest`
5. `guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest`

**Reference:** `vkey-generator/src/main.rs:6-12`

## Complete Integration Workflow

### 1. Modify Circuit

```bash
# Edit guest program
vim packages/zk-guest-sp1/guest/src/main.rs

# Example: Add new constraint
fn verify_circuit_constraints(inputs: &CircuitInputs) -> Result<()> {
    // ... existing constraints ...

    // New constraint
    let custom_check = verify_custom_logic(&inputs);
    if !custom_check {
        return Err(anyhow!("Custom constraint failed"));
    }

    Ok(())
}
```

### 2. Rebuild Guest Program

```bash
# Navigate to guest directory
cd packages/zk-guest-sp1/guest

# Build for RISC-V target (release mode required)
cargo build --release --target riscv32im-succinct-zkvm-elf

# Or use cargo prove from workspace root
cd ../../..
cargo prove build --release

# Verify ELF exists
ls -lh packages/zk-guest-sp1/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest
# Should show ~1-2 MB file
```

### 3. Generate New VKey Hash

```bash
# Run vkey-generator
cargo run --package vkey-generator

# Output:
# SP1 Withdraw Circuit VKey Hash: 0x0000000000000000000000000000000000000000000000000000000000000000
# VKey hash written to: target/vkey_hash.txt

# Copy the hash (you'll need it for next step)
cat target/vkey_hash.txt
```

### 4. Update Shield Pool Program

```bash
# Open constants file
vim programs/shield-pool/src/constants.rs

# Update the SP1_VKEY_HASH constant
# Before:
# pub const SP1_VKEY_HASH: &str = "0xOLDHASH...";

# After:
pub const SP1_VKEY_HASH: &str = "0x0064c7b959bfd20407b69859a8126b8efaa6df25191373b91cb78eb03a0bd12f";

# Save and exit
```

### 5. Rebuild Shield Pool Program

```bash
# Build Solana BPF program
cargo build-sbf

# Verify build succeeded
ls -lh target/deploy/shield_pool.so
```

### 6. Deploy to Solana

**Devnet:**
```bash
# Deploy to devnet
solana program deploy \
  target/deploy/shield_pool.so \
  --program-id target/deploy/shield_pool-keypair.json \
  --url devnet

# Verify deployment
solana program show <PROGRAM_ID> --url devnet
```

**Mainnet:**
```bash
# Deploy to mainnet (requires SOL for deployment fees)
solana program deploy \
  target/deploy/shield_pool.so \
  --program-id target/deploy/shield_pool-keypair.json \
  --url mainnet-beta

# Verify deployment
solana program show <PROGRAM_ID> --url mainnet-beta
```

### 7. Verify Integration

**Test Proof Generation:**
```bash
# Generate a test proof with new circuit
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/test_proof.bin \
  --pubout out/test_public.json
```

**Submit Test Withdraw:**
```bash
# Submit proof to relay service
curl -X POST http://localhost:3002/withdraw \
  -H "Content-Type: application/json" \
  -d @withdraw_request.json

# Check if proof verifies on-chain with new vkey hash
```

## Output

### Console Output

```
SP1 Withdraw Circuit VKey Hash: 0x0000000000000000000000000000000000000000000000000000000000000000
VKey hash written to: target/vkey_hash.txt
```

### File Output

**Location:** `target/vkey_hash.txt`

**Contents:** 32-byte hex string (with 0x prefix)
```
0x0064c7b959bfd20407b69859a8126b8efaa6df25191373b91cb78eb03a0bd12f
```

**Usage:**
- Copy this value to shield-pool program constants
- Store for documentation/changelog purposes
- Compare with previous hash to verify circuit changes

## Advanced Usage

### Extract Full Verification Key

The current tool only extracts the hash. To extract the full verification key:

```rust
// Modify vkey-generator/src/main.rs
use sp1_sdk::{HashableKey, ProverClient};
use std::fs;

fn main() -> Result<()> {
    let client = ProverClient::from_env();
    let guest_elf = find_guest_elf()?;
    let (pk, vk) = client.setup(&guest_elf);

    // Get hash
    let vkey_hash = vk.bytes32();
    println!("VKey Hash: {}", vkey_hash);

    // Serialize full vkey
    let vkey_bytes = bincode::serialize(&vk)?;
    fs::write("target/vkey_full.bin", &vkey_bytes)?;
    println!("Full vkey written to: target/vkey_full.bin");

    // Also save proving key if needed
    let pk_bytes = bincode::serialize(&pk)?;
    fs::write("target/proving_key.bin", &pk_bytes)?;

    Ok(())
}
```

### Automate VKey Update

Create a script to automate the workflow:

```bash
#!/bin/bash
# update_vkey.sh - Rebuild circuit and update vkey hash

set -e  # Exit on error

echo "🔧 Rebuilding guest program..."
cd packages/zk-guest-sp1/guest
cargo build --release --target riscv32im-succinct-zkvm-elf
cd ../../..

echo "🔑 Generating new vkey hash..."
NEW_HASH=$(cargo run --package vkey-generator 2>&1 | grep "0x" | awk '{print $NF}')
echo "New hash: $NEW_HASH"

echo "📝 Updating shield-pool constants..."
sed -i.bak "s/pub const SP1_VKEY_HASH: &str = \"0x[a-f0-9]*\";/pub const SP1_VKEY_HASH: \&str = \"$NEW_HASH\";/" programs/shield-pool/src/constants.rs

echo "🔨 Rebuilding shield-pool..."
cargo build-sbf

echo "✅ Done! New vkey hash: $NEW_HASH"
echo "⚠️  Don't forget to deploy the updated program!"
```

Usage:
```bash
chmod +x update_vkey.sh
./update_vkey.sh
```

### Compare VKey Hashes

Track vkey changes across circuit versions:

```bash
# Save vkey history
echo "$(date -I) - $(git log --oneline -1) - $(cat target/vkey_hash.txt)" >> vkey_history.txt

# View history
cat vkey_history.txt
# Output:
# 2025-01-15 - abc123 Initial circuit - 0x0064c7...
# 2025-01-20 - def456 Add fee constraint - 0x1234ab...
# 2025-01-25 - ghi789 Fix merkle bug - 0x5678cd...
```

### CI/CD Integration

Add to GitHub Actions workflow:

```yaml
# .github/workflows/verify_vkey.yml
name: Verify VKey Consistency

on: [pull_request]

jobs:
  vkey-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install SP1
        run: |
          curl -L https://sp1.succinct.xyz | bash
          echo "$HOME/.sp1/bin" >> $GITHUB_PATH

      - name: Build guest
        run: |
          cd packages/zk-guest-sp1/guest
          cargo build --release --target riscv32im-succinct-zkvm-elf

      - name: Generate vkey hash
        id: vkey
        run: |
          HASH=$(cargo run --package vkey-generator | grep "0x" | awk '{print $NF}')
          echo "hash=$HASH" >> $GITHUB_OUTPUT

      - name: Check if constants match
        run: |
          EXPECTED=$(grep "SP1_VKEY_HASH" programs/shield-pool/src/constants.rs | grep -o "0x[a-f0-9]*")
          ACTUAL="${{ steps.vkey.outputs.hash }}"
          if [ "$EXPECTED" != "$ACTUAL" ]; then
            echo "❌ VKey hash mismatch!"
            echo "Expected: $EXPECTED"
            echo "Actual:   $ACTUAL"
            echo "Did you forget to update constants.rs?"
            exit 1
          fi
          echo "✅ VKey hash matches constants.rs"
```

## Implementation Details

### Source Code

**File:** `packages/vkey-generator/src/main.rs`

**Key Functions:**

```rust
// Search for guest ELF in multiple locations
fn find_guest_elf() -> Result<Vec<u8>> {
    let paths = [/* ... */];
    for path in &paths {
        if Path::new(path).exists() {
            return Ok(fs::read(path)?);
        }
    }
    Err(anyhow!("Could not find guest ELF"))
}

fn main() -> Result<()> {
    // Create prover client
    let client = ProverClient::from_env();

    // Load guest ELF
    let guest_elf = find_guest_elf()?;

    // Setup: generates proving key and verification key
    let (_, vk) = client.setup(&guest_elf);

    // Compute 32-byte hash
    let vkey_hash = vk.bytes32();

    // Output to console and file
    println!("SP1 Withdraw Circuit VKey Hash: {}", vkey_hash);
    fs::write("target/vkey_hash.txt", &vkey_hash)?;

    Ok(())
}
```

**Reference:** `vkey-generator/src/main.rs`

## Related Documentation

- **[ZK Guest SP1](./zk-guest-sp1.md)** - Withdraw circuit implementation
- **[Shield Pool Program](../onchain/shield-pool.md)** - On-chain verifier that uses vkey hash
- **[Development Workflow](./zk-guest-sp1.md#development-workflow)** - Circuit modification guide
- **[Operations Guide](../operations/runbook.md)** - Production deployment procedures
- **[Packages Overview](./overview.md)** - All Cloak packages

## External Resources

- **[SP1 Verification Keys](https://docs.succinct.xyz/verification/onchain/solana-groth16.html)** - SP1 documentation on vkeys
- **[SP1 Setup Phase](https://docs.succinct.xyz/generating-proofs/setup.html)** - How SP1 generates keys
