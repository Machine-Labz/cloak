# Roadmap (ZK-first)

## ✅ COMPLETED MILESTONES

**M0 – Merkle & Indexer** ✅ **COMPLETE**
- ✅ Build append-only tree, `/merkle/root`, `/merkle/proof/:index`, `/notes/range?start&end`
- ✅ Route-based deposit handling via `/deposit`
- ✅ Fixed `getMaxLeafIndex()` logic for proper leaf assignment

**M1 – Deposit Path** ✅ **COMPLETE**
- ✅ `transact_deposit` instruction + event (leaf_commit, encrypted_output)
- ✅ FE shows "Private balance" via local scan
- ✅ 0% fee structure implemented

**M2 – SP1 Withdraw Circuit** ✅ **COMPLETE**
- ✅ Circuit: inclusion, nullifier, conservation, outputs_hash
- ✅ Local prove/verify harness + golden tests
- ✅ BLAKE3-256 hashing with standard `blake3` crate

**M3 – On-chain Verifier + Program** ✅ **COMPLETE**
- ✅ Pinocchio `shield-pool::withdraw` + CPI to SP1 verifier
- ✅ Roots ring, nullifier shards, payouts & fee
- ✅ Program ID: `c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp`
- ✅ Optimized fee structure: 0.5% + 0.0025 SOL fixed
- ✅ Standard `blake3` crate integration
- ✅ Dual network testing (localnet + testnet)

## 🚀 CURRENT STATUS: PRODUCTION READY

**Core Protocol:** Fully functional with dual network support (localnet + testnet)
**Testing:** Comprehensive test suite with separate localnet/testnet binaries
**Dependencies:** All issues resolved, using standard crates
**Fee Structure:** Optimized and consistent across all components
**Architecture:** Clean separation of concerns with proper tooling structure

## 🔮 FUTURE ENHANCEMENTS

**M4 – Relay + API (2–4 days)**
- `POST /withdraw`, `GET /status/:id`, queue (no Jito)
- Mint receipt (optional)

**M5 – Hardening (1 sprint)**
- Encoding invariants, rate limits, metrics, threat-model doc

**M6 – Frontend Integration**
- User interface for deposits/withdrawals
- Wallet integration

**M7 – Multi-token Support**
- Beyond SOL to other SPL tokens