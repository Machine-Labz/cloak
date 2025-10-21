# SP1 TEE Integration Fix - Verification Guide

## Problem Fixed

The indexer was **always falling back to local proof generation** despite having SP1 TEE properly configured. The root cause was missing the `.private_key()` method call when building the ProverClient.

## Changes Made

### File: `services/indexer/src/sp1_tee_client.rs`

#### 1. Fixed `generate_proof()` method (lines 67-80)

**Before:**
```rust
let client = ProverClient::builder()
    .network()
    .private()
    .build();  // ❌ Missing .private_key() - causes silent fallback to local
```

**After:**
```rust
// Get the private key from config
let private_key = self.config.private_key.as_ref()
    .ok_or_else(|| anyhow::anyhow!("NETWORK_PRIVATE_KEY is required for TEE proving"))?;

let client = ProverClient::builder()
    .network()
    .private()
    .private_key(private_key)  // ✅ Now properly authenticated!
    .build();
```

#### 2. Fixed `health_check()` method (lines 151-173)

Applied the same fix to ensure health checks also use proper TEE authentication.

### File: `services/indexer/SP1_TEE_INTEGRATION.md`

Updated documentation to explain the critical importance of the `.private_key()` method call and why it's required.

## How to Verify the Fix

### Step 1: Rebuild the indexer

```bash
cd /Users/marcelofeitoza/Development/solana/cloak
cargo build --package indexer --release
```

### Step 2: Ensure environment variables are set

**IMPORTANT:** Environment configuration is centralised in the repository root `.env`. Add the TEE variables there before running the indexer from any working directory (use `services/indexer/env.example` as a reference if needed).

Add these variables:

```bash
# SP1 TEE Configuration
SP1_TEE_ENABLED=true
SP1_TEE_WALLET_ADDRESS=0xYourWalletAddressHere
SP1_TEE_RPC_URL=https://rpc.sp1-lumiere.xyz
SP1_TEE_TIMEOUT_SECONDS=300
NETWORK_PRIVATE_KEY=your_private_key_here
```

**Verification:** When you start the indexer, check the startup logs. You should see:
```
SP1 TEE: enabled=true, wallet=0xYourWalletAddress, private_key_present=true
```

**If you see `enabled=false`**, the root `.env` file is not being loaded.

### Step 3: Start the indexer

```bash
cd services/indexer
cargo run --release
```

### Step 4: Send a test proof request

In another terminal:

```bash
curl -X POST http://localhost:3001/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "private_inputs": "{\"test\": \"private\"}",
    "public_inputs": "{\"test\": \"public\"}",
    "outputs": "{\"test\": \"output\"}"
  }' | jq '.proof_method'
```

### Step 5: Verify the response

**Expected output (TEE working):**
```json
"tee"
```

**Old broken behavior (before fix):**
```json
"local"
```

### Step 6: Check the logs

Look for these log messages indicating TEE is being used:

```
🔐 TEE is enabled - attempting TEE proof generation
   Wallet: 0xYourWalletAddress
   Private key present: true
✅ TEE client created successfully
Building TEE client with private_key...
✅ TEE client built successfully
✅ TEE proof generation succeeded
🎉 TEE proof generation request completed successfully
```

**vs. logs indicating fallback to local (broken behavior):**

```
🏠 TEE disabled, using local proof generation
🎉 Local proof generation request completed successfully
```

## Technical Details

### Why the fix works

The SP1 SDK requires **two critical configurations** for TEE proving:

#### 1. Client Authentication (`.private_key()`)

The `ProverClient::builder()` chain requires explicit authentication:

1. `.network()` - Configures network-based proving
2. `.private()` - Enables private proving mode (TEE)
3. `.private_key(key)` - **REQUIRED**: Authenticates with the TEE using your private key
4. `.build()` - Builds the client

**Without step 3**, the SDK cannot authenticate with the TEE infrastructure at `tee.sp1-lumiere.xyz`, so it silently falls back to local proving.

#### 2. Fulfillment Strategy (`.strategy(FulfillmentStrategy::Reserved)`)

When generating proofs, TEE requires the `Reserved` fulfillment strategy:

```rust
client.prove(&pk, &stdin)
    .groth16()
    .strategy(FulfillmentStrategy::Reserved)  // ⚠️ REQUIRED for TEE!
    .run()
```

**Without this**, you'll get the error:
```
Private proving is available with reserved fulfillment strategy only.
Use FulfillmentStrategy::Reserved.
```

### Reference Implementation

This fix is based on the official SP1 TEE example:
`/tmp/sp1-tee-private-proving/bin/server/tests/fibonacci.rs`

```rust
let client = ProverClient::builder()
    .network()
    .private()
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
    .build();
```

## Build Verification

Confirmed the code compiles successfully:

```bash
$ cargo build --package indexer
   Compiling indexer v0.1.0
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.58s
```

## Next Steps

1. Deploy the updated indexer to your testnet environment
2. Monitor the logs to confirm TEE is being used
3. Verify `proof_method: "tee"` in API responses
4. Monitor TEE usage and costs on the SP1 network dashboard

## Additional Resources

- [SP1 TEE Documentation](https://docs.succinct.xyz/docs/sp1/prover-network/private-proving)
- [SP1 TEE Example Repository](https://github.com/succinctlabs/sp1-tee-private-proving)
- [Indexer TEE Integration Guide](./SP1_TEE_INTEGRATION.md)
