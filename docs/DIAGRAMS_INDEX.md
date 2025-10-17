# Cloak - Architecture Documentation Index

This directory contains comprehensive documentation about the Cloak privacy protocol architecture, including detailed diagrams of how the entire system works.

## 📚 Documentation Files

### 🎯 Start Here

1. **[ARCHITECTURE_DIAGRAM.md](./ARCHITECTURE_DIAGRAM.md)** - **MAIN DOCUMENT**
   - Complete system architecture overview
   - Detailed deposit and withdraw flows
   - Data structures and cryptographic primitives
   - Component breakdown (Program, ZK, Services)
   - Security properties and guarantees
   - Performance metrics and limits
   - Complete transaction lifecycle examples

2. **[VISUAL_FLOW.md](./VISUAL_FLOW.md)** - **VISUAL DIAGRAMS**
   - ASCII art flow diagrams
   - Step-by-step deposit flow
   - Step-by-step withdraw flow
   - Zero-knowledge circuit visualization
   - Data structure diagrams
   - Fee structure breakdown

3. **[TECH_STACK.md](./TECH_STACK.md)** - **TECHNICAL DETAILS**
   - Technology stack overview
   - Package structure and organization
   - Build and deployment processes
   - Testing strategy
   - Monitoring and observability
   - Security considerations
   - Performance optimizations
   - Development commands

### 📖 Additional Documentation

4. **[COMPLETE_FLOW_STATUS.md](./COMPLETE_FLOW_STATUS.md)**
   - Current production status
   - Completed features
   - Recent updates and improvements

5. **[zk/design.md](./zk/design.md)**
   - High-level ZK design
   - Deposit and withdraw overview

6. **[zk/circuit-withdraw.md](./zk/circuit-withdraw.md)**
   - Circuit specification
   - Constraints and public inputs

7. **[zk/api-contracts.md](./zk/api-contracts.md)**
   - Indexer API endpoints
   - Relay API endpoints

## 🎨 Quick Visual Overview

```
┌─────────────┐
│    USER     │  Deposits SOL → Creates commitment
└──────┬──────┘                  Encrypts note
       │
       ▼
┌──────────────────────────┐
│   SOLANA BLOCKCHAIN      │  Stores commitment
│   Shield-Pool Program    │  Emits event
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│   INDEXER SERVICE        │  Builds Merkle tree
│   PostgreSQL             │  Serves proofs
└──────────────────────────┘

         ┌─────────────────┐
         │  USER wants to  │  Scans notes
         │  withdraw       │  Gets Merkle proof
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  SP1 PROVER     │  Generates ZK proof
         │  (30-60 sec)    │  ~260 bytes
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  RELAY SERVICE  │  Submits transaction
         │  (optional)     │  Provides privacy
         └────────┬────────┘
                  │
                  ▼
┌──────────────────────────────────┐
│   SOLANA BLOCKCHAIN              │  Verifies proof
│   Shield-Pool Program            │  Checks nullifier
│   • Verify ZK proof              │  Transfers SOL
│   • Check nullifier not used     │  Marks spent
│   • Validate outputs             │
│   • Transfer funds               │
└──────────────────────────────────┘
```

## 🔑 Key Concepts

### Privacy Primitives

- **Commitment**: `C = H(amount || r || pk_spend)` - Hides deposit details
- **Nullifier**: `nf = H(sk_spend || leaf_index)` - Prevents double-spending
- **Merkle Tree**: 31-level tree, BLAKE3-256 hashing
- **ZK Proof**: SP1 circuit, Groth16 (260 bytes)

### Transaction Flow

1. **Deposit**: User → Solana → Indexer builds tree → Admin pushes root
2. **Withdraw**: User scans notes → Generates proof → Relay submits → Solana verifies

### Fee Structure

- **Deposits**: 0% (FREE)
- **Withdrawals**: 0.5% + 0.0025 SOL

## 📊 System Components

| Component | Language | Purpose |
|-----------|----------|---------|
| **Shield-Pool Program** | Rust (Anchor) | On-chain verification, fund custody |
| **SP1 Guest** | Rust (zkVM) | Zero-knowledge circuit |
| **SP1 Host** | Rust | Proof generation |
| **Indexer** | Rust (Actix) | Merkle tree management, API |
| **Relay** | Rust (Actix) | Transaction submission |
| **Web Frontend** | TypeScript (Next.js) | User interface |

## 🔐 Security Guarantees

✅ **Privacy**
- Sender anonymity via commitment hiding
- Recipient privacy (can withdraw to new address)
- No link between deposit and withdrawal

✅ **Security**
- Double-spend prevention via nullifiers
- Counterfeit prevention via Merkle proofs
- Front-running protection (nullifier tied to secret key)
- Conservation of value enforced in circuit

⚠️ **Known Limitations**
- Output amounts visible on-chain (MVP)
- Timing analysis possible
- Admin role for root updates (can be decentralized)

## 📈 Performance Metrics

- **Proof Generation**: 30-60 seconds (CPU)
- **Proof Size**: 260 bytes (Groth16)
- **Transaction Size**: ~1.2 KB
- **Compute Units**: ~50K CUs (well within 200K limit)
- **Merkle Capacity**: 2^31 = 2.1B commitments

## 🚀 Getting Started

```bash
# Read the full architecture
cat docs/ARCHITECTURE_DIAGRAM.md | less

# View visual flows
cat docs/VISUAL_FLOW.md | less

# Explore technical details
cat docs/TECH_STACK.md | less

# Build and test
just build
just test-localnet

# Start services
docker compose up -d
```

## 🎯 Program Information

```
Program ID:  c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp
Status:      ✅ Production Ready
Networks:    Localnet, Testnet, Devnet
```

## 📞 Quick Reference

### For Developers
- Start with **ARCHITECTURE_DIAGRAM.md** for system understanding
- Use **TECH_STACK.md** for implementation details
- Refer to **zk/** directory for circuit specifications

### For Auditors
- Review **ARCHITECTURE_DIAGRAM.md** for security properties
- Check **zk/circuit-withdraw.md** for constraint system
- Examine **zk/threat-model.md** for security analysis

### For Users
- Read **VISUAL_FLOW.md** for simple explanations
- Understand deposit and withdraw flows
- Learn about privacy guarantees and limitations

## 🔗 Related Documentation

- **[../README.md](../README.md)** - Project overview
- **[roadmap.md](./roadmap.md)** - Future plans
- **[glossary.md](./glossary.md)** - Term definitions
- **[CHANGELOG.md](./CHANGELOG.md)** - Recent changes

---

## 💡 Understanding the Architecture

The Cloak protocol implements a **privacy-preserving exit router** on Solana using zero-knowledge proofs. Here's the core idea:

1. **Deposit Phase**: Users deposit SOL along with a commitment that hides the amount and recipient. The commitment is added to a Merkle tree.

2. **Privacy Set**: As more users deposit, the anonymity set grows. All commitments look identical on-chain.

3. **Withdraw Phase**: Users prove they own a valid commitment in the tree without revealing which one. They can withdraw to any address.

4. **Zero-Knowledge**: The ZK proof ensures:
   - The commitment exists in the tree
   - The user knows the secret key
   - The nullifier prevents double-spending
   - The outputs are correctly computed
   - All without revealing the original deposit

This provides **practical privacy** on Solana while maintaining **security** and **decentralization**.

---

**Last Updated**: January 2025  
**Status**: ✅ Production Ready

For questions or contributions, see the main [README](../README.md).

