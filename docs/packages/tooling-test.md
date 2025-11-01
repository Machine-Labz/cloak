---
title: Testing Toolkit
description: Shared Rust helpers for localnet/testnet end-to-end testing of the shield pool and relay stack.
---

# Testing Toolkit

`tooling/test` contains reusable Rust helpers for integration tests, smoke tests, and manual QA scenarios across Cloak's privacy-preserving protocol.

## Overview

The testing toolkit provides a comprehensive suite of utilities for testing Cloak's components in isolation and integration. It includes helpers for Solana program testing, service integration, proof generation, and end-to-end workflow validation.

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                    TESTING TOOLKIT ARCHITECTURE                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Test      │    │   Solana     │    │   Service       │    │
│  │   Config    │    │   Helpers    │    │   Helpers       │    │
│  │             │    │              │    │                 │    │
│  │ • Localnet  │    │ • Keypairs   │    │ • Indexer      │    │
│  │ • Testnet   │    │ • RPC Client │    │ • Relay        │    │
│  │ • Mainnet   │    │ • PDAs       │    │ • WebSocket     │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
│         │                   │                   │               │
│         │                   │                   │               │
│         ▼                   ▼                   ▼               │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Proof     │    │   Merkle     │    │   Integration   │    │
│  │   Helpers   │    │   Tree       │    │   Tests         │    │
│  │             │    │   Helpers    │    │                 │    │
│  │ • Witness   │    │ • Path Gen   │    │ • E2E Flows     │    │
│  │ • Generation│    │ • Verification│   │ • Smoke Tests   │    │
│  │ • Validation│    │ • Test Data  │    │ • Load Tests    │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Core Capabilities

### Solana Integration

**Keypair Management:**
- Load Solana keypairs from JSON or binary files
- Generate test keypairs for different scenarios
- Manage admin and user keypairs securely

**RPC Client Helpers:**
- Query cluster health and status
- Fund user accounts from admin keypairs
- Handle transaction confirmation and retries
- Monitor account balances and program state

**PDA Address Construction:**
- Generate PDA addresses for shield pool program
- Construct roots ring, nullifier shard, and treasury addresses
- Validate PDA derivation consistency

### Service Integration

**Indexer Service Helpers:**
- Query Merkle tree state and proofs
- Validate root updates and leaf additions
- Test note discovery and encryption/decryption

**Relay Service Helpers:**
- Submit withdraw requests and track status
- Validate proof generation and submission
- Test job queue processing and error handling

**WebSocket Subscriptions:**
- Real-time updates for root changes
- Withdraw status notifications
- Service health monitoring

## Key Structures

### Test Configuration

**TestConfig Structure:**
```rust
pub struct TestConfig {
    pub environment: Environment,
    pub rpc_url: String,
    pub ws_url: String,
    pub shield_pool_program_id: Pubkey,
    pub scramble_registry_program_id: Pubkey,
    pub indexer_url: String,
    pub relay_url: String,
    pub admin_keypair_path: String,
    pub user_keypair_path: String,
    pub min_balance: u64,
    pub test_amounts: TestAmounts,
}

pub enum Environment {
    Localnet,
    Testnet,
    Mainnet,
}

pub struct TestAmounts {
    pub small: u64,    // 0.001 SOL
    pub medium: u64,   // 0.01 SOL
    pub large: u64,    // 0.1 SOL
    pub max: u64,      // 1 SOL
}
```

**Configuration Methods:**
```rust
impl TestConfig {
    pub fn localnet() -> Self {
        Self {
            environment: Environment::Localnet,
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: "ws://localhost:8900".to_string(),
            shield_pool_program_id: Pubkey::from_str("...").unwrap(),
            scramble_registry_program_id: Pubkey::from_str("...").unwrap(),
            indexer_url: "http://localhost:3001".to_string(),
            relay_url: "http://localhost:3002".to_string(),
            admin_keypair_path: "admin-keypair.json".to_string(),
            user_keypair_path: "user-keypair.json".to_string(),
            min_balance: 1_000_000_000, // 1 SOL
            test_amounts: TestAmounts {
                small: 1_000_000,   // 0.001 SOL
                medium: 10_000_000, // 0.01 SOL
                large: 100_000_000, // 0.1 SOL
                max: 1_000_000_000, // 1 SOL
            },
        }
    }
    
    pub fn testnet() -> Self {
        // Testnet configuration
    }
    
    pub fn mainnet() -> Self {
        // Mainnet configuration
    }
}
```

### Merkle Proof Structures

**MerkleProof Structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub path_elements: Vec<String>,  // Hex-encoded sibling hashes
    pub path_indices: Vec<u32>,      // Left/right indicators
    pub leaf: String,                // Hex-encoded leaf commitment
    pub root: String,                // Hex-encoded tree root
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleRootResponse {
    pub root: String,
    pub next_index: u32,
    pub height: u32,
    pub updated_at: String,
}
```

### Deposit Request Structures

**DepositRequest Structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    pub amount: u64,
    pub commitment: String,          // Hex-encoded commitment
    pub encrypted_output: String,    // Hex-encoded encrypted output
    pub leaf_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResponse {
    pub txid: String,
    pub leaf_index: u32,
    pub commitment: String,
    pub encrypted_output: String,
}
```

## Helper Functions

### Keypair Management

**Load Keypair:**
```rust
pub fn load_keypair(path: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    
    if path.ends_with(".json") {
        // JSON format
        let json: serde_json::Value = serde_json::from_slice(&data)?;
        let bytes: Vec<u8> = json["secretKey"].as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        Ok(Keypair::from_bytes(&bytes)?)
    } else {
        // Binary format
        Ok(Keypair::from_bytes(&data)?)
    }
}
```

**Generate Test Keypair:**
```rust
pub fn generate_test_keypair(name: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
    let keypair = Keypair::new();
    let path = format!("{}-keypair.json", name);
    save_keypair(&keypair, &path)?;
    Ok(keypair)
}
```

### Cluster Health

**Check Cluster Health:**
```rust
pub async fn check_cluster_health(rpc_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new(rpc_url.to_string());
    
    // Check RPC availability
    let version = client.get_version().await?;
    println!("Connected to Solana cluster: {}", version.solana_core);
    
    // Check cluster health
    let health = client.get_health().await?;
    if health != RpcResponse::Ok {
        return Err("Cluster is unhealthy".into());
    }
    
    // Check recent blockhash
    let blockhash = client.get_latest_blockhash().await?;
    println!("Latest blockhash: {}", blockhash);
    
    Ok(())
}
```

### Account Funding

**Ensure User Funding:**
```rust
pub async fn ensure_user_funding(
    rpc_url: &str,
    user: &Keypair,
    admin: &Keypair,
    min_balance: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new(rpc_url.to_string());
    
    // Check user balance
    let balance = client.get_balance(&user.pubkey()).await?;
    
    if balance < min_balance {
        let amount = min_balance - balance;
        println!("Funding user account with {} lamports", amount);
        
        // Transfer from admin to user
        let transfer_instruction = system_instruction::transfer(
            &admin.pubkey(),
            &user.pubkey(),
            amount,
        );
        
        let recent_blockhash = client.get_latest_blockhash().await?;
        let transaction = Transaction::new_signed_with_payer(
            &[transfer_instruction],
            Some(&admin.pubkey()),
            &[admin],
            recent_blockhash,
        );
        
        client.send_and_confirm_transaction(&transaction).await?;
        println!("User account funded successfully");
    }
    
    Ok(())
}
```

### PDA Address Construction

**Get PDA Addresses:**
```rust
pub fn get_pda_addresses(program_id: &Pubkey) -> Result<PdaAddresses, Box<dyn std::error::Error>> {
    let (pool_pda, _) = Pubkey::find_program_address(
        &[b"pool"],
        program_id,
    );
    
    let (roots_ring_pda, _) = Pubkey::find_program_address(
        &[b"roots_ring"],
        program_id,
    );
    
    let (treasury_pda, _) = Pubkey::find_program_address(
        &[b"treasury"],
        program_id,
    );
    
    Ok(PdaAddresses {
        pool: pool_pda,
        roots_ring: roots_ring_pda,
        treasury: treasury_pda,
    })
}

pub struct PdaAddresses {
    pub pool: Pubkey,
    pub roots_ring: Pubkey,
    pub treasury: Pubkey,
}
```

### Instruction Builders

**Build Deposit Instruction:**
```rust
pub fn build_deposit_instruction(
    program_id: &Pubkey,
    user: &Pubkey,
    pool_pda: &Pubkey,
    treasury_pda: &Pubkey,
    amount: u64,
    commitment: [u8; 32],
    encrypted_output: Vec<u8>,
) -> Result<Instruction, Box<dyn std::error::Error>> {
    let accounts = vec![
        AccountMeta::new(*user, true),
        AccountMeta::new(*pool_pda, false),
        AccountMeta::new(*treasury_pda, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    
    let data = DepositInstruction {
        amount,
        commitment,
        encrypted_output,
    };
    
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: data.try_to_vec()?,
    })
}
```

**Build Withdraw Instruction:**
```rust
pub fn build_withdraw_instruction(
    program_id: &Pubkey,
    user: &Pubkey,
    pool_pda: &Pubkey,
    roots_ring_pda: &Pubkey,
    treasury_pda: &Pubkey,
    outputs: Vec<Output>,
    public_inputs: PublicInputs,
    proof_bytes: [u8; 260],
) -> Result<Instruction, Box<dyn std::error::Error>> {
    let accounts = vec![
        AccountMeta::new(*user, true),
        AccountMeta::new(*pool_pda, false),
        AccountMeta::new(*roots_ring_pda, false),
        AccountMeta::new(*treasury_pda, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    
    let data = WithdrawInstruction {
        outputs,
        public_inputs,
        proof_bytes,
    };
    
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: data.try_to_vec()?,
    })
}
```

## Service Integration Helpers

### Indexer Service

**Indexer Client:**
```rust
pub struct IndexerClient {
    base_url: String,
    client: reqwest::Client,
}

impl IndexerClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn get_root(&self) -> Result<MerkleRootResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/merkle/root", self.base_url))
            .send()
            .await?;
        
        let root: MerkleRootResponse = response.json().await?;
        Ok(root)
    }
    
    pub async fn get_proof(&self, leaf_index: u32) -> Result<MerkleProof, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/merkle/proof/{}", self.base_url, leaf_index))
            .send()
            .await?;
        
        let proof: MerkleProof = response.json().await?;
        Ok(proof)
    }
}
```

### Relay Service

**Relay Client:**
```rust
pub struct RelayClient {
    base_url: String,
    client: reqwest::Client,
}

impl RelayClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn submit_withdraw(&self, request: WithdrawRequest) -> Result<WithdrawResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/withdraw", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        let withdraw: WithdrawResponse = response.json().await?;
        Ok(withdraw)
    }
    
    pub async fn get_withdraw_status(&self, request_id: &str) -> Result<WithdrawStatusResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/withdraw/{}", self.base_url, request_id))
            .send()
            .await?;
        
        let status: WithdrawStatusResponse = response.json().await?;
        Ok(status)
    }
}
```

## Proof Generation Helpers

### Witness Building

**Build Withdraw Witness:**
```rust
pub fn build_withdraw_witness(
    note: &SpendableNote,
    outputs: Vec<Output>,
    merkle_proof: &MerkleProof,
) -> Result<WithdrawWitness, Box<dyn std::error::Error>> {
    let witness = WithdrawWitness {
        amount: note.amount,
        r: note.r,
        sk_spend: note.sk_spend,
        leaf_index: note.leaf_index,
        merkle_path: MerklePath {
            path_elements: merkle_proof.path_elements
                .iter()
                .map(|s| hex::decode(s).unwrap().try_into().unwrap())
                .collect(),
            path_indices: merkle_proof.path_indices.clone(),
        },
        outputs,
    };
    
    Ok(witness)
}
```

### Proof Validation

**Validate Proof:**
```rust
pub fn validate_proof(
    proof: &ProofBundle,
    expected_public_inputs: &PublicInputs,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Validate public inputs match
    if proof.public_inputs.root != expected_public_inputs.root {
        return Ok(false);
    }
    
    if proof.public_inputs.nullifier != expected_public_inputs.nullifier {
        return Ok(false);
    }
    
    if proof.public_inputs.outputs_hash != expected_public_inputs.outputs_hash {
        return Ok(false);
    }
    
    if proof.public_inputs.amount != expected_public_inputs.amount {
        return Ok(false);
    }
    
    // Validate proof size
    if proof.groth16_proof.len() != 260 {
        return Ok(false);
    }
    
    Ok(true)
}
```

## Integration Test Examples

### End-to-End Deposit Test

**Complete Deposit Flow:**
```rust
#[tokio::test]
async fn test_deposit_flow() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::localnet();
    let client = RpcClient::new(config.rpc_url.clone());
    
    // Setup
    let admin = load_keypair(&config.admin_keypair_path)?;
    let user = load_keypair(&config.user_keypair_path)?;
    
    check_cluster_health(&config.rpc_url).await?;
    ensure_user_funding(&config.rpc_url, &user, &admin, config.min_balance).await?;
    
    // Get PDA addresses
    let pda_addresses = get_pda_addresses(&config.shield_pool_program_id)?;
    
    // Build deposit instruction
    let amount = config.test_amounts.medium;
    let commitment = [0x01u8; 32];
    let encrypted_output = vec![0x02u8; 100];
    
    let instruction = build_deposit_instruction(
        &config.shield_pool_program_id,
        &user.pubkey(),
        &pda_addresses.pool,
        &pda_addresses.treasury,
        amount,
        commitment,
        encrypted_output,
    )?;
    
    // Execute transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );
    
    let signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("Deposit transaction: {}", signature);
    
    // Verify deposit
    let indexer_client = IndexerClient::new(config.indexer_url);
    let root = indexer_client.get_root().await?;
    assert_ne!(root.root, "0000000000000000000000000000000000000000000000000000000000000000");
    
    Ok(())
}
```

### End-to-End Withdraw Test

**Complete Withdraw Flow:**
```rust
#[tokio::test]
async fn test_withdraw_flow() -> Result<(), Box<dyn std::error::Error>> {
let config = TestConfig::localnet();
    
    // Setup
    let admin = load_keypair(&config.admin_keypair_path)?;
let user = load_keypair(&config.user_keypair_path)?;
    
    check_cluster_health(&config.rpc_url).await?;
    ensure_user_funding(&config.rpc_url, &user, &admin, config.min_balance).await?;
    
    // Get Merkle proof
    let indexer_client = IndexerClient::new(config.indexer_url.clone());
    let proof = indexer_client.get_proof(0).await?;
    
    // Build withdraw request
    let outputs = vec![
        Output {
            address: [0x01u8; 32],
            amount: 500_000,
        },
        Output {
            address: [0x02u8; 32],
            amount: 300_000,
        },
    ];
    
    let public_inputs = PublicInputs {
        root: hex::decode(&proof.root).unwrap().try_into().unwrap(),
        nullifier: [0x03u8; 32],
        outputs_hash: [0x04u8; 32],
        amount: 1_000_000,
    };
    
    let proof_bytes = [0x05u8; 260];
    
    let request = WithdrawRequest {
        outputs,
        policy: Policy { fee_bps: 60 },
        public_inputs,
        proof_bytes: base64::encode(proof_bytes),
    };
    
    // Submit withdraw
    let relay_client = RelayClient::new(config.relay_url);
    let response = relay_client.submit_withdraw(request).await?;
    
    // Wait for completion
    let mut attempts = 0;
    while attempts < 30 {
        let status = relay_client.get_withdraw_status(&response.request_id).await?;
        
        match status.state.as_str() {
            "settled" => {
                println!("Withdraw completed: {}", status.txid.unwrap());
                break;
            }
            "failed" => {
                return Err("Withdraw failed".into());
            }
            _ => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    }
    
    Ok(())
}
```

## Running Tests

### Test Execution

**Run All Tests:**
```bash
cargo test -p tooling-test
```

**Run Specific Test:**
```bash
cargo test -p tooling-test test_deposit_flow
```

**Run with Logging:**
```bash
RUST_LOG=debug cargo test -p tooling-test -- --nocapture
```

### Test Environment Setup

**Localnet Setup:**
```bash
# Start Solana test validator
solana-test-validator --reset --quiet &

# Deploy programs
solana program deploy target/deploy/shield_pool.so
solana program deploy target/deploy/scramble_registry.so

# Start services
docker-compose up -d

# Run tests
cargo test -p tooling-test
```

**Testnet Setup:**
```bash
# Set testnet cluster
solana config set --url https://api.testnet.solana.com

# Deploy programs
solana program deploy target/deploy/shield_pool.so
solana program deploy target/deploy/scramble_registry.so

# Run tests
cargo test -p tooling-test --features testnet
```

## Best Practices

### Test Organization

**Test Structure:**
- Unit tests for individual functions
- Integration tests for service interactions
- End-to-end tests for complete workflows
- Smoke tests for deployment validation

**Test Data Management:**
- Use deterministic test data for reproducibility
- Generate unique test data for each test run
- Clean up test data after completion
- Use test fixtures for complex scenarios

### Error Handling

**Test Error Handling:**
```rust
#[tokio::test]
async fn test_with_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::localnet();
    
    // Setup with error handling
    let result = async {
        let admin = load_keypair(&config.admin_keypair_path)?;
        let user = load_keypair(&config.user_keypair_path)?;
        
        check_cluster_health(&config.rpc_url).await?;
        ensure_user_funding(&config.rpc_url, &user, &admin, config.min_balance).await?;
        
        // Test logic here
        
        Ok::<(), Box<dyn std::error::Error>>(())
    }.await;
    
    match result {
        Ok(_) => println!("Test passed"),
        Err(e) => {
            println!("Test failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
```

### Performance Testing

**Load Testing:**
```rust
#[tokio::test]
async fn test_load_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::localnet();
    let client = RpcClient::new(config.rpc_url.clone());
    
    let start = std::time::Instant::now();
    
    // Execute multiple operations
    for i in 0..100 {
        let admin = load_keypair(&config.admin_keypair_path)?;
        let user = load_keypair(&config.user_keypair_path)?;
        
        ensure_user_funding(&config.rpc_url, &user, &admin, config.min_balance).await?;
        
        // Additional operations
    }
    
    let duration = start.elapsed();
    println!("Load test completed in {:?}", duration);
    
    Ok(())
}
```

This testing toolkit provides a comprehensive foundation for testing Cloak's privacy-preserving protocol across all components and scenarios.
