//! Register a miner on a network
//!
//! Run with:
//!   cargo run --package cloak-miner --example register_miner -- [network]
//!
//! Examples:
//!   cargo run --package cloak-miner --example register_miner -- testnet
//!   cargo run --package cloak-miner --example register_miner -- localnet

use anyhow::Result;
use cloak_miner::{
    constants::Network,
    instructions::{build_register_miner_ix, derive_miner_pda},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};

fn main() -> Result<()> {
    println!("=== Register Miner ===\n");

    // Parse network from command line args (default to localnet)
    let network_str = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localnet".to_string());
    let network = Network::from_str(&network_str).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Get network config
    let rpc_url = network.default_rpc_url();
    let program_id = network
        .scramble_program_id()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Network: {:?}", network);
    println!("RPC URL: {}", rpc_url);
    println!("Program ID: {}", program_id);

    // Load miner keypair
    let miner_keypair =
        read_keypair_file(shellexpand::tilde("~/.config/solana/id.json").to_string())
            .map_err(|e| anyhow::anyhow!("Failed to load miner keypair: {}", e))?;

    let miner_authority = miner_keypair.pubkey();
    let (miner_pda, _bump) = derive_miner_pda(&program_id, &miner_authority);

    println!("Miner authority: {}", miner_authority);
    println!("Miner PDA: {}\n", miner_pda);

    // Create RPC client
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Build register_miner instruction
    let ix = build_register_miner_ix(&program_id, &miner_pda, &miner_authority);

    println!("1. Submitting registration transaction...");

    // Create and send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&miner_keypair.pubkey()),
        &[&miner_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&tx)?;

    println!("\n✓ Miner registered successfully!");
    println!("   Signature: {}", signature);

    Ok(())
}
