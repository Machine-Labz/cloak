# Cloak

Cloak is a **privacy-preserving exit router** on Solana. This repo hosts the whole system, with a strong focus on the **Zero-Knowledge layer**:

- **ZK Notes (UTXO-style):** commitments, nullifiers, Merkle proofs
- **Withdraw circuit (SP1):** inclusion + nullifier + conservation
- **On-chain verification:** Pinocchio program `shield-pool`
- **Indexer API:** append-only Merkle tree & proofs
- **Relay:** submits withdraw txs (no Jito in MVP)

## 🚀 Getting Started

### Prerequisites

**Required Tools:**
- Rust (stable or nightly)
- Solana CLI tools
- SP1 toolchain (for ZK proving)

**Installation:**

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

# 5. RISC-V target is automatically installed with SP1
# No manual installation needed - SP1 handles this automatically

# 6. Verify installations
rustc --version
solana --version
cargo prove --version
```

**Build the project:**
```bash
git clone <repo-url>
cd cloak
cargo build --release
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

**Build Issues:**
- Ensure all prerequisites are installed
- Try `cargo clean` before rebuilding
- Check that `RUSTUP_TOOLCHAIN` environment variable is not set incorrectly

## Quick links

- ZK overview: `docs/zk/README.md`
- Circuit spec: `docs/zk/circuit-withdraw.md`
- Encoding contract: `docs/zk/encoding.md`
- Merkle tree & proofs: `docs/zk/merkle.md`
- SP1 prover details: `docs/zk/prover-sp1.md`
- On-chain verifier & program: `docs/zk/onchain-verifier.md`
- API contracts (Indexer/Relay): `docs/zk/api-contracts.md`
- Threat model: `docs/zk/threat-model.md`
- Roadmap: `docs/roadmap.md`

> Build order (MVP): Merkle+Indexer → Deposit tx/event → SP1 withdraw circuit → On-chain verifier → Relay → Web wiring.

## 📚 Documentation Site

- Run `yarn` inside `docs-site/` and `yarn start` to launch the Docusaurus portal backed by the markdown docs in `docs/`.
- Generated site covers architecture, workflows, on-chain programs, services, PoW, tooling, and operations.
