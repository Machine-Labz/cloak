# Cloak Docs

This folder is the developer guide. Start here:

- **🎉 Complete Flow Status:** `COMPLETE_FLOW_STATUS.md` - **PRODUCTION READY**
- **Glossary:** `glossary.md`
- **Roadmap:** `roadmap.md` - **ALL CORE MILESTONES COMPLETE**
- **Changelog:** `CHANGELOG.md` - **Recent changes and improvements**
- **ZK Layer (start here):** `zk/README.md`
- **Non-ZK stubs:** `non-zk/` (Index/Relay/Frontend responsibilities & done criteria)

## 🚀 Quick Start

```bash
# Build and test everything
just build
just test-localnet  # Test on localnet
just test-testnet   # Test on testnet
```

**Program ID:** `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`

## 🆕 Recent Updates (January 2025)

- **Fee Structure Optimized:** 0% deposits, 0.5% + 0.0025 SOL withdrawals
- **Test Architecture Reorganized:** Moved to `tooling/test/` with separate localnet/testnet binaries
- **Dependency Fixes:** Replaced `solana-blake3-hasher` with standard `blake3` crate
- **Indexer Bug Fixes:** Fixed `getMaxLeafIndex()` logic for proper leaf assignment
- **Dual Network Support:** Both localnet and testnet testing fully operational
