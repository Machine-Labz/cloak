use anyhow::Result;
use serde::{Deserialize, Serialize};
use shield_pool::instructions::ShieldPoolInstruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
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
    pub recipient2_keypair_path: String,
    pub program_keypair_path: String,
    pub indexer_url: String,
}

impl TestConfig {
    /// Create configuration for localnet testing
    pub fn localnet() -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(),
            program_id: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".to_string(),
            amount: 100_000_000, // 0.1 SOL
            user_keypair_path: "user-keypair.json".to_string(),
            recipient_keypair_path: "recipient-keypair.json".to_string(),
            recipient2_keypair_path: "recipient-2-keypair.json".to_string(),
            program_keypair_path: "testnet-program-keypair.json".to_string(),
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
            recipient2_keypair_path: "recipient-2-keypair.json".to_string(),
            program_keypair_path: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json".to_string(),
            indexer_url: "http://localhost:3001".to_string(),
        }
    }

    /// Check if this is a testnet configuration
    pub fn is_testnet(&self) -> bool {
        self.rpc_url.contains("testnet.solana.com")
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
    pub root: String,
}

/// Get PDA addresses for Shield Pool program with mint support
pub fn get_pda_addresses(
    program_id: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], program_id);
    let (commitments_pda, _) =
        Pubkey::find_program_address(&[b"commitments", mint.as_ref()], program_id);
    let (roots_ring_pda, _) =
        Pubkey::find_program_address(&[b"roots_ring", mint.as_ref()], program_id);
    let (nullifier_shard_pda, _) =
        Pubkey::find_program_address(&[b"nullifier_shard", mint.as_ref()], program_id);
    let (treasury_pda, _) =
        Pubkey::find_program_address(&[b"treasury", mint.as_ref()], program_id);
    (
        pool_pda,
        commitments_pda,
        roots_ring_pda,
        nullifier_shard_pda,
        treasury_pda,
    )
}

/// Get PDA addresses for native SOL (backward compatibility)
pub fn get_pda_addresses_sol(program_id: &Pubkey) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    get_pda_addresses(program_id, &Pubkey::default())
}

/// Create deposit instruction
pub fn create_deposit_instruction(
    user_pubkey: &Pubkey,
    pool_pubkey: &Pubkey,
    commitments_pubkey: &Pubkey,
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
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            AccountMeta::new(*commitments_pubkey, false),
        ],
        data,
    }
}

/// Create admin push root instruction
pub fn create_admin_push_root_instruction(
    admin_pubkey: &Pubkey,
    roots_ring_pubkey: &Pubkey,
    program_id: &Pubkey,
    merkle_root: &[u8; 32],
) -> Instruction {
    let mut data = Vec::new();
    data.push(ShieldPoolInstruction::AdminPushRoot as u8); // AdminPushRoot discriminator
    data.extend_from_slice(merkle_root);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*admin_pubkey, true),
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
    recipient_accounts: &[Pubkey],
    program_id: &Pubkey,
    proof_bytes: &[u8],
    public_inputs: &[u8],
    nullifier: &[u8; 32],
    outputs: &[(Pubkey, u64)],
) -> Instruction {
    let mut data = Vec::new();
    data.push(ShieldPoolInstruction::Withdraw as u8); // Withdraw discriminator
    data.extend_from_slice(proof_bytes);
    data.extend_from_slice(public_inputs);
    data.extend_from_slice(nullifier);

    data.push(outputs.len() as u8);
    for (recipient, amount) in outputs {
        data.extend_from_slice(recipient.as_ref());
        data.extend_from_slice(&amount.to_le_bytes());
    }

    let mut accounts = vec![
        AccountMeta::new(*pool_pubkey, false),
        AccountMeta::new(*treasury_pubkey, false),
        AccountMeta::new_readonly(*roots_ring_pubkey, false),
        AccountMeta::new(*nullifier_shard_pubkey, false),
    ];

    // Add all recipient accounts first
    for recipient in recipient_accounts {
        accounts.push(AccountMeta::new(*recipient, false));
    }

    // Add system program last
    accounts.push(AccountMeta::new_readonly(
        solana_sdk::system_program::ID,
        false,
    ));

    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

pub fn load_keypair(path: &str) -> Result<Keypair> {
    let keypair_data = std::fs::read(path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair file '{}': {}", path, e))?;

    if keypair_data.is_empty() {
        return Err(anyhow::anyhow!("Keypair file '{}' is empty", path));
    }

    // Try to parse as JSON array first (Solana CLI format)
    if let Ok(json_bytes) = serde_json::from_slice::<Vec<u8>>(&keypair_data) {
        if json_bytes.len() == 64 {
            let keypair = Keypair::try_from(&json_bytes[..]).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse keypair from JSON array in '{}': {}",
                    path,
                    e
                )
            })?;
            return Ok(keypair);
        }
    }

    // Fallback to raw bytes (other formats)
    let keypair = Keypair::try_from(&keypair_data[..]).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse keypair from '{}': {}. File size: {} bytes",
            path,
            e,
            keypair_data.len()
        )
    })?;

    Ok(keypair)
}

pub fn write_keypair(path: &str, keypair: &Keypair) -> Result<()> {
    let keypair_bytes = keypair.to_bytes();
    let json = serde_json::to_string(&keypair_bytes.to_vec())
        .map_err(|e| anyhow::anyhow!("Failed to serialize keypair for '{}': {}", path, e))?;
    std::fs::write(path, json)
        .map_err(|e| anyhow::anyhow!("Failed to write keypair file '{}': {}", path, e))?;
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
    println!(
        "   - Secondary Recipient Keypair: {}",
        config.recipient2_keypair_path
    );
    println!("   - Program Keypair: {}", config.program_keypair_path);
    println!("   - Indexer URL: {}", config.indexer_url);
}

/// Check cluster health
pub fn check_cluster_health(rpc_url: &str) -> Result<()> {
    println!("🔍 Checking cluster health...");
    let client = RpcClient::new(rpc_url);
    let _version = client.get_version()?;
    println!("   ✅ Cluster is healthy");
    Ok(())
}

/// Ensure user has sufficient funding
pub fn ensure_user_funding(
    rpc_url: &str,
    user_keypair: &Keypair,
    admin_keypair: &Keypair,
) -> Result<()> {
    let client = RpcClient::new(rpc_url);
    let user_balance = client.get_balance(&user_keypair.pubkey())?;

    if user_balance >= SOL_TO_LAMPORTS {
        println!(
            "💰 User has sufficient SOL ({} SOL), skipping transfer",
            user_balance / SOL_TO_LAMPORTS
        );
        return Ok(());
    }

    println!("💰 Transferring SOL to user...");
    let transfer_amount = SOL_TO_LAMPORTS - user_balance;

    let transfer_ix = system_instruction::transfer(
        &admin_keypair.pubkey(),
        &user_keypair.pubkey(),
        transfer_amount,
    );

    let blockhash = client.get_latest_blockhash()?;
    let mut transfer_tx =
        Transaction::new_with_payer(&[transfer_ix], Some(&admin_keypair.pubkey()));
    transfer_tx.sign(&[admin_keypair], blockhash);

    client.send_and_confirm_transaction(&transfer_tx)?;
    println!("   ✅ Transfer completed");
    Ok(())
}
