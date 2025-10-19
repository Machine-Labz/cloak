# Cloak Miner

Standalone PoW miner for the Cloak protocol's scrambler gate system. Inspired by [Ore](https://ore.supply/), this is an independent CLI tool that miners run to earn fees.

## Architecture

```
┌──────────────┐
│ Cloak Miner  │  ← You run this (Ore-style standalone)
│              │
│ - Mines PoW  │
│ - Reveals    │
│ - Tracks     │
└──────────────┘
       │
       │ Submit mine_claim + reveal_claim txs
       ↓
┌──────────────┐
│ Scramble     │
│ Registry     │  ← On-chain program
│              │
│ - Validates  │
│ - Stores     │
└──────────────┘
       ↑
       │ consume_claim CPI
       │
┌──────────────┐
│ Shield Pool  │  ← Users withdraw
│ Withdraw     │
└──────────────┘
```

## How It Works

### 1. Mining Phase
Miners run `cloak-miner` to continuously:
- Fetch current difficulty from on-chain registry
- Fetch recent SlotHash (anti-precomputation)
- Mine valid nonces using BLAKE3
- Submit `mine_claim` transaction (commit to hash)
- Submit `reveal_claim` transaction (reveal solution)

### 2. Consumption Phase
When users withdraw from the shield pool:
- They reference an existing revealed claim
- The shield-pool program validates via `consume_claim` CPI
- Miners earn fees when their claims are consumed

## Installation

```bash
# Build from source
cargo build --release --package cloak-miner

# Binary will be at:
# target/release/cloak-miner
```

## Usage

### 1. Generate/Prepare Keypair

```bash
# Generate new keypair
solana-keygen new -o ~/.config/solana/miner.json

# Fund it with SOL for transaction fees
solana airdrop 1 ~/.config/solana/miner.json --url devnet
```

### 2. Register as Miner (One-time)

```bash
# Mainnet (default)
cloak-miner --keypair ~/.config/solana/miner.json register

# Devnet
cloak-miner --network devnet --keypair ~/.config/solana/miner.json register

# Localnet (requires SCRAMBLE_PROGRAM_ID env var)
SCRAMBLE_PROGRAM_ID=<YOUR_LOCAL_PROGRAM_ID> \
  cloak-miner --network localnet --keypair ~/.config/solana/miner.json register
```

This creates your miner PDA on-chain.

### 3. Start Mining

```bash
# Mainnet (default)
cloak-miner --keypair ~/.config/solana/miner.json mine

# Devnet
cloak-miner --network devnet --keypair ~/.config/solana/miner.json mine --timeout 30 --interval 10

# Localnet with custom RPC
SCRAMBLE_PROGRAM_ID=<YOUR_LOCAL_PROGRAM_ID> \
  cloak-miner --network localnet --rpc-url http://localhost:8899 --keypair ~/.config/solana/miner.json mine
```

**Parameters:**
- `--network` - Network to use: `mainnet` (default), `devnet`, or `localnet`
- `--rpc-url` - Custom RPC URL (overrides network default)
- `--keypair` - Path to miner keypair
- `--timeout` - Mining timeout per attempt (seconds, default: 30)
- `--interval` - Delay between mining rounds (seconds, default: 10)
- `--target-claims` - Number of active claims to maintain (future feature, default: 5)

**Environment Variables:**
```bash
# Network selection
export CLOAK_NETWORK=devnet           # mainnet, devnet, or localnet

# Optional: Custom RPC URL
export SOLANA_RPC_URL=https://api.devnet.solana.com

# Keypair path
export MINER_KEYPAIR_PATH=~/.config/solana/miner.json

# For localnet only: Program ID
export SCRAMBLE_PROGRAM_ID=<YOUR_LOCAL_PROGRAM_ID>

# Then simply:
cloak-miner mine
```

### 4. Check Status

```bash
# Mainnet
cloak-miner --keypair ~/.config/solana/miner.json status

# Devnet
cloak-miner --network devnet --keypair ~/.config/solana/miner.json status

# Localnet
SCRAMBLE_PROGRAM_ID=<ID> cloak-miner --network localnet --keypair ~/.config/solana/miner.json status
```

Shows:
- Miner registration status
- Current registry difficulty
- Active claims (future feature)

## Mining Economics

### Revenue Model
Miners earn fees when their claims are consumed:
- Users pay a fee to use revealed claims
- Fee distribution TBD (could be claim-by-claim or pooled)
- Difficulty adjusts based on network demand

### Cost Model
Miners pay for:
- `mine_claim` transaction: ~5,000 lamports
- `reveal_claim` transaction: ~5,000 lamports
- Total: ~0.00001 SOL per claim

### Profitability
Depends on:
- Current difficulty (higher = fewer successful mines)
- Network demand (more withdrawals = more claim consumption)
- Your hash rate (CPU speed)
- Transaction fees

## Mining Algorithm

### Preimage Structure (137 bytes)
```
Domain:       "CLOAK:SCRAMBLE:v1" (17 bytes)
Slot:         u64 LE              (8 bytes)
Slot Hash:    [u8; 32]            (32 bytes)
Miner Pubkey: [u8; 32]            (32 bytes)
Batch Hash:   [u8; 32]            (32 bytes)
Nonce:        u128 LE             (16 bytes)
```

### Difficulty Check
```rust
BLAKE3(preimage) < difficulty_target  // 256-bit LE comparison
```

### Example Difficulty
- `[0x10, 0x00, ..., 0x00]` = ~1/16 chance per hash
- `[0x01, 0x00, ..., 0x00]` = ~1/256 chance per hash
- `[0x00, 0x01, ..., 0x00]` = ~1/65536 chance per hash

## Advanced Usage

### Running Multiple Miners
You can run multiple miner instances with different keypairs:

```bash
# Terminal 1
cloak-miner --keypair ./miner1.json mine

# Terminal 2
cloak-miner --keypair ./miner2.json mine

# Terminal 3
cloak-miner --keypair ./miner3.json mine
```

### Custom Mining Strategy
The current implementation mines with auto-generated job IDs. For production:
- Listen to an API/queue for actual withdraw requests
- Mine claims for specific user job batches
- Coordinate with other miners to avoid duplicate work

### Monitoring
Use logging levels for debugging:

```bash
# Info level (default)
RUST_LOG=info cloak-miner --keypair ./miner.json mine

# Debug level (verbose)
RUST_LOG=debug cloak-miner --keypair ./miner.json mine

# Trace level (very verbose)
RUST_LOG=trace cloak-miner --keypair ./miner.json mine
```

## Development

### Project Structure
```
packages/cloak-miner/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Module exports
│   ├── engine.rs        # Mining engine (BLAKE3 + nonce search)
│   ├── rpc.rs           # RPC helpers (fetch registry, SlotHash)
│   ├── batch.rs         # Batch commitment (BLAKE3 of job IDs)
│   ├── instructions.rs  # Instruction builders
│   └── manager.rs       # ClaimManager (lifecycle)
├── Cargo.toml
└── README.md
```

### Testing

```bash
# Run unit tests
cargo test --package cloak-miner

# Run with output
cargo test --package cloak-miner -- --nocapture

# Integration tests (requires localnet)
cargo test --package cloak-miner -- --ignored
```

### Building Release Binary

```bash
cargo build --release --package cloak-miner

# Optimized build with LTO
RUSTFLAGS="-C target-cpu=native" cargo build --release --package cloak-miner
```

## Troubleshooting

### "Miner not registered"
Run `cloak-miner register` first.

### "Mining timeout"
Difficulty is too high. This is expected - just wait for the next round.

### "Failed to fetch registry"
- Check program ID is correct
- Ensure registry is initialized
- Verify RPC connection

### "Insufficient lamports"
Fund your keypair with more SOL:
```bash
solana airdrop 1 <YOUR_PUBKEY> --url devnet
```

## Comparison to Ore

| Feature | Ore | Cloak Miner |
|---------|-----|-------------|
| Algorithm | Equix | BLAKE3 |
| Target | Global bus | Per-claim |
| Difficulty | Dynamic | Registry-based |
| Revenue | Token rewards | Fee distribution |
| Anti-precomp | Recent hash | SlotHash sysvar |
| Claim lifecycle | N/A | Mine → Reveal → Consume |

## Roadmap

- [ ] Claim enumeration and status tracking
- [ ] Fee distribution mechanism
- [ ] Pool mining support
- [ ] Multi-threaded mining engine
- [ ] GPU acceleration
- [ ] Difficulty estimation and profitability calculator
- [ ] Web dashboard for monitoring

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
