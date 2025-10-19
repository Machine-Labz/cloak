//! Cloak Miner CLI
//!
//! Standalone PoW miner for Cloak protocol.
//! Continuously mines and reveals claims that can be consumed by withdraw transactions.
//!
//! Usage:
//!   # Mainnet (default)
//!   cloak-miner --keypair <PATH> mine
//!
//!   # Devnet
//!   cloak-miner --network devnet --keypair <PATH> mine
//!
//!   # Localnet
//!   SCRAMBLE_PROGRAM_ID=<ID> cloak-miner --network localnet --keypair <PATH> mine
//!
//!   # Other commands
//!   cloak-miner --network devnet --keypair <PATH> register
//!   cloak-miner --network devnet --keypair <PATH> status

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cloak_miner::{
    build_register_miner_ix, constants::Network, derive_miner_pda, derive_registry_pda,
    fetch_registry, ClaimManager,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "cloak-miner")]
#[command(about = "Standalone PoW miner for Cloak protocol", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Network to connect to (mainnet, devnet, localnet)
    #[arg(long, short = 'n', env = "CLOAK_NETWORK", default_value = "mainnet")]
    network: String,

    /// RPC URL for Solana cluster (overrides network default)
    #[arg(long, env = "SOLANA_RPC_URL")]
    rpc_url: Option<String>,

    /// Path to miner keypair file
    #[arg(long, env = "MINER_KEYPAIR_PATH")]
    keypair: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Register as a new miner (one-time setup)
    Register,

    /// Start mining claims continuously
    Mine {
        /// Mining timeout per attempt in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Minimum interval between mining attempts in seconds
        #[arg(long, default_value = "10")]
        interval: u64,

        /// Target number of active claims to maintain
        #[arg(long, default_value = "5")]
        target_claims: usize,
    },

    /// Check miner status and active claims
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Parse network
    let network = Network::from_str(&cli.network).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Get RPC URL (use explicit or network default)
    let rpc_url = cli
        .rpc_url
        .unwrap_or_else(|| network.default_rpc_url().to_string());

    // Get program ID for network
    let program_id = network
        .scramble_program_id()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Load keypair
    let keypair = read_keypair_file(&cli.keypair)
        .map_err(|e| anyhow::anyhow!("Failed to load keypair from {:?}: {}", cli.keypair, e))?;

    info!("Network: {:?}", network);
    info!("Miner pubkey: {}", keypair.pubkey());
    info!("RPC URL: {}", rpc_url);
    info!("Program ID: {}", program_id);

    match cli.command {
        Commands::Register => register_miner(&rpc_url, &program_id, keypair).await,
        Commands::Mine {
            timeout,
            interval,
            target_claims,
        } => {
            mine_continuously(
                &rpc_url,
                &program_id,
                keypair,
                timeout,
                interval,
                target_claims,
            )
            .await
        }
        Commands::Status => check_status(&rpc_url, &program_id, keypair).await,
    }
}

/// Register miner (one-time setup)
async fn register_miner(
    rpc_url: &str,
    program_id: &solana_sdk::pubkey::Pubkey,
    keypair: Keypair,
) -> Result<()> {
    info!("Registering miner...");

    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Check if already registered
    let (miner_pda, _) = derive_miner_pda(program_id, &keypair.pubkey());

    if let Ok(account) = client.get_account(&miner_pda) {
        if account.data.len() > 0 {
            info!("✓ Miner already registered: {}", miner_pda);
            return Ok(());
        }
    }

    // Build and submit registration transaction
    let register_ix = build_register_miner_ix(program_id, &miner_pda, &keypair.pubkey());

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[register_ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    let signature = client
        .send_and_confirm_transaction(&tx)
        .context("Failed to register miner")?;

    info!("✓ Miner registered successfully!");
    info!("  Miner PDA: {}", miner_pda);
    info!("  Signature: {}", signature);

    Ok(())
}

/// Mine claims continuously
async fn mine_continuously(
    rpc_url: &str,
    program_id: &solana_sdk::pubkey::Pubkey,
    keypair: Keypair,
    timeout_secs: u64,
    interval_secs: u64,
    target_claims: usize,
) -> Result<()> {
    info!("Starting continuous mining...");
    info!("  Timeout: {}s per attempt", timeout_secs);
    info!("  Interval: {}s between attempts", interval_secs);
    info!("  Target claims: {}", target_claims);

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        warn!("Received shutdown signal, stopping miner...");
        r.store(false, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl-C handler")?;

    // Initialize claim manager
    let mut manager = ClaimManager::new(
        rpc_url.to_string(),
        keypair,
        &program_id.to_string(),
        timeout_secs,
    )
    .context("Failed to initialize ClaimManager")?;

    info!("✓ ClaimManager initialized");

    // Fetch registry to display difficulty
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let (registry_pda, _) = derive_registry_pda(program_id);

    match fetch_registry(&client, &registry_pda) {
        Ok(registry) => {
            info!(
                "Current difficulty: {:x?}...",
                &registry.current_difficulty[0..4]
            );
            info!("Reveal window: {} slots", registry.reveal_window);
            info!("Claim window: {} slots", registry.claim_window);
        }
        Err(e) => {
            warn!("Could not fetch registry: {}", e);
        }
    }

    let mut mining_round = 0u64;

    // Main mining loop
    while running.load(Ordering::SeqCst) {
        mining_round += 1;
        info!("=== Mining Round {} ===", mining_round);

        // Generate a unique job ID for this mining attempt
        // In production, this could be based on user requests or a pool
        let job_id = format!("auto-mine-{}", mining_round);

        match manager.get_claim_for_job(&job_id).await {
            Ok(claim_pda) => {
                info!("✓ Claim mined and revealed: {}", claim_pda);
                info!("  Miners can now earn fees when this claim is consumed");
            }
            Err(e) => {
                error!("✗ Mining failed: {}", e);
                error!("  Retrying in {}s...", interval_secs);
            }
        }

        // Wait before next mining attempt
        if running.load(Ordering::SeqCst) {
            info!("Waiting {}s before next mining round...\n", interval_secs);
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    info!("Miner stopped gracefully");
    Ok(())
}

/// Check miner status
async fn check_status(
    rpc_url: &str,
    program_id: &solana_sdk::pubkey::Pubkey,
    keypair: Keypair,
) -> Result<()> {
    info!("Checking miner status...");

    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Check miner PDA
    let (miner_pda, _) = derive_miner_pda(program_id, &keypair.pubkey());

    match client.get_account(&miner_pda) {
        Ok(account) => {
            info!("✓ Miner registered");
            info!("  PDA: {}", miner_pda);
            info!("  Data length: {} bytes", account.data.len());
            info!("  Owner: {}", account.owner);
        }
        Err(_) => {
            warn!("✗ Miner not registered");
            warn!("  Run 'cloak-miner register' first");
            return Ok(());
        }
    }

    // Check registry
    let (registry_pda, _) = derive_registry_pda(program_id);
    match fetch_registry(&client, &registry_pda) {
        Ok(registry) => {
            info!("\n✓ Registry state:");
            info!("  Admin: {}", registry.admin);
            info!("  Difficulty: {:x?}...", &registry.current_difficulty[0..8]);
            info!("  Reveal window: {} slots", registry.reveal_window);
            info!("  Claim window: {} slots", registry.claim_window);
            info!("  Max k: {}", registry.max_k);
        }
        Err(e) => {
            warn!("Could not fetch registry: {}", e);
        }
    }

    // TODO: Query and display active claims for this miner
    info!("\nNote: Active claim enumeration not yet implemented");

    Ok(())
}
