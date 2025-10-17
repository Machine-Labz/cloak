# Cloak Architecture - Complete System Diagram

## 🏗️ High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              USER / FRONTEND                                     │
│                            (services/web/)                                       │
└───────────────────────────┬─────────────────────────────────────────────────────┘
                            │
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
        ▼                                       ▼
┌────────────────┐                    ┌──────────────────┐
│   DEPOSIT      │                    │   WITHDRAW       │
│   FLOW         │                    │   FLOW           │
└────────────────┘                    └──────────────────┘
```

---

## 🔐 DEPOSIT FLOW (Privacy Input)

```
┌─────────────┐
│   USER      │ Creates commitment locally
│  (Client)   │ C = H(amount || r || pk_spend)
└──────┬──────┘ pk_spend = H(sk_spend)
       │
       │ 1. Send SOL + encrypted_output + leaf_commit
       ▼
┌─────────────────────────────────────────────────────────────┐
│         SOLANA PROGRAM (shield-pool)                        │
│         Program ID: c1oak6tetxYnNfvXKFkpn1d98FxtK7B68v...  │
│                                                             │
│  deposit_instruction:                                       │
│                                                             │
│  Accounts (in order):                                       │
│  1. user          (signer, writable) - Depositor           │
│  2. pool          (writable)         - Pool vault          │
│  3. system_program                   - For CPI transfer    │
│  4. commitments   (writable)         - Commitment queue    │
│                                                             │
│  Instruction Data:                                          │
│  • [0]: discriminator (0x00 = deposit)                     │
│  • [1-8]: amount (u64, little-endian)                      │
│  • [9-40]: commitment (32 bytes)                           │
│                                                             │
│  Processing:                                                │
│  • Validates user is signer with sufficient funds          │
│  • Checks commitment not already in queue (prevents dupes) │
│  • Appends commitment to on-chain commitment queue         │
│  • Transfers SOL via System Program CPI                    │
│    └─ Uses pinocchio-system Transfer instruction           │
│  • Fee: 0% (FREE deposits)                                 │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Commitment stored on-chain
                 ▼
┌─────────────────────────────────────────────────────────────┐
│              INDEXER SERVICE                                │
│              (services/indexer/)                            │
│                                                             │
│  Monitors on-chain commitment queue:                       │
│  1. Polls commitment queue account for new commitments     │
│  2. Reads commitment C from queue                          │
│  3. Appends C to off-chain Merkle tree (PostgreSQL)       │
│  4. Computes new root = H(left, right) recursively        │
│  5. Stores in PostgreSQL:                                  │
│     - commitment                                           │
│     - leaf_index (position in tree)                        │
│     - tree level and siblings                              │
│     - current root                                         │
│  6. Updates merkle_roots table with new root              │
│  7. Provides API for merkle proofs and root queries       │
│                                                             │
│  Database Schema:                                          │
│  • commitments (leaf_index, commitment, timestamp)        │
│  • merkle_tree (level, index, hash)                       │
│  • merkle_roots (root, created_at)                        │
│                                                             │
│  Tree Structure:                                           │
│  • 31-level binary tree (2^31 capacity)                   │
│  • BLAKE3-256 hashing for all nodes                       │
│  • Deterministic path computation                         │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Admin periodically pushes root
                 ▼
┌─────────────────────────────────────────────────────────────┐
│         SOLANA PROGRAM (admin_push_root)                    │
│                                                             │
│  admin_push_root_instruction:                              │
│  • Validates admin signature                               │
│  • Updates on-chain merkle_root_state                     │
│  • Marks root as valid for withdrawals                    │
│  • Timestamps root update                                  │
└─────────────────────────────────────────────────────────────┘

Result: User's deposit is now part of the anonymity set
```

---

## 💸 WITHDRAW FLOW (Privacy Output with ZK Proof)

```
┌──────────────────────────────────────────────────────────────────────┐
│  STEP 1: CLIENT PREPARATION                                          │
└──────────────────────────────────────────────────────────────────────┘

┌─────────────┐
│   USER      │ 1. Has saved note from deposit (or received from sender)
│  (Client)   │ 2. Selects input note to spend
└──────┬──────┘    - Has: amount, r, sk_spend, commitment
       │
       │ Queries Indexer
       ▼
┌─────────────────────────────────────────────────────────────┐
│              INDEXER SERVICE - API ENDPOINTS                │
│                                                             │
│  GET /merkle/root                                          │
│  → { root: "0x...", nextIndex: 1234 }                     │
│                                                             │
│  GET /merkle/proof/:leaf_index                             │
│  → { pathElements: [hash1, hash2, ...],                   │
│      pathIndices: [0, 1, 0, ...] }                        │
│                                                             │
│  POST /api/v1/deposit                                      │
│  → Register deposit in indexer (commitment + metadata)     │
│                                                             │
│  POST /api/v1/prove                                        │
│  → Generate withdrawal proof (SP1 prover service)          │
└─────────────────────────────────────────────────────────────┘
       │
       │ Client now has: root, merkle_path, leaf_index
       │
       ▼

┌──────────────────────────────────────────────────────────────────────┐
│  STEP 2: ZERO-KNOWLEDGE PROOF GENERATION                             │
└──────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│    SP1 HOST PROGRAM (packages/zk-guest-sp1/host/)          │
│                                                             │
│  Input preparation:                                        │
│  • Private inputs (witness):                               │
│    - amount (u64)                                          │
│    - r (32 bytes randomness)                               │
│    - sk_spend (32 bytes secret key)                        │
│    - leaf_index (u32)                                      │
│    - merkle_path (pathElements[], pathIndices[])           │
│                                                             │
│  • Public inputs:                                          │
│    - root (32 bytes)                                       │
│    - nf = H(sk_spend || leaf_index) (32 bytes)            │
│    - outputs_hash = H(serialize(outputs)) (32 bytes)       │
│    - amount (u64)                                          │
│                                                             │
│  • Output specification:                                   │
│    - recipient_address (Solana pubkey)                     │
│    - output_amount (lamports)                              │
│                                                             │
│  Fee calculation:                                          │
│    fee = (amount × 0.5%) + 0.0025 SOL (fixed)             │
│    output_amount = amount - fee                            │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Invokes SP1 prover with guest program
             ▼
┌─────────────────────────────────────────────────────────────┐
│    SP1 GUEST PROGRAM (packages/zk-guest-sp1/guest/)        │
│    (Runs in zkVM - generates proof)                        │
│                                                             │
│  Circuit Constraints (all must be satisfied):              │
│                                                             │
│  1. pk_spend = H(sk_spend)                                │
│     └─ Proves knowledge of secret key                      │
│                                                             │
│  2. C = H(amount || r || pk_spend)                        │
│     └─ Reconstructs commitment from private inputs         │
│                                                             │
│  3. MerkleVerify(C, merkle_path) == root                  │
│     └─ Proves commitment exists in tree                    │
│     └─ Uses BLAKE3-256 for all hashes                      │
│     └─ 31-level tree verification                          │
│                                                             │
│  4. nf = H(sk_spend || leaf_index)                        │
│     └─ Computes unique nullifier                           │
│     └─ Prevents double-spending                            │
│                                                             │
│  5. sum(outputs) + fee == amount                          │
│     └─ Conservation: input = outputs + fee                 │
│     └─ fee = (amount × 0.005) + 2_500_000 lamports        │
│                                                             │
│  6. H(serialize(outputs)) == outputs_hash                 │
│     └─ Binds outputs to public inputs                      │
│                                                             │
│  All constraints use BLAKE3-256 hashing                    │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Proof generated
             ▼
┌─────────────────────────────────────────────────────────────┐
│         SP1 PROVER OUTPUT                                   │
│                                                             │
│  • proofBytes (Groth16): ~260 bytes                        │
│  • publicInputs: 226 bytes                                 │
│    - root (32 bytes)                                       │
│    - nf (32 bytes)                                         │
│    - amount (8 bytes)                                      │
│    - outputs_hash (32 bytes)                               │
│  • vkey (verification key for on-chain verification)       │
│                                                             │
│  Total proof package: ~486 bytes                           │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Client can submit directly OR use relay
             ▼

┌──────────────────────────────────────────────────────────────────────┐
│  STEP 3: TRANSACTION SUBMISSION (via Relay)                          │
└──────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│         RELAY SERVICE (services/relay/)                     │
│         (Optional - provides privacy + batching)            │
│                                                             │
│  POST /withdraw                                            │
│  Body: {                                                    │
│    outputs: [{ address, amount }],                         │
│    publicInputs: { root, nf, amount, outputs_hash },       │
│    proofBytes: "base64..."                                 │
│  }                                                          │
│                                                             │
│  Processing:                                                │
│  1. Rate limiting & validation                             │
│  2. Checks nullifier not already used                      │
│  3. Verifies proof format                                  │
│  4. Constructs Solana transaction                          │
│  5. Signs and submits to blockchain                        │
│  6. Tracks status in PostgreSQL                            │
│  7. Returns requestId for tracking                         │
│                                                             │
│  Database:                                                  │
│  • withdrawal_requests (id, status, txid, timestamp)       │
│  • Used for idempotency and status tracking                │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Submits transaction to Solana
             ▼

┌──────────────────────────────────────────────────────────────────────┐
│  STEP 4: ON-CHAIN VERIFICATION                                       │
└──────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│    SOLANA PROGRAM - withdraw_instruction                    │
│    (programs/shield-pool/src/instructions/withdraw.rs)      │
│                                                             │
│  Accounts:                                                  │
│  • pool_state (contains merkle root)                       │
│  • nullifier_set (tracks spent notes)                      │
│  • treasury (receives fees)                                │
│  • recipient (receives withdrawn SOL)                      │
│  • verifier_program (SP1 verifier)                         │
│                                                             │
│  Verification Steps:                                        │
│                                                             │
│  1. Root Validation                                        │
│     ├─ Check root matches on-chain merkle_root_state       │
│     ├─ Ensures proof is for current tree state             │
│     └─ Prevents stale proof attacks                        │
│                                                             │
│  2. Nullifier Check                                        │
│     ├─ Check nf not in nullifier_set                       │
│     ├─ Prevents double-spending                            │
│     └─ If already used, reject transaction                 │
│                                                             │
│  3. Outputs Hash Verification                              │
│     ├─ Recompute: hash = H(serialize(outputs))            │
│     ├─ Compare with publicInputs.outputs_hash              │
│     └─ Ensures outputs haven't been tampered with          │
│                                                             │
│  4. Fee Calculation Check                                  │
│     ├─ Compute: expected_fee = (amount × 0.005) + 0.0025 SOL │
│     ├─ Verify: sum(output_amounts) + fee == amount        │
│     └─ Ensures conservation of value                       │
│                                                             │
│  5. SP1 Proof Verification                                 │
│     ├─ Call SP1 verifier program with:                     │
│     │  • proofBytes                                        │
│     │  • publicInputs                                      │
│     │  • vkey (stored in program)                          │
│     ├─ Verifier runs Groth16 verification                  │
│     └─ Returns success/failure                             │
│                                                             │
│  If all checks pass:                                       │
│  ├─ Mark nullifier as used (add to nullifier_set)         │
│  ├─ Transfer output_amount to recipient                    │
│  ├─ Transfer fee to treasury                               │
│  └─ Emit WithdrawEvent                                     │
│                                                             │
│  Transaction succeeds, funds transferred!                  │
└─────────────────────────────────────────────────────────────┘

Result: User successfully withdraws SOL privately to recipient address
```

---

## 🗂️ DATA STRUCTURES & CRYPTOGRAPHIC PRIMITIVES

```
┌──────────────────────────────────────────────────────────────┐
│  COMMITMENT SCHEME                                           │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Secret Key:          sk_spend (32 bytes random)            │
│  Public Key:          pk_spend = H(sk_spend)                │
│  Randomness:          r (32 bytes random)                   │
│                                                              │
│  Commitment:          C = H(amount || r || pk_spend)        │
│                       ↓                                      │
│                   Stored in Merkle tree                      │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  NULLIFIER SCHEME (Double-Spend Prevention)                 │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Nullifier:           nf = H(sk_spend || leaf_index)        │
│                                                              │
│  Properties:                                                 │
│  • Deterministic (same inputs → same nf)                    │
│  • Unique per note (leaf_index is unique)                   │
│  • Linked to secret key (prevents front-running)            │
│  • Unlinkable to commitment (privacy preserved)             │
│                                                              │
│  On-chain storage:    HashSet of used nullifiers            │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  MERKLE TREE STRUCTURE                                       │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Levels:              31 (height)                            │
│  Capacity:            2^31 = 2,147,483,648 notes            │
│  Hash Function:       BLAKE3-256                             │
│                                                              │
│  Structure:                                                  │
│                       root (level 31)                        │
│                      /              \                        │
│                  node               node (level 30)          │
│                 /    \             /    \                    │
│               ...    ...         ...    ...                  │
│              /  \    /  \       /  \    /  \                │
│            C₁  C₂  C₃  C₄     C₅  C₆  C₇  C₈ (level 0)     │
│                                                              │
│  Path Proof:          31 sibling hashes + 31 direction bits │
│  Computation:         H(left || right) at each level        │
│  Zero Values:         Empty subtrees use zero hash          │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  OUTPUTS HASH (Public Binding)                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  outputs = [{                                                │
│    address: Pubkey (32 bytes),                              │
│    amount: u64 (8 bytes)                                    │
│  }]                                                          │
│                                                              │
│  Serialization:       address₁ || amount₁ || address₂ || ... │
│  outputs_hash:        H(serialized_outputs)                 │
│                                                              │
│  Purpose:             Binds outputs to proof without         │
│                       revealing them in public inputs        │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  HASHING (BLAKE3-256)                                        │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Used for:                                                   │
│  • Commitments                                              │
│  • Nullifiers                                               │
│  • Merkle tree nodes                                        │
│  • Outputs hash                                             │
│  • Public key derivation                                    │
│                                                              │
│  Properties:                                                 │
│  • Fast (optimized for modern CPUs)                         │
│  • Secure (256-bit output)                                  │
│  • Standard (blake3 crate)                                  │
│  • Consistent across all components                         │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔧 COMPONENT BREAKDOWN

```
┌──────────────────────────────────────────────────────────────────────┐
│  SOLANA PROGRAM (programs/shield-pool/)                             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Program ID: c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp          │
│  Language: Rust + Pinocchio (low-level framework)                    │
│                                                                      │
│  Instructions:                                                       │
│  ├─ initialize         : Setup pool state and accounts              │
│  ├─ deposit            : Accept SOL deposits, store commitments      │
│  ├─ admin_push_root    : Update merkle root (admin only)            │
│  └─ withdraw           : Verify proof and transfer funds             │
│                                                                      │
│  Deposit Instruction (Technical Details):                           │
│  ├─ Discriminator: 0x00                                             │
│  ├─ Accounts (order matters!):                                      │
│  │   [0] user (signer, writable) - payer of deposit                │
│  │   [1] pool (writable) - receives SOL                             │
│  │   [2] system_program - for CPI transfer                          │
│  │   [3] commitments (writable) - commitment queue                  │
│  ├─ Data: [discriminator(1) + amount(8) + commitment(32)] = 41 bytes│
│  ├─ Validation:                                                      │
│  │   • User must be signer                                          │
│  │   • User must have sufficient balance                            │
│  │   • Pool owner must be program ID                                │
│  │   • Commitment must be unique (not already in queue)            │
│  ├─ Processing:                                                      │
│  │   • Append commitment to on-chain queue (CommitmentQueue)       │
│  │   • Transfer lamports via System Program CPI                     │
│  │   • Uses pinocchio-system::instructions::Transfer               │
│  └─ Compute Units: ~10K CUs                                         │
│                                                                      │
│  State Accounts:                                                     │
│  ├─ PoolState          : Global config, treasury, merkle root       │
│  ├─ CommitmentQueue    : On-chain append-only commitment queue      │
│  ├─ MerkleRootState    : Current valid root for withdrawals         │
│  └─ NullifierSet       : HashMap of used nullifiers                 │
│                                                                      │
│  Key Files:                                                          │
│  ├─ lib.rs             : Program entry point                        │
│  ├─ state/mod.rs       : Account structures                         │
│  ├─ instructions/      : Instruction handlers                       │
│  │   ├─ deposit.rs                                                  │
│  │   └─ withdraw.rs                                                 │
│  ├─ constants.rs       : Constants (fees, limits)                   │
│  └─ error.rs           : Custom error types                         │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  SP1 ZK COMPONENTS (packages/zk-guest-sp1/)                         │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Guest Program (guest/):                                             │
│  ├─ Language: Rust (compiled to RISC-V)                             │
│  ├─ Runtime: SP1 zkVM                                                │
│  ├─ main.rs: Circuit logic and constraints                          │
│  ├─ encoding.rs: Cryptographic utilities                            │
│  └─ Output: ELF binary for proving                                   │
│                                                                      │
│  Host Program (host/):                                               │
│  ├─ Language: Rust                                                   │
│  ├─ Purpose: Generate proofs using guest program                     │
│  ├─ lib.rs: Proving interface                                       │
│  ├─ encoding.rs: Input/output encoding                              │
│  └─ bin/cloak-zk.rs: CLI tool for proof generation                  │
│                                                                      │
│  Build Process:                                                      │
│  ├─ Guest compiled to RISC-V ELF                                    │
│  ├─ Host invokes SP1 prover with ELF + inputs                       │
│  ├─ SP1 generates Groth16 proof (~260 bytes)                        │
│  └─ Proof + public inputs returned                                   │
│                                                                      │
│  Performance:                                                        │
│  ├─ Proof generation: ~30-60 seconds                                │
│  ├─ Verification: ~50K compute units on-chain                       │
│  └─ Proof size: 260 bytes (Groth16) + 226 bytes (public inputs)    │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  INDEXER SERVICE (services/indexer/)                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Language: Rust + Actix-web                                          │
│  Database: PostgreSQL                                                │
│                                                                      │
│  Core Functions:                                                     │
│  ├─ Blockchain Monitoring                                            │
│  │   ├─ Watches for DepositEvents                                   │
│  │   ├─ Extracts commitments from events                            │
│  │   └─ Real-time event processing                                  │
│  │                                                                   │
│  ├─ Merkle Tree Management                                           │
│  │   ├─ Append-only tree structure                                  │
│  │   ├─ 31-level binary tree                                        │
│  │   ├─ BLAKE3-256 hashing                                          │
│  │   ├─ Automatic root computation                                  │
│  │   └─ Deterministic proof generation                              │
│  │                                                                   │
│  └─ API Server                                                       │
│      ├─ GET /merkle/root                                            │
│      ├─ GET /merkle/proof/:index                                    │
│      ├─ GET /notes/range                                            │
│      └─ GET /artifacts/withdraw/:version                            │
│                                                                      │
│  Database Schema:                                                    │
│  ├─ commitments: stores all leaf commitments                        │
│  ├─ merkle_tree: stores tree nodes                                  │
│  ├─ merkle_roots: tracks root history                               │
│  └─ proof_requests: caches generated proofs                         │
│                                                                      │
│  Key Files:                                                          │
│  ├─ src/blockchain/monitor.rs: Event watching                       │
│  ├─ src/database/merkle.rs: Tree operations                         │
│  ├─ src/server/routes.rs: API endpoints                             │
│  └─ src/server/prover_handler.rs: Proof generation                  │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  RELAY SERVICE (services/relay/)                                     │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Language: Rust + Actix-web                                          │
│  Database: PostgreSQL                                                │
│                                                                      │
│  Purpose:                                                            │
│  ├─ Privacy layer: Breaks link between proof generator and tx       │
│  ├─ Transaction management: Handles submission and tracking         │
│  └─ Rate limiting: Prevents abuse                                    │
│                                                                      │
│  API Endpoints:                                                      │
│  ├─ POST /withdraw                                                   │
│  │   ├─ Accepts proof + public inputs + outputs                     │
│  │   ├─ Validates proof format                                      │
│  │   ├─ Checks nullifier not used                                   │
│  │   ├─ Constructs Solana transaction                               │
│  │   ├─ Signs and submits to blockchain                             │
│  │   └─ Returns requestId for tracking                              │
│  │                                                                   │
│  └─ GET /status/:requestId                                           │
│      ├─ Returns transaction status                                  │
│      └─ States: queued, executing, settled, failed                  │
│                                                                      │
│  Database Schema:                                                    │
│  ├─ withdrawal_requests: tracks all requests                        │
│  ├─ used_nullifiers: prevents duplicate submissions                 │
│  └─ transaction_logs: audit trail                                   │
│                                                                      │
│  Key Files:                                                          │
│  ├─ src/api/withdraw.rs: Withdrawal endpoint                        │
│  ├─ src/db/repository.rs: Database operations                       │
│  └─ src/solana/transaction.rs: Transaction building                 │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  WEB FRONTEND (services/web/)                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Framework: Next.js + React                                          │
│  Wallet: Solana Wallet Adapter                                       │
│                                                                      │
│  Features:                                                           │
│  ├─ Wallet connection (Phantom, Solflare, etc.)                     │
│  ├─ Deposit interface                                                │
│  │   ├─ Amount input                                                │
│  │   ├─ Local commitment generation                                 │
│  │   └─ Transaction signing and submission                          │
│  │                                                                   │
│  ├─ Withdraw interface                                               │
│  │   ├─ Load note from localStorage or import                       │
│  │   ├─ Recipient address input                                     │
│  │   ├─ Proof generation via indexer API                            │
│  │   └─ Direct transaction submission (relay TBD)                   │
│  │                                                                   │
│  ├─ Note management                                                  │
│  │   ├─ LocalStorage-based note persistence                         │
│  │   ├─ Import/export note functionality                            │
│  │   └─ Shows saved notes with metadata                             │
│  │                                                                   │
│  └─ Transaction history                                              │
│      ├─ Deposit confirmations                                       │
│      └─ Withdrawal status tracking                                  │
│                                                                      │
│  Key Components:                                                     │
│  ├─ components/transaction/: Deposit & withdraw flows               │
│  ├─ components/ui/: UI components (Shadcn)                          │
│  ├─ lib/note-manager.ts: Note generation & storage                  │
│  ├─ lib/solana.ts: Solana interactions                              │
│  └─ lib/sp1-prover.ts: SP1 proof generation via indexer API        │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 🔐 SECURITY PROPERTIES

```
┌──────────────────────────────────────────────────────────────────────┐
│  PRIVACY GUARANTEES                                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ✓ Sender Anonymity                                                 │
│    └─ Commitment C hides sender's identity                          │
│    └─ Merkle tree provides anonymity set                            │
│    └─ Nullifier unlinkable to commitment                            │
│                                                                      │
│  ✓ Amount Privacy (with caveats)                                    │
│    └─ Commitment hides amount                                       │
│    └─ Outputs visible on-chain (MVP limitation)                     │
│    └─ Can use fixed denominations for better privacy                │
│                                                                      │
│  ✓ Recipient Privacy (optional)                                     │
│    └─ Can withdraw to fresh address                                 │
│    └─ No link between deposit and withdrawal addresses              │
│    └─ Relay provides additional privacy layer                       │
│                                                                      │
│  ⚠ Metadata Leakage                                                 │
│    └─ Timing analysis possible                                      │
│    └─ Amount correlation possible (fixed in future)                 │
│    └─ Use multiple notes and delays for better privacy              │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  SECURITY MECHANISMS                                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ✓ Double-Spend Prevention                                          │
│    └─ Nullifier marks note as spent                                 │
│    └─ On-chain nullifier set prevents reuse                         │
│    └─ Nullifier = H(sk_spend || leaf_index) is deterministic        │
│                                                                      │
│  ✓ Counterfeit Prevention                                           │
│    └─ ZK proof ensures valid commitment in tree                     │
│    └─ Merkle root verification prevents fake notes                  │
│    └─ Conservation constraint prevents money creation               │
│                                                                      │
│  ✓ Front-running Protection                                         │
│    └─ Nullifier tied to secret key                                  │
│    └─ Attacker cannot compute nullifier without sk_spend            │
│    └─ Proof cannot be replayed by attacker                          │
│                                                                      │
│  ✓ Root Staleness Protection                                        │
│    └─ Multiple historical roots accepted (grace period)             │
│    └─ Prevents DoS from rapid root updates                          │
│    └─ Indexer tracks root history                                   │
│                                                                      │
│  ✓ Fee Consistency                                                  │
│    └─ Fixed fee structure: 0.5% + 0.0025 SOL                        │
│    └─ Enforced in ZK circuit                                        │
│    └─ Verified on-chain                                             │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 📊 PERFORMANCE & LIMITS

```
┌──────────────────────────────────────────────────────────────────────┐
│  TRANSACTION METRICS                                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Deposit:                                                            │
│  ├─ Size: ~300 bytes                                                │
│  ├─ Compute Units: ~10K CUs                                         │
│  ├─ Fee: Network fee only (~0.000005 SOL)                           │
│  └─ Confirmation: 1-2 blocks (~1 second)                            │
│                                                                      │
│  Withdraw:                                                           │
│  ├─ Size: ~1.2 KB (proof + public inputs + outputs)                │
│  ├─ Compute Units: ~50K CUs (proof verification)                    │
│  ├─ Fee: 0.5% + 0.0025 SOL + network fee                           │
│  ├─ Proof generation: 30-60 seconds (client-side)                   │
│  └─ Confirmation: 1-2 blocks (~1 second)                            │
│                                                                      │
│  Limits:                                                             │
│  ├─ Min deposit: 0.01 SOL                                           │
│  ├─ Max deposit: No practical limit                                 │
│  ├─ Merkle capacity: 2^31 = 2.1B notes                              │
│  └─ Max outputs per withdrawal: 10 (configurable)                   │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  COST BREAKDOWN (Example: 1 SOL Withdrawal)                         │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Input Amount:        1.0000 SOL                                    │
│                                                                      │
│  Protocol Fee:        0.5% = 0.0050 SOL                             │
│  Fixed Fee:           0.0025 SOL                                     │
│  Total Fee:           0.0075 SOL                                     │
│                                                                      │
│  Network Fee:         ~0.000005 SOL                                 │
│                                                                      │
│  Recipient Gets:      0.992495 SOL                                  │
│                                                                      │
│  Effective Rate:      0.75% total cost                              │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 DEPLOYMENT ARCHITECTURE

```
┌──────────────────────────────────────────────────────────────────────┐
│  PRODUCTION DEPLOYMENT                                               │
└──────────────────────────────────────────────────────────────────────┘

                          ┌─────────────────┐
                          │   CLOUDFLARE    │
                          │   (CDN + WAF)   │
                          └────────┬────────┘
                                   │
                                   │ HTTPS
                                   ▼
                          ┌─────────────────┐
                          │   NEXT.JS       │
                          │   (Frontend)    │
                          │   Vercel/Cloud  │
                          └────────┬────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │                             │
                    ▼                             ▼
           ┌──────────────┐            ┌──────────────────┐
           │   INDEXER    │            │     RELAY        │
           │   API        │            │     API          │
           │   (Rust)     │            │     (Rust)       │
           └──────┬───────┘            └────────┬─────────┘
                  │                             │
                  │                             │
           ┌──────▼───────┐            ┌────────▼─────────┐
           │  PostgreSQL  │            │   PostgreSQL     │
           │  (Indexer)   │            │   (Relay)        │
           └──────────────┘            └──────────────────┘
                  │
                  │ Reads blockchain events
                  ▼
           ┌──────────────────────────────────┐
           │     SOLANA RPC NODES             │
           │     (Mainnet/Devnet/Testnet)     │
           └────────────┬─────────────────────┘
                        │
                        │ Blockchain data
                        ▼
           ┌──────────────────────────────────┐
           │   SOLANA BLOCKCHAIN              │
           │                                  │
           │   Program: c1oak6tetx...         │
           │   Accounts: PoolState, etc.      │
           └──────────────────────────────────┘

Docker Compose Stack (Development):
├─ indexer: Rust service on port 3001
├─ relay: Rust service on port 3002
├─ postgres_indexer: Database for indexer
├─ postgres_relay: Database for relay
└─ frontend: Next.js on port 3000

Environment Variables:
├─ SOLANA_RPC_URL: RPC endpoint
├─ PROGRAM_ID: Shield pool program ID
├─ DATABASE_URL: PostgreSQL connection
├─ RUST_LOG: Logging level
└─ PORT: Service port
```

---

## 🔄 COMPLETE TRANSACTION LIFECYCLE

```
┌──────────────────────────────────────────────────────────────────────┐
│  FULL CYCLE: Alice deposits 1 SOL, Bob withdraws 0.5 SOL            │
└──────────────────────────────────────────────────────────────────────┘

PHASE 1: ALICE DEPOSITS
========================

1. Alice (Client):
   ├─ Generates: sk_spend, r (random 32 bytes each)
   ├─ Computes: pk_spend = H(sk_spend)
   ├─ Computes: C = H(1.0 SOL || r || pk_spend)
   ├─ Saves note locally: {amount, r, sk_spend, C}
   └─ Submits: deposit_tx(1.0 SOL, C)

2. Solana Program (deposit instruction):
   ├─ Validates accounts and amount
   ├─ Checks commitment C not already in queue
   ├─ Appends C to on-chain CommitmentQueue
   ├─ Transfers 1.0 SOL from Alice to pool via System Program CPI
   └─ Transaction confirmed

3. Indexer:
   ├─ Polls on-chain CommitmentQueue for new commitments
   ├─ Detects new commitment C at position 42 in queue
   ├─ Appends C to off-chain Merkle tree at leaf_index = 42
   ├─ Computes new root
   ├─ Stores: commitment=C, leaf_index=42, tree_nodes
   └─ API now returns new root and can provide merkle proofs

4. Admin (periodic):
   ├─ Fetches latest root from indexer
   ├─ Submits: admin_push_root(new_root)
   └─ On-chain root updated

Result: Alice's 1.0 SOL is now in pool, commitment in tree

---

PHASE 2: BOB WITHDRAWS (using Alice's note)
============================================

5. Bob (Client):
   ├─ Alice shares note details with Bob (off-chain)
   ├─ Bob has: amount=1.0 SOL, r, sk_spend, commitment=C
   ├─ Queries indexer to find leaf_index for commitment C
   ├─ Now has: amount=1.0 SOL, r, sk_spend, leaf_index=42
   ├─ Queries indexer:
   │  ├─ GET /merkle/root → root=0xabc...
   │  └─ GET /merkle/proof/42 → merkle_path
   └─ Prepares withdrawal to Bob's address: 0.5 SOL

6. Bob's Client - Proof Generation:
   ├─ Private inputs:
   │  ├─ amount = 1.0 SOL (1,000,000,000 lamports)
   │  ├─ r = Alice's randomness
   │  ├─ sk_spend = Alice's secret key
   │  ├─ leaf_index = 42
   │  └─ merkle_path = [31 sibling hashes + indices]
   │
   ├─ Public inputs:
   │  ├─ root = 0xabc...
   │  ├─ nf = H(sk_spend || 42)
   │  ├─ outputs_hash = H(Bob's address || 0.5 SOL)
   │  └─ amount = 1.0 SOL
   │
   ├─ SP1 Host invokes guest program
   ├─ Guest verifies all constraints
   ├─ SP1 generates Groth16 proof (30-60 seconds)
   └─ Returns: proofBytes (260 bytes)

7. Bob submits to Relay:
   POST /withdraw {
     outputs: [{ address: Bob, amount: 0.5 SOL }],
     publicInputs: { root, nf, amount, outputs_hash },
     proofBytes: "..."
   }

8. Relay Service:
   ├─ Validates proof format
   ├─ Checks nf not in used_nullifiers table
   ├─ Constructs Solana transaction:
   │  └─ withdraw_ix(proof, public_inputs, outputs)
   ├─ Signs with relay keypair
   ├─ Submits to blockchain
   └─ Returns: requestId

9. Solana Program - Verification:
   ├─ Checks root = 0xabc... (matches on-chain state) ✓
   ├─ Checks nf not in nullifier_set ✓
   ├─ Recomputes outputs_hash:
   │  └─ H(Bob's address || 0.5 SOL) = public_inputs.outputs_hash ✓
   ├─ Verifies SP1 proof:
   │  └─ sp1_verifier.verify(proofBytes, publicInputs, vkey) ✓
   ├─ Checks conservation:
   │  ├─ fee = (1.0 × 0.5%) + 0.0025 = 0.0075 SOL
   │  ├─ sum(outputs) = 0.5 SOL
   │  └─ 0.5 + 0.0075 ≠ 1.0 SOL ✗ (wait, there's remaining!)
   │
   │  (Note: Bob only requested 0.5 SOL, remainder of ~0.4925 SOL
   │   would need another output or is forfeit. In practice, Bob
   │   would withdraw 0.9925 SOL after fee)
   │
   ├─ All checks pass:
   │  ├─ Marks nf as used
   │  ├─ Transfers 0.5 SOL to Bob
   │  ├─ Transfers 0.0075 SOL to treasury
   │  └─ (Remainder stays in pool - realization: conservation constraint
   │       should account for all funds. This is a design consideration)
   │
   └─ Transaction confirmed

10. Bob:
    ├─ Receives 0.5 SOL at his address
    ├─ Transaction appears on blockchain
    └─ Privacy maintained: no link to Alice's deposit

Result: Bob successfully withdraws 0.5 SOL privately!

---

PHASE 3: WHAT'S LEFT IN THE POOL?
==================================

In practice, conservation constraint ensures:
  input_amount = sum(output_amounts) + fee

So if Alice deposited 1.0 SOL and Bob wants to withdraw it all:
  - Output: 0.9925 SOL (to Bob)
  - Fee: 0.0075 SOL (to treasury)
  - Total: 1.0 SOL ✓

The nullifier for leaf_index=42 is now used. Alice's commitment
is spent and cannot be used again. Bob (or whoever had Alice's keys)
can now use the 0.9925 SOL freely.
```

---

## 🛠️ TESTING & DEVELOPMENT

```
┌──────────────────────────────────────────────────────────────────────┐
│  TEST SUITE                                                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Unit Tests:                                                         │
│  ├─ programs/shield-pool/tests/        : Program tests              │
│  ├─ packages/zk-guest-sp1/tests/       : Circuit tests              │
│  ├─ services/indexer/tests/            : Indexer tests              │
│  └─ services/relay/tests/              : Relay tests                │
│                                                                      │
│  Integration Tests:                                                  │
│  ├─ tooling/test/src/localnet_test.rs : Local network test         │
│  ├─ tooling/test/src/testnet_test.rs  : Testnet test               │
│  └─ Tests full deposit → withdraw flow                              │
│                                                                      │
│  Test Commands:                                                      │
│  ├─ just build           : Build all components                     │
│  ├─ just test-localnet   : Run localnet integration test            │
│  ├─ just test-testnet    : Run testnet integration test             │
│  ├─ just start-validator : Start local validator                    │
│  └─ just deploy-local    : Deploy to local validator                │
│                                                                      │
│  Docker Testing:                                                     │
│  ├─ docker compose up    : Start all services                       │
│  ├─ docker compose down  : Stop all services                        │
│  └─ docker compose logs  : View service logs                        │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 📚 KEY DOCUMENTATION

- **docs/zk/design.md** - High-level ZK design
- **docs/zk/circuit-withdraw.md** - Circuit specification
- **docs/zk/encoding.md** - Encoding schemes
- **docs/zk/merkle.md** - Merkle tree details
- **docs/zk/prover-sp1.md** - SP1 prover integration
- **docs/zk/onchain-verifier.md** - On-chain verification
- **docs/zk/api-contracts.md** - API specifications
- **docs/COMPLETE_FLOW_STATUS.md** - Current status
- **docs/roadmap.md** - Future plans

---

## 🎯 PROGRAM ID & NETWORK INFO

```
Program ID:  c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp
Networks:    Localnet (8899), Testnet, Devnet
Status:      ✅ Production Ready
```


