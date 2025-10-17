# Cloak - Quick Reference Card

## 🎯 One-Minute Overview

**Cloak** is a privacy protocol on Solana that lets you:
- Deposit SOL privately (creates hidden commitment)
- Withdraw to any address (uses zero-knowledge proof)
- Break the link between deposit and withdrawal addresses

**Key Innovation**: Zero-knowledge proofs prove you own funds without revealing which deposit is yours.

---

## 🔄 How It Works (Simple)

```
DEPOSIT                          WITHDRAW
-------                          --------
1. User deposits 1 SOL           1. User selects note to spend
   ↓                                ↓
2. Creates commitment C          2. Gets Merkle proof from indexer
   C = Hash(amount, secret)         ↓
   ↓                             3. Generates ZK proof (30-60s)
3. Commitment added to tree         "I own a note in the tree"
   Tree has 1000s of deposits       ↓
   ↓                             4. Submits proof + new address
4. All deposits look the same       ↓
   (privacy!)                    5. Program verifies proof
                                    ↓
                                 6. SOL sent to new address
                                    (privacy!)
```

---

## 💡 Key Components

| Component | What It Does |
|-----------|--------------|
| **Shield-Pool** | Solana program that holds SOL and verifies proofs |
| **Indexer** | Builds Merkle tree of commitments, provides proofs |
| **SP1 Prover** | Generates zero-knowledge proofs |
| **Relay** | Submits transactions (optional privacy layer) |
| **Frontend** | User interface for deposits/withdrawals |

---

## 🔐 Privacy Model

### What's Hidden
- ✅ Link between deposit and withdrawal addresses
- ✅ Which commitment you're spending (hidden in anonymity set)
- ✅ Original depositor identity

### What's Visible
- ⚠️ Deposit amounts (on-chain)
- ⚠️ Withdrawal amounts (on-chain)
- ⚠️ Transaction timing
- ⚠️ Commitment exists in tree (but not which one is being spent)

### Privacy Tips
1. Wait before withdrawing (timing decorrelation)
2. Use common amounts (1, 5, 10 SOL for larger anonymity sets)
3. Withdraw to fresh address
4. Use relay to hide your IP

---

## 💰 Fees

```
Deposit:   0% protocol fee (FREE)
           + ~0.000005 SOL network fee

Withdraw:  0.5% of amount (variable)
           + 0.0025 SOL (fixed)
           + ~0.000005 SOL network fee

Example:
Deposit 1.0 SOL → Pool receives: 1.0 SOL
Withdraw 1.0 SOL → You receive: 0.9925 SOL
                   Treasury gets: 0.0075 SOL
```

---

## 🔑 Cryptographic Primitives

```
Commitment:    C = BLAKE3(amount || r || pk_spend)
               Stored in Merkle tree, hides details

Nullifier:     nf = BLAKE3(sk_spend || leaf_index)
               Prevents double-spending, revealed when spending

Merkle Tree:   31 levels, BLAKE3-256, 2^31 capacity
               Proves commitment exists without revealing which

ZK Proof:      SP1 (Groth16), 260 bytes
               Proves: "I own a note" without revealing it
```

---

## 🛡️ Security Properties

✅ **Double-Spend Prevention**
- Each note can only be spent once
- Nullifier marked on-chain after spending

✅ **Counterfeit Prevention**
- Must prove commitment exists in Merkle tree
- Cannot create fake notes

✅ **Front-Running Protection**
- Nullifier tied to secret key
- Attacker cannot steal withdrawal

✅ **Conservation of Value**
- Enforced in ZK circuit: input = outputs + fee
- Cannot create or destroy SOL

---

## 🧪 Testing

```bash
# Build everything
just build

# Test on local network
just start-validator  # Terminal 1
just test-localnet    # Terminal 2

# Test on testnet
just test-testnet

# Start services
docker compose up -d
```

---

## 📊 Performance

| Metric | Value |
|--------|-------|
| Proof Generation | 30-60 seconds (CPU) |
| Proof Size | 260 bytes |
| Transaction Size | ~1.2 KB |
| Compute Units | ~50K CUs (25% of limit) |
| Merkle Capacity | 2^31 = 2.1 billion notes |
| Throughput | ~50 TPS (Solana limited) |

---

## 🏗️ Architecture (High Level)

```
┌──────────┐
│   USER   │ Creates commitment, encrypts note
└────┬─────┘
     │
     ▼
┌────────────────┐
│ SOLANA PROGRAM │ Stores commitment, emits event
└────┬───────────┘
     │
     ▼
┌────────────────┐
│    INDEXER     │ Builds Merkle tree, serves proofs
└────────────────┘

     ┌───────────┐
     │   USER    │ Scans notes, requests proof
     └─────┬─────┘
           │
           ▼
     ┌───────────┐
     │ SP1 PROVER│ Generates ZK proof (30-60s)
     └─────┬─────┘
           │
           ▼
     ┌───────────┐
     │   RELAY   │ Submits transaction (optional)
     └─────┬─────┘
           │
           ▼
     ┌───────────────┐
     │SOLANA PROGRAM │ Verifies proof, transfers SOL
     └───────────────┘
```

---

## 🚀 Quick Start

```bash
# 1. Clone and build
git clone <repo>
cd cloak
just build

# 2. Start services
docker compose up -d

# 3. Test deposit
curl -X POST http://localhost:3001/deposit \
  -H "Content-Type: application/json" \
  -d '{"commitment": "0x...", "encrypted_output": "0x..."}'

# 4. Generate proof (CLI)
cd packages/zk-guest-sp1
cargo run --bin cloak-zk -- generate-proof \
  --amount 1000000000 \
  --leaf-index 0 \
  --root 0x...

# 5. Submit withdrawal
curl -X POST http://localhost:3002/withdraw \
  -H "Content-Type: application/json" \
  -d @withdrawal_request.json
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **ARCHITECTURE_DIAGRAM.md** | Complete system architecture |
| **VISUAL_FLOW.md** | Visual flow diagrams |
| **TECH_STACK.md** | Technical stack details |
| **zk/circuit-withdraw.md** | ZK circuit specification |
| **zk/api-contracts.md** | API endpoints |

---

## 🎯 Program Info

```
Program ID:    c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp
Status:        ✅ Production Ready
Networks:      Localnet (8899), Testnet, Devnet
Last Updated:  January 2025
```

---

## 🔧 Environment Setup

```bash
# .env file
SOLANA_RPC_URL=http://localhost:8899
PROGRAM_ID=c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp
DATABASE_URL=postgresql://user:pass@localhost/cloak
RUST_LOG=info
```

---

## ⚡ Common Commands

```bash
# Build
just build                 # Build all components
just build-program         # Build Solana program only
just build-zk              # Build ZK guest program

# Deploy
just deploy-local          # Deploy to localnet
just deploy-testnet        # Deploy to testnet

# Test
just test-localnet         # Integration test (local)
just test-testnet          # Integration test (testnet)
just test-unit             # Unit tests only

# Services
docker compose up          # Start all services
docker compose down        # Stop all services
docker compose logs -f     # View logs

# Database
sqlx migrate run           # Run migrations
sqlx database reset        # Reset database
```

---

## 🐛 Troubleshooting

**Proof generation fails**
- Check SP1 is installed: `sp1up`
- Ensure sufficient memory: 4+ GB
- Try: `cargo clean && cargo build`

**Transaction fails**
- Check RPC connection: `solana cluster-version`
- Verify program deployed: `solana program show <PROGRAM_ID>`
- Check account balance: `solana balance`

**Indexer not syncing**
- Check RPC endpoint accessible
- Verify database connection
- Check logs: `docker compose logs indexer`

**Invalid proof error**
- Ensure root is current (not stale)
- Verify merkle proof correct
- Check public inputs match

---

## 📞 Support

- **Docs**: `docs/` directory
- **Issues**: GitHub Issues
- **Logs**: `docker compose logs`
- **Debug**: Set `RUST_LOG=debug`

---

**Quick Tip**: Start by reading `ARCHITECTURE_DIAGRAM.md` for the full picture, then use this card as a reference!

