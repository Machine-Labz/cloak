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
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

const DEFAULT_RELAY_URL: &str = "http://localhost:3002";

#[derive(Deserialize)]
struct BacklogResponse {
    #[allow(dead_code)]
    pending_count: usize,
    #[allow(dead_code)]
    queued_jobs: Vec<String>,
}

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

/// Miner statistics for tracking performance
#[derive(Debug)]
struct MinerStats {
    total_claims_mined: AtomicU64,
    total_mining_time: AtomicU64, // milliseconds
    total_hash_attempts: AtomicU64,
    successful_mining_rounds: AtomicU64,
    failed_mining_rounds: AtomicU64,
    start_time: Instant,
}

impl MinerStats {
    fn new() -> Self {
        Self {
            total_claims_mined: AtomicU64::new(0),
            total_mining_time: AtomicU64::new(0),
            total_hash_attempts: AtomicU64::new(0),
            successful_mining_rounds: AtomicU64::new(0),
            failed_mining_rounds: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    fn record_successful_mining(&self, mining_time_ms: u64, hash_attempts: u64) {
        self.total_claims_mined.fetch_add(1, Ordering::Relaxed);
        self.total_mining_time
            .fetch_add(mining_time_ms, Ordering::Relaxed);
        self.total_hash_attempts
            .fetch_add(hash_attempts, Ordering::Relaxed);
        self.successful_mining_rounds
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_failed_mining(&self) {
        self.failed_mining_rounds.fetch_add(1, Ordering::Relaxed);
    }

    fn get_average_hash_rate(&self) -> f64 {
        let total_time_secs = self.total_mining_time.load(Ordering::Relaxed) as f64 / 1000.0;
        if total_time_secs > 0.0 {
            self.total_hash_attempts.load(Ordering::Relaxed) as f64 / total_time_secs
        } else {
            0.0
        }
    }

    fn get_success_rate(&self) -> f64 {
        let total = self.successful_mining_rounds.load(Ordering::Relaxed)
            + self.failed_mining_rounds.load(Ordering::Relaxed);
        if total > 0 {
            self.successful_mining_rounds.load(Ordering::Relaxed) as f64 / total as f64 * 100.0
        } else {
            0.0
        }
    }

    fn print_summary(&self) {
        let uptime = self.start_time.elapsed();
        let claims_mined = self.total_claims_mined.load(Ordering::Relaxed);
        let avg_hash_rate = self.get_average_hash_rate();
        let success_rate = self.get_success_rate();

        println!("Miner Statistics:");
        println!("  Uptime: {:.1}s", uptime.as_secs_f64());
        println!("  Claims mined: {}", claims_mined);
        println!("  Average hash rate: {:.0} H/s", avg_hash_rate);
        println!("  Success rate: {:.1}%", success_rate);
        println!(
            "  Claims per hour: {:.1}",
            claims_mined as f64 / (uptime.as_secs_f64() / 3600.0)
        );
    }
}

/// Check if there's pending demand from the relay
async fn check_relay_demand(relay_url: &str) -> Result<(bool, usize)> {
    let url = format!("{}/backlog", relay_url);
    match reqwest::get(&url).await {
        Ok(response) => {
            match response.json::<BacklogResponse>().await {
                Ok(backlog) => {
                    let has_demand = backlog.pending_count > 0;
                    Ok((has_demand, backlog.pending_count))
                }
                Err(e) => {
                    warn!("Failed to parse backlog response: {}", e);
                    Ok((true, 0)) // Assume demand to avoid blocking mining
                }
            }
        }
        Err(e) => {
            warn!("Failed to check relay backlog at {}: {}", url, e);
            Ok((true, 0)) // Assume demand if relay unreachable
        }
    }
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
    let miner_pubkey = keypair.pubkey();
    
    // Get relay URL from env or use default
    let relay_url = std::env::var("RELAY_URL")
        .unwrap_or_else(|_| DEFAULT_RELAY_URL.to_string());

    println!("Cloak Miner Starting");
    println!("Miner: {}", miner_pubkey);
    println!("Timeout: {}s per attempt", timeout_secs);
    println!("Interval: {}s between attempts", interval_secs);
    println!("Target claims: {}", target_claims);

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("Received shutdown signal, stopping miner...");
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

    println!("ClaimManager initialized");
    println!("Continuous mining mode enabled\n");

    // Initialize miner statistics
    let stats = Arc::new(MinerStats::new());

    // Fetch registry to display difficulty
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let (registry_pda, _) = derive_registry_pda(program_id);

    match fetch_registry(&client, &registry_pda) {
        Ok(registry) => {
            println!("Registry State:");
            println!("  Difficulty: {:x?}...", &registry.current_difficulty[0..4]);
            println!("  Reveal window: {} slots", registry.reveal_window);
            println!("  Claim window: {} slots", registry.claim_window);
        }
        Err(e) => {
            warn!("Could not fetch registry: {}", e);
        }
    }

    // Check initial miner balance
    let min_sol_required = 0.01; // Minimum SOL needed for transactions
    
    match client.get_balance(&miner_pubkey) {
        Ok(balance) => {
            let sol_balance = balance as f64 / LAMPORTS_PER_SOL as f64;
            println!("Initial SOL balance: {:.4} SOL", sol_balance);
            
            if sol_balance < min_sol_required {
                error!("Insufficient SOL balance: {:.4} SOL (minimum required: {:.4} SOL)", sol_balance, min_sol_required);
                error!("Miner needs funding before it can submit transactions.");
                error!("Exiting to avoid transaction failures.");
                return Ok(());
            }
        }
        Err(e) => {
            warn!("Could not fetch balance: {}", e);
            error!("Cannot verify funds - exiting to avoid potential transaction failures.");
            return Ok(());
        }
    }

    let mut mining_round = 0u64;

    // Main mining loop
    while running.load(Ordering::SeqCst) {
        mining_round += 1;

        // Print statistics every 10 rounds
        if mining_round % 10 == 1 {
            stats.print_summary();
        }

        println!("=== Mining Round {} ===", mining_round);

        // Check current number of active claims (with cleanup)
        let current_claims = manager.get_active_claims_count().await.unwrap_or(0);
        println!(
            "Current active claims: {}/{}",
            current_claims, target_claims
        );

        // Check for relay demand
        let min_buffer = 2; // Always keep at least 2 claims ready for incoming requests
        let (has_demand, pending_count) = match check_relay_demand(&relay_url).await.unwrap_or((true, 0)) {
            (demand, count) => {
                info!("📊 Relay check: {} pending jobs, has_demand={}", count, demand);
                (demand, count)
            }
        };

        // If we have enough claims, wait before checking again
        if current_claims >= target_claims {
            if has_demand {
                println!("📦 {} pending withdrawals, but sufficient claims available", pending_count);
            }
            println!("Waiting {}s before next check...\n", interval_secs);
            if running.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
            continue;
        }
        
        // Decision logic with clear reasoning
        if !has_demand && current_claims >= min_buffer {
            println!("⚡ No demand detected ({} pending) and buffer sufficient ({} claims)", pending_count, current_claims);
            println!("Skipping mining to avoid waste. Waiting {}s...\n", interval_secs);
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            continue;
        } else if has_demand {
            println!("📦 Mining triggered by demand: {} pending withdrawals", pending_count);
        } else {
            println!("🔄 Mining to maintain minimum buffer: {} claims < {} buffer", current_claims, min_buffer);
        }

        // Mine new claims to reach target
        let claims_needed = target_claims.saturating_sub(current_claims);
        println!("Mining {} new claim(s)...", claims_needed);

        for i in 0..claims_needed {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            // Use wildcard batch_hash ([0; 32]) for continuous mining
            // This allows the claim to be used by any withdraw request
            // Force mining new claims by calling mine_and_reveal directly (bypassing cache)
            let wildcard_batch_hash = [0u8; 32];
            let mining_start = Instant::now();

            match manager.mine_and_reveal(wildcard_batch_hash).await {
                Ok((claim_pda, solution)) => {
                    let mining_time = mining_start.elapsed();
                    let mining_time_ms = mining_time.as_millis() as u64;

                    // Record successful mining with actual hash attempts
                    stats.record_successful_mining(mining_time_ms, solution.attempts);

                    println!("Claim {} mined and revealed: {}", i + 1, claim_pda);
                    println!("  Mining time: {:.2}s", mining_time.as_secs_f64());
                    println!("  Hash attempts: {}", solution.attempts);
                    println!(
                        "  Hash rate: {:.0} H/s",
                        solution.attempts as f64 / solution.mining_time.as_secs_f64()
                    );
                    println!("  Miners can now earn fees when this claim is consumed");
                }
                Err(e) => {
                    stats.record_failed_mining();
                    error!("Mining claim {} failed: {}", i + 1, e);
                    error!("Continuing with next claim...");
                }
            }
        }

        // Wait before next mining round
        if running.load(Ordering::SeqCst) {
            println!("Waiting {}s before next mining round...\n", interval_secs);
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    println!("Miner stopped gracefully");
    stats.print_summary();
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
