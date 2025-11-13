---
title: Project Roadmap
description: Development milestones for the Cloak privacy protocol.
---

# Project Roadmap

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

**M4 – Enhanced Relay Service**
- Advanced transaction management APIs
- Status tracking and job queuing
- Transaction receipt system

**M5 – Security Hardening**
- Comprehensive rate limiting
- Enhanced monitoring and metrics
- Extended threat modeling documentation

**M6 – User Interface**
- Web application for deposits and withdrawals
- Wallet integration and key management
- Transaction history and analytics

**M7 – Multi-Asset Support**
- SPL token integration
- Cross-token privacy pools
