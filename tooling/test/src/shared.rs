use anyhow::Result;
use serde::{Deserialize, Serialize};
use shield_pool::instructions::ShieldPoolInstruction;
use solana_client::rpc_client::RpcClient;
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
            amount: 213_567_839, // 0.213567839 SOL (0.12 + 0.09 + fees, accounting for integer division)
            user_keypair_path: "user-keypair.json".to_string(),
            recipient_keypair_path: "recipient-keypair.json".to_string(),
            recipient2_keypair_path: "recipient-2-keypair.json".to_string(),
            program_keypair_path: "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json".to_string(),
            indexer_url: "https://api.cloaklabz.xyz".to_string(),
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
}

/// Get PDA addresses for Shield Pool program
pub fn get_pda_addresses(
    program_id: &Pubkey,
) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], program_id);
    let (commitments_pda, _) =
        Pubkey::find_program_address(&[b"commitments"], program_id);
    let (roots_ring_pda, _) =
        Pubkey::find_program_address(&[b"roots_ring"], program_id);
    let (nullifier_shard_pda, _) =
        Pubkey::find_program_address(&[b"nullifier_shard"], program_id);
    let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], program_id);
    (
        pool_pda,
        commitments_pda,
        roots_ring_pda,
        nullifier_shard_pda,
        treasury_pda,
    )
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

/// Load keypair from file
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

/// Batch fund multiple user accounts efficiently (3 batches of ~30% each)
/// If funding fails, automatically refunds all SOL back to admin
pub fn batch_fund_accounts(
    rpc_url: &str,
    user_keypairs: &[&Keypair],
    admin_keypair: &Keypair,
    num_batches: usize,
) -> Result<()> {
    let client = RpcClient::new(rpc_url);

    // Fund with 0.25 SOL (enough for test transactions with fees)
    const FUNDING_AMOUNT: u64 = 250_000_000; // 0.25 SOL

    // Check which accounts need funding
    let mut accounts_to_fund = Vec::new();
    for keypair in user_keypairs {
        let balance = client.get_balance(&keypair.pubkey())?;
        if balance < FUNDING_AMOUNT {
            accounts_to_fund.push(keypair);
        }
    }

    if accounts_to_fund.is_empty() {
        println!("   ✅ All accounts already funded");
        return Ok(());
    }

    println!("   📊 Need to fund {}/{} accounts", accounts_to_fund.len(), user_keypairs.len());

    // Split into batches
    let batch_size = (accounts_to_fund.len() + num_batches - 1) / num_batches;
    let batches: Vec<_> = accounts_to_fund.chunks(batch_size).collect();

    println!("   📦 Creating {} batches with ~{} accounts each", batches.len(), batch_size);

    // Track successfully funded accounts for potential rollback
    let mut funded_accounts: Vec<&Keypair> = Vec::new();

    for (batch_idx, batch) in batches.iter().enumerate() {
        println!("   🔄 Funding batch {}/{} ({} accounts)...", batch_idx + 1, batches.len(), batch.len());

        let mut instructions = Vec::new();
        let mut batch_funded_accounts = Vec::new();

        for keypair in batch.iter() {
            let current_balance = client.get_balance(&keypair.pubkey())?;
            let transfer_amount = FUNDING_AMOUNT.saturating_sub(current_balance);

            if transfer_amount > 0 {
                let transfer_ix = system_instruction::transfer(
                    &admin_keypair.pubkey(),
                    &keypair.pubkey(),
                    transfer_amount,
                );
                instructions.push(transfer_ix);
                batch_funded_accounts.push(*keypair);
            }
        }

        if instructions.is_empty() {
            println!("      ✅ Batch {} - all accounts already funded", batch_idx + 1);
            continue;
        }

        let blockhash = client.get_latest_blockhash()?;
        let mut batch_tx = Transaction::new_with_payer(&instructions, Some(&admin_keypair.pubkey()));
        batch_tx.sign(&[admin_keypair], blockhash);

        match client.send_and_confirm_transaction(&batch_tx) {
            Ok(_) => {
                println!("      ✅ Batch {} - funded {} accounts", batch_idx + 1, instructions.len());
                // Track successfully funded accounts
                funded_accounts.extend(batch_funded_accounts);
            }
            Err(e) => {
                println!("      ❌ Batch {} failed: {}", batch_idx + 1, e);
                println!("   🔄 Rolling back: refunding SOL from {} previously funded accounts...", funded_accounts.len());

                // Attempt to refund all previously funded accounts
                if let Err(rollback_err) = rollback_funding(&client, &funded_accounts, admin_keypair) {
                    println!("      ⚠️  Warning: Rollback encountered issues: {}", rollback_err);
                    println!("      ℹ️  Some accounts may still have funded SOL");
                } else {
                    println!("   ✅ Rollback complete - all SOL refunded to admin");
                }

                return Err(anyhow::anyhow!("Batch funding failed: {}", e));
            }
        }
    }

    println!("   ✅ All {} accounts funded successfully", accounts_to_fund.len());
    Ok(())
}

/// Public cleanup function to refund all funded accounts (used after test failures)
pub fn cleanup_funded_accounts(
    rpc_url: &str,
    user_keypairs: &[&Keypair],
    admin_keypair: &Keypair,
) -> Result<()> {
    let client = RpcClient::new(rpc_url);
    println!("\n🧹 Cleaning up: Refunding SOL from all funded accounts...");
    rollback_funding(&client, user_keypairs, admin_keypair)
}

/// Rollback funding by transferring all SOL back from funded accounts to admin
fn rollback_funding(
    client: &RpcClient,
    funded_accounts: &[&Keypair],
    admin_keypair: &Keypair,
) -> Result<()> {
    if funded_accounts.is_empty() {
        return Ok(());
    }

    let mut total_refunded = 0u64;
    let mut successful_refunds = 0usize;
    let mut failed_refunds = 0usize;

    for (idx, keypair) in funded_accounts.iter().enumerate() {
        match client.get_balance(&keypair.pubkey()) {
            Ok(balance) => {
                // Transfer all balance (close the account)
                // We need to leave exactly 5000 lamports for the transaction fee
                if balance > 5000 {
                    let refund_amount = balance.saturating_sub(5000);

                    let transfer_ix = system_instruction::transfer(
                        &keypair.pubkey(),
                        &admin_keypair.pubkey(),
                        refund_amount,
                    );

                    let blockhash = match client.get_latest_blockhash() {
                        Ok(bh) => bh,
                        Err(e) => {
                            println!("      ⚠️  Account {}/{}: Failed to get blockhash: {}", idx + 1, funded_accounts.len(), e);
                            failed_refunds += 1;
                            continue;
                        }
                    };

                    let mut refund_tx = Transaction::new_with_payer(&[transfer_ix], Some(&keypair.pubkey()));
                    refund_tx.sign(&[*keypair], blockhash);

                    match client.send_and_confirm_transaction(&refund_tx) {
                        Ok(_) => {
                            total_refunded += refund_amount;
                            successful_refunds += 1;
                            if (successful_refunds) % 5 == 0 {
                                println!("      ✓ Refunded {}/{} accounts", successful_refunds, funded_accounts.len());
                            }
                        }
                        Err(e) => {
                            println!("      ⚠️  Account {}/{}: Failed to refund {} lamports: {}",
                                idx + 1, funded_accounts.len(), refund_amount, e);
                            failed_refunds += 1;
                        }
                    }
                }
            }
            Err(e) => {
                println!("      ⚠️  Account {}/{}: Failed to get balance: {}", idx + 1, funded_accounts.len(), e);
                failed_refunds += 1;
            }
        }
    }

    println!("      📊 Rollback summary:");
    println!("         - Successfully refunded: {} accounts", successful_refunds);
    println!("         - Failed refunds: {} accounts", failed_refunds);
    println!("         - Total SOL refunded: {} ({} SOL)", total_refunded, total_refunded as f64 / SOL_TO_LAMPORTS as f64);

    if failed_refunds > 0 {
        Err(anyhow::anyhow!("{} accounts failed to refund", failed_refunds))
    } else {
        Ok(())
    }
}
