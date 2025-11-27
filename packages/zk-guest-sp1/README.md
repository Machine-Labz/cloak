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

**CRITICAL:** This package requires the SP1 toolchain to be installed. Without it, you'll get the error:
```
error: override toolchain 'succinct' is not installed
```

**Install SP1 toolchain:**
```bash
# Install SP1
curl -L https://sp1.succinct.xyz | bash
source ~/.zshenv  # or ~/.bashrc depending on your shell
sp1up

# Install the succinct toolchain
cargo prove install-toolchain

# Add RISC-V target (requires succinct toolchain)
rustup target add riscv32im-succinct-zkvm-elf --toolchain succinct

# Verify installation
cargo prove --version
```

**Additional requirements:**
- Rust (stable or nightly)
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

## Troubleshooting

### Common Build Errors

**1. SP1 Toolchain Not Installed:**
```
error: override toolchain 'succinct' is not installed
```
**Solution:**
```bash
curl -L https://sp1.succinct.xyz | bash
source ~/.zshenv  # or ~/.bashrc depending on your shell
sp1up
cargo prove install-toolchain
```

**2. RISC-V Target Not Supported:**
```
error: toolchain 'nightly-aarch64-apple-darwin' does not support target 'riscv32im-succinct-zkvm-elf'
```
**Solution:**
```bash
rustup target add riscv32im-succinct-zkvm-elf --toolchain succinct
```

**3. Build Script Failures:**
```
Failed to run rustc --version
```
**Solution:**
- Ensure `RUSTUP_TOOLCHAIN` environment variable is not set incorrectly
- Try `cargo clean` and rebuild
- Verify SP1 installation with `sp1 --version`

**4. Different VKey Hash Between Developers (Same Branch):**
```
SP1 Withdraw Circuit VKey Hash: 0x<different_hash>
```
**Problem:** The vkey_hash is computed by `ProverClient::setup()` which depends on **BOTH**:
1. The compiled ELF binary
2. **The SP1 SDK version** used to compute the hash

Even with identical source code and ELF, different SP1 SDK versions will produce different vkey_hashes!

**Root Causes:**
- Different SP1 SDK versions in `Cargo.lock` (even if `Cargo.toml` matches)
- Different SP1 toolchain versions (`sp1 --version`)
- Different Rust toolchain versions in the `succinct` toolchain
- Different pre-built ELF artifacts

**Solution - Ensure Consistent VKey Hash:**

1. **Verify SP1 SDK versions match:**
   ```bash
   # Check resolved SP1 SDK version in Cargo.lock
   cargo tree -p sp1-sdk
   
   # Check SP1 toolchain version
   sp1 --version
   
   # Compare with your friend - these MUST match!
   ```

2. **Sync Cargo.lock (CRITICAL):**
   ```bash
   # Ensure Cargo.lock is committed and synced
   git add Cargo.lock
   git commit -m "Lock SP1 SDK version"
   git push
   
   # Friend should pull and use exact same Cargo.lock
   git pull
   cargo build --locked  # Use --locked to prevent updates
   ```

3. **Use the same pre-built ELF (Recommended):**
   ```bash
   # Share the pre-built ELF from .artifacts/zk-guest-sp1-guest
   # Ensure everyone uses the exact same binary file
   # Verify with: sha256sum packages/zk-guest-sp1/.artifacts/zk-guest-sp1-guest
   ```

4. **Or build from source with identical environment:**
   ```bash
   # Clean all build artifacts
   cargo clean -p zk-guest-sp1-guest
   cargo clean -p zk-guest-sp1-host
   rm -rf packages/zk-guest-sp1/.artifacts
   rm -rf packages/zk-guest-sp1/host/target
   rm -rf packages/zk-guest-sp1/guest/target
   
   # Ensure same Cargo.lock (git pull)
   # Ensure same SP1 toolchain (sp1up)
   
   # Force rebuild from source with locked dependencies
   ZK_GUEST_FORCE_BUILD=1 cargo build -p zk-guest-sp1-host --release --locked
   
   # Verify vkey_hash
   cargo run -p zk-guest-sp1-host --release --bin get_vkey_hash -- --capability
   ```

5. **Diagnostic checklist:**
   ```bash
   # Run diagnostic to compare with friend
   cargo run -p zk-guest-sp1-host --release --bin get_vkey_hash -- --capability
   
   # Compare:
   # - ELF SHA256 (must match)
   # - SP1 SDK version from Cargo.lock (must match)
   # - SP1 toolchain version (must match)
   ```

**Important:** The vkey_hash must match between all parties (provers and verifiers) for proofs to be valid. The most common cause of mismatched vkey_hashes on the same branch is **different SP1 SDK versions in Cargo.lock** - always commit and sync Cargo.lock!

### Environment Variables

**Optional SP1 configuration:**
```bash
export SP1_PROVER=network  # or 'local' for local proving
export SP1_NETWORK_RPC=https://rpc.succinct.xyz
```

## Development

The encoding module is shared between guest and host to ensure consistency. Unit tests verify hash computations match the specification, and integration tests ensure the full prove/verify cycle works correctly.

For debugging, the guest program will panic with descriptive messages if constraints fail, helping identify issues during development.