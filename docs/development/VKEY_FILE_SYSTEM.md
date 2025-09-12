# File-Based VKey Management System

This document explains the file-based VKey hash management system implemented in Cloak.

## Overview

The system automatically generates and manages SP1 VKey hashes using files, eliminating the need for shell scripts or manual environment variables.

## Architecture

```
packages/vkey-generator/     # VKey generation tool
├── Cargo.toml
└── src/main.rs             # Generates vkey_hash.txt

programs/shield-pool/        # Solana program
├── build.rs                # Reads vkey_hash.txt at build time
└── src/lib.rs              # Uses VKEY_HASH environment variable

vkey_hash.txt               # Generated VKey hash file
```

## How It Works

### 1. VKey Generation
- **Tool**: `packages/vkey-generator`
- **Function**: Generates VKey hash from SP1 guest circuit
- **Output**: Writes `vkey_hash.txt` with the current VKey hash
- **Command**: `cargo run -p vkey-generator --release`

### 2. VKey Consumption
- **Build Script**: `programs/shield-pool/build.rs`
- **Function**: Reads `vkey_hash.txt` at build time
- **Output**: Sets `VKEY_HASH` environment variable for the program
- **Fallback**: Uses hardcoded VKey if file not found

### 3. Program Usage
- **Code**: `programs/shield-pool/src/lib.rs`
- **Function**: Uses `env!("VKEY_HASH")` to get the VKey hash
- **Result**: Program compiled with correct VKey hash

## Usage

### Generate VKey Hash
```bash
# Generate VKey and write to file
make vkey-hash

# Or directly
cargo run -p vkey-generator --release
```

### Build Program
```bash
# Generate VKey + build program
make build

# Or build directly (reads existing vkey_hash.txt)
cd programs/shield-pool && cargo build-sbf
```

### Full Workflow
```bash
# Complete build process
make build
```

## File Locations

The build script looks for `vkey_hash.txt` in these locations (in order):
1. `vkey_hash.txt` (current directory)
2. `../../vkey_hash.txt` (project root)
3. `../../../vkey_hash.txt` (parent directory)
4. `packages/vkey-generator/vkey_hash.txt` (generator directory)

## VKey Generation Process

1. **Find Guest ELF**: Searches multiple paths for the compiled SP1 guest
2. **Load Proving Key**: Uses `ProverClient::from_env().setup(&guest_elf)`
3. **Extract VKey Hash**: Calls `vk.bytes32()` to get the hash
4. **Write File**: Saves the hash to `vkey_hash.txt`

## Build Process

1. **Read VKey File**: `build.rs` searches for `vkey_hash.txt`
2. **Set Environment**: Sets `VKEY_HASH` environment variable
3. **Compile Program**: Rust `env!()` macro includes the VKey
4. **Rebuild Trigger**: `cargo:rerun-if-changed` ensures rebuilds when VKey changes

## Benefits

✅ **No Shell Scripts**: Pure Rust implementation
✅ **Automatic Detection**: Finds VKey files in multiple locations
✅ **Fallback Safety**: Uses hardcoded VKey if file not found
✅ **Build Integration**: Seamlessly integrated with Cargo build system
✅ **Change Detection**: Automatically rebuilds when VKey changes
✅ **Cross-Platform**: Works on all platforms that support Rust

## Example Output

```bash
$ make build
🔑 Generating VKey hash...
    Finished `release` profile [optimized] target(s) in 0.44s
     Running `target/release/vkey-generator`
SP1 Withdraw Circuit VKey Hash: 0x009b498bc2e9a58ee6af8fb35829fab9b96859ce80e12ef4863289ed979fcde6
VKey hash written to: vkey_hash.txt
✅ VKey hash written to vkey_hash.txt
🔨 Building shield pool program...
   Compiling shield-pool v0.1.0 (/Users/marcelofeitoza/Development/solana/cloak/programs/shield-pool)
warning: shield-pool@0.1.0: Using VKey hash from: ../../vkey_hash.txt
    Finished `release` profile [optimized] target(s) in 1.11s
✅ Build complete!
```

## Troubleshooting

### VKey File Not Found
- **Error**: `No vkey_hash.txt found, using fallback VKey hash`
- **Solution**: Run `make vkey-hash` first to generate the file

### Guest ELF Not Found
- **Error**: `Could not find guest ELF in any expected location`
- **Solution**: Ensure SP1 guest is built first: `cargo build -p zk-guest-sp1-guest --release`

### VKey Mismatch
- **Symptom**: Proof verification failures
- **Solution**: Regenerate VKey: `make vkey-hash && make build`

## Integration with CI/CD

```yaml
- name: Generate VKey Hash
  run: cargo run -p vkey-generator --release

- name: Build Shield Pool
  run: cd programs/shield-pool && cargo build-sbf
```

## Files Modified

- `packages/vkey-generator/` - New VKey generation tool
- `programs/shield-pool/build.rs` - Build script for VKey reading
- `programs/shield-pool/src/lib.rs` - Uses `env!("VKEY_HASH")`
- `Makefile` - Updated for file-based workflow
- `Cargo.toml` - Added vkey-generator to workspace
