# Cloak Miner - Quick Start

## Command Structure

```
cloak-miner [GLOBAL OPTIONS] <COMMAND> [COMMAND OPTIONS]
```

**Global options come BEFORE the command!**

## Quick Examples

### Mainnet
```bash
# Register (one-time)
cloak-miner --keypair ~/.config/solana/miner.json register

# Mine
cloak-miner --keypair ~/.config/solana/miner.json mine

# Check status
cloak-miner --keypair ~/.config/solana/miner.json status
```

### Devnet
```bash
# Register
cloak-miner --network devnet --keypair ~/.config/solana/miner.json register

# Mine with custom params
cloak-miner --network devnet --keypair ~/.config/solana/miner.json mine --timeout 60 --interval 15

# Status
cloak-miner --network devnet --keypair ~/.config/solana/miner.json status
```

### Localnet
```bash
# Set program ID (required for localnet)
export SCRAMBLE_PROGRAM_ID=YourProgramIdHere123...

# Register
cloak-miner --network localnet --keypair ./miner.json register

# Mine
cloak-miner --network localnet --keypair ./miner.json mine

# Status
cloak-miner --network localnet --keypair ./miner.json status
```

## Using Environment Variables

```bash
# Set once
export CLOAK_NETWORK=devnet
export MINER_KEYPAIR_PATH=~/.config/solana/miner.json

# Then just
cloak-miner mine
cloak-miner status
cloak-miner register
```

## Common Mistakes

❌ **WRONG** (options after command):
```bash
cloak-miner mine --network devnet --keypair ./miner.json
```

✅ **CORRECT** (options before command):
```bash
cloak-miner --network devnet --keypair ./miner.json mine
```

## Full Help

```bash
# Main help
cloak-miner --help

# Command-specific help
cloak-miner --keypair ./miner.json mine --help
```
