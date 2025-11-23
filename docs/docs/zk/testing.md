# ZK Testing Strategy & Implementation

This document outlines the comprehensive testing strategy for Cloak's zero-knowledge privacy layer, including unit tests, property tests, integration tests, and end-to-end validation.

## Testing Overview

### Testing Pyramid

```mermaid
graph TD
    subgraph "ZK TESTING PYRAMID"
        E2E[E2E Tests (localnet)<br/>• Deposit → Indexer → Proof → Withdraw → Success<br/>• Multiple users, complex scenarios]
        
        INT[Integration Tests<br/>• On-chain verification<br/>• Service interactions<br/>• API contract validation]
        
        PROP[Property Tests<br/>• Randomized Merkle trees<br/>• Double-spend prevention<br/>• Edge case validation]
        
        UNIT[Unit Tests<br/>• Constraint verification<br/>• Hash function consistency<br/>• Encoding/decoding]
    end
    
    E2E --> INT
    INT --> PROP
    PROP --> UNIT
```

## Unit Tests

### Guest Program Tests

**Constraint Verification Tests:**
```rust
#[cfg(test)]
mod guest_tests {
    use super::*;
    
    #[test]
    fn test_spend_key_derivation() {
        let sk_spend = [0x01u8; 32];
        let pk_spend = derive_spend_key(sk_spend);
        
        // Verify deterministic output
        let expected = blake3(&sk_spend);
        assert_eq!(pk_spend, expected);
        
        // Verify different inputs produce different outputs
        let sk_spend2 = [0x02u8; 32];
        let pk_spend2 = derive_spend_key(sk_spend2);
        assert_ne!(pk_spend, pk_spend2);
    }
    
    #[test]
    fn test_commitment_computation() {
        let amount = 1_000_000u64;
        let r = [0x01u8; 32];
        let pk_spend = [0x02u8; 32];
        
        let commitment = compute_commitment(amount, r, pk_spend);
        
        // Verify deterministic output
        let mut preimage = Vec::new();
        preimage.extend_from_slice(&amount.to_le_bytes());
        preimage.extend_from_slice(&r);
        preimage.extend_from_slice(&pk_spend);
        let expected = blake3(&preimage);
        
        assert_eq!(commitment, expected);
        
        // Verify different inputs produce different commitments
        let commitment2 = compute_commitment(amount + 1, r, pk_spend);
        assert_ne!(commitment, commitment2);
    }
    
    #[test]
    fn test_nullifier_generation() {
        let sk_spend = [0x03u8; 32];
        let leaf_index = 42u32;
        
        let nullifier = compute_nullifier(sk_spend, leaf_index);
        
        // Verify deterministic output
        let mut preimage = Vec::new();
        preimage.extend_from_slice(&sk_spend);
        preimage.extend_from_slice(&leaf_index.to_le_bytes());
        let expected = blake3(&preimage);
        
        assert_eq!(nullifier, expected);
        
        // Verify different leaf indices produce different nullifiers
        let nullifier2 = compute_nullifier(sk_spend, leaf_index + 1);
        assert_ne!(nullifier, nullifier2);
    }
}
```

**Merkle Path Verification Tests:**
```rust
#[test]
fn test_merkle_path_verification() {
    // Test with known Merkle tree
    let mut tree = MerkleTree::new();
    
    // Add some leaves
    let commitments = vec![
        [0x01u8; 32],
        [0x02u8; 32],
        [0x03u8; 32],
        [0x04u8; 32],
    ];
    
    let mut indices = Vec::new();
    for commitment in commitments {
        let index = tree.append_leaf(commitment);
        indices.push(index);
    }
    
    // Verify all paths
    for (i, &commitment) in commitments.iter().enumerate() {
        let path = tree.generate_merkle_path(indices[i]).unwrap();
        let is_valid = verify_merkle_path(commitment, &path, indices[i], tree.root).unwrap();
        assert!(is_valid);
    }
}

#[test]
fn test_merkle_path_invalid() {
    let leaf = [0x01u8; 32];
    let path = MerklePath {
        path_elements: vec![[0x02u8; 32]; 32],
        path_indices: vec![0; 32],
    };
    let leaf_index = 0;
    let wrong_root = [0x03u8; 32];
    
    let is_valid = verify_merkle_path(leaf, &path, leaf_index, wrong_root).unwrap();
    assert!(!is_valid);
}
```

**Amount Conservation Tests:**
```rust
#[test]
fn test_amount_conservation() {
    let amount = 1_000_000u64;
    let outputs = vec![
        Output { address: [0x01u8; 32], amount: 500_000 },
        Output { address: [0x02u8; 32], amount: 300_000 },
    ];
    
    let total_outputs: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(amount);
    
    // Should pass: 500_000 + 300_000 + 7_500 = 807_500
    assert_eq!(total_outputs + fee, amount);
}

#[test]
fn test_amount_conservation_failure() {
    let amount = 1_000_000u64;
    let outputs = vec![
        Output { address: [0x01u8; 32], amount: 600_000 },
        Output { address: [0x02u8; 32], amount: 500_000 },
    ];
    
    let total_outputs: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(amount);
    
    // Should fail: 600_000 + 500_000 + 7_500 = 1_107_500 > 1_000_000
    assert_ne!(total_outputs + fee, amount);
}

#[test]
fn test_fee_calculation() {
    let amount = 1_000_000u64;
    let fee = calculate_fee(amount);
    
    // 0.5% + 0.0025 SOL base fee
    let expected_variable = (amount * 5) / 1000; // 5,000
    let expected_fixed = 2_500_000; // 0.0025 SOL
    let expected_total = expected_variable + expected_fixed; // 2,505,000
    
    assert_eq!(fee, expected_total);
}
```

**Outputs Hash Tests:**
```rust
#[test]
fn test_outputs_hash_computation() {
    let outputs = vec![
        Output { address: [0x01u8; 32], amount: 500_000 },
        Output { address: [0x02u8; 32], amount: 300_000 },
    ];
    
    let outputs_hash = compute_outputs_hash(&outputs);
    
    // Verify deterministic output
    let mut serialized = Vec::new();
    for output in &outputs {
        serialized.extend_from_slice(&output.address);
        serialized.extend_from_slice(&output.amount.to_le_bytes());
    }
    let expected = blake3(&serialized);
    
    assert_eq!(outputs_hash, expected);
}

#[test]
fn test_outputs_hash_order_sensitivity() {
    let outputs1 = vec![
        Output { address: [0x01u8; 32], amount: 500_000 },
        Output { address: [0x02u8; 32], amount: 300_000 },
    ];
    
    let outputs2 = vec![
        Output { address: [0x02u8; 32], amount: 300_000 },
        Output { address: [0x01u8; 32], amount: 500_000 },
    ];
    
    let hash1 = compute_outputs_hash(&outputs1);
    let hash2 = compute_outputs_hash(&outputs2);
    
    // Different order should produce different hash
    assert_ne!(hash1, hash2);
}
```

## Property Tests

### Randomized Merkle Tree Tests

**Merkle Tree Property Tests:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_merkle_inclusion_property(
        leaves in prop::collection::vec(any::<[u8; 32]>(), 1..100),
        leaf_index in 0..100usize
    ) {
        let mut tree = MerkleTree::new();
        
        // Add leaves
        for leaf in &leaves {
            tree.append_leaf(*leaf);
        }
        
        // Test inclusion for valid leaf index
        if leaf_index < leaves.len() {
            let path = tree.generate_merkle_path(leaf_index as u32).unwrap();
            let is_valid = verify_merkle_path(
                leaves[leaf_index], 
                &path, 
                leaf_index as u32, 
                tree.root
            ).unwrap();
            assert!(is_valid);
        }
    }
    
    #[test]
    fn test_merkle_path_invalid_property(
        leaves in prop::collection::vec(any::<[u8; 32]>(), 1..100),
        leaf_index in 0..100usize,
        wrong_sibling in any::<[u8; 32]>()
    ) {
        let mut tree = MerkleTree::new();
        
        // Add leaves
        for leaf in &leaves {
            tree.append_leaf(*leaf);
        }
        
        // Test with wrong sibling
        if leaf_index < leaves.len() {
            let mut path = tree.generate_merkle_path(leaf_index as u32).unwrap();
            path.path_elements[0] = wrong_sibling; // Corrupt first sibling
            
            let is_valid = verify_merkle_path(
                leaves[leaf_index], 
                &path, 
                leaf_index as u32, 
                tree.root
            ).unwrap();
            assert!(!is_valid);
        }
    }
}
```

### Double-Spend Prevention Tests

**Nullifier Uniqueness Tests:**
```rust
proptest! {
    #[test]
    fn test_nullifier_uniqueness_property(
        sk_spend in any::<[u8; 32]>(),
        leaf_index1 in 0..1000u32,
        leaf_index2 in 0..1000u32
    ) {
        let nullifier1 = compute_nullifier(sk_spend, leaf_index1);
        let nullifier2 = compute_nullifier(sk_spend, leaf_index2);
        
        if leaf_index1 != leaf_index2 {
            assert_ne!(nullifier1, nullifier2);
        } else {
            assert_eq!(nullifier1, nullifier2);
        }
    }
    
    #[test]
    fn test_nullifier_collision_resistance(
        sk_spend1 in any::<[u8; 32]>(),
        sk_spend2 in any::<[u8; 32]>(),
        leaf_index in 0..1000u32
    ) {
        let nullifier1 = compute_nullifier(sk_spend1, leaf_index);
        let nullifier2 = compute_nullifier(sk_spend2, leaf_index);
        
        if sk_spend1 != sk_spend2 {
            assert_ne!(nullifier1, nullifier2);
        } else {
            assert_eq!(nullifier1, nullifier2);
        }
    }
}
```

### Edge Case Tests

**Overflow Protection Tests:**
```rust
#[test]
fn test_amount_overflow_protection() {
    let max_amount = u64::MAX;
    let outputs = vec![
        Output { address: [0x01u8; 32], amount: max_amount / 2 },
        Output { address: [0x02u8; 32], amount: max_amount / 2 },
    ];
    
    // Should not panic on overflow
    let total_outputs: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(max_amount);
    
    // Verify overflow handling
    assert!(total_outputs.checked_add(fee).is_some());
}

#[test]
fn test_fee_overflow_protection() {
    let amount = u64::MAX;
    let fee = calculate_fee(amount);
    
    // Fee calculation should not overflow
    assert!(fee < amount);
}
```

## Integration Tests

### On-Chain Verification Tests

**Shield Pool Program Tests:**
```rust
#[cfg(test)]
mod onchain_tests {
    use super::*;
    
    #[test]
    fn test_withdraw_success() {
        let mut test_context = TestContext::new();
        
        // Setup: Deposit and get commitment
        let commitment = test_context.deposit(1_000_000);
        
        // Generate proof
        let proof_bundle = test_context.generate_proof(commitment);
        
        // Execute withdraw
        let result = test_context.withdraw(proof_bundle);
        
        // Verify success
        assert!(result.is_ok());
        
        // Verify recipients credited
        assert_eq!(test_context.get_balance(&recipient1), 500_000);
        assert_eq!(test_context.get_balance(&recipient2), 300_000);
        
        // Verify fee sent to treasury
        assert_eq!(test_context.get_treasury_balance(), 7_500);
        
        // Verify nullifier marked as used
        assert!(test_context.is_nullifier_used(&proof_bundle.public_inputs.nullifier));
    }
    
    #[test]
    fn test_invalid_root_rejected() {
        let mut test_context = TestContext::new();
        
        // Setup: Deposit and get commitment
        let commitment = test_context.deposit(1_000_000);
        
        // Generate proof with wrong root
        let mut proof_bundle = test_context.generate_proof(commitment);
        proof_bundle.public_inputs.root = [0xFFu8; 32]; // Wrong root
        
        // Execute withdraw
        let result = test_context.withdraw(proof_bundle);
        
        // Verify rejection
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("RootNotFound"));
    }
    
    #[test]
    fn test_outputs_mismatch_rejected() {
        let mut test_context = TestContext::new();
        
        // Setup: Deposit and get commitment
        let commitment = test_context.deposit(1_000_000);
        
        // Generate proof with wrong outputs
        let mut proof_bundle = test_context.generate_proof(commitment);
        proof_bundle.public_inputs.outputs_hash = [0xFFu8; 32]; // Wrong hash
        
        // Execute withdraw
        let result = test_context.withdraw(proof_bundle);
        
        // Verify rejection
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OutputsMismatch"));
    }
    
    #[test]
    fn test_double_spend_rejected() {
        let mut test_context = TestContext::new();
        
        // Setup: Deposit and get commitment
        let commitment = test_context.deposit(1_000_000);
        
        // Generate proof
        let proof_bundle = test_context.generate_proof(commitment);
        
        // Execute first withdraw
        let result1 = test_context.withdraw(proof_bundle.clone());
        assert!(result1.is_ok());
        
        // Try to execute same withdraw again
        let result2 = test_context.withdraw(proof_bundle);
        
        // Verify rejection
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("DoubleSpend"));
    }
}
```

### Service Integration Tests

**Indexer-Relay Integration Tests:**
```rust
#[tokio::test]
async fn test_indexer_relay_integration() {
    let mut test_env = TestEnvironment::new().await;
    
    // 1. Deposit via relay
    let deposit_result = test_env.relay.deposit(1_000_000).await;
    assert!(deposit_result.is_ok());
    
    // 2. Wait for indexer to process
    test_env.wait_for_indexer_update().await;
    
    // 3. Verify indexer has new root
    let root_response = test_env.indexer.get_root().await.unwrap();
    assert_ne!(root_response.root, [0u8; 32]);
    
    // 4. Get Merkle proof
    let proof_response = test_env.indexer.get_proof(0).await.unwrap();
    assert_eq!(proof_response.path_elements.len(), 32);
    
    // 5. Submit withdraw via relay
    let withdraw_result = test_env.relay.withdraw(proof_response).await;
    assert!(withdraw_result.is_ok());
}
```

## End-to-End Tests

### Localnet E2E Tests

**Complete Deposit-Withdraw Flow:**
```rust
#[tokio::test]
async fn test_complete_deposit_withdraw_flow() {
    let mut test_env = TestEnvironment::new().await;
    
    // 1. User deposits SOL
    let deposit_amount = 1_000_000;
    let deposit_result = test_env.user.deposit(deposit_amount).await;
    assert!(deposit_result.is_ok());
    
    // 2. Indexer processes deposit
    test_env.wait_for_indexer_update().await;
    
    // 3. User discovers notes
    let notes = test_env.user.discover_notes().await.unwrap();
    assert_eq!(notes.len(), 1);
    
    // 4. User generates proof
    let outputs = vec![
        Output { address: test_env.recipient1, amount: 500_000 },
        Output { address: test_env.recipient2, amount: 300_000 },
    ];
    
    let proof_bundle = test_env.user.generate_proof(notes[0].clone(), outputs).await.unwrap();
    
    // 5. User submits withdraw
    let withdraw_result = test_env.user.withdraw(proof_bundle).await;
    assert!(withdraw_result.is_ok());
    
    // 6. Verify recipients credited
    let balance1 = test_env.get_balance(test_env.recipient1).await;
    let balance2 = test_env.get_balance(test_env.recipient2).await;
    
    assert_eq!(balance1, 500_000);
    assert_eq!(balance2, 300_000);
    
    // 7. Verify fee sent to treasury
    let treasury_balance = test_env.get_treasury_balance().await;
    assert_eq!(treasury_balance, 7_500);
}
```

**Multiple User E2E Test:**
```rust
#[tokio::test]
async fn test_multiple_users_e2e() {
    let mut test_env = TestEnvironment::new().await;
    
    // Create multiple users
    let users = vec![
        TestUser::new("user1").await,
        TestUser::new("user2").await,
        TestUser::new("user3").await,
    ];
    
    // Each user deposits
    for user in &users {
        let deposit_result = user.deposit(1_000_000).await;
        assert!(deposit_result.is_ok());
    }
    
    // Wait for all deposits to be processed
    test_env.wait_for_indexer_update().await;
    
    // Each user withdraws
    for user in &users {
        let notes = user.discover_notes().await.unwrap();
        assert_eq!(notes.len(), 1);
        
        let outputs = vec![
            Output { address: test_env.recipient1, amount: 500_000 },
        ];
        
        let proof_bundle = user.generate_proof(notes[0].clone(), outputs).await.unwrap();
        let withdraw_result = user.withdraw(proof_bundle).await;
        assert!(withdraw_result.is_ok());
    }
    
    // Verify final balances
    let final_balance = test_env.get_balance(test_env.recipient1).await;
    assert_eq!(final_balance, 1_500_000); // 3 users × 500,000 each
}
```

## Golden File Tests

### Test Vector Management

**Golden Test Vectors:**
```rust
#[cfg(test)]
mod golden_tests {
    use super::*;
    
    #[test]
    fn test_commitment_golden_vectors() {
        let test_vectors = include_str!("../test_vectors/commitments.json");
        let vectors: Vec<CommitmentTestVector> = serde_json::from_str(test_vectors).unwrap();
        
        for vector in vectors {
            let commitment = compute_commitment(
                vector.amount,
                vector.r,
                vector.pk_spend
            );
            
            assert_eq!(commitment, vector.expected_commitment);
        }
    }
    
    #[test]
    fn test_nullifier_golden_vectors() {
        let test_vectors = include_str!("../test_vectors/nullifiers.json");
        let vectors: Vec<NullifierTestVector> = serde_json::from_str(test_vectors).unwrap();
        
        for vector in vectors {
            let nullifier = compute_nullifier(
                vector.sk_spend,
                vector.leaf_index
            );
            
            assert_eq!(nullifier, vector.expected_nullifier);
        }
    }
    
    #[test]
    fn test_merkle_path_golden_vectors() {
        let test_vectors = include_str!("../test_vectors/merkle_paths.json");
        let vectors: Vec<MerklePathTestVector> = serde_json::from_str(test_vectors).unwrap();
        
        for vector in vectors {
            let is_valid = verify_merkle_path(
                vector.leaf,
                &vector.path,
                vector.leaf_index,
                vector.expected_root
            ).unwrap();
            
            assert_eq!(is_valid, vector.expected_valid);
        }
    }
}
```

**Test Vector Structure:**
```json
{
  "commitments": [
    {
      "amount": 1000000,
      "r": "0101010101010101010101010101010101010101010101010101010101010101",
      "pk_spend": "0202020202020202020202020202020202020202020202020202020202020202",
      "expected_commitment": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456"
    }
  ],
  "nullifiers": [
    {
      "sk_spend": "0303030303030303030303030303030303030303030303030303030303030303",
      "leaf_index": 42,
      "expected_nullifier": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
    }
  ],
  "merkle_paths": [
    {
      "leaf": "0404040404040404040404040404040404040404040404040404040404040404",
      "path": {
        "path_elements": ["0505050505050505050505050505050505050505050505050505050505050505"],
        "path_indices": [0]
      },
      "leaf_index": 0,
      "expected_root": "0606060606060606060606060606060606060606060606060606060606060606",
      "expected_valid": true
    }
  ]
}
```

## Performance Tests

### Benchmarking

**Proof Generation Benchmarks:**
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");
    
    group.bench_function("small_witness", |b| {
        let witness = generate_test_witness(1_000_000, 1);
        b.iter(|| generate_withdraw_proof(witness.clone()))
    });
    
    group.bench_function("large_witness", |b| {
        let witness = generate_test_witness(10_000_000, 10);
        b.iter(|| generate_withdraw_proof(witness.clone()))
    });
    
    group.finish();
}

fn benchmark_merkle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_operations");
    
    group.bench_function("append_leaf", |b| {
        let mut tree = MerkleTree::new();
        b.iter(|| tree.append_leaf([0x01u8; 32]))
    });
    
    group.bench_function("generate_proof", |b| {
        let mut tree = MerkleTree::new();
        for i in 0..1000 {
            tree.append_leaf([i as u8; 32]);
        }
        b.iter(|| tree.generate_merkle_path(500))
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_proof_generation, benchmark_merkle_operations);
criterion_main!(benches);
```

## Test Infrastructure

### Test Environment Setup

**Test Environment:**
```rust
pub struct TestEnvironment {
    pub indexer: IndexerClient,
    pub relay: RelayClient,
    pub shield_pool: ShieldPoolClient,
    pub users: Vec<TestUser>,
    pub recipients: Vec<Pubkey>,
}

impl TestEnvironment {
    pub async fn new() -> Self {
        // Start local services
        let indexer = IndexerClient::new("http://localhost:3001").await;
        let relay = RelayClient::new("http://localhost:3002").await;
        let shield_pool = ShieldPoolClient::new("http://localhost:8899").await;
        
        // Deploy programs
        shield_pool.deploy().await.unwrap();
        
        // Initialize services
        indexer.initialize().await.unwrap();
        relay.initialize().await.unwrap();
        
        Self {
            indexer,
            relay,
            shield_pool,
            users: Vec::new(),
            recipients: vec![
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            ],
        }
    }
    
    pub async fn wait_for_indexer_update(&self) {
        // Wait for indexer to process new deposits
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### Test Utilities

**Test Helper Functions:**
```rust
pub fn generate_test_witness(amount: u64, num_outputs: usize) -> WithdrawWitness {
    let mut outputs = Vec::new();
    for i in 0..num_outputs {
        outputs.push(Output {
            address: [i as u8; 32],
            amount: amount / num_outputs as u64,
        });
    }
    
    WithdrawWitness {
        amount,
        r: [0x01u8; 32],
        sk_spend: [0x02u8; 32],
        leaf_index: 0,
        merkle_path: generate_test_merkle_path(),
        outputs,
    }
}

pub fn generate_test_merkle_path() -> MerklePath {
    MerklePath {
        path_elements: vec![[0x03u8; 32]; 32],
        path_indices: vec![0; 32],
    }
}
```

## Continuous Integration

### GitHub Actions Test Pipeline

**Test Workflow:**
```yaml
name: ZK Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: riscv32im-succinct-zkvm-elf
          
      - name: Run Unit Tests
        run: cargo test --package zk-guest-sp1-guest
        
      - name: Run Property Tests
        run: cargo test --package zk-guest-sp1-host --features proptest
        
  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
          
      postgres:
        image: postgres:15
        options: >-
          --health-cmd "pg_isready -U cloak"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
          
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        
      - name: Start Solana Test Validator
        run: |
          solana-test-validator --reset --quiet &
          sleep 10
          
      - name: Deploy Programs
        run: |
          solana program deploy target/deploy/shield_pool.so
          solana program deploy target/deploy/scramble_registry.so
          
      - name: Run Integration Tests
        run: cargo test --package zk-guest-sp1-host --test integration
        
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        
      - name: Start Services
        run: |
          docker-compose up -d
          sleep 30
          
      - name: Run E2E Tests
        run: cargo test --package zk-guest-sp1-host --test e2e
        
      - name: Upload Test Results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: target/test-results/
```

This comprehensive testing strategy ensures the reliability, security, and correctness of Cloak's zero-knowledge privacy layer across all components and use cases.
