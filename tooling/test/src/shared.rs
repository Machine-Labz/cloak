use anyhow::Result;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use shield_pool::instructions::ShieldPoolInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

/// Configuration for tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub rpc_url: String,
    pub program_id: String,
    pub amount: u64,
    pub user_keypair_path: String,
    pub recipient_keypair_path: String,
    pub program_keypair_path: String,
    pub indexer_url: String,
}

impl TestConfig {
    /// Create configuration for localnet testing
    pub fn localnet() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:8899".to_string(),
            program_id: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".to_string(),
            amount: 1_000_000_000, // 1 SOL
            user_keypair_path: "user-keypair.json".to_string(),
            recipient_keypair_path: "recipient-keypair.json".to_string(),
            program_keypair_path: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json".to_string(),
            indexer_url: "http://localhost:3001".to_string(),
        }
    }

    /// Create configuration for testnet testing
    pub fn testnet() -> Self {
        Self {
            rpc_url: "https://api.testnet.solana.com".to_string(),
            program_id: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".to_string(),
            amount: 100_000_000, // 0.1 SOL
            user_keypair_path: "user-keypair.json".to_string(),
            recipient_keypair_path: "recipient-keypair.json".to_string(),
            program_keypair_path: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json".to_string(),
            indexer_url: "http://localhost:3001".to_string(),
        }
    }
}

/// Common test data structures
pub const SOL_TO_LAMPORTS: u64 = 1_000_000_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    #[serde(rename = "pathElements")]
    pub path_elements: Vec<String>,
    #[serde(rename = "pathIndices")]
    pub path_indices: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositRequest {
    #[serde(rename = "leafCommit")]
    pub leaf_commit: String,
    #[serde(rename = "encryptedOutput")]
    pub encrypted_output: String,
    #[serde(rename = "txSignature")]
    pub tx_signature: String,
    pub slot: u64,
}

/// Load a keypair from file
pub fn load_keypair(path: &str) -> Result<Keypair> {
    let keypair_data = std::fs::read(path)?;

    // Try to parse as JSON first (array of numbers)
    if let Ok(json_data) = serde_json::from_slice::<Vec<u8>>(&keypair_data) {
        Keypair::try_from(&json_data[..])
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON keypair from {}: {}", path, e))
    } else {
        // Fall back to binary format
        Keypair::try_from(&keypair_data[..])
            .map_err(|e| anyhow::anyhow!("Failed to parse binary keypair from {}: {}", path, e))
    }
}

/// Get program ID from keypair
pub fn get_program_id(keypair_path: &str) -> Result<Pubkey> {
    let keypair = load_keypair(keypair_path)?;
    Ok(keypair.pubkey())
}

/// Check cluster health
pub fn check_cluster_health(rpc_url: &str) -> Result<()> {
    println!("🔍 Checking cluster health...");
    let client = solana_client::rpc_client::RpcClient::new(rpc_url);
    match client.get_health() {
        Ok(_) => {
            println!("   ✅ Cluster is healthy");
            Ok(())
        }
        Err(e) => {
            println!("   ❌ Cluster health check failed: {}", e);
            Err(anyhow::anyhow!("Cluster is not accessible: {}", e))
        }
    }
}

/// Ensure user has sufficient SOL
pub fn ensure_user_funding(
    rpc_url: &str,
    user_keypair: &Keypair,
    admin_keypair: &Keypair,
) -> Result<()> {
    let client = solana_client::rpc_client::RpcClient::new(rpc_url);
    let user_balance = client.get_balance(&user_keypair.pubkey())?;

    if user_balance < 5_000_000_000 {
        println!("\n💰 Transferring SOL from admin to user...");

        println!("   🔍 Getting latest blockhash...");
        let blockhash = client.get_latest_blockhash()?;
        println!("   🔍 Blockhash: {}", blockhash);

        let transfer_ix = system_instruction::transfer(
            &admin_keypair.pubkey(),
            &user_keypair.pubkey(),
            2_000_000_000, // 2 SOL
        );
        let mut transfer_tx =
            Transaction::new_with_payer(&[transfer_ix], Some(&admin_keypair.pubkey()));
        transfer_tx.sign(&[&admin_keypair], blockhash);

        println!("   🔍 Sending transaction...");

        // Use send_and_confirm_transaction to avoid duplicate transaction issues
        match client.send_and_confirm_transaction(&transfer_tx) {
            Ok(signature) => {
                println!("   ✅ Transfer successful with signature: {}", signature);
            }
            Err(e) => {
                let error_msg = e.to_string();
                println!("   ❌ Transfer failed: {}", error_msg);
                return Err(anyhow::anyhow!("Transfer failed: {}", error_msg));
            }
        }
    } else {
        println!(
            "\n💰 User has sufficient SOL ({} SOL), skipping transfer",
            user_balance / SOL_TO_LAMPORTS
        );
    }

    Ok(())
}

/// Print test configuration
pub fn print_config(config: &TestConfig) {
    println!("🔧 Test Configuration:");
    println!("   - RPC URL: {}", config.rpc_url);
    println!("   - Program ID: {}", config.program_id);
    println!("   - Amount: {} SOL", config.amount / SOL_TO_LAMPORTS);
    println!("   - User Keypair: {}", config.user_keypair_path);
    println!("   - Recipient Keypair: {}", config.recipient_keypair_path);
    println!("   - Program Keypair: {}", config.program_keypair_path);
    println!("   - Indexer URL: {}", config.indexer_url);
}

/// Validate configuration
pub fn validate_config(config: &TestConfig) -> Result<(), String> {
    if !std::path::Path::new(&config.user_keypair_path).exists() {
        return Err(format!(
            "User keypair file not found: {}",
            config.user_keypair_path
        ));
    }
    if !std::path::Path::new(&config.recipient_keypair_path).exists() {
        return Err(format!(
            "Recipient keypair file not found: {}",
            config.recipient_keypair_path
        ));
    }
    if !std::path::Path::new(&config.program_keypair_path).exists() {
        return Err(format!(
            "Program keypair file not found: {}",
            config.program_keypair_path
        ));
    }
    if config.amount == 0 {
        return Err("Amount must be greater than 0".to_string());
    }
    Ok(())
}

/// Create deposit instruction
pub fn create_deposit_instruction(
    user_pubkey: &Pubkey,
    pool_pubkey: &Pubkey,
    roots_ring_pubkey: &Pubkey,
    program_id: &Pubkey,
    amount: u64,
    commitment: &[u8; 32],
) -> Instruction {
    let mut data = Vec::new();
    data.push(ShieldPoolInstruction::Deposit as u8); // Deposit discriminator
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(commitment);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*user_pubkey, true),
            AccountMeta::new(*pool_pubkey, false),
            AccountMeta::new(*roots_ring_pubkey, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ],
        data,
    }
}

/// Create admin push root instruction
pub fn create_admin_push_root_instruction(
    admin_pubkey: &Pubkey,
    roots_ring_pubkey: &Pubkey,
    program_id: &Pubkey,
    root: &[u8; 32],
) -> Instruction {
    let mut data = Vec::new();
    data.push(ShieldPoolInstruction::AdminPushRoot as u8); // AdminPushRoot discriminator
    data.extend_from_slice(root);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*admin_pubkey, true),
            AccountMeta::new(*roots_ring_pubkey, false),
        ],
        data,
    }
}

/// Create withdraw instruction
pub fn create_withdraw_instruction(
    pool_pubkey: &Pubkey,
    treasury_pubkey: &Pubkey,
    roots_ring_pubkey: &Pubkey,
    nullifier_shard_pubkey: &Pubkey,
    recipient_pubkey: &Pubkey,
    program_id: &Pubkey,
    proof_bytes: &[u8],
    raw_public_inputs: &[u8],
    nullifier: &[u8; 32],
    num_outputs: u8,
    recipient_amount: u64,
) -> Instruction {
    let mut data = Vec::new();
    data.push(ShieldPoolInstruction::Withdraw as u8); // Withdraw discriminator - Byte 0
    data.extend_from_slice(proof_bytes); // Groth16 proof bytes - Bytes 1-260 (260 bytes)
    data.extend_from_slice(raw_public_inputs); // Raw public inputs - Bytes 261-364 (104 bytes)
    data.extend_from_slice(nullifier); // 32 bytes (for nullifier check) - Bytes 365-396
    data.push(num_outputs); // 1 byte - Bytes 397
    data.extend_from_slice(&recipient_pubkey.to_bytes()); // 32 bytes - Bytes 398-429
    data.extend_from_slice(&recipient_amount.to_le_bytes()); // 8 bytes - Bytes 430-437

    println!("   - Instruction data length: {} bytes", data.len());
    println!("   - Expected length: 438 bytes (1 + 260 + 104 + 32 + 1 + 32 + 8)");

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*pool_pubkey, false),
            AccountMeta::new(*treasury_pubkey, false),
            AccountMeta::new(*roots_ring_pubkey, false),
            AccountMeta::new(*nullifier_shard_pubkey, false),
            AccountMeta::new(*recipient_pubkey, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ],
        data,
    }
}

/// Verify Merkle path (exact copy of SP1 guest program's verify_merkle_path function)
pub fn verify_merkle_path(
    leaf: &[u8; 32],
    path_elements: &[[u8; 32]],
    path_indices: &[u8],
    root: &[u8; 32],
) -> bool {
    if path_elements.len() != path_indices.len() {
        return false;
    }

    let mut current = *leaf;

    for (element, &index) in path_elements.iter().zip(path_indices.iter()) {
        let mut hasher = Hasher::new();
        if index == 0 {
            // current is left, element is right
            hasher.update(&current);
            hasher.update(element);
        } else if index == 1 {
            // element is left, current is right
            hasher.update(element);
            hasher.update(&current);
        } else {
            return false; // Invalid index
        }
        current = hasher.finalize().into();
    }

    current == *root
}
