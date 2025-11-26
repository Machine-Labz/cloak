//! Initialize shield pool
//!
//! Run with:
//!   cargo run --package cloak-miner --example init_shield_pool

use std::str::FromStr;

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};

const SHIELD_POOL_PROGRAM_ID: &str = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp";
const ADMIN_KEYPAIR_PATH: &str = "mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa.json";

fn main() -> Result<()> {
    println!("=== Initialize Shield Pool ===\n");

    let rpc_url = "http://localhost:8899";
    let program_id = Pubkey::from_str(SHIELD_POOL_PROGRAM_ID)?;

    println!("RPC URL: {}", rpc_url);
    println!("Program ID: {}", program_id);

    // Load admin keypair
    let admin_keypair = read_keypair_file(ADMIN_KEYPAIR_PATH).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load admin keypair from {}: {}",
            ADMIN_KEYPAIR_PATH,
            e
        )
    })?;

    println!("Admin: {}", admin_keypair.pubkey());

    // Derive PDAs
    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], &program_id);
    let (commitments_pda, _) = Pubkey::find_program_address(&[b"commitments"], &program_id);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let (nullifier_shard_pda, _) = Pubkey::find_program_address(&[b"nullifier_shard"], &program_id);
    let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], &program_id);

    println!("\nPDAs:");
    println!("  Pool: {}", pool_pda);
    println!("  Commitments: {}", commitments_pda);
    println!("  Roots Ring: {}", roots_ring_pda);
    println!("  Nullifier Shard: {}", nullifier_shard_pda);
    println!("  Treasury: {}", treasury_pda);

    // Connect to RPC
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Check if already initialized
    if let Ok(account) = client.get_account(&pool_pda) {
        if account.lamports > 0 {
            println!("\n✓ Shield pool already initialized!");
            return Ok(());
        }
    }

    // Fund admin if needed
    let admin_balance = client.get_balance(&admin_keypair.pubkey())?;
    if admin_balance < 1_000_000_000 {
        println!("\n⚠️  Admin needs SOL. Run:");
        println!(
            "   solana airdrop 5 {} --url http://127.0.0.1:8899",
            admin_keypair.pubkey()
        );
        return Ok(());
    }

    println!("\n1. Building initialize instruction...");

    // Initialize instruction: discriminator = 3
    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_keypair.pubkey(), true), // Admin (signer, payer)
            AccountMeta::new(pool_pda, false),              // Pool PDA
            AccountMeta::new(commitments_pda, false),       // Commitments PDA
            AccountMeta::new(roots_ring_pda, false),        // Roots Ring PDA
            AccountMeta::new(nullifier_shard_pda, false),   // Nullifier Shard PDA
            AccountMeta::new(treasury_pda, false),          // Treasury PDA
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // System Program
        ],
        data: vec![3u8], // Initialize discriminator
    };

    println!("2. Submitting transaction...");

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&admin_keypair.pubkey()),
        &[&admin_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;

    println!("\n✓ Shield pool initialized successfully!");
    println!("  Signature: {}", signature);
    println!("  Pool PDA: {}", pool_pda);

    Ok(())
}
