---
title: Quickstart
description: Spin up a local Cloak environment with the indexer, relay, SP1 prover artifacts, and the web client.
---

# Quickstart

This quickstart guides you through building the core binaries, starting the services, and exercising a end-to-end deposit/withdraw loop on localnet.

## Prerequisites

### Required Tools

- **Rust** 1.75+ with nightly toolchain
- **Solana CLI** 2.3.1 (matches the workspace lock file)
- **Node.js** 18+ (for the web app and Docusaurus site)
- **PostgreSQL** 14+ (Docker Compose recipe available)
- **Succinct SP1 toolchain** (REQUIRED for ZK functionality)

### Installation

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install nightly

# 2. Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 3. Install SP1 toolchain (REQUIRED for ZK functionality)
curl -L https://sp1.succinct.xyz | bash
source ~/.zshenv  # or ~/.bashrc depending on your shell
sp1up

# 4. Install the succinct toolchain
cargo prove install-toolchain

# 5. Add WASM32 target for Solana BPF builds
rustup target add bpfel-unknown-unknown

# 6. Verify installations
rustc --version
solana --version
cargo prove --version
```

### Troubleshooting

**SP1 Toolchain Error:**
If you see `error: override toolchain 'succinct' is not installed`, run:
```bash
# Install SP1 and the succinct toolchain
curl -L https://sp1.succinct.xyz | bash
source ~/.zshenv  # or ~/.bashrc depending on your shell
sp1up
cargo prove install-toolchain
```

**RISC-V Target Error:**
If you see `error: toolchain 'nightly-aarch64-apple-darwin' does not support target 'riscv32im-succinct-zkvm-elf'`, this is normal! SP1 automatically handles the RISC-V target installation. Just run:
```bash
# Build the project - SP1 will handle the RISC-V target automatically
cargo build --release
```

### Optional but Recommended

- `cargo-watch` for hot reload while developing Rust crates
- `just` to run helper recipes from the root `justfile`

## 1. Clone and Bootstrap

```bash
git clone https://github.com/cloak-labz/cloak.git
cd cloak
cargo fetch
```

Generate local keypairs (the repo includes development fixtures):

```bash
solana-keygen new -o user-keypair.json
solana-keygen new -o recipient-keypair.json
solana-keygen new -o admin-keypair.json
```

## 2. Build Core Binaries

```bash
# Programs
cargo build-sbf --manifest-path programs/shield-pool/Cargo.toml
cargo build-sbf --manifest-path programs/scramble-registry/Cargo.toml

# ZK guest/host
cargo build -p zk-guest-sp1-host

# Miner CLI
cargo build -p cloak-miner

# Relay + Indexer services
cargo build -p relay -p indexer
```

> `cargo build-sbf` requires the Solana LLVM toolchain. See the official Solana docs if the toolchain is missing.

## 3. Start Databases (Docker)

```bash
docker compose up -d postgres
```

This starts Postgres on `localhost:5432` using the credentials from `services/relay/config.toml` and `services/indexer/.env`.

## 4. Launch Services

### Indexer

```bash
cd services/indexer
cp env.example .env
cargo run
```

### Relay

```bash
cd ../relay
cp config.toml.example config.toml  # if you use template configuration
cargo run
```

> The relay automatically creates job tables, connects, and initialises the PoW `ClaimFinder` when `SCRAMBLE_REGISTRY_PROGRAM_ID` is set in `config.solana`.

### Web App

```bash
cd ../web
npm install
npm run dev
```

## 5. Seed Shield Pool State

1. Deploy `shield-pool` and `scramble-registry` to localnet (see `programs/*/README.md`).
2. Register miners via `cloak-miner register --network localnet`.
3. Use the indexer admin endpoints to push initial Merkle roots.

## 6. Exercise a Flow

1. Deposit via the web UI (`/transaction`, Deposit tab). The indexer records the new commitment.
2. Generate a withdraw proof using `zk-guest-sp1-host` CLI or the relay local proving endpoint (`POST /jobs/:job_id/prove-local`).
3. Ensure the relay finds a wildcard claim (check logs with `[METRICS] Found available claim`).
4. Submit the withdraw job and confirm the resulting Solana transaction.

## Troubleshooting

- Consult the [Runbook](../operations/runbook.md) for validator/operator workflows.
- Review the [Metrics Guide](../operations/metrics-guide.md) for PoW instrumentation and success rates.
- The relay exposes `GET /health` and job status endpoints for debugging.
- Use `solana logs` and on-chain program logs to diagnose proof verification failures.

## Where to Next

- Dive into the [Workflows](../workflows/deposit.md) for step-by-step deposit/withdraw sequencing.
- Understand the [Zero-Knowledge Layer](../zk/README.md) to customise the circuit.
- Explore the [Services](../offchain/indexer.md) section for API details.
