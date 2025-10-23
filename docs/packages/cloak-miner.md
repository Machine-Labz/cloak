---
title: Cloak Miner CLI
description: Standalone proof-of-work miner that discovers wildcard claims for the scramble registry.
---

# Cloak Miner CLI

`cloak-miner` is a standalone Rust CLI application that miners run to earn fees by producing proof-of-work claims for the Cloak protocol. Inspired by [Ore](https://ore.supply/), it operates independently from the relay service, creating a decentralized mining ecosystem.

**Source:** `packages/cloak-miner/`

## Overview

The miner continuously generates cryptographic proofs-of-work that are stored as "claims" in the scramble-registry program. These claims are later consumed by the relay service during withdraw transactions, and miners earn fees when their claims are used.

### Architecture

```
┌──────────────┐
│ Cloak Miner  │  ← Runs independently (like Ore miners)
│              │
│ - Mines PoW  │
│ - Reveals    │
│ - Tracks     │
└──────┬───────┘
       │
       │ Submit mine_claim + reveal_claim transactions
       ▼
┌──────────────┐
│ Scramble     │
│ Registry     │  ← On-chain program
│              │
│ - Validates  │
│ - Stores     │
└──────┬───────┘
       ▲
       │ consume_claim CPI (when withdrawals happen)
       │
┌──────────────┐
│ Shield Pool  │  ← Relay service triggers this
│ Withdraw     │
└──────────────┘
```

### How It Works

1. **Mining Phase:**
   - Miner runs `cloak-miner` CLI continuously
   - Fetches current difficulty from on-chain registry
   - Fetches recent SlotHash (anti-precomputation)
   - Mines valid nonces using BLAKE3
   - Submits `mine_claim` transaction (commits hash)
   - Submits `reveal_claim` transaction (reveals solution)

2. **Consumption Phase:**
   - Users submit withdrawals to the relay
   - Relay searches for available revealed claims
   - Shield-pool program validates and consumes claim via CPI
   - Miner earns fees when claim is consumed

## Installation

### Build from Source

```bash
# Navigate to repository root
cd cloak

# Build release binary
cargo build --release --package cloak-miner

# Binary location
# target/release/cloak-miner
```

### Verify Installation

```bash
./target/release/cloak-miner --version
# cloak-miner 0.1.0
```

## Quick Start

### 1. Generate Keypair

```bash
# Generate new miner keypair
solana-keygen new -o ~/.config/solana/miner.json

# Fund it with SOL for transaction fees
solana airdrop 1 ~/.config/solana/miner.json --url devnet
```

### 2. Register as Miner (One-Time)

```bash
# Mainnet
cloak-miner --keypair ~/.config/solana/miner.json register

# Devnet
cloak-miner --network devnet --keypair ~/.config/solana/miner.json register

# Localnet (requires SCRAMBLE_PROGRAM_ID env var)
SCRAMBLE_PROGRAM_ID=<program_id> \
  cloak-miner --network localnet --keypair ~/.config/solana/miner.json register
```

This creates your miner PDA on-chain.

### 3. Start Mining

```bash
# Mainnet (default)
cloak-miner --keypair ~/.config/solana/miner.json mine

# Devnet with custom settings
cloak-miner --network devnet \
  --keypair ~/.config/solana/miner.json \
  --timeout 30 \
  --interval 10 \
  mine

# Localnet
SCRAMBLE_PROGRAM_ID=<program_id> \
  cloak-miner --network localnet \
  --rpc-url http://localhost:8899 \
  --keypair ~/.config/solana/miner.json \
  mine
```

## Command Reference

### register

Register a new miner account on-chain (one-time operation).

**Usage:**
```bash
cloak-miner [OPTIONS] register
```

**Example:**
```bash
cloak-miner --network devnet --keypair ./miner.json register
```

**What it does:**
- Derives miner PDA from keypair
- Creates miner account on scramble-registry
- Records registration slot
- Initializes miner statistics

**Cost:** ~0.002 SOL for account creation + transaction fees

---

### mine

Start the mining loop to discover and submit PoW claims.

**Usage:**
```bash
cloak-miner [OPTIONS] mine
```

**Options:**
- `--timeout <SECONDS>` - Mining timeout per attempt (default: 30)
- `--interval <SECONDS>` - Delay between mining rounds (default: 10)
- `--target-claims <N>` - Target number of active claims (future feature, default: 5)

**Example:**
```bash
cloak-miner --network devnet \
  --keypair ./miner.json \
  --timeout 60 \
  --interval 5 \
  mine
```

**Mining Loop:**
1. Fetch current difficulty and slot hash
2. Mine nonce until hash < difficulty target
3. Submit `mine_claim` transaction
4. Wait for reveal window
5. Submit `reveal_claim` transaction
6. Repeat

**Logs:**
```
🔍 Mining started (difficulty: 0x10000000...)
⛏️  Found solution! Nonce: 123456789 (hash: 0x00123...)
📤 Submitting mine_claim transaction...
✅ Claim mined: claimPda123...
⏳ Waiting for reveal window...
📤 Submitting reveal_claim transaction...
✅ Claim revealed and ready for consumption
```

---

### status

Check miner registration status and current difficulty.

**Usage:**
```bash
cloak-miner [OPTIONS] status
```

**Example:**
```bash
cloak-miner --network devnet --keypair ./miner.json status
```

**Output:**
```
Miner Status:
  Authority: miner123...
  Registered: Yes
  Total Mined: 42
  Total Consumed: 38
  Registered At Slot: 1000000

Registry Status:
  Current Difficulty: 0x10000000...
  Total Claims: 1234
  Active Claims: 567
```

## Global Options

These options apply to all commands:

**Network Selection:**
```bash
--network <NETWORK>     # mainnet, devnet, or localnet (default: mainnet)
--rpc-url <URL>        # Custom RPC URL (overrides network default)
```

**Keypair:**
```bash
--keypair <PATH>        # Path to miner keypair file (required)
```

**Logging:**
```bash
# Set via environment variable
RUST_LOG=info          # info, debug, trace
```

## Environment Variables

Configure miner via environment variables:

```bash
# Network selection
export CLOAK_NETWORK=devnet        # mainnet, devnet, or localnet

# Custom RPC URL
export SOLANA_RPC_URL=https://api.devnet.solana.com

# Keypair path
export MINER_KEYPAIR_PATH=~/.config/solana/miner.json

# Localnet only: Program ID
export SCRAMBLE_PROGRAM_ID=<program_id>

# Logging
export RUST_LOG=info               # trace, debug, info, warn, error
```

**Then simply:**
```bash
cloak-miner mine
```

## Mining Algorithm

### Preimage Structure (137 bytes)

```
Domain:       "CLOAK:SCRAMBLE:v1"  (17 bytes)
Slot:         u64 LE               (8 bytes)
Slot Hash:    [u8; 32]             (32 bytes)
Miner Pubkey: [u8; 32]             (32 bytes)
Batch Hash:   [u8; 32]             (32 bytes) - [0; 32] for wildcard
Nonce:        u128 LE              (16 bytes)
```

### Difficulty Check

```rust
BLAKE3(preimage) < difficulty_target  // 256-bit LE comparison
```

### Example Difficulties

- `[0x10, 0x00, ..., 0x00]` ≈ 1/16 chance per hash
- `[0x01, 0x00, ..., 0x00]` ≈ 1/256 chance per hash
- `[0x00, 0x01, ..., 0x00]` ≈ 1/65536 chance per hash

### Wildcard vs Specific Claims

**Wildcard Claims:**
- `batch_hash = [0; 32]`
- Can be used for ANY withdraw
- Higher utility and demand

**Specific Claims:**
- `batch_hash = BLAKE3(job_ids...)`
- Only usable for specific batch
- Targeted mining (future feature)

**Current Implementation:** All claims are wildcard.

## Mining Economics

### Revenue Model

Miners earn fees when their claims are consumed during withdrawals:

**Fee Distribution:**
- Configured in scramble-registry (`fee_share_bps`)
- Paid from withdraw amount to miner
- Sent to miner's authority account

**Fee Amount:**
- TBD: Depends on protocol configuration
- Could be fixed per claim or percentage of withdraw

### Cost Model

**Per Claim Costs:**
- `mine_claim` transaction: ~5,000 lamports
- `reveal_claim` transaction: ~5,000 lamports
- **Total: ~0.00001 SOL per claim**

### Profitability Factors

**Depends on:**
1. **Difficulty:** Higher = fewer successful mines
2. **Network Demand:** More withdrawals = more claim consumption
3. **Your Hash Rate:** CPU speed determines attempts per second
4. **Transaction Fees:** Solana network congestion
5. **Fee Distribution:** Protocol fee configuration

**Example Calculation:**
```
Cost per claim: 0.00001 SOL
Fee per consumption: 0.0001 SOL (example)
Breakeven: 1 claim consumed per 10 mined
Current consumption rate: Check via `status` command
```

## Architecture Details

### Module Structure

```
packages/cloak-miner/src/
├── main.rs          - CLI entry point and argument parsing
├── lib.rs           - Module exports
├── engine.rs        - Mining engine (BLAKE3 + nonce search)
├── rpc.rs          - RPC helpers (registry, SlotHash)
├── batch.rs         - Batch commitment (BLAKE3 of job IDs)
├── instructions.rs  - Instruction builders
├── manager.rs       - Claim lifecycle management
└── constants.rs     - Program IDs and constants
```

**Reference:** `packages/cloak-miner/src/`

### Mining Engine (`engine.rs`)

**Key Functions:**
```rust
pub fn mine_solution(
    difficulty: &[u8; 32],
    slot: u64,
    slot_hash: &[u8; 32],
    miner_pubkey: &[u8; 32],
    batch_hash: &[u8; 32],
    timeout: Duration,
) -> Option<(u128, [u8; 32])>
```

**Algorithm:**
1. Build preimage with domain, slot, hashes, and starting nonce
2. Loop: Hash preimage with BLAKE3
3. Check if hash < difficulty target
4. If yes: Return (nonce, hash)
5. If no: Increment nonce and retry
6. Timeout after specified duration

**Optimization:**
- Single-threaded (can be parallelized in future)
- Nonce space: 0 to u128::MAX
- Early termination on timeout

### RPC Module (`rpc.rs`)

**Key Functions:**
```rust
pub async fn fetch_registry_difficulty(
    rpc_client: &RpcClient,
    registry_program_id: &Pubkey,
) -> Result<[u8; 32]>

pub async fn fetch_slot_hash(
    rpc_client: &RpcClient,
) -> Result<(u64, [u8; 32])>

pub async fn fetch_miner_account(
    rpc_client: &RpcClient,
    miner_pda: &Pubkey,
) -> Result<MinerAccount>
```

### Instruction Builders (`instructions.rs`)

**Key Functions:**
```rust
pub fn build_mine_claim_ix(...) -> Instruction
pub fn build_reveal_claim_ix(...) -> Instruction
pub fn build_register_miner_ix(...) -> Instruction
```

## Advanced Usage

### Running Multiple Miners

Run multiple instances with different keypairs:

```bash
# Terminal 1
cloak-miner --keypair ./miner1.json mine

# Terminal 2
cloak-miner --keypair ./miner2.json mine

# Terminal 3
cloak-miner --keypair ./miner3.json mine
```

**Note:** Each miner needs its own keypair and SOL for fees.

### Custom Mining Strategy

**Current:** Mines wildcard claims continuously

**Future Enhancements:**
- Listen to API/queue for specific withdraw batches
- Mine targeted claims for higher fees
- Coordinate with other miners to avoid duplicate work
- Pool mining with shared revenue

### Monitoring & Metrics

**Logging Levels:**
```bash
# Info level (default - recommended)
RUST_LOG=info cloak-miner mine

# Debug level (verbose)
RUST_LOG=debug cloak-miner mine

# Trace level (very verbose)
RUST_LOG=trace cloak-miner mine
```

**Key Metrics:**
- Mining attempts per second
- Successful mines per hour
- Claim consumption rate
- Fee revenue earned
- SOL spent on transactions

### Performance Tuning

**CPU Optimization:**
```bash
# Build with native CPU optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --package cloak-miner
```

**Mining Parameters:**
```bash
# Longer timeout for higher difficulty
cloak-miner --timeout 120 mine

# Shorter interval for faster rounds
cloak-miner --interval 1 mine
```

**Future:** GPU acceleration, SIMD optimizations

## Troubleshooting

### "Miner not registered"

**Symptom:**
```
Error: Miner account not found
```

**Solution:**
```bash
# Register first
cloak-miner --network devnet --keypair ./miner.json register
```

---

### "Mining timeout"

**Symptom:**
```
⏰ Mining timeout after 30s (no solution found)
```

**Cause:** Difficulty is too high for current hash rate

**Solution:**
- This is expected and normal
- Wait for next round
- Or increase timeout: `--timeout 60`
- Difficulty will adjust down if few solutions found

---

### "Failed to fetch registry"

**Symptom:**
```
Error: Failed to query registry program
```

**Solutions:**
1. Check program ID is correct:
   ```bash
   solana program show <SCRAMBLE_PROGRAM_ID>
   ```

2. Ensure registry is initialized:
   ```bash
   # Should have registry account
   solana account <REGISTRY_PDA>
   ```

3. Verify RPC connection:
   ```bash
   solana cluster-version
   ```

---

### "Insufficient lamports"

**Symptom:**
```
Error: Insufficient funds for transaction
```

**Solution:**
```bash
# Check balance
solana balance <YOUR_MINER_PUBKEY>

# Fund keypair (devnet)
solana airdrop 1 <YOUR_MINER_PUBKEY> --url devnet

# Fund keypair (mainnet - use faucet or transfer)
```

---

### "Reveal window expired"

**Symptom:**
```
Error: RevealWindowExpired
```

**Cause:** Too much time passed between mine_claim and reveal_claim

**Solution:**
- Reduce `--interval` to reveal claims faster
- Check network congestion (transactions delayed)
- Ensure miner is running continuously

---

### High CPU Usage

**Expected:** Mining is CPU-intensive by design

**Management:**
```bash
# Limit mining timeout
cloak-miner --timeout 30 mine

# Add delays between rounds
cloak-miner --interval 15 mine

# Use nice/cpulimit on Linux
nice -n 19 cloak-miner mine
```

## Comparison to Ore

| Feature | Ore | Cloak Miner |
|---------|-----|-------------|
| **Algorithm** | Equix | BLAKE3 |
| **Target** | Global bus | Per-claim registry |
| **Difficulty** | Dynamic | Registry-based |
| **Revenue** | Token rewards | Fee distribution |
| **Anti-precomp** | Recent hash | SlotHash sysvar |
| **Lifecycle** | Single submit | Mine → Reveal → Consume |
| **Decentralization** | Independent miners | Independent miners |

## Development

### Build & Test

```bash
# Build
cargo build --package cloak-miner

# Run tests
cargo test --package cloak-miner

# Run with output
cargo test --package cloak-miner -- --nocapture
```

### Example Usage

```rust
use cloak_miner::engine::mine_solution;

let difficulty = [0x10, 0x00, /* ... */];
let slot = 1000000;
let slot_hash = [/* ... */];
let miner_pubkey = [/* ... */];
let batch_hash = [0u8; 32]; // wildcard

let solution = mine_solution(
    &difficulty,
    slot,
    &slot_hash,
    &miner_pubkey,
    &batch_hash,
    Duration::from_secs(30),
);

if let Some((nonce, proof_hash)) = solution {
    println!("Found nonce: {}", nonce);
    println!("Proof hash: {:?}", proof_hash);
}
```

## Roadmap

**Planned Features:**
- [ ] Claim enumeration and status tracking
- [ ] Fee distribution reporting
- [ ] Pool mining support
- [ ] Multi-threaded mining engine
- [ ] GPU acceleration (CUDA/OpenCL)
- [ ] Difficulty estimation and profitability calculator
- [ ] Web dashboard for monitoring
- [ ] Automatic claim consumption tracking
- [ ] Batch-specific claim mining
- [ ] Docker container for easy deployment

## Related Documentation

- **[Packages Overview](./overview.md)** - All Cloak packages
- **[PoW Overview](../pow/overview.md)** - Mining system architecture
- **[Scramble Registry](../onchain/scramble-registry.md)** - On-chain program
- **[Relay Service](../offchain/relay.md)** - How claims are consumed
- **[PoW Withdraw Workflow](../workflows/pow-withdraw.md)** - End-to-end flow
