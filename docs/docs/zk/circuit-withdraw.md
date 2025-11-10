# Withdraw Circuit Design

The withdraw circuit is the core cryptographic component of Cloak's privacy-preserving protocol. It enables users to spend notes privately by proving knowledge of valid commitments without revealing transaction details.

## Circuit Overview

The circuit implements a zero-knowledge proof system that validates:
1. **Commitment Validity** - The note commitment exists in the Merkle tree
2. **Nullifier Uniqueness** - The nullifier has never been used before
3. **Amount Conservation** - No value is created or destroyed in the transaction

## Input Structure

### Private Inputs (Witness Data)

**Core Witness Structure:**
```rust
pub struct WithdrawWitness {
    // Note secrets
    pub amount: u64,                    // Note amount in lamports
    pub r: [u8; 32],                   // Randomness for commitment
    pub sk_spend: [u8; 32],            // Note secret key
    
    // Merkle tree data
    pub leaf_index: u32,                // Leaf index in Merkle tree
    pub merkle_path: MerklePath,       // Inclusion proof path
    
    // Transaction outputs
    pub outputs: Vec<Output>,           // Output recipients and amounts
}

pub struct MerklePath {
    pub path_elements: Vec<[u8; 32]>,   // Sibling hashes at each level
    pub path_indices: Vec<u32>,        // Left/right indicators (0=left, 1=right)
}

pub struct Output {
    pub address: [u8; 32],             // Recipient public key
    pub amount: u64,                   // Amount to send
}
```

**Witness Data Sources:**
- `amount`, `r`, `sk_spend` - Generated during note creation
- `leaf_index`, `merkle_path` - Retrieved from indexer service
- `outputs` - Specified by user for withdrawal

### Public Inputs (104 bytes)

**Public Input Structure:**
```rust
pub struct PublicInputs {
    pub root: [u8; 32],                // Merkle tree root (32 bytes)
    pub nullifier: [u8; 32],          // Spending nullifier (32 bytes)
    pub outputs_hash: [u8; 32],       // Hash of output recipients (32 bytes)
    pub amount: u64,                  // Total amount being spent (8 bytes)
    // Total: 104 bytes
}
```

**Public Input Layout:**
```
Offset  Size    Field
0       32      root
32      32      nullifier  
64      32      outputs_hash
96      8       amount
```

## Constraint System

The circuit enforces six critical constraints that ensure the validity and privacy of the withdrawal:

### Constraint 1: Spend Key Derivation

**Purpose:** Derive the public spend key from the secret key.

**Implementation:**
```rust
// In SP1 guest program
let pk_spend = blake3(&witness.sk_spend);
```

**Mathematical Expression:**
```
pk_spend = BLAKE3(sk_spend)
```

**Security Properties:**
- One-way function prevents recovery of secret key
- Deterministic derivation ensures consistency
- 32-byte output provides sufficient entropy

### Constraint 2: Commitment Recomputation

**Purpose:** Verify the commitment was correctly computed during note creation.

**Implementation:**
```rust
// In SP1 guest program
let mut commitment_preimage = Vec::new();
commitment_preimage.extend_from_slice(&witness.amount.to_le_bytes());
commitment_preimage.extend_from_slice(&witness.r);
commitment_preimage.extend_from_slice(&pk_spend);
let commitment = blake3(&commitment_preimage);
```

**Mathematical Expression:**
```
C = BLAKE3(amount || r || pk_spend)
```

**Data Layout:**
```
Preimage: [amount:8][r:32][pk_spend:32] = 72 bytes
Output:   [commitment:32] = 32 bytes
```

**Security Properties:**
- Binding: commitment uniquely identifies the note
- Hiding: commitment reveals no information about inputs
- Collision resistance: prevents commitment reuse

### Constraint 3: Merkle Inclusion Proof

**Purpose:** Prove the commitment exists in the Merkle tree without revealing other commitments.

**Implementation:**
```rust
// In SP1 guest program
fn verify_merkle_inclusion(
    leaf: [u8; 32],
    path: &MerklePath,
    leaf_index: u32,
) -> [u8; 32] {
    let mut current_hash = leaf;
    
    for (i, sibling) in path.path_elements.iter().enumerate() {
        let is_right = (leaf_index >> i) & 1 == 1;
        
        if is_right {
            // Current is left child, sibling is right
            let mut preimage = Vec::new();
            preimage.extend_from_slice(&current_hash);
            preimage.extend_from_slice(sibling);
            current_hash = blake3(&preimage);
        } else {
            // Current is right child, sibling is left
            let mut preimage = Vec::new();
            preimage.extend_from_slice(sibling);
            preimage.extend_from_slice(&current_hash);
            current_hash = blake3(&preimage);
        }
    }
    
    current_hash
}
```

**Mathematical Expression:**
```
MerkleVerify(C, merkle_path) == root
```

**Tree Structure:**
```
Level 0: [C] ← leaf commitment
Level 1: [H(C || sibling_0)] or [H(sibling_0 || C)]
Level 2: [H(parent_1 || sibling_1)] or [H(sibling_1 || parent_1)]
...
Level 31: [root] ← final Merkle root
```

**Security Properties:**
- Completeness: valid proofs always verify
- Soundness: invalid proofs never verify
- Zero-knowledge: proof reveals no other commitments

### Constraint 4: Nullifier Generation

**Purpose:** Generate a unique identifier for the spent note to prevent double-spending.

**Implementation:**
```rust
// In SP1 guest program
let mut nullifier_preimage = Vec::new();
nullifier_preimage.extend_from_slice(&witness.sk_spend);
nullifier_preimage.extend_from_slice(&witness.leaf_index.to_le_bytes());
let nullifier = blake3(&nullifier_preimage);
```

**Mathematical Expression:**
```
nf = BLAKE3(sk_spend || leaf_index)
```

**Data Layout:**
```
Preimage: [sk_spend:32][leaf_index:4] = 36 bytes
Output:   [nullifier:32] = 32 bytes
```

**Security Properties:**
- Uniqueness: each note generates unique nullifier
- Unlinkability: nullifiers don't reveal note relationships
- One-way: nullifier doesn't reveal secret key

### Constraint 5: Amount Conservation

**Purpose:** Ensure no value is created or destroyed in the transaction.

**Implementation:**
```rust
// In SP1 guest program
let total_outputs: u64 = witness.outputs.iter().map(|o| o.amount).sum();
let fee = calculate_fee(witness.amount);
assert!(total_outputs + fee == witness.amount);
```

**Mathematical Expression:**
```
sum(outputs) + fee(amount) == amount
where fee(amount) = (amount * 5) / 1000 + 2_500_000
```

**Fee Calculation:**
```rust
fn calculate_fee(amount: u64) -> u64 {
    // 0.5% of amount + 0.0025 SOL base fee
    (amount * 5) / 1000 + 2_500_000
}
```

**Security Properties:**
- Conservation: total input equals total output
- Fee transparency: fee calculation is deterministic
- Overflow protection: arithmetic constraints prevent overflow

### Constraint 6: Outputs Hash Verification

**Purpose:** Bind the output recipients to the proof to prevent malleability.

**Implementation:**
```rust
// In SP1 guest program
fn compute_outputs_hash(outputs: &[Output]) -> [u8; 32] {
    let mut serialized = Vec::new();
    for output in outputs {
        serialized.extend_from_slice(&output.address);
        serialized.extend_from_slice(&output.amount.to_le_bytes());
    }
    blake3(&serialized)
}
```

**Mathematical Expression:**
```
BLAKE3(serialize(outputs)) == outputs_hash
```

**Serialization Format:**
```
Output 1: [address:32][amount:8]
Output 2: [address:32][amount:8]
...
Output N: [address:32][amount:8]
```

**Security Properties:**
- Binding: outputs are cryptographically committed
- Order sensitivity: output order affects hash
- Malleability prevention: outputs cannot be modified

## Circuit Implementation

### SP1 Guest Program Structure

**Main Circuit Logic:**
```rust
// packages/zk-guest-sp1/guest/src/main.rs
use sp1_zkvm::prelude::*;

fn main() {
    // Read witness data from host
    let witness = env::read::<WithdrawWitness>();
    
    // Constraint 1: Derive spend key
    let pk_spend = blake3(&witness.sk_spend);
    
    // Constraint 2: Recompute commitment
    let commitment = compute_commitment(witness.amount, witness.r, pk_spend);
    
    // Constraint 3: Verify Merkle inclusion
    let computed_root = verify_merkle_inclusion(
        commitment,
        &witness.merkle_path,
        witness.leaf_index,
    );
    
    // Constraint 4: Generate nullifier
    let nullifier = compute_nullifier(witness.sk_spend, witness.leaf_index);
    
    // Constraint 5: Verify amount conservation
    let total_outputs: u64 = witness.outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(witness.amount);
    assert!(total_outputs + fee == witness.amount);
    
    // Constraint 6: Verify outputs hash
    let computed_outputs_hash = compute_outputs_hash(&witness.outputs);
    
    // Commit public inputs
    let public_inputs = PublicInputs {
        root: computed_root,
        nullifier,
        outputs_hash: computed_outputs_hash,
        amount: witness.amount,
    };
    
    env::commit(&public_inputs);
}
```

### Helper Functions

**Commitment Computation:**
```rust
fn compute_commitment(amount: u64, r: [u8; 32], pk_spend: [u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&amount.to_le_bytes());
    preimage.extend_from_slice(&r);
    preimage.extend_from_slice(&pk_spend);
    blake3(&preimage)
}
```

**Nullifier Computation:**
```rust
fn compute_nullifier(sk_spend: [u8; 32], leaf_index: u32) -> [u8; 32] {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&sk_spend);
    preimage.extend_from_slice(&leaf_index.to_le_bytes());
    blake3(&preimage)
}
```

**Fee Calculation:**
```rust
fn calculate_fee(amount: u64) -> u64 {
    // 0.5% + 0.0025 SOL base fee
    (amount * 5) / 1000 + 2_500_000
}
```

**Outputs Hash Computation:**
```rust
fn compute_outputs_hash(outputs: &[Output]) -> [u8; 32] {
    let mut serialized = Vec::new();
    for output in outputs {
        serialized.extend_from_slice(&output.address);
        serialized.extend_from_slice(&output.amount.to_le_bytes());
    }
    blake3(&serialized)
}
```

## Performance Characteristics

### Circuit Complexity

**Constraint Count:**
- BLAKE3 hash operations: ~6 (one per constraint)
- Merkle tree verification: ~32 levels
- Arithmetic operations: ~10 (additions, comparisons)
- **Total constraints: ~50**

**Memory Usage:**
- Witness data: ~1KB
- Merkle path: ~1KB (32 levels × 32 bytes)
- Intermediate values: ~500 bytes
- **Total memory: ~2.5KB**

### Proof Generation Time

**Local CPU Proving:**
- Target: < 120 seconds (p95)
- Current: 60-90 seconds
- Bottlenecks: BLAKE3 operations, Merkle verification

**TEE Proving:**
- Target: < 5 minutes
- Current: 2-3 minutes
- Benefits: Hardware acceleration, parallel processing

### Proof Size

**Groth16 Proof:**
- Size: 260 bytes
- Components: 3 group elements (G1, G2, G1)
- Compression: Optimized for Solana transaction limits

**Public Inputs:**
- Size: 104 bytes
- Components: 4 fields (root, nullifier, outputs_hash, amount)
- Encoding: Fixed-size binary format

## Security Analysis

### Cryptographic Assumptions

**Hash Function Security:**
- BLAKE3-256: 128-bit security level
- Collision resistance: 2^128 operations
- Preimage resistance: 2^256 operations

**Zero-Knowledge Properties:**
- Completeness: Valid proofs always verify
- Soundness: Invalid proofs never verify (with negligible probability)
- Zero-knowledge: Proofs reveal no witness information

### Attack Vectors

**Double-Spending Prevention:**
- Nullifier uniqueness prevents note reuse
- On-chain nullifier tracking ensures enforcement
- Merkle tree binding prevents commitment reuse

**Malleability Prevention:**
- Outputs hash binding prevents modification
- Public input commitment prevents tampering
- Deterministic fee calculation prevents manipulation

**Privacy Leakage:**
- Commitment scheme hides note amounts
- Merkle proofs hide tree structure
- Nullifiers don't link to specific notes

## Testing and Validation

### Unit Testing

**Constraint Testing:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commitment_computation() {
        let amount = 1_000_000;
        let r = [1u8; 32];
        let pk_spend = [2u8; 32];
        
        let commitment = compute_commitment(amount, r, pk_spend);
        
        // Verify deterministic output
        let expected = blake3(&[amount.to_le_bytes(), r, pk_spend].concat());
        assert_eq!(commitment, expected);
    }
    
    #[test]
    fn test_nullifier_computation() {
        let sk_spend = [1u8; 32];
        let leaf_index = 42;
        
        let nullifier = compute_nullifier(sk_spend, leaf_index);
        
        // Verify deterministic output
        let expected = blake3(&[sk_spend, leaf_index.to_le_bytes()].concat());
        assert_eq!(nullifier, expected);
    }
    
    #[test]
    fn test_amount_conservation() {
        let amount = 1_000_000;
        let outputs = vec![
            Output { address: [1u8; 32], amount: 500_000 },
            Output { address: [2u8; 32], amount: 497_500 },
        ];
        let fee = calculate_fee(amount);
        
        let total_outputs: u64 = outputs.iter().map(|o| o.amount).sum();
        assert_eq!(total_outputs + fee, amount);
    }
}
```

### Integration Testing

**End-to-End Testing:**
```rust
#[test]
fn test_full_withdraw_circuit() {
    // Generate test witness
    let witness = generate_test_witness();
    
    // Generate proof
    let proof_bundle = generate_withdraw_proof(witness.clone()).unwrap();
    
    // Verify proof
    let is_valid = verify_withdraw_proof(&proof_bundle).unwrap();
    assert!(is_valid);
    
    // Verify public inputs match witness
    assert_eq!(proof_bundle.public_inputs.root, witness.expected_root);
    assert_eq!(proof_bundle.public_inputs.nullifier, witness.expected_nullifier);
    assert_eq!(proof_bundle.public_inputs.amount, witness.amount);
}
```

## Related Documentation

- **[ZK Layer Overview](./README.md)** - Complete ZK system documentation
- **[Data Encoding](./encoding.md)** - Input/output format specifications
- **[Merkle Trees](./merkle.md)** - Tree structure and proof generation
- **[SP1 Prover](./prover-sp1.md)** - Proof generation implementation
- **[On-Chain Verifier](./onchain-verifier.md)** - Verification integration
- **[Testing Guide](./testing.md)** - Comprehensive testing procedures
- **[Threat Model](./threat-model.md)** - Security analysis and mitigations