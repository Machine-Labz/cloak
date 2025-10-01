# Changelog

## January 2025

### Major Changes

#### Fee Structure Optimization
- **Deposits:** Reduced from variable fee to 0% (no cost to users)
- **Withdrawals:** Optimized to 0.5% variable + 0.0025 SOL fixed fee
- **Implementation:** Updated across all components:
  - Solana program (`programs/shield-pool/src/instructions/withdraw.rs`)
  - SP1 guest program (`packages/zk-guest-sp1/guest/src/encoding.rs`)
  - SP1 host program (`packages/zk-guest-sp1/host/src/encoding.rs`)

#### Test Architecture Reorganization
- **Before:** Single `test_complete_flow_rust/` directory with hardcoded configurations
- **After:** Moved to `tooling/test/` with separate binaries:
  - `localnet_test.rs` - Local network testing
  - `testnet_test.rs` - Testnet testing
- **Benefits:** Cleaner separation, easier maintenance, better CI/CD integration

#### Dependency Management
- **Removed:** `solana-blake3-hasher` dependency (caused deployment issues)
- **Added:** Standard `blake3` crate for consistent hashing
- **Updated:** All BLAKE3 calls to use `blake3::hash()` instead of `solana_blake3_hasher::hash()`

#### Indexer Improvements
- **Fixed:** `getMaxLeafIndex()` logic in `services/indexer/src/db/storage.ts`
- **Added:** Better error handling and debug logging
- **Resolved:** Duplicate key errors in Merkle tree operations

#### Program ID Management
- **Updated:** Program ID to `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`
- **Consistent:** Used across all test files and program configurations
- **Deployment:** Updated deployment commands to use `testnet-program-keypair.json`

### Technical Details

#### Fee Calculation Formula
```rust
// Consistent across all components
let fixed_fee = 2_500_000; // 0.0025 SOL
let variable_fee = (amount * 5) / 1_000; // 0.5% = 5/1000
let total_fee = fixed_fee + variable_fee;
```

#### BLAKE3 Integration
```rust
// Before (problematic)
use solana_blake3_hasher as blake3;
let hash = solana_blake3_hasher::hash(input);

// After (standard)
use blake3;
let hash = blake3::hash(input);
```

#### Test Structure
```toml
# tooling/test/Cargo.toml
[[bin]]
name = "localnet-test"
path = "src/localnet_test.rs"

[[bin]]
name = "testnet-test"
path = "src/testnet_test.rs"
```

### Files Modified

#### Core Program Files
- `programs/shield-pool/src/lib.rs` - Updated BLAKE3 import
- `programs/shield-pool/src/instructions/withdraw.rs` - Updated fee calculation and BLAKE3 calls
- `programs/shield-pool/Cargo.toml` - Removed `solana-blake3-hasher` dependency

#### SP1 Components
- `packages/zk-guest-sp1/guest/src/encoding.rs` - Updated fee calculation
- `packages/zk-guest-sp1/host/src/encoding.rs` - Updated fee calculation

#### Indexer Service
- `services/indexer/src/db/storage.ts` - Fixed `getMaxLeafIndex()` logic
- `services/indexer/src/lib/merkle.ts` - Added debug logging

#### Test Files
- `tooling/test/src/localnet_test.rs` - Localnet test configuration
- `tooling/test/src/testnet_test.rs` - Testnet test configuration
- `tooling/test/Cargo.toml` - Dual binary configuration

#### Documentation
- `docs/COMPLETE_FLOW_STATUS.md` - Updated with current status
- `docs/roadmap.md` - Updated milestone completion
- `docs/glossary.md` - Updated fee terminology
- `docs/README.md` - Added recent updates section

### Testing Commands

```bash
# Build everything
just build

# Test localnet (requires local validator on port 8899)
just test-localnet

# Test testnet (requires testnet SOL)
just test-testnet

# Start local validator
just start-validator

# Deploy to local validator
just deploy-local
```

### Impact

- **User Experience:** Reduced fees make the protocol more competitive
- **Developer Experience:** Cleaner test structure and better error handling
- **Reliability:** Fixed indexer bugs prevent duplicate key errors
- **Maintainability:** Standard dependencies and consistent fee calculations
- **Scalability:** Dual network support enables comprehensive testing
