# Cloak Documentation

Comprehensive developer documentation for the Cloak privacy protocol.

## 📚 Getting Started

- **[Introduction](overview/introduction.md)** - High-level overview and system goals
- **[Quickstart](overview/quickstart.md)** - Set up your local development environment
- **[System Architecture](overview/system-architecture.md)** - Component-level system design
- **[Complete Flow Status](COMPLETE_FLOW_STATUS.md)** - Current production status
- **[Zero-Knowledge Layer](zk/README.md)** - Protocol internals and ZK circuit design

## 🚀 Quick Start

```bash
# Build and test everything
just build
just test-localnet  # Test on localnet
just test-testnet   # Test on testnet
```

**Program ID:** `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`

## ⚡ Key Features

- **Privacy-Preserving Withdrawals:** Deposit and withdraw SOL without linking transactions
- **Zero-Knowledge Proofs:** SP1-powered Groth16 proofs verified on-chain
- **Wildcard Mining:** Economic incentives through proof-of-work claims
- **Production Ready:** Fully functional end-to-end privacy protocol
