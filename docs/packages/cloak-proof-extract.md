---
title: Cloak Proof Extract
description: no_std library for parsing SP1 proof bundles to extract Groth16 proofs and public inputs.
---

# Cloak Proof Extract

`cloak-proof-extract` is a lightweight, `no_std` compatible library that extracts Groth16 proof bytes and public inputs from SP1 proof bundles. It bridges the gap between SP1's proof format and the raw bytes required for on-chain verification.

**Source:** `packages/cloak-proof-extract/`

## Overview

SP1 generates proof bundles (via `SP1ProofWithPublicValues`) that contain metadata, proof bytes, and public inputs in a serialized format. However, the shield-pool program needs:
- **260-byte Groth16 proof** - Raw proof bytes for verification
- **104-byte public inputs** - Canonical `root||nf||outputs_hash||amount` format

This library provides extraction utilities that work both with and without SP1 dependencies, making it suitable for off-chain services and on-chain programs.

### Why This Library?

**Problem:**
- SP1 proof bundles are large (~50-100 KB) and contain extra metadata
- On-chain verifiers need compact, fixed-size proof formats
- Relay services need to extract proofs without full SP1 dependencies
- Need `no_std` compatibility for potential on-chain usage

**Solution:**
- Efficient extraction of 260-byte Groth16 proofs from bundles
- Parse 104-byte public inputs into structured format
- Multiple extraction strategies (heuristic + SP1 deserialization)
- `no_std` compatible with optional `alloc` feature
- Optional SP1 integration for high-level deserialization

### Architecture

```
┌──────────────────────────────────────┐
│  SP1 Proof Bundle (50-100 KB)       │
│  ┌────────────────────────────────┐ │
│  │ Metadata                       │ │
│  ├────────────────────────────────┤ │
│  │ Groth16 Proof (260 bytes) ←──┐│ │
│  ├────────────────────────────────┤ ││
│  │ Public Inputs (104 bytes) ←──┐││ │
│  ├────────────────────────────────┤│││
│  │ Additional SP1 Data           ││││
│  └────────────────────────────────┘│││
└─────────────────────────────────────┘││
                                       ││
         ┌─────────────────────────────┘│
         │  cloak-proof-extract          │
         │  ┌──────────────────────┐     │
         │  │ extract_groth16_260()│◄────┘
         │  └──────────┬───────────┘
         │             ▼
         │  ┌─────────────────────┐
         │  │ [u8; 260]           │  ← Groth16 proof
         │  └─────────────────────┘
         │
         │  ┌──────────────────────┐
         │  │parse_public_inputs() │
         │  └──────────┬───────────┘
         │             ▼
         │  ┌─────────────────────┐
         │  │ PublicInputs {      │
         │  │   root: [u8; 32],   │
         │  │   nf: [u8; 32],     │
         │  │   outputs_hash: ...,│
         │  │   amount: u64       │
         │  │ }                   │
         │  └─────────────────────┘
         └─────────────────────────
```

## Features

### Core Functionality

**`extract_groth16_260()`**
- Extracts 260-byte Groth16 proof from SP1 bundle
- Uses heuristic scanning with known offset fallback
- Rejects all-zero slices to avoid false positives
- Works without SP1 dependencies

**`parse_public_inputs_104()`**
- Parses 104-byte blob into structured format
- Format: `root:32 || nf:32 || outputs_hash:32 || amount:8 LE`
- Returns `PublicInputs` struct with named fields
- Validates byte lengths

**`extract_groth16_260_sp1()`** (with `sp1` feature)
- Deserializes SP1 bundle using `SP1ProofWithPublicValues`
- Directly extracts proof from parsed structure
- More reliable than heuristic for complex bundles

**`parse_public_inputs_104_sp1()`** (with `sp1` feature)
- Extracts public inputs from SP1 bundle
- Avoids manual offset calculations
- Validates format automatically

### Feature Flags

**`std`** (default enabled)
- Enables `Display` and `Error` implementations
- Provides `std::error::Error` trait for errors
- Recommended for off-chain services

**`sp1`**
- Enables SP1 deserialization functions
- Requires `sp1-sdk` dependency
- Recommended for relay services

**`hex`**
- Enables hex serialization via Serde
- Provides `to_hex()` and `from_hex()` helpers
- Useful for JSON APIs

**`alloc`**
- Enables heap allocation for `no_std` environments
- Required for some extraction strategies
- Use when `std` is not available

## Installation

### Add to Cargo.toml

**For relay services (with SP1):**
```toml
[dependencies]
cloak-proof-extract = { path = "../../packages/cloak-proof-extract", features = ["sp1", "hex"] }
```

**For no_std environments:**
```toml
[dependencies]
cloak-proof-extract = { path = "../../packages/cloak-proof-extract", default-features = false, features = ["alloc"] }
```

**For on-chain programs:**
```toml
[dependencies]
cloak-proof-extract = { path = "../../packages/cloak-proof-extract", default-features = false }
```

### Verify Installation

```bash
# Build with default features
cargo build --package cloak-proof-extract

# Build with SP1 feature
cargo build --package cloak-proof-extract --features sp1

# Build for no_std
cargo build --package cloak-proof-extract --no-default-features
```

## Usage

### Basic Extraction (Heuristic)

Extract proof without SP1 dependencies:

```rust
use cloak_proof_extract::{extract_groth16_260, Error};

fn process_proof_bundle(bundle: &[u8]) -> Result<[u8; 260], Error> {
    // Extract 260-byte Groth16 proof
    let proof = extract_groth16_260(bundle)?;

    // Proof is ready for on-chain submission
    Ok(proof)
}

// Example
let bundle = std::fs::read("out/proof.bin")?;
let proof = extract_groth16_260(&bundle)?;
println!("Extracted {} byte proof", proof.len());
```

**Reference:** `cloak-proof-extract/src/lib.rs`

### Extract with SP1 Deserialization

Use SP1's parser for reliable extraction:

```rust
use cloak_proof_extract::{extract_groth16_260_sp1, Error};

fn process_sp1_bundle(bundle: &[u8]) -> Result<[u8; 260], Error> {
    // Deserialize SP1 bundle and extract proof
    let proof = extract_groth16_260_sp1(bundle)?;
    Ok(proof)
}

// Example
let bundle = std::fs::read("out/proof.bin")?;
let proof = extract_groth16_260_sp1(&bundle)?;
println!("Extracted proof: {:?}", proof);
```

**Note:** Requires `sp1` feature flag.

**Reference:** `cloak-proof-extract/src/lib.rs`

### Parse Public Inputs

Extract and parse public inputs from bundle:

```rust
use cloak_proof_extract::{parse_public_inputs_104, PublicInputs};

fn extract_public_inputs(public_blob: &[u8]) -> Result<PublicInputs, Error> {
    // Parse 104-byte public inputs
    let inputs = parse_public_inputs_104(public_blob)?;

    println!("Root: {:?}", inputs.root);
    println!("Nullifier: {:?}", inputs.nf);
    println!("Outputs hash: {:?}", inputs.outputs_hash);
    println!("Amount: {}", inputs.amount);

    Ok(inputs)
}

// Example with known offset
let bundle = std::fs::read("out/proof.bin")?;
let offset = 1234; // Known offset in bundle
let inputs = parse_public_inputs_104(&bundle[offset..offset+104])?;
```

**Reference:** `cloak-proof-extract/src/lib.rs`

### Parse Public Inputs from SP1 Bundle

Extract public inputs without manual offsets:

```rust
use cloak_proof_extract::{parse_public_inputs_104_sp1, PublicInputs};

fn extract_sp1_public_inputs(bundle: &[u8]) -> Result<PublicInputs, Error> {
    // Deserialize and extract public inputs automatically
    let inputs = parse_public_inputs_104_sp1(bundle)?;

    // Verify amount
    assert_eq!(inputs.amount, 1_000_000);

    Ok(inputs)
}

// Example
let bundle = std::fs::read("out/proof.bin")?;
let inputs = parse_public_inputs_104_sp1(&bundle)?;
```

**Reference:** `cloak-proof-extract/src/lib.rs`

### Complete Relay Service Integration

Full example for relay service:

```rust
use cloak_proof_extract::{
    extract_groth16_260_sp1,
    parse_public_inputs_104_sp1,
    PublicInputs,
    Error,
};

struct WithdrawArtifacts {
    proof: [u8; 260],
    public_inputs: PublicInputs,
}

fn process_withdraw_proof(bundle: &[u8]) -> Result<WithdrawArtifacts, Error> {
    // Extract Groth16 proof
    let proof = extract_groth16_260_sp1(bundle)?;

    // Extract public inputs
    let public_inputs = parse_public_inputs_104_sp1(bundle)?;

    // Verify public inputs match expectations
    if public_inputs.amount == 0 {
        return Err(Error::InvalidFormat);
    }

    Ok(WithdrawArtifacts {
        proof,
        public_inputs,
    })
}

// Use in relay service
let bundle = receive_proof_from_client()?;
let artifacts = process_withdraw_proof(&bundle)?;

// Submit to Solana
submit_withdraw_transaction(
    &artifacts.proof,
    &artifacts.public_inputs,
)?;
```

## API Reference

### extract_groth16_260()

Extract 260-byte Groth16 proof using heuristic scanning.

```rust
pub fn extract_groth16_260(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], Error>
```

**Parameters:**
- `sp1_proof_bundle` - Full SP1 proof bundle bytes

**Returns:**
- `Ok([u8; 260])` - Extracted Groth16 proof
- `Err(Error::InvalidFormat)` - Bundle format invalid or proof not found

**Algorithm:**
1. Search for known offset patterns
2. Scan for length-prefixed proof data
3. Validate 260-byte length
4. Reject all-zero slices
5. Return proof bytes

**Example:**
```rust
let bundle = fs::read("proof.bin")?;
let proof = extract_groth16_260(&bundle)?;
assert_eq!(proof.len(), 260);
```

---

### extract_groth16_260_sp1()

Extract proof using SP1 deserialization (requires `sp1` feature).

```rust
#[cfg(feature = "sp1")]
pub fn extract_groth16_260_sp1(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], Error>
```

**Parameters:**
- `sp1_proof_bundle` - Full SP1 proof bundle bytes

**Returns:**
- `Ok([u8; 260])` - Extracted Groth16 proof
- `Err(Error::InvalidFormat)` - Deserialization failed or invalid format

**Advantages:**
- More reliable than heuristic
- Uses SP1's official deserialization
- Handles format changes automatically

**Example:**
```rust
#[cfg(feature = "sp1")]
let proof = extract_groth16_260_sp1(&bundle)?;
```

---

### parse_public_inputs_104()

Parse 104-byte public inputs into structured format.

```rust
pub fn parse_public_inputs_104(bytes: &[u8]) -> Result<PublicInputs, Error>
```

**Parameters:**
- `bytes` - Exactly 104 bytes in format `root||nf||outputs_hash||amount`

**Returns:**
- `Ok(PublicInputs)` - Parsed public inputs
- `Err(Error::InvalidFormat)` - Length mismatch or invalid data

**Format:**
```
Offset 0-31:   root (32 bytes)
Offset 32-63:  nf (32 bytes)
Offset 64-95:  outputs_hash (32 bytes)
Offset 96-103: amount (8 bytes LE)
```

**Example:**
```rust
let public_blob = &bundle[offset..offset+104];
let inputs = parse_public_inputs_104(public_blob)?;
println!("Amount: {}", inputs.amount);
```

---

### parse_public_inputs_104_sp1()

Extract and parse public inputs from SP1 bundle (requires `sp1` feature).

```rust
#[cfg(feature = "sp1")]
pub fn parse_public_inputs_104_sp1(sp1_proof_bundle: &[u8]) -> Result<PublicInputs, Error>
```

**Parameters:**
- `sp1_proof_bundle` - Full SP1 proof bundle bytes

**Returns:**
- `Ok(PublicInputs)` - Parsed public inputs
- `Err(Error::InvalidFormat)` - Deserialization failed

**Advantages:**
- No manual offset calculation needed
- Automatically finds public inputs in bundle
- Validates format

**Example:**
```rust
#[cfg(feature = "sp1")]
let inputs = parse_public_inputs_104_sp1(&bundle)?;
assert_eq!(inputs.root.len(), 32);
```

## Data Structures

### PublicInputs

Structured representation of 104-byte public inputs.

```rust
pub struct PublicInputs {
    pub root: [u8; 32],           // Merkle root
    pub nf: [u8; 32],             // Nullifier
    pub outputs_hash: [u8; 32],   // Outputs commitment
    pub amount: u64,              // Amount (little-endian)
}
```

**Serialization:**
- Implements `Serialize` and `Deserialize`
- With `hex` feature: can serialize to hex strings
- With `std` feature: implements `Debug` and `Display`

**Example:**
```rust
let inputs = PublicInputs {
    root: [0x01; 32],
    nf: [0x02; 32],
    outputs_hash: [0x03; 32],
    amount: 1_000_000,
};

// Serialize to bytes
let bytes = inputs.to_bytes(); // 104 bytes

// With hex feature
let hex_str = serde_json::to_string(&inputs)?;
```

---

### Error

Error type for extraction failures.

```rust
pub enum Error {
    InvalidFormat,  // Bundle format invalid or unexpected
}
```

**With `std` feature:**
- Implements `std::error::Error`
- Implements `Display` for error messages

**Example:**
```rust
match extract_groth16_260(&bundle) {
    Ok(proof) => println!("Success: {} bytes", proof.len()),
    Err(Error::InvalidFormat) => eprintln!("Invalid bundle format"),
}
```

## Extraction Strategies

### Heuristic Scanning

**How it works:**
1. **Known Offset Search:**
   - Checks common SP1 bundle offsets (e.g., after metadata)
   - Validates 260-byte proof at each offset

2. **Length-Prefixed Search:**
   - Scans for length prefix patterns (e.g., `0x04 0x01` for 260)
   - Reads following 260 bytes
   - Validates non-zero data

3. **Validation:**
   - Rejects all-zero slices (likely padding)
   - Checks proof length exactly 260 bytes
   - Returns first valid match

**Pros:**
- No SP1 dependency required
- Fast for well-formed bundles
- Suitable for `no_std` environments

**Cons:**
- May fail on unusual bundle formats
- Requires manual updates if SP1 format changes
- Less reliable than deserialization

**Reference:** `cloak-proof-extract/src/lib.rs`

### SP1 Deserialization

**How it works:**
1. **Deserialize Bundle:**
   - Use `bincode::deserialize::<SP1ProofWithPublicValues>()`
   - Parses full SP1 structure

2. **Extract Proof:**
   - Access `.proof` field directly
   - Convert to 260-byte array

3. **Extract Public Inputs:**
   - Access `.public_values` field
   - Parse as 104-byte blob

**Pros:**
- Most reliable extraction method
- Handles all SP1 bundle formats
- Automatically adapts to SP1 updates

**Cons:**
- Requires `sp1-sdk` dependency
- Not suitable for `no_std` environments
- Larger binary size

**Reference:** `cloak-proof-extract/src/lib.rs`

## Testing

### Unit Tests

Test extraction functions with fixture data:

```bash
# Run all tests
cargo test --package cloak-proof-extract

# Test with SP1 feature
cargo test --package cloak-proof-extract --features sp1

# Test without std
cargo test --package cloak-proof-extract --no-default-features --features alloc
```

**Test Coverage:**
- Heuristic extraction on valid bundles
- SP1 deserialization extraction
- Public inputs parsing
- Error handling for invalid bundles
- Edge cases (empty bundles, all-zero data)

**Reference:** `cloak-proof-extract/tests/`

### Integration Tests

Test with real SP1 proof bundles:

```rust
#[test]
fn test_extract_real_proof() {
    // Load fixture bundle
    let bundle = std::fs::read("tests/fixtures/proof.bin")
        .expect("fixture not found");

    // Extract proof
    let proof = extract_groth16_260(&bundle)
        .expect("extraction failed");

    // Verify length
    assert_eq!(proof.len(), 260);

    // Verify non-zero
    assert!(!proof.iter().all(|&b| b == 0));
}

#[cfg(feature = "sp1")]
#[test]
fn test_extract_sp1_proof() {
    let bundle = std::fs::read("tests/fixtures/proof.bin")
        .expect("fixture not found");

    let proof = extract_groth16_260_sp1(&bundle)
        .expect("SP1 extraction failed");

    assert_eq!(proof.len(), 260);
}
```

### Benchmark Tests

Compare extraction strategies:

```bash
# Run benchmarks
cargo bench --package cloak-proof-extract --features sp1
```

**Typical Results:**
- Heuristic extraction: ~50-100 μs
- SP1 deserialization: ~500-1000 μs
- Public inputs parsing: ~10-20 μs

## Use Cases

### 1. Relay Service

Extract proofs from client submissions:

```rust
// In relay service worker
async fn process_withdraw_job(job: &WithdrawJob) -> Result<(), Error> {
    // Client uploaded SP1 bundle
    let bundle = fetch_proof_bundle(&job.proof_upload_url).await?;

    // Extract proof for on-chain submission
    let proof = extract_groth16_260_sp1(&bundle)?;
    let public_inputs = parse_public_inputs_104_sp1(&bundle)?;

    // Build Solana transaction
    let tx = build_withdraw_tx(&proof, &public_inputs)?;

    // Submit to chain
    submit_transaction(&tx).await?;

    Ok(())
}
```

**Reference:** `services/relay/src/worker.rs`

### 2. Proof Verification Service

Validate client proofs before queueing:

```rust
fn validate_proof_bundle(bundle: &[u8]) -> Result<bool, Error> {
    // Try to extract proof
    let proof = extract_groth16_260_sp1(bundle)?;

    // Check proof is not all zeros
    if proof.iter().all(|&b| b == 0) {
        return Ok(false);
    }

    // Extract public inputs
    let inputs = parse_public_inputs_104_sp1(bundle)?;

    // Validate amount is positive
    if inputs.amount == 0 {
        return Ok(false);
    }

    // Additional validation...
    Ok(true)
}
```

### 3. CLI Tool

Convert SP1 bundles to raw proof files:

```rust
// Example: proof-converter CLI
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input_file = &args[1];
    let output_file = &args[2];

    // Load SP1 bundle
    let bundle = fs::read(input_file)?;

    // Extract proof
    let proof = extract_groth16_260_sp1(&bundle)?;

    // Save raw proof
    fs::write(output_file, &proof)?;

    println!("Extracted {} byte proof to {}", proof.len(), output_file);

    Ok(())
}
```

Usage:
```bash
cargo run --bin proof-converter -- input.bin output_proof.bin
```

### 4. On-Chain Program (Future)

Validate proof format before verification:

```rust
// In Solana program (no_std)
#![no_std]

use cloak_proof_extract::{parse_public_inputs_104, Error};

#[program]
pub mod shield_pool {
    fn withdraw(ctx: Context<Withdraw>, proof: [u8; 260], public_blob: [u8; 104]) -> Result<()> {
        // Parse public inputs
        let inputs = parse_public_inputs_104(&public_blob)
            .map_err(|_| ErrorCode::InvalidPublicInputs)?;

        // Use parsed inputs
        let root = inputs.root;
        let nf = inputs.nf;

        // Verify proof...

        Ok(())
    }
}
```

**Note:** Requires `default-features = false` for `no_std` compatibility.

## Troubleshooting

### "Invalid bundle format" Error

**Symptom:**
```rust
Err(Error::InvalidFormat)
```

**Causes:**
1. Bundle is not a valid SP1 proof
2. Bundle is corrupted or truncated
3. Bundle format changed (SP1 SDK version mismatch)

**Solutions:**

**Verify bundle is valid:**
```bash
# Check file size (should be 50-100 KB)
ls -lh proof.bin

# Check file is not empty or all zeros
hexdump -C proof.bin | head
```

**Try SP1 deserialization:**
```rust
// If heuristic fails, try SP1 feature
#[cfg(feature = "sp1")]
let proof = extract_groth16_260_sp1(&bundle)?;
```

**Regenerate proof:**
```bash
# Generate fresh proof bundle
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove ...
```

---

### Public Inputs Length Mismatch

**Symptom:**
```
Error: InvalidFormat (expected 104 bytes, got 96)
```

**Cause:** Public inputs blob is not exactly 104 bytes.

**Solution:**

**Check blob length:**
```rust
println!("Public blob length: {}", public_blob.len());
assert_eq!(public_blob.len(), 104);
```

**Extract from correct offset:**
```rust
// Wrong: guessing offset
let inputs = parse_public_inputs_104(&bundle[1000..1104])?;

// Right: use SP1 extraction
let inputs = parse_public_inputs_104_sp1(&bundle)?;
```

---

### All-Zero Proof Rejected

**Symptom:**
Heuristic extraction fails on valid bundle.

**Cause:** Proof happens to be all zeros (extremely rare but possible).

**Solution:**
Use SP1 deserialization instead of heuristic:

```rust
// Heuristic may reject all-zero proof
let proof = extract_groth16_260(&bundle); // May fail

// SP1 deserialization doesn't check for zeros
#[cfg(feature = "sp1")]
let proof = extract_groth16_260_sp1(&bundle)?; // Works
```

---

### SP1 Feature Not Available

**Symptom:**
```
error: cannot find function `extract_groth16_260_sp1` in crate `cloak_proof_extract`
```

**Cause:** `sp1` feature not enabled.

**Solution:**
```toml
# Add sp1 feature
[dependencies]
cloak-proof-extract = { path = "...", features = ["sp1"] }
```

Or compile with feature:
```bash
cargo build --features sp1
```

## Advanced Usage

### Custom Extraction Logic

Implement custom extraction for specialized bundles:

```rust
use cloak_proof_extract::{PublicInputs, Error};

fn extract_custom_proof(bundle: &[u8]) -> Result<([u8; 260], PublicInputs), Error> {
    // Custom offset based on your bundle format
    let proof_offset = 512;
    let public_offset = 1024;

    // Extract proof
    let mut proof = [0u8; 260];
    if bundle.len() < proof_offset + 260 {
        return Err(Error::InvalidFormat);
    }
    proof.copy_from_slice(&bundle[proof_offset..proof_offset+260]);

    // Extract public inputs
    let public_blob = &bundle[public_offset..public_offset+104];
    let inputs = parse_public_inputs_104(public_blob)?;

    Ok((proof, inputs))
}
```

### Batch Processing

Process multiple proof bundles efficiently:

```rust
use rayon::prelude::*;
use cloak_proof_extract::extract_groth16_260_sp1;

fn process_bundles(bundles: Vec<Vec<u8>>) -> Vec<Result<[u8; 260], Error>> {
    bundles
        .par_iter()
        .map(|bundle| extract_groth16_260_sp1(bundle))
        .collect()
}

// Use
let bundles = load_proof_bundles()?;
let proofs = process_bundles(bundles);

for (i, result) in proofs.iter().enumerate() {
    match result {
        Ok(proof) => println!("Bundle {}: extracted {} bytes", i, proof.len()),
        Err(e) => eprintln!("Bundle {}: failed - {:?}", i, e),
    }
}
```

### Hex Encoding (with `hex` feature)

Serialize proofs as hex for JSON APIs:

```rust
#[cfg(feature = "hex")]
use cloak_proof_extract::PublicInputs;

let inputs = parse_public_inputs_104_sp1(&bundle)?;

// Serialize to JSON with hex encoding
let json = serde_json::to_string(&inputs)?;
// {"root":"0x0123...","nf":"0xabcd...","outputs_hash":"0x5678...","amount":1000000}

// Deserialize from JSON
let inputs: PublicInputs = serde_json::from_str(&json)?;
```

## Related Documentation

- **[ZK Guest SP1](./zk-guest-sp1.md)** - Circuit that generates SP1 proofs
- **[Relay Service](../offchain/relay.md)** - Uses this library to process proofs
- **[Shield Pool Program](../onchain/shield-pool.md)** - On-chain verifier
- **[Withdraw Workflow](../workflows/withdraw.md)** - End-to-end proof flow
- **[Packages Overview](./overview.md)** - All Cloak packages

## External Resources

- **[SP1 Proof Format](https://docs.succinct.xyz/generating-proofs/proof-types.html)** - SP1 proof bundle structure
- **[Groth16 Proofs](https://docs.succinct.xyz/verification/onchain/solana-groth16.html)** - Groth16 on Solana
