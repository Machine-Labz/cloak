# ZK Layer Documentation

The Zero-Knowledge (ZK) layer is the cryptographic core of Cloak's privacy-preserving protocol. It enables users to spend notes privately by proving knowledge of valid commitments without revealing the underlying transaction details.

## Overview

**Primary Goal:** Enable private note spending by proving:
1. **Leaf Inclusion** - Commitment `C` exists in the Merkle tree
2. **Nullifier Uniqueness** - Nullifier `nf` has never been used before
3. **Amount Conservation** - `sum(outputs) + fee == amount` (no value created/destroyed)

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                        ZK LAYER ARCHITECTURE                    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Client    │    │   SP1 Guest  │    │   On-Chain      │    │
│  │   Witness   │    │   Circuit    │    │   Verifier      │    │
│  │             │    │              │    │                 │    │
│  │ • Secrets   │───►│ • Constraints│───►│ • Proof Check   │    │
│  │ • Merkle    │    │ • BLAKE3     │    │ • Nullifier     │    │
│  │ • Outputs   │    │ • Groth16    │    │ • Conservation  │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
│         │                   │                   │               │
│         │                   │                   │               │
│         ▼                   ▼                   ▼               │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Indexer   │    │   SP1 Host   │    │   Shield Pool   │    │
│  │   Service   │    │   Prover     │    │   Program       │    │
│  │             │    │              │    │                 │    │
│  │ • Merkle    │    │ • Proof Gen  │    │ • Verification  │    │
│  │ • Proofs    │    │ • Witness    │    │ • Execution     │    │
│  │ • Discovery │    │ • Artifacts  │    │ • State Update  │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Core Components

**Proving System:**
- **SP1 zkVM** - Succinct's zero-knowledge virtual machine
- **Groth16** - zkSNARK proving system (260-byte proofs)
- **RISC-V** - Target architecture for SP1 guest programs

**Cryptographic Primitives:**
- **BLAKE3-256** - Primary hash function for commitments and nullifiers
- **Merkle Trees** - 32-level inclusion proofs for commitments
- **Commitment Scheme** - `C = H(amount || r || pk_spend)`

**On-Chain Integration:**
- **Pinocchio Framework** - Solana program development
- **SP1 Solana Verifier** - On-chain proof verification
- **CPI Calls** - Cross-program invocation for verification

### Data Flow

```text
1. Witness Preparation
   Client → Secrets + Merkle Proof + Outputs → SP1 Guest

2. Proof Generation  
   SP1 Guest → Constraints + BLAKE3 → Groth16 Proof (260 bytes)

3. On-Chain Verification
   Shield Pool → SP1 Verifier → Proof Validation → Transaction Execution
```

## Circuit Design

### Input Structure

**Private Inputs (Witness):**
```rust
pub struct WithdrawWitness {
    pub amount: u64,                    // Note amount in lamports
    pub r: [u8; 32],                   // Randomness for commitment
    pub sk_spend: [u8; 32],            // Note secret key
    pub leaf_index: u32,                // Merkle tree leaf index
    pub merkle_path: MerklePath,       // Inclusion proof path
    pub outputs: Vec<Output>,           // Output recipients and amounts
}
```

**Public Inputs (104 bytes):**
```rust
pub struct PublicInputs {
    pub root: [u8; 32],                // Merkle tree root
    pub nullifier: [u8; 32],          // Spending nullifier
    pub outputs_hash: [u8; 32],       // Hash of output recipients
    pub amount: u64,                  // Total amount being spent
}
```

### Constraint System

The circuit enforces six critical constraints:

1. **Spend Key Derivation**
   ```rust
   pk_spend = BLAKE3(sk_spend)
   ```

2. **Commitment Recomputation**
   ```rust
   C = BLAKE3(amount || r || pk_spend)
   ```

3. **Merkle Inclusion Proof**
   ```rust
   MerkleVerify(C, merkle_path) == root
   ```

4. **Nullifier Generation**
   ```rust
   nf = BLAKE3(sk_spend || leaf_index)
   ```

5. **Amount Conservation**
   ```rust
   sum(outputs) + fee(amount) == amount
   where fee(amount) = (amount * 5) / 1000 + 2_500_000
   ```

6. **Outputs Hash Verification**
   ```rust
   BLAKE3(serialize(outputs)) == outputs_hash
   ```

## Proof Generation

### SP1 Guest Program

**Circuit Implementation:**
```rust
// packages/zk-guest-sp1/guest/src/main.rs
use sp1_zkvm::prelude::*;

fn main() {
    // Read witness data
    let witness = env::read::<WithdrawWitness>();
    
    // Constraint 1: Derive spend key
    let pk_spend = blake3(&witness.sk_spend);
    
    // Constraint 2: Recompute commitment
    let commitment = compute_commitment(witness.amount, witness.r, pk_spend);
    
    // Constraint 3: Verify Merkle inclusion
    let root = verify_merkle_inclusion(commitment, witness.merkle_path);
    
    // Constraint 4: Generate nullifier
    let nullifier = compute_nullifier(witness.sk_spend, witness.leaf_index);
    
    // Constraint 5: Verify amount conservation
    let total_outputs: u64 = witness.outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(witness.amount);
    assert!(total_outputs + fee == witness.amount);
    
    // Constraint 6: Verify outputs hash
    let outputs_hash = compute_outputs_hash(&witness.outputs);
    
    // Commit public inputs
    env::commit(&PublicInputs {
        root,
        nullifier,
        outputs_hash,
        amount: witness.amount,
    });
}
```

### SP1 Host Integration

**Proof Generation Process:**
```rust
// packages/zk-guest-sp1/host/src/main.rs
use sp1_zkvm::prelude::*;

async fn generate_withdraw_proof(witness: WithdrawWitness) -> Result<ProofBundle> {
    // 1. Build guest ELF
    let elf = include_bytes!("../../guest/target/riscv32im-succinct-zkvm-elf/release/guest");
    
    // 2. Create prover client
    let prover = ProverClient::new();
    
    // 3. Generate proof
    let proof_bundle = prover
        .prove(elf, witness)
        .await?;
    
    // 4. Extract Groth16 proof (260 bytes)
    let groth16_proof = extract_groth16_260(&proof_bundle)?;
    
    // 5. Parse public inputs (104 bytes)
    let public_inputs = parse_public_inputs_104(&proof_bundle)?;
    
    Ok(ProofBundle {
        groth16_proof,
        public_inputs,
        sp1_proof_bundle: proof_bundle,
    })
}
```

## On-Chain Verification

### Shield Pool Integration

**Proof Verification:**
```rust
// programs/shield-pool/src/instructions/withdraw.rs
use sp1_solana::verify_proof;

pub fn withdraw(
    ctx: Context<Withdraw>,
    proof: [u8; 260],
    public_inputs: [u8; 104],
    outputs: Vec<Output>,
) -> Result<()> {
    // 1. Verify SP1 proof
    sp1_solana::verify_proof(&proof, &public_inputs, &VKEY_HASH)?;
    
    // 2. Parse public inputs
    let root = &public_inputs[0..32];
    let nullifier = &public_inputs[32..64];
    let outputs_hash = &public_inputs[64..96];
    let amount = u64::from_le_bytes(public_inputs[96..104].try_into()?);
    
    // 3. Verify Merkle root
    require!(ctx.accounts.roots_ring.contains(root), ErrorCode::RootNotFound);
    
    // 4. Check nullifier not spent
    require!(!ctx.accounts.nullifier_shard.contains(nullifier), ErrorCode::NullifierUsed);
    
    // 5. Mark nullifier as spent
    ctx.accounts.nullifier_shard.insert(nullifier)?;
    
    // 6. Verify outputs hash
    let computed_hash = compute_outputs_hash(&outputs);
    require!(computed_hash == outputs_hash, ErrorCode::OutputsHashMismatch);
    
    // 7. Verify amount conservation
    let fee = calculate_fee(amount);
    let outputs_sum: u64 = outputs.iter().map(|o| o.amount).sum();
    require!(outputs_sum + fee == amount, ErrorCode::AmountMismatch);
    
    // 8. Transfer funds
    for output in outputs {
        transfer_from_pool(output.address, output.amount)?;
    }
    
    // 9. Transfer fee to treasury
    transfer_from_pool(ctx.accounts.treasury.key(), fee)?;
    
    Ok(())
}
```

### Verification Key Management

**VKey Hash Extraction:**
```rust
// packages/vkey-generator/src/main.rs
use sp1_zkvm::prelude::*;

fn main() {
    // Load guest ELF
    let elf = include_bytes!("../zk-guest-sp1/guest/target/riscv32im-succinct-zkvm-elf/release/guest");
    
    // Extract verification key hash
    let vkey_hash = extract_vkey_hash(elf)?;
    
    println!("VKey Hash: {}", hex::encode(vkey_hash));
    
    // Output for program integration
    println!("const VKEY_HASH: [u8; 32] = {:?};", vkey_hash);
}
```

## Performance Characteristics

### Proof Generation Metrics

| Metric | Target | Current Performance |
|--------|--------|-------------------|
| **Proof Size** | 260 bytes | 260 bytes (Groth16) |
| **Public Inputs** | 104 bytes | 104 bytes |
| **Generation Time** | < 120s (p95) | 60-90s (local CPU) |
| **Verification Time** | < 1s | ~200ms (on-chain) |
| **Memory Usage** | < 4GB | 2-3GB (SP1) |

### Optimization Strategies

**Local Proving:**
- Multi-threaded SP1 execution
- Optimized BLAKE3 implementation
- Cached Merkle tree operations

**TEE Proving:**
- Remote proving services
- Hardware acceleration
- Reduced generation time (2-5 minutes)

**On-Chain Optimization:**
- Efficient verification key storage
- Optimized constraint checking
- Minimal compute unit consumption

## Security Considerations

### Cryptographic Security

**Hash Function Security:**
- BLAKE3-256 provides 128-bit security level
- Collision resistance for commitments and nullifiers
- Preimage resistance for secret key derivation

**Zero-Knowledge Properties:**
- Proof reveals no information about private inputs
- Nullifiers prevent double-spending without linking transactions
- Merkle proofs hide tree structure and other commitments

**Soundness Guarantees:**
- Circuit constraints ensure mathematical correctness
- On-chain verification prevents invalid proofs
- Nullifier tracking prevents replay attacks

### Implementation Security

**Witness Protection:**
- Private inputs never leave client environment
- Secure random number generation for `r` and `sk_spend`
- Memory clearing after proof generation

**Proof Validation:**
- Multiple verification layers (SP1 + on-chain)
- Constraint satisfaction guarantees
- Public input binding prevents malleability

## Development Workflow

### Circuit Development

**1. Modify Circuit:**
```bash
# Edit guest program
vim packages/zk-guest-sp1/guest/src/main.rs
```

**2. Rebuild Guest ELF:**
```bash
# Build for RISC-V target
cargo build --release --target riscv32im-succinct-zkvm-elf
```

**3. Generate New VKey Hash:**
```bash
# Extract verification key hash
cargo run --package vkey-generator
```

**4. Update Program:**
```bash
# Update VKEY_HASH in shield-pool program
# Rebuild and deploy program
cargo build-sbf --package shield-pool
solana program deploy target/deploy/shield_pool.so
```

### Testing Workflow

**Unit Tests:**
```bash
# Test circuit constraints
cargo test --package zk-guest-sp1

# Test proof generation
cargo test --package cloak-proof-extract
```

**Integration Tests:**
```bash
# Test end-to-end workflow
cargo test --package tooling-test
```

## Related Documentation

- **[Circuit Design](./circuit-withdraw.md)** - Detailed constraint explanations
- **[Data Encoding](./encoding.md)** - Input/output format specifications
- **[Merkle Trees](./merkle.md)** - Tree structure and proof generation
- **[SP1 Prover](./prover-sp1.md)** - Proof generation implementation
- **[On-Chain Verifier](./onchain-verifier.md)** - Verification integration
- **[API Contracts](./api-contracts.md)** - Service interface definitions
- **[Testing Guide](./testing.md)** - Comprehensive testing procedures
- **[Threat Model](./threat-model.md)** - Security analysis and mitigations
- **[Design Principles](./design.md)** - Architectural decisions and rationale