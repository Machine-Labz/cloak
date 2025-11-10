# SP1 Prover Implementation Guide

This document provides a comprehensive guide to Cloak's SP1-based zero-knowledge proof generation system, including guest program implementation, host integration, artifact management, and deployment workflows.

## Overview

Cloak uses Succinct's SP1 zkVM to generate Groth16 zero-knowledge proofs for private withdrawals. The system consists of a guest program (Rust) that implements circuit constraints and a host program that orchestrates proof generation.

## Architecture

### SP1 Integration

```text
┌───────────────────────────────────────────────────────────────┐
│                    SP1 PROVER ARCHITECTURE                    │
├───────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌───────────────┐    ┌─────────────────┐  │
│  │   Host      │    │   SP1 zkVM    │    │   Guest         │  │
│  │   Program   │    │   Runtime     │    │   Program       │  │
│  │             │    │               │    │                 │  │
│  │ • Witness   │───►│ • Execution   │───►│ • Constraints   │  │
│  │   Prep      │    │ • Proving     │    │ • BLAKE3        │  │
│  │ • Artifact  │    │ • Verification│    │ • Merkle        │  │
│  │   Mgmt      │    │ • Output      │    │ • Validation    │  │
│  └─────────────┘    └───────────────┘    └─────────────────┘  │
│         │                   │                   │             │
│         │                   │                   │             │
│         ▼                   ▼                   ▼             │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │   Client    │    │   Groth16    │    │   On-Chain      │   │
│  │   App       │    │   Proof      │    │   Verifier      │   │
│  │             │    │              │    │                 │   │
│  │ • UI/UX     │    │ • 260 bytes  │    │ • SP1 Verifier  │   │
│  │ • Key Mgmt  │    │ • Public     │    │ • VKey Hash     │   │
│  │ • Proof     │    │   Inputs     │    │ • Validation    │   │
│  └─────────────┘    └──────────────┘    └─────────────────┘   │
└───────────────────────────────────────────────────────────────┘
```

## Guest Program Implementation

### Core Structure

**Guest Program Location:** `packages/zk-guest-sp1/guest/src/main.rs`

**Main Function:**
```rust
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

### Witness Structure

**Private Inputs:**
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

### Constraint Implementations

**1. Spend Key Derivation:**
```rust
fn derive_spend_key(sk_spend: [u8; 32]) -> [u8; 32] {
    blake3(&sk_spend)
}
```

**2. Commitment Recomputation:**
```rust
fn compute_commitment(amount: u64, r: [u8; 32], pk_spend: [u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&amount.to_le_bytes());
    preimage.extend_from_slice(&r);
    preimage.extend_from_slice(&pk_spend);
    blake3(&preimage)
}
```

**3. Merkle Inclusion Proof:**
```rust
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

**4. Nullifier Generation:**
```rust
fn compute_nullifier(sk_spend: [u8; 32], leaf_index: u32) -> [u8; 32] {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&sk_spend);
    preimage.extend_from_slice(&leaf_index.to_le_bytes());
    blake3(&preimage)
}
```

**5. Amount Conservation:**
```rust
fn verify_amount_conservation(amount: u64, outputs: &[Output]) {
    let total_outputs: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(amount);
    
    // Assert: total_outputs + fee == amount
    assert!(total_outputs + fee == amount);
}

fn calculate_fee(amount: u64) -> u64 {
    // 0.5% + 0.0025 SOL base fee
    (amount * 5) / 1000 + 2_500_000
}
```

**6. Outputs Hash Verification:**
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

### Public Inputs Structure

**Public Inputs (104 bytes):**
```rust
pub struct PublicInputs {
    pub root: [u8; 32],                // Merkle tree root (32 bytes)
    pub nullifier: [u8; 32],          // Spending nullifier (32 bytes)
    pub outputs_hash: [u8; 32],       // Hash of output recipients (32 bytes)
    pub amount: u64,                  // Total amount being spent (8 bytes)
}
```

## Host Program Implementation

### Host Program Location

**Host Program:** `packages/zk-guest-sp1/host/src/main.rs`

### Proof Generation Process

**Main Host Function:**
```rust
use sp1_zkvm::prelude::*;

async fn generate_withdraw_proof(witness: WithdrawWitness) -> Result<ProofBundle> {
    // 1. Load guest ELF
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

### Witness Preparation

**Client-Side Witness Building:**
```rust
pub async fn build_withdraw_witness(
    note: SpendableNote,
    outputs: Vec<Output>,
    indexer_url: &str,
) -> Result<WithdrawWitness> {
    // 1. Get Merkle root
    let root_response = reqwest::get(&format!("{}/api/v1/merkle/root", indexer_url))
        .await?
        .json::<MerkleRootResponse>()
        .await?;
    
    // 2. Get Merkle proof
    let proof_response = reqwest::get(&format!("{}/api/v1/merkle/proof/{}", indexer_url, note.leaf_index))
        .await?
        .json::<MerkleProofResponse>()
        .await?;
    
    // 3. Build witness
    let witness = WithdrawWitness {
        amount: note.amount,
        r: note.r,
        sk_spend: note.sk_spend,
        leaf_index: note.leaf_index,
        merkle_path: MerklePath {
            path_elements: proof_response.path_elements,
            path_indices: proof_response.path_indices,
        },
        outputs,
    };
    
    Ok(witness)
}
```

### Proof Extraction

**Groth16 Proof Extraction:**
```rust
use cloak_proof_extract::extract_groth16_260;

pub fn extract_groth16_260(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], String> {
    // Extract 260-byte Groth16 proof from SP1 bundle
    cloak_proof_extract::extract_groth16_260(sp1_proof_bundle)
        .map_err(|e| format!("Failed to extract Groth16 proof: {}", e))
}
```

**Public Inputs Parsing:**
```rust
use cloak_proof_extract::parse_public_inputs_104;

pub fn parse_public_inputs_104(sp1_proof_bundle: &[u8]) -> Result<PublicInputs, String> {
    // Parse 104-byte public inputs from SP1 bundle
    cloak_proof_extract::parse_public_inputs_104(sp1_proof_bundle)
        .map_err(|e| format!("Failed to parse public inputs: {}", e))
}
```

## Artifact Management

### Artifact Bundle Structure

**Artifact Bundle API:**
```http
GET /artifacts/withdraw/{version}
```

**Response Format:**
```json
{
  "guestElfUrl": "ipfs://QmHash...",
  "vk": "base64_encoded_verification_key",
  "sha256": {
    "elf": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "vk": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
  },
  "sp1Version": "0.8.0",
  "createdAt": "2025-01-15T10:30:00Z",
  "metadata": {
    "constraints": 6,
    "publicInputs": 104,
    "proofSize": 260
  }
}
```

### Artifact Caching

**Local Artifact Cache:**
```rust
pub struct ArtifactCache {
    cache_dir: PathBuf,
    artifacts: HashMap<String, ArtifactBundle>,
}

impl ArtifactCache {
    pub async fn get_artifact(&mut self, version: &str) -> Result<ArtifactBundle> {
        if let Some(artifact) = self.artifacts.get(version) {
            return Ok(artifact.clone());
        }
        
        // Download from indexer
        let artifact = self.download_artifact(version).await?;
        
        // Cache locally
        self.cache_artifact(version, &artifact).await?;
        
        Ok(artifact)
    }
    
    async fn download_artifact(&self, version: &str) -> Result<ArtifactBundle> {
        let url = format!("{}/artifacts/withdraw/{}", self.indexer_url, version);
        let response = reqwest::get(&url).await?;
        let artifact: ArtifactBundle = response.json().await?;
        
        // Verify SHA256 hashes
        self.verify_artifact_hashes(&artifact)?;
        
        Ok(artifact)
    }
}
```

### Verification Key Management

**VKey Hash Extraction:**
```rust
use vkey_generator::extract_vkey_hash;

pub fn extract_vkey_hash_from_elf(elf_path: &Path) -> Result<[u8; 32], String> {
    let elf_bytes = std::fs::read(elf_path)
        .map_err(|e| format!("Failed to read ELF file: {}", e))?;
    
    extract_vkey_hash(&elf_bytes)
        .map_err(|e| format!("Failed to extract VKey hash: {}", e))
}
```

## Client Integration

### Frontend Integration

**TypeScript Client:**
```typescript
export class CloakProver {
  private artifactCache: Map<string, ArtifactBundle> = new Map();
  
  async generateProof(
    note: SpendableNote,
    outputs: Output[],
    indexerUrl: string
  ): Promise<ProofBundle> {
    // 1. Build witness
    const witness = await this.buildWitness(note, outputs, indexerUrl);
    
    // 2. Get artifact
    const artifact = await this.getArtifact('latest');
    
    // 3. Generate proof
    const proofBundle = await this.runProver(witness, artifact);
    
    return proofBundle;
  }
  
  private async buildWitness(
    note: SpendableNote,
    outputs: Output[],
    indexerUrl: string
  ): Promise<WithdrawWitness> {
    // Get Merkle root
    const rootResponse = await fetch(`${indexerUrl}/api/v1/merkle/root`);
    const rootData = await rootResponse.json();
    
    // Get Merkle proof
    const proofResponse = await fetch(`${indexerUrl}/api/v1/merkle/proof/${note.leafIndex}`);
    const proofData = await proofResponse.json();
    
    return {
      amount: note.amount,
      r: note.r,
      skSpend: note.skSpend,
      leafIndex: note.leafIndex,
      merklePath: {
        pathElements: proofData.pathElements,
        pathIndices: proofData.pathIndices,
      },
      outputs,
    };
  }
}
```

### Relay Integration

**Relay Prover Endpoint:**
```rust
// In relay service
pub async fn prove_withdraw(
    job_id: Uuid,
    witness: WithdrawWitness,
) -> Result<ProofBundle, String> {
    // 1. Get artifact
    let artifact = artifact_cache.get_artifact("latest").await?;
    
    // 2. Generate proof
    let proof_bundle = generate_withdraw_proof(witness).await?;
    
    // 3. Store proof
    proof_store.store_proof(job_id, &proof_bundle).await?;
    
    Ok(proof_bundle)
}
```

## Performance Optimization

### Proving Performance

**Local Proving:**
- **Target:** 60-90 seconds (p95)
- **Memory:** 2-3GB RAM
- **CPU:** Multi-threaded execution
- **Storage:** ~1GB temporary files

**TEE Proving:**
- **Target:** 2-3 minutes
- **Memory:** Reduced requirements
- **CPU:** Hardware acceleration
- **Storage:** Minimal temporary files

### Optimization Strategies

**Memory Optimization:**
```rust
// Use streaming for large witness data
pub fn build_witness_streaming(
    note: SpendableNote,
    outputs: Vec<Output>,
) -> impl Iterator<Item = u8> {
    // Stream witness data instead of loading all at once
    WitnessStream::new(note, outputs)
}
```

**Parallel Processing:**
```rust
// Parallel constraint evaluation
pub fn evaluate_constraints_parallel(witness: &WithdrawWitness) -> PublicInputs {
    let (pk_spend, commitment, nullifier, outputs_hash) = rayon::join!(
        || blake3(&witness.sk_spend),
        || compute_commitment(witness.amount, witness.r, pk_spend),
        || compute_nullifier(witness.sk_spend, witness.leaf_index),
        || compute_outputs_hash(&witness.outputs)
    );
    
    // ... rest of evaluation
}
```

## Error Handling

### Common Errors

**Proving Errors:**
```rust
pub enum ProvingError {
    WitnessInvalid { reason: String },
    ConstraintViolation { constraint: String },
    MerkleProofInvalid { leaf_index: u32 },
    AmountConservationFailed { expected: u64, actual: u64 },
    ArtifactNotFound { version: String },
    ProofGenerationFailed { reason: String },
}
```

**Error Recovery:**
```rust
impl CloakProver {
    pub async fn generate_proof_with_retry(
        &self,
        witness: WithdrawWitness,
        max_retries: u32,
    ) -> Result<ProofBundle> {
        for attempt in 0..max_retries {
            match self.generate_proof(witness.clone()).await {
                Ok(proof) => return Ok(proof),
                Err(e) => {
                    if attempt == max_retries - 1 {
                        return Err(e);
                    }
                    
                    // Wait before retry
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                }
            }
        }
        
        Err("Max retries exceeded".into())
    }
}
```

## Testing & Validation

### Unit Tests

**Constraint Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spend_key_derivation() {
        let sk_spend = [0x01u8; 32];
        let pk_spend = derive_spend_key(sk_spend);
        
        // Verify deterministic output
        let expected = blake3(&sk_spend);
        assert_eq!(pk_spend, expected);
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
    }
}
```

**Integration Tests:**
```rust
#[test]
fn test_end_to_end_proving() {
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

## Deployment & CI/CD

### Build Process

**Guest ELF Build:**
```bash
# Build guest program for RISC-V target
cargo build --release --target riscv32im-succinct-zkvm-elf --package zk-guest-sp1-guest

# Verify ELF
file target/riscv32im-succinct-zkvm-elf/release/guest
```

**Host Program Build:**
```bash
# Build host program
cargo build --release --package zk-guest-sp1-host

# Run tests
cargo test --package zk-guest-sp1-host
```

### CI/CD Pipeline

**GitHub Actions Workflow:**
```yaml
name: SP1 Prover CI/CD

on:
  push:
    branches: [main]
    paths: ['packages/zk-guest-sp1/**']

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: riscv32im-succinct-zkvm-elf
          
      - name: Build Guest ELF
        run: cargo build --release --target riscv32im-succinct-zkvm-elf --package zk-guest-sp1-guest
        
      - name: Run Tests
        run: cargo test --package zk-guest-sp1-host
        
      - name: Extract VKey Hash
        run: cargo run --package vkey-generator
        
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: sp1-artifacts
          path: target/riscv32im-succinct-zkvm-elf/release/guest
```

## Monitoring & Metrics

### Key Metrics

**Proving Metrics:**
```rust
pub struct ProvingMetrics {
    pub proof_generation_time_ms: f64,
    pub proof_size_bytes: usize,
    pub public_inputs_size_bytes: usize,
    pub witness_size_bytes: usize,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}
```

**Performance Monitoring:**
- Proof generation latency (p50, p95, p99)
- Memory usage during proving
- CPU utilization
- Success/failure rates
- Artifact download times

This SP1 prover implementation provides a robust foundation for zero-knowledge proof generation in Cloak's privacy-preserving protocol.