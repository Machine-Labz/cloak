use std::str::FromStr;

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::config::SolanaConfig;

/// Push a merkle root to the on-chain roots ring
pub async fn push_root_to_chain(root_hash: &str, config: &SolanaConfig) -> Result<()> {
    // Check if admin keypair is configured
    let admin_keypair_bytes = match &config.admin_keypair {
        Some(bytes) => bytes,
        None => {
            tracing::warn!("Admin keypair not configured, skipping root push");
            return Ok(());
        }
    };

    // Validate configuration
    if config.shield_pool_program_id.is_empty() {
        tracing::warn!("Shield pool program ID not configured, skipping root push");
        return Ok(());
    }

    // Create admin keypair from bytes
    let admin_keypair = Keypair::try_from(admin_keypair_bytes.as_slice())
        .context("Failed to create admin keypair from bytes")?;

    tracing::info!(
        root_hash = root_hash,
        admin_pubkey = %admin_keypair.pubkey(),
        "Pushing root to on-chain roots ring"
    );

    // Connect to Solana with timeout
    let client =
        RpcClient::new_with_timeout(config.rpc_url.clone(), std::time::Duration::from_secs(30));

    // Convert root hex to bytes
    let root_bytes: [u8; 32] = hex::decode(root_hash)
        .context("Failed to decode root hash as hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Root hash must be exactly 32 bytes"))?;

    // Create instruction data: [discriminator: 1 byte][root: 32 bytes]
    let mut instruction_data = vec![1u8]; // AdminPushRoot discriminator
    instruction_data.extend_from_slice(&root_bytes);

    // Derive roots_ring PDA from program ID
    let program_id = Pubkey::from_str(&config.shield_pool_program_id)
        .context("Invalid shield pool program ID")?;
    let (roots_ring_pda, _bump) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    tracing::info!(
        roots_ring_pda = %roots_ring_pda,
        "Derived roots_ring PDA from program ID"
    );

    // Create instruction
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new(roots_ring_pda, false),
        ],
        data: instruction_data,
    };

    // Get recent blockhash with retry
    let recent_blockhash = client
        .get_latest_blockhash()
        .context("Failed to get recent blockhash")?;

    // Create transaction
    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&admin_keypair.pubkey()));
    transaction.sign(&[&admin_keypair], recent_blockhash);

    // Send transaction with confirmation
    tracing::info!("Sending root push transaction...");
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .context("Failed to send and confirm root push transaction")?;

    tracing::info!(
        signature = %signature,
        "Root successfully pushed to on-chain roots ring"
    );

    Ok(())
}
