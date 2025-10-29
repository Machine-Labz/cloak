---
title: ZK Guest SP1
description: SP1 zero-knowledge circuit for Cloak withdraw proofs with guest program and host CLI.
---

# ZK Guest SP1 - Withdraw Circuit

`zk-guest-sp1` implements Cloak's withdraw circuit as an SP1 guest program with a host CLI and library for zero-knowledge proof generation. This package is the cryptographic core of the privacy protocol, enforcing all withdraw constraints in zero-knowledge.

**Source:** `packages/zk-guest-sp1/`

## Overview

The withdraw circuit proves that a user can validly withdraw funds from the shield pool without revealing which deposit is being spent. It enforces six critical constraints using BLAKE3 hashing and SP1's zkVM to generate Groth16 proofs that can be verified on-chain.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Withdraw Circuit                      │
│                                                         │
│  Private Inputs          Circuit Logic      Public Ins  │
│  ┌──────────────┐       ┌──────────┐       ┌────────┐  │
│  │ • sk_spend   │──────▶│ Verify   │──────▶│ root   │  │
│  │ • r (nonce)  │       │ 6 constr-│       │ nf     │  │
│  │ • amount     │       │ aints    │       │ amt    │  │
│  │ • leaf_index │       │ using    │       │ out_h  │  │
│  │ • merkle_path│       │ BLAKE3   │       └────────┘  │
│  └──────────────┘       └──────────┘                    │
│                              ▼                          │
│                    ┌─────────────────┐                  │
│                    │ SP1 Guest (RISC-V)                 │
│                    │ Generates ZK proof                 │
│                    └─────────┬───────┘                  │
└──────────────────────────────┼─────────────────────────┘
                               ▼
                     ┌──────────────────┐
                     │ Groth16 Proof    │
                     │ (260 bytes)      │
                     │ + Public (104 B) │
                     └──────────────────┘
```

### How It Works

1. **User provides private witness:**
   - Secret spending key (`sk_spend`)
   - Randomness nonce (`r`) used in commitment
   - Deposit amount and leaf index in Merkle tree
   - Merkle inclusion proof path

2. **Circuit verifies constraints:**
   - All 6 constraints must pass (see below)
   - Uses BLAKE3 for all hash computations
   - Merkle path verification proves tree membership
   - Amount conservation ensures no inflation

3. **SP1 generates proof:**
   - Executes guest program in zkVM
   - Compiles to Groth16 proof (260 bytes)
   - Commits public inputs (104 bytes)
   - Proof is verifiable on-chain by shield-pool program

4. **Relay submits to chain:**
   - Proof + public inputs sent to Solana
   - On-chain verifier checks SP1 proof
   - Shield-pool program verifies public inputs match
   - Funds released if valid

## Circuit Constraints

The guest program enforces six cryptographic constraints:

### Constraint 1: Spend Key Derivation
```
pk_spend = H(sk_spend)
```
**Purpose:** Derives the public spend key from the secret key.

**Verification:** The circuit computes `pk_spend` from the private `sk_spend` input. This binds the proof to the owner of the secret key without revealing it.

**Reference:** `guest/src/main.rs:93`

### Constraint 2: Commitment Recomputation
```
C = H(amount || r || pk_spend)
```
**Purpose:** Recomputes the deposit commitment using private inputs.

**Verification:**
- Concatenates `amount` (u64 LE), `r` (32 bytes), and `pk_spend` (32 bytes)
- Hashes with BLAKE3 to get commitment `C`
- This commitment must match the leaf in the Merkle tree

**Encoding:** `amount:u64 LE || r:32 || pk_spend:32` → BLAKE3 → 32 bytes

**Reference:** `guest/src/main.rs:96`, `encoding.rs:62-68`

### Constraint 3: Merkle Inclusion Proof
```
MerkleVerify(C, path_elements, path_indices) == root
```
**Purpose:** Proves the commitment exists in the Merkle tree.

**Verification:**
- Starts with commitment `C` as leaf
- Iteratively hashes with siblings in `path_elements`
- Uses `path_indices` (0=left, 1=right) to determine hash order
- Final computed root must equal public `root` input

**Algorithm:**
```rust
current = C
for (sibling, index) in zip(path_elements, path_indices):
    if index == 0:
        current = H(current || sibling)  // current left
    else:
        current = H(sibling || current)  // current right
assert current == root
```

**Reference:** `guest/src/main.rs:99-107`, `encoding.rs:102-131`

### Constraint 4: Nullifier Computation
```
nf = H(sk_spend || leaf_index)
```
**Purpose:** Computes a unique nullifier to prevent double-spends.

**Verification:**
- Concatenates `sk_spend` (32 bytes) and `leaf_index` (u32 LE)
- Hashes with BLAKE3 to get nullifier `nf`
- Must match public `nf` input
- On-chain program stores nullifiers to detect reuse

**Encoding:** `sk_spend:32 || leaf_index:u32 LE` → BLAKE3 → 32 bytes

**Reference:** `guest/src/main.rs:110-113`, `encoding.rs:76-81`

### Constraint 5: Amount Conservation
```
sum(outputs) + fee(amount, fee_bps) == amount
```
**Purpose:** Ensures no inflation - total out equals total in minus fees.

**Verification:**
- Sums all output amounts
- Computes fee from amount and fee basis points
- Checks: `outputs_sum + fee == amount`

**Fee Calculation:**
```
fixed_fee = 2,500,000 lamports (0.0025 SOL)
variable_fee = (amount * fee_bps) / 10,000
total_fee = fixed_fee + variable_fee
```

**Example:** For 1 SOL withdraw with 60 bps (0.6%):
- Fixed: 0.0025 SOL
- Variable: 0.006 SOL (0.6% of 1 SOL)
- Total fee: 0.0085 SOL
- Outputs sum: 0.9915 SOL

**Reference:** `guest/src/main.rs:118-128`, `encoding.rs:94-98`

### Constraint 6: Outputs Hash Binding
```
outputs_hash = H(serialize(outputs))
```
**Purpose:** Binds specific recipients and amounts to the proof.

**Verification:**
- Serializes all outputs as `address:32 || amount:u64` concatenated
- Hashes with BLAKE3
- Must match public `outputs_hash` input
- Prevents proof reuse with different recipients

**Serialization:**
```
output[0].address (32 bytes) ||
output[0].amount (8 bytes LE) ||
output[1].address (32 bytes) ||
output[1].amount (8 bytes LE) ||
... for all outputs
```

**Reference:** `guest/src/main.rs:131-134`, `encoding.rs:85-92`

## Directory Structure

```
packages/zk-guest-sp1/
├── Cargo.toml                    # Workspace configuration
├── README.md                     # Package README
│
├── guest/                        # SP1 guest program (no_std)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # Guest entrypoint with circuit logic
│       └── encoding.rs          # BLAKE3 hashing & serialization (no_std)
│
├── host/                        # Host CLI and library (std)
│   ├── Cargo.toml
│   ├── build.rs                 # Build script for ELF inclusion
│   └── src/
│       ├── lib.rs               # Library API for proof generation
│       ├── main.rs              # (Deprecated - use bin/cloak-zk.rs)
│       ├── encoding.rs          # Shared encoding functions (std)
│       └── bin/
│           ├── cloak-zk.rs      # Main CLI: prove command
│           ├── dump_vkey.rs     # Dump verification key
│           ├── get_vkey_hash.rs # Get vkey hash for program
│           └── generate_examples.rs  # Generate test data
│
├── examples/                    # Example input data
│   ├── private.example.json     # Private witness inputs
│   ├── public.example.json      # Public inputs
│   └── outputs.example.json     # Outputs array
│
├── tests/
│   └── golden.rs               # Integration tests
│
└── out/                        # Generated proof artifacts (gitignored)
    ├── proof.bin               # SP1 proof bundle
    └── public.json             # Public inputs (104 bytes)
```

## Installation

### Prerequisites

**Rust Toolchain:**
```bash
# Install Rust (stable or nightly)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# SP1 requires nightly for some features
rustup install nightly
```

**SP1 Toolchain:**
```bash
# Install SP1
curl -L https://sp1.succinct.xyz | bash
source ~/.zshenv  # or ~/.bashrc depending on your shell
sp1up

# Install the succinct toolchain (REQUIRED)
cargo prove install-toolchain

# Verify installation
cargo prove --version
```

**Build Tools:**
```bash
# RISC-V target for guest program (requires succinct toolchain)
rustup target add riscv32im-succinct-zkvm-elf --toolchain succinct
```

### Build from Source

```bash
# Navigate to repository root
cd cloak

# Build host CLI
cargo build --release --package zk-guest-sp1-host

# Build guest program
cd packages/zk-guest-sp1/guest
cargo build --release --target riscv32im-succinct-zkvm-elf

# Or use SP1's build tool
cd ..
cargo prove build --release
```

### Verify Installation

```bash
# Check host binary
cargo run --package zk-guest-sp1-host --bin cloak-zk -- --version

# Check guest ELF exists
ls -lh target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest
# Should show ~1-2 MB ELF file
```

## Quick Start

### 1. Generate Example Inputs

```bash
# Generate test data with consistent hashes
cargo run --package zk-guest-sp1-host --bin generate_examples

# Output:
# ✅ Generated examples/private.example.json
# ✅ Generated examples/public.example.json
# ✅ Generated examples/outputs.example.json
```

### 2. Generate a Proof

```bash
# Using the CLI
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/proof.bin \
  --pubout out/public.json

# Output:
# 📖 Reading input files...
# ✅ Input files loaded
# 🔧 Setting up SP1 prover client...
# 🔑 Generating proving key (this may take 1-2 minutes)...
# ✅ Proving key generated
# 📝 Preparing circuit inputs...
# 🔨 Generating Groth16 proof (this may take ~2 minutes)...
# 📊 Total cycles: 1,234,567
# ✅ Proof generated!
# 💾 Saving proof to disk...
# Proof generated successfully!
# Proof size: 89234 bytes
# Public inputs size: 104 bytes
```

### 3. Verify the Proof (Development)

```bash
# The relay service verifies proofs, but you can test locally
# This is built into the host library verification
```

## Host CLI Reference

### cloak-zk prove

Generate a zero-knowledge proof for a withdraw transaction.

**Usage:**
```bash
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove \
  --private <PATH> \
  --public <PATH> \
  --outputs <PATH> \
  --proof <OUTPUT_PATH> \
  --pubout <OUTPUT_PATH>
```

**Arguments:**

- `--private <PATH>` - Path to private inputs JSON file
- `--public <PATH>` - Path to public inputs JSON file
- `--outputs <PATH>` - Path to outputs array JSON file
- `--proof <OUTPUT_PATH>` - Where to save the SP1 proof bundle
- `--pubout <OUTPUT_PATH>` - Where to save the 104-byte public inputs

**Example:**
```bash
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/withdraw_proof.bin \
  --pubout out/withdraw_public.json
```

**Output Files:**

**`proof.bin`** - SP1 proof bundle (bincode serialized):
- Contains full `SP1ProofWithPublicValues` structure
- Includes Groth16 proof (260 bytes) embedded
- Can be deserialized with `SP1ProofWithPublicValues::load()`
- Relay uses `cloak-proof-extract` to extract the 260-byte proof

**`public.json`** - Public inputs (104 bytes raw):
- Format: `root:32 || nf:32 || outputs_hash:32 || amount:8`
- This is the canonical public input blob verified on-chain
- All values in little-endian byte order

**Performance:**
- Proof generation: ~2 minutes (local prover)
- Proof generation: ~30-45 seconds (SP1 network TEE)
- Cycle count: ~1-2M cycles (varies by Merkle depth)
- Memory: ~4 GB RAM recommended

**Reference:** `host/src/bin/cloak-zk.rs`

## Data Formats

### Private Inputs (private.json)

Contains secret witness data known only to the user.

```json
{
  "amount": 1000000,
  "r": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "sk_spend": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
  "leaf_index": 42,
  "merkle_path": {
    "path_elements": [
      "1111111111111111111111111111111111111111111111111111111111111111",
      "2222222222222222222222222222222222222222222222222222222222222222",
      "3333333333333333333333333333333333333333333333333333333333333333"
    ],
    "path_indices": [0, 1, 0]
  }
}
```

**Fields:**

- `amount` (u64): Deposit amount in lamports
- `r` (hex32): 32-byte randomness nonce used in commitment
- `sk_spend` (hex32): 32-byte secret spending key
- `leaf_index` (u32): Position of commitment in Merkle tree (0-indexed)
- `merkle_path.path_elements` (array of hex32): Sibling hashes for inclusion proof
- `merkle_path.path_indices` (array of u8): Path directions (0=left, 1=right)

**Security:** This file contains secrets and should NEVER be shared or logged.

### Public Inputs (public.json)

Contains public data that will be verified on-chain.

```json
{
  "root": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "nf": "0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba",
  "outputs_hash": "5555555555555555555555555555555555555555555555555555555555555555",
  "amount": 1000000
}
```

**Fields:**

- `root` (hex32): Current Merkle root from indexer
- `nf` (hex32): Nullifier to prevent double-spends
- `outputs_hash` (hex32): BLAKE3 hash of outputs array
- `amount` (u64): Total amount being withdrawn (must match private)

**Note:** The guest program commits these as a 104-byte blob in the order: `root || nf || outputs_hash || amount`.

### Outputs (outputs.json)

Specifies recipients and amounts for the withdraw.

```json
[
  {
    "address": "recipient_pubkey_base58_or_hex32",
    "amount": 400000
  },
  {
    "address": "another_recipient_pubkey_base58_or_hex32",
    "amount": 594000
  }
]
```

**Fields:**

- `address` (string): Solana pubkey as base58 OR 32-byte hex string
- `amount` (u64): Amount in lamports to send to this recipient

**Constraints:**
- Minimum: 1 output
- Maximum: 10 outputs (protocol limit)
- Sum of outputs must equal `amount - fee`

**Address Formats:**

Both formats are supported:
```json
// Base58 (Solana standard)
"address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"

// Hex (32 bytes)
"address": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
```

## Encoding Specification

All cryptographic operations use **BLAKE3-256** with **little-endian** integer serialization.

### Integer Serialization

```rust
u64 → 8 bytes LE
u32 → 4 bytes LE
u16 → 2 bytes LE
```

**Example:**
```rust
amount = 1,000,000u64
serialized = [0x40, 0x42, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00]
```

### Hash Computations

**Commitment:**
```
C = BLAKE3(amount:u64 LE || r:32 || pk_spend:32)
  = BLAKE3(8 bytes || 32 bytes || 32 bytes)
  = 32 bytes
```

**Spend Key:**
```
pk_spend = BLAKE3(sk_spend:32)
  = 32 bytes
```

**Nullifier:**
```
nf = BLAKE3(sk_spend:32 || leaf_index:u32 LE)
  = BLAKE3(32 bytes || 4 bytes)
  = 32 bytes
```

**Outputs Hash:**
```
outputs_hash = BLAKE3(
    address₀:32 || amount₀:u64 LE ||
    address₁:32 || amount₁:u64 LE ||
    ... for all outputs
)
```

**Merkle Parent:**
```
if path_index == 0:
    parent = BLAKE3(current:32 || sibling:32)
else:
    parent = BLAKE3(sibling:32 || current:32)
```

### Fee Calculation

```rust
fixed_fee = 2,500,000 lamports       // 0.0025 SOL
variable_fee = (amount * fee_bps) / 10,000
total_fee = fixed_fee + variable_fee
```

**Example Calculations:**

| Amount (SOL) | Fee BPS | Fixed Fee | Variable Fee | Total Fee | Outputs Sum |
|--------------|---------|-----------|--------------|-----------|-------------|
| 1.0          | 60 (0.6%) | 0.0025 | 0.006       | 0.0085    | 0.9915      |
| 0.5          | 100 (1%) | 0.0025  | 0.005       | 0.0075    | 0.4925      |
| 10.0         | 60 (0.6%) | 0.0025 | 0.06        | 0.0625    | 9.9375      |

**Reference:** `encoding.rs:94-98`

## Library API

The host package exports a library for programmatic proof generation.

### generate_proof()

Generate an SP1 proof from JSON input strings.

```rust
use zk_guest_sp1_host::generate_proof;

pub fn generate_proof(
    private_inputs: &str,
    public_inputs: &str,
    outputs: &str,
) -> Result<ProofResult>

pub struct ProofResult {
    pub proof_bytes: Vec<u8>,        // SP1 bundle (bincode)
    pub public_inputs: Vec<u8>,      // 104-byte canonical blob
    pub generation_time_ms: u64,     // Time taken to generate
    pub total_cycles: u64,           // SP1 execution cycles
    pub total_syscalls: u64,         // SP1 syscall count
    pub execution_report: String,    // Formatted execution report
}
```

**Example Usage:**
```rust
let private_json = r#"{
    "amount": 1000000,
    "r": "0123...",
    "sk_spend": "fedc...",
    "leaf_index": 42,
    "merkle_path": {...}
}"#;

let public_json = r#"{
    "root": "abcd...",
    "nf": "0987...",
    "outputs_hash": "5555...",
    "amount": 1000000
}"#;

let outputs_json = r#"[
    {"address": "9xQe...", "amount": 400000},
    {"address": "7yRt...", "amount": 594000}
]"#;

let result = generate_proof(private_json, public_json, outputs_json)?;

println!("Proof size: {} bytes", result.proof_bytes.len());
println!("Cycles: {}", result.total_cycles);
println!("Time: {}ms", result.generation_time_ms);

// Save proof
std::fs::write("proof.bin", &result.proof_bytes)?;
```

**Reference:** `host/src/lib.rs:24-82`

### generate_proof_from_data()

Generate proof from serde_json Values (alternative API).

```rust
pub fn generate_proof_from_data(
    private: &serde_json::Value,
    public: &serde_json::Value,
    outputs: &serde_json::Value,
) -> Result<ProofResult>
```

**Example:**
```rust
let private_value: serde_json::Value = serde_json::from_str(private_json)?;
let public_value: serde_json::Value = serde_json::from_str(public_json)?;
let outputs_value: serde_json::Value = serde_json::from_str(outputs_json)?;

let result = generate_proof_from_data(&private_value, &public_value, &outputs_value)?;
```

**Reference:** `host/src/lib.rs:113-123`

## Proving Modes

### Local Prover (CPU)

**Default mode** - Generates proofs locally using your CPU.

```bash
# No environment setup needed
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove ...
```

**Characteristics:**
- Proof generation: ~2 minutes
- Memory: ~4 GB RAM
- CPU-intensive (100% core utilization)
- Free (no network costs)
- Privacy: All computation local

**Use Cases:**
- Development and testing
- Privacy-sensitive scenarios
- Offline proof generation

### SP1 Network (TEE)

**Network mode** - Offloads proof generation to SP1's TEE network.

```bash
# Set prover mode to network
export SP1_PROVER=network
export SP1_PRIVATE_KEY=your_private_key  # From SP1 dashboard

cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove ...
```

**Characteristics:**
- Proof generation: ~30-45 seconds
- No local resource requirements
- Network costs apply (check SP1 pricing)
- Requires internet connection
- Submitted to TEE for proving

**Use Cases:**
- Production deployments
- Faster proof generation
- Resource-constrained clients
- Mobile/web wallets

**Setup:**
1. Sign up at [SP1 Network](https://network.succinct.xyz/)
2. Get API key from dashboard
3. Export `SP1_PRIVATE_KEY` environment variable
4. Set `SP1_PROVER=network`

**Reference:** [SP1 Network Documentation](https://docs.succinct.xyz/generating-proofs/prover-network.html)

## Performance & Optimization

### Cycle Counting

Track execution performance by analyzing cycle counts.

```bash
# Proof generation shows cycle count
cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove ...

# Output:
# 📊 Total cycles: 1,234,567
```

**Typical Cycle Counts:**

| Merkle Depth | Cycles | Proof Time (Local) | Proof Time (Network) |
|--------------|--------|--------------------|---------------------|
| 10 levels    | ~800K  | ~90 seconds        | ~25 seconds         |
| 20 levels    | ~1.2M  | ~120 seconds       | ~35 seconds         |
| 32 levels    | ~1.8M  | ~180 seconds       | ~45 seconds         |

**Cycle Breakdown:**

The majority of cycles are spent on:
1. **Merkle path verification** (~40-50%) - Hashing at each level
2. **Hash computations** (~30-40%) - Commitment, nullifier, outputs hash
3. **Circuit logic** (~10-20%) - Constraint checks and serialization

### Optimization Tips

**1. Minimize Merkle Depth:**
- Use the smallest tree depth that accommodates expected deposits
- Each level adds ~30K-40K cycles
- Trade-off: Smaller depth = fewer max deposits

**2. Reduce Output Count:**
- Each additional output adds serialization overhead
- Typical: 1-3 outputs is optimal
- Maximum: 10 outputs supported

**3. Use SP1 Network for Production:**
- ~2.5x faster than local proving
- Offloads computation from user devices
- Better UX for end users

**4. Enable Release Mode:**
- Always build guest program with `--release`
- Debug builds are significantly slower
- SP1 SDK requires release mode for proving

```bash
# Correct (release mode)
cargo prove build --release

# Incorrect (debug mode - will fail)
cargo prove build
```

### Memory Requirements

**Local Prover:**
- Minimum: 2 GB RAM
- Recommended: 4 GB RAM
- Optimal: 8 GB+ RAM (for faster proving, typically completes in ~2 minutes)

**Guest Program:**
- Stack: ~512 KB
- Heap: ~2 MB (for Merkle path and intermediate hashes)
- Total: &lt;3 MB in zkVM

## Testing

### Unit Tests

Test encoding functions and cryptographic primitives.

```bash
# Run encoding tests
cargo test --package zk-guest-sp1-host

# Test output:
# test encoding::test_hash_commitment_matches_docs ... ok
# test encoding::test_nullifier_matches_docs ... ok
# test encoding::test_outputs_hash_order_sensitive ... ok
# test encoding::test_merkle_verify_ok_and_fails_on_swapped_sibling ... ok
# test encoding::test_fee_calculation ... ok
# test encoding::test_address_parsing ... ok
```

**Reference:** `guest/src/encoding.rs:194-294`

### Integration Tests

Test full prove/verify cycle with golden test data.

```bash
# Run integration tests (requires release mode)
cargo test --package zk-guest-sp1 --release golden

# Test output:
# test test_valid_proof_generation_and_verification ... ok
# test test_invalid_merkle_path_fails ... ok
# test test_conservation_failure ... ok
# test test_invalid_outputs_hash_fails ... ok
```

**What's Tested:**
- Valid proof generation and verification
- Merkle path validation (rejects invalid paths)
- Amount conservation (rejects invalid sums)
- Outputs hash binding (rejects mismatched hashes)

**Reference:** `tests/golden.rs`

### Constraint Tests

Test individual circuit constraints in isolation.

```bash
# Test constraint logic (no proof generation)
cargo test --package zk-guest-sp1

# Tests run fast without SP1 proving
```

**Test Cases:**
- `test_invalid_merkle_path_fails` - Ensures invalid Merkle paths are rejected
- `test_conservation_failure` - Ensures amount conservation is enforced
- `test_invalid_outputs_hash_fails` - Ensures outputs hash must match

**Reference:** `tests/golden.rs:275-355`

## Development Workflow

### 1. Setup Development Environment

```bash
# Install dependencies
rustup install nightly
rustup target add riscv32im-succinct-zkvm-elf
curl -L https://sp1.succinct.xyz | bash && sp1up

# Clone and build
git clone <repo>
cd cloak/packages/zk-guest-sp1
cargo build --release
```

### 2. Modify Circuit Constraints

**Edit guest program:**
```bash
vim guest/src/main.rs  # Main circuit logic
vim guest/src/encoding.rs  # Hash functions
```

**Rebuild guest:**
```bash
cd guest
cargo build --release --target riscv32im-succinct-zkvm-elf
```

**Important:** Guest and host encoding modules must stay in sync. Any changes to hash computations must be replicated in both.

### 3. Test Changes

```bash
# Unit tests (fast)
cargo test --package zk-guest-sp1-host

# Integration tests (slow - generates proofs)
cargo test --package zk-guest-sp1 --release
```

### 4. Update Verification Key

When the guest program changes, the verification key changes.

```bash
# Rebuild guest
cd guest && cargo build --release --target riscv32im-succinct-zkvm-elf

# Generate new vkey hash
cd ../..
cargo run --package vkey-generator

# Output:
# SP1 Withdraw Circuit VKey Hash: 0x0000000000000000000000000000000000000000000000000000000000000000

# Update shield-pool program with new hash
vim programs/shield-pool/src/constants.rs
# Replace SP1_VKEY_HASH constant

# Redeploy shield-pool program
cargo build-sbf
solana program deploy ...
```

### 5. Generate Example Data

```bash
# Create consistent test data
cargo run --package zk-guest-sp1-host --bin generate_examples

# Manually edit if needed
vim examples/private.example.json
vim examples/public.example.json
vim examples/outputs.example.json
```

## Troubleshooting

### "succinct toolchain is not installed"

**Symptom:**
```
error: override toolchain 'succinct' is not installed: the RUSTUP_TOOLCHAIN environment variable specifies an uninstalled toolchain
```

**Solution:**
```bash
# Install SP1 and the succinct toolchain
curl -L https://sp1.succinct.xyz | bash
source ~/.zshenv  # or ~/.bashrc depending on your shell
sp1up
cargo prove install-toolchain

# Add RISC-V target to succinct toolchain
rustup target add riscv32im-succinct-zkvm-elf --toolchain succinct

# Verify installation
cargo prove --version

# Clean and rebuild
cargo clean
cargo build --release
```

---

### "Could not find guest ELF in any expected location"

**Symptom:**
```
Error: Could not find guest ELF in any expected location
```

**Solution:**
```bash
# Build the guest program first
cd packages/zk-guest-sp1/guest
cargo build --release --target riscv32im-succinct-zkvm-elf

# Verify ELF exists
ls -lh target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest

# Or use cargo prove
cd ..
cargo prove build --release
```

---

### "Circuit constraint verification failed"

**Symptom:**
```
Error: Circuit constraint verification failed
Caused by: Merkle path verification failed
```

**Cause:** Input data violates one of the six constraints.

**Solutions:**

**Check Merkle path:**
- Ensure `path_elements` and `path_indices` match the tree
- Verify `root` is the current root from indexer
- Path indices must be 0 (left) or 1 (right) only

**Check amount conservation:**
```rust
// Calculate expected values
fee = fixed_fee + (amount * fee_bps) / 10_000
expected_outputs_sum = amount - fee

// Verify
actual_outputs_sum = sum of all output amounts
assert!(actual_outputs_sum == expected_outputs_sum)
```

**Check outputs hash:**
```rust
// Recompute hash
computed_hash = BLAKE3(serialize_outputs(outputs))
// Must match public.outputs_hash
```

**Check nullifier:**
```rust
// Recompute nullifier
computed_nf = BLAKE3(sk_spend || leaf_index as u32 LE)
// Must match public.nf
```

---

### "SP1 proof generation panicked"

**Symptom:**
```
Error: SP1 proof generation panicked - this usually means invalid input data or circuit constraint failure
```

**Cause:** Guest program panicked during execution, usually due to constraint violation.

**Solution:**
1. Run unit tests to verify encoding logic:
   ```bash
   cargo test --package zk-guest-sp1-host
   ```

2. Check input JSON files are well-formed:
   ```bash
   jq . examples/private.example.json
   jq . examples/public.example.json
   jq . examples/outputs.example.json
   ```

3. Enable debug logging to see which constraint failed:
   ```bash
   RUST_LOG=debug cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove ...
   ```

4. Generate fresh example data:
   ```bash
   cargo run --package zk-guest-sp1-host --bin generate_examples
   ```

---

### "Proof verification failed"

**Symptom:**
Proof generates successfully but verification fails.

**Cause:** Mismatch between guest ELF and verification key.

**Solution:**
```bash
# Ensure guest is rebuilt
cd guest && cargo build --release --target riscv32im-succinct-zkvm-elf

# Regenerate verification key
cd ../..
cargo run --package vkey-generator

# Use the new vkey hash in shield-pool program
# Redeploy shield-pool with updated constant
```

---

### High Memory Usage

**Symptom:**
Process uses excessive RAM during proof generation.

**Management:**
```bash
# Use SP1 network instead of local prover
export SP1_PROVER=network
export SP1_PRIVATE_KEY=your_key

# Or limit system resources (Linux)
systemd-run --scope -p MemoryMax=4G cargo run --package zk-guest-sp1-host --bin cloak-zk -- prove ...
```

---

### Slow Proof Generation

**Symptom:**
Proof generation takes >20 minutes locally.

**Optimizations:**

1. **Use SP1 Network:**
   ```bash
   export SP1_PROVER=network
   # Reduces time from 15min → 3min
   ```

2. **Build with Native Optimizations:**
   ```bash
   RUSTFLAGS="-C target-cpu=native" cargo build --release
   ```

3. **Reduce Merkle Depth:**
   - Use shallower trees if possible
   - Each level adds ~40K cycles

4. **Check CPU Throttling:**
   ```bash
   # Linux: Check CPU frequency scaling
   cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
   # Should be "performance" not "powersave"
   ```

## Advanced Usage

### Custom Circuit Modifications

To add new constraints or modify existing ones:

**1. Edit Guest Program:**
```rust
// guest/src/main.rs
fn verify_circuit_constraints(inputs: &CircuitInputs) -> Result<()> {
    // ... existing constraints ...

    // Add new constraint
    let custom_check = my_custom_verification(&inputs);
    if !custom_check {
        return Err(anyhow!("Custom constraint failed"));
    }

    Ok(())
}
```

**2. Update Encoding Module (if needed):**
```rust
// guest/src/encoding.rs and host/src/encoding.rs
pub fn compute_custom_hash(data: &[u8]) -> [u8; 32] {
    hash_blake3(data)
}
```

**3. Sync Host Encoding:**
Ensure any encoding changes are replicated in `host/src/encoding.rs`.

**4. Update Tests:**
```rust
// tests/golden.rs
#[test]
fn test_custom_constraint() {
    // Test new constraint in isolation
}
```

**5. Regenerate VKey:**
```bash
cargo prove build --release
cargo run --package vkey-generator
# Update shield-pool program with new hash
```

### Parallel Proving

Generate multiple proofs concurrently.

```rust
use rayon::prelude::*;
use zk_guest_sp1_host::generate_proof;

let inputs: Vec<(String, String, String)> = vec![
    (private1, public1, outputs1),
    (private2, public2, outputs2),
    (private3, public3, outputs3),
];

let results: Vec<_> = inputs
    .par_iter()
    .map(|(priv, pub, out)| {
        generate_proof(priv, pub, out)
    })
    .collect();

for result in results {
    match result {
        Ok(proof) => println!("Proof generated: {} cycles", proof.total_cycles),
        Err(e) => eprintln!("Proof failed: {}", e),
    }
}
```

**Note:** Each proof generation is CPU-intensive. Limit parallelism to available CPU cores.

### WASM Integration

The host library includes WASM compatibility.

```toml
# Cargo.toml
[dependencies]
zk-guest-sp1-host = { path = "../../packages/zk-guest-sp1/host" }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_withdraw_proof(
    private_json: &str,
    public_json: &str,
    outputs_json: &str,
) -> Result<Vec<u8>, JsValue> {
    zk_guest_sp1_host::generate_proof(private_json, public_json, outputs_json)
        .map(|r| r.proof_bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

**Note:** WASM proving requires SP1 network mode (local proving not supported in browser).

## Comparison to Other ZK Frameworks

| Feature | SP1 | Circom | Halo2 | Plonky2 |
|---------|-----|--------|-------|---------|
| **Language** | Rust | DSL | Rust | Rust |
| **Proof System** | Groth16 | Groth16 | PLONK | PLONK |
| **Proof Size** | 260 bytes | 260 bytes | ~400 bytes | ~1 KB |
| **Verify Time** | ~1 ms | ~1 ms | ~2 ms | ~10 ms |
| **Prove Time** | 10-15 min | 5-10 min | 15-20 min | 3-5 min |
| **Trusted Setup** | Required | Required | No | No |
| **Recursion** | Yes | Limited | Yes | Yes |
| **Solana Support** | ✅ (sp1-solana) | ❌ | ❌ | ❌ |

**Why SP1 for Cloak:**
- Native Rust (no DSL to learn)
- Excellent Solana integration via `sp1-solana` verifier
- Compact 260-byte proofs (cheaper to verify on-chain)
- Strong tooling and network prover support
- Active development and community

## Related Tools

**Internal Packages:**
- **[vkey-generator](./vkey-generator.md)** - Extract verification key hash from compiled guest ELF
- **[cloak-proof-extract](./cloak-proof-extract.md)** - Parse SP1 bundles to extract 260-byte Groth16 proofs

**External Resources:**
- **[SP1 Documentation](https://docs.succinct.xyz/)** - Official SP1 docs
- **[SP1 Network](https://network.succinct.xyz/)** - TEE proving service
- **[SP1 GitHub](https://github.com/succinctlabs/sp1)** - Source code and examples

## Related Documentation

- **[ZK Design](../zk/design.md)** - High-level ZK protocol design
- **[Encoding Specification](../zk/encoding.md)** - Detailed encoding rules
- **[Shield Pool Program](../onchain/shield-pool.md)** - On-chain verifier integration
- **[Withdraw Workflow](../workflows/withdraw.md)** - End-to-end withdraw flow
- **[Operations Guide](../operations/runbook.md)** - Production deployment
- **[Packages Overview](./overview.md)** - All Cloak packages
