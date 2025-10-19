//! Initialize scramble registry on localnet
//!
//! Run with:
//!   cargo run --package cloak-miner --example init_registry

use anyhow::Result;
use cloak_miner::{constants::Network, derive_registry_pda};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

const REGISTRY_SIZE: usize = 196; // Size from state/registry.rs

fn main() -> Result<()> {
    println!("=== Initialize Scramble Registry ===\n");

    // Get network config
    let network = Network::Localnet;
    let rpc_url = network.default_rpc_url();
    let program_id = network.scramble_program_id()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Network: {:?}", network);
    println!("RPC URL: {}", rpc_url);
    println!("Program ID: {}", program_id);

    // Load admin keypair (your default Solana keypair)
    let admin_keypair = read_keypair_file(
        shellexpand::tilde("~/.config/solana/id.json").to_string(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to load admin keypair: {}", e))?;

    println!("Admin: {}", admin_keypair.pubkey());

    // Derive registry PDA
    let (registry_pda, bump) = derive_registry_pda(&program_id);
    println!("\nRegistry PDA: {}", registry_pda);
    println!("Bump: {}", bump);

    // Connect to RPC
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Check if registry already exists
    if let Ok(account) = client.get_account(&registry_pda) {
        if account.data.len() > 0 {
            println!("\n✓ Registry already initialized!");
            println!("   Registry PDA: {}", registry_pda);
            return Ok(());
        }
    }

    println!("\n1. Building initialize instruction...");

    // Initialize instruction data
    let mut init_data = Vec::new();
    init_data.push(0u8); // Discriminator for initialize_registry

    // Initial difficulty (easy for testing: require first byte < 0x80)
    let initial_difficulty = {
        let mut diff = [0x00u8; 32];
        diff[0] = 0x80; // ~50% success rate
        diff
    };
    init_data.extend_from_slice(&initial_difficulty);

    // Min difficulty (very easy)
    let min_difficulty = [0xFFu8; 32];
    init_data.extend_from_slice(&min_difficulty);

    // Max difficulty (very hard)
    let mut max_difficulty = [0x00u8; 32];
    max_difficulty[0] = 0x01;
    init_data.extend_from_slice(&max_difficulty);

    // Target interval slots
    let target_interval_slots = 10u64;
    init_data.extend_from_slice(&target_interval_slots.to_le_bytes());

    // Fee share bps (10% = 1000)
    let fee_share_bps = 1000u16;
    init_data.extend_from_slice(&fee_share_bps.to_le_bytes());

    // Reveal window (10 slots)
    let reveal_window = 10u64;
    init_data.extend_from_slice(&reveal_window.to_le_bytes());

    // Claim window (300 slots ~ 2 minutes)
    let claim_window = 300u64;
    init_data.extend_from_slice(&claim_window.to_le_bytes());

    // Max k (max jobs per claim)
    let max_k = 5u16;
    init_data.extend_from_slice(&max_k.to_le_bytes());

    println!("   Initial difficulty: {:x?}...", &initial_difficulty[0..4]);
    println!("   Reveal window: {} slots", reveal_window);
    println!("   Claim window: {} slots", claim_window);
    println!("   Max k: {}", max_k);

    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(registry_pda, false),
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data: init_data,
    };

    println!("\n2. Submitting transaction...");

    // Create and send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&admin_keypair.pubkey()),
        &[&admin_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;

    println!("\n✓ Registry initialized successfully!");
    println!("   Signature: {}", signature);
    println!("   Registry PDA: {}", registry_pda);

    Ok(())
}
