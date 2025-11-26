# Cloak Localnet Setup Guide

Quick reference for running a complete local development environment.

## 1. Start Local Validator

```bash
solana-test-validator --reset
```

Keep this running in a separate terminal.

## 2. Build & Deploy Programs

```bash
# Build Solana programs
cargo build-sbf

# Deploy scramble-registry
solana program deploy target/deploy/scramble_registry.so \
  --program-id scramb1eReg1stryPoWM1n1ngSo1anaC1oak11111111.json \
  --url http://127.0.0.1:8899

# Deploy shield-pool  
solana program deploy target/deploy/shield_pool.so \
  --program-id c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json \
  --url http://127.0.0.1:8899
```

## 3. Initialize Programs

```bash
# Initialize scramble-registry
cargo run --package cloak-miner --example init_registry -- localnet

# Initialize shield-pool
cargo run --package cloak-miner --example init_shield_pool -- localnet
```

## 4. Setup Miner

```bash
# Fund miner wallet
solana airdrop 100 $(solana-keygen pubkey miner.json) --url http://127.0.0.1:8899

# Register miner (creates escrow with 1 SOL)
cargo run -p cloak-miner -- \
  --network localnet \
  --keypair miner.json \
  register --initial-escrow 1.0
```

## 5. Start Indexer

```bash
cd services/indexer
cargo run
```

Indexer runs on port 3001 by default.

## 6. Run Miner

```bash
INDEXER_URL=http://localhost:3001 cargo run -p cloak-miner -- \
  --network localnet \
  --keypair miner.json \
  mine
```

## Quick Reference

| Service | Port | Command |
|---------|------|---------|
| Solana Validator | 8899 | `solana-test-validator` |
| Indexer | 3001 | `cargo run -p indexer` |
| Relay | 3002 | `cargo run -p relay` |

| Program | ID |
|---------|-----|
| Scramble Registry | `9yoeUduVanEN5RGp144Czfa5GXNiLdGmDMAboM4vfqsm` |
| Shield Pool | `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp` |



## Miner Data Locations

```
~/.cloak-miner/
└── notes-<PUBKEY>.json    # Decoy deposit notes
```
