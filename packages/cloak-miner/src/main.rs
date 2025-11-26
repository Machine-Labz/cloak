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

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cloak_miner::{
    build_register_miner_with_escrow_ix, constants::Network, derive_miner_escrow_pda,
    derive_miner_pda, derive_registry_pda, fetch_registry, ClaimManager, DecoyManager, DecoyNote,
    DecoyResult, NoteStorage,
};
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use tracing::{debug, error, info, warn};

const DEFAULT_RELAY_URL: &str =
    // "https://api.cloaklabz.xyz";
    "http://localhost:3002";

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
    Register {
        /// Initial escrow amount in SOL (for decoy operations)
        #[arg(long, default_value = "0")]
        initial_escrow: f64,
    },

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

    /// Decoy operations for privacy
    #[command(subcommand)]
    Decoy(DecoyCommands),
}

#[derive(Subcommand)]
enum DecoyCommands {
    /// Create a decoy deposit (generates note and deposits to shield-pool)
    Deposit {
        /// Amount to deposit in SOL
        #[arg(long)]
        amount: f64,
    },

    /// Show decoy note storage status
    Status,

    /// Top up miner escrow with SOL
    TopUp {
        /// Amount to add to escrow in SOL
        #[arg(long)]
        amount: f64,
    },
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
        Commands::Register { initial_escrow } => {
            register_miner(&rpc_url, &program_id, keypair, initial_escrow).await
        }
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
        Commands::Decoy(decoy_cmd) => match decoy_cmd {
            DecoyCommands::Deposit { amount } => {
                decoy_deposit(&rpc_url, &program_id, keypair, amount).await
            }
            DecoyCommands::Status => decoy_status(&rpc_url, &program_id, keypair).await,
            DecoyCommands::TopUp { amount } => {
                decoy_top_up(&rpc_url, &program_id, keypair, amount).await
            }
        },
    }
}

/// Register miner (one-time setup)
async fn register_miner(
    rpc_url: &str,
    program_id: &solana_sdk::pubkey::Pubkey,
    keypair: Keypair,
    initial_escrow_sol: f64,
) -> Result<()> {
    info!("Registering miner...");

    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Check if already registered
    let (miner_pda, _) = derive_miner_pda(program_id, &keypair.pubkey());
    let (escrow_pda, _) = derive_miner_escrow_pda(program_id, &keypair.pubkey());

    if let Ok(account) = client.get_account(&miner_pda) {
        if !account.data.is_empty() {
            info!("✓ Miner already registered: {}", miner_pda);
            info!("  Escrow PDA: {}", escrow_pda);
            return Ok(());
        }
    }

    // Convert SOL to lamports
    let initial_escrow_lamports = (initial_escrow_sol * LAMPORTS_PER_SOL as f64) as u64;

    // Build and submit registration transaction with escrow
    let register_ix = build_register_miner_with_escrow_ix(
        program_id,
        &miner_pda,
        &escrow_pda,
        &keypair.pubkey(),
        initial_escrow_lamports,
    );

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
    info!("  Escrow PDA: {}", escrow_pda);
    if initial_escrow_lamports > 0 {
        info!(
            "  Initial escrow: {} SOL",
            initial_escrow_lamports as f64 / LAMPORTS_PER_SOL as f64
        );
    }
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
            let status = response.status();
            let response_text = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("<failed to read response>"));

            if !status.is_success() {
                warn!(
                    "Relay returned non-success status {}: {}",
                    status, response_text
                );
                return Ok((true, 0)); // Assume demand to avoid blocking mining
            }

            match serde_json::from_str::<BacklogResponse>(&response_text) {
                Ok(backlog) => {
                    let has_demand = backlog.pending_count > 0;
                    Ok((has_demand, backlog.pending_count))
                }
                Err(e) => {
                    warn!("Failed to parse backlog response from {}: {}", url, e);
                    warn!(
                        "Response body (first 200 chars): {}",
                        &response_text.chars().take(200).collect::<String>()
                    );
                    Ok((true, 0)) // Assume demand to avoid blocking mining
                }
            }
        }
        Err(e) => {
            warn!("Failed to connect to relay at {}: {}", url, e);
            warn!("Tip: For local development, set RELAY_URL=http://localhost:3002");
            Ok((true, 0)) // Assume demand if relay unreachable
        }
    }
}

/// Minimum escrow balance required to continue mining (in lamports)
/// Below this threshold, miner should stop and re-register with new address
const MIN_ESCROW_BALANCE: u64 = 50_000_000; // 0.05 SOL

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
    let relay_url = std::env::var("RELAY_URL").unwrap_or_else(|_| DEFAULT_RELAY_URL.to_string());

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              🎭 CLOAK MINER STARTING 🎭                   ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!(
        "║ Miner: {}...{}",
        &miner_pubkey.to_string()[..8],
        &miner_pubkey.to_string()[36..]
    );
    println!(
        "║ Timeout: {}s | Interval: {}s | Target Claims: {}",
        timeout_secs, interval_secs, target_claims
    );
    println!("║ Decoy Mode: ALWAYS ON (privacy by default)               ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  Received shutdown signal, stopping miner...");
        r.store(false, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl-C handler")?;

    // Initialize claim manager (needs a clone of keypair)
    let keypair_bytes = keypair.to_bytes();
    let keypair_for_claims: Keypair = keypair_bytes
        .as_slice()
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to clone keypair: {:?}", e))?;

    let mut manager = ClaimManager::new(
        rpc_url.to_string(),
        keypair_for_claims,
        &program_id.to_string(),
        timeout_secs,
    )
    .context("Failed to initialize ClaimManager")?;

    // Initialize decoy manager (ALWAYS enabled - mandatory for privacy)
    let keypair_for_decoy: Keypair = keypair_bytes
        .as_slice()
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to clone keypair: {:?}", e))?;

    let mut decoy_manager = DecoyManager::new(rpc_url, keypair_for_decoy, program_id)
        .context("Failed to initialize DecoyManager")?;

    println!("\n📊 Initial Decoy Status:");
    decoy_manager.print_status();

    // Check initial escrow balance
    let initial_escrow = decoy_manager.get_escrow_balance_pub().unwrap_or(0);
    if initial_escrow < MIN_ESCROW_BALANCE {
        error!("╔══════════════════════════════════════════════════════════╗");
        error!("║           ⛔ INSUFFICIENT ESCROW BALANCE ⛔               ║");
        error!("╠══════════════════════════════════════════════════════════╣");
        error!(
            "║ Current: {:.4} SOL | Required: {:.4} SOL",
            initial_escrow as f64 / LAMPORTS_PER_SOL as f64,
            MIN_ESCROW_BALANCE as f64 / LAMPORTS_PER_SOL as f64
        );
        error!("║                                                          ║");
        error!("║ To mine, you must register with escrow funds:            ║");
        error!("║   cloak-miner register --initial-escrow 1.0              ║");
        error!("╚══════════════════════════════════════════════════════════╝");
        return Ok(());
    }

    println!("\n✅ ClaimManager initialized");
    println!("🎭 Decoy system active - all transactions create privacy noise\n");

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
                error!(
                    "Insufficient SOL balance: {:.4} SOL (minimum required: {:.4} SOL)",
                    sol_balance, min_sol_required
                );
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
        let (has_demand, pending_count) =
            match check_relay_demand(&relay_url).await.unwrap_or((true, 0)) {
                (demand, count) => {
                    info!(
                        "📊 Relay check: {} pending jobs, has_demand={}",
                        count, demand
                    );
                    (demand, count)
                }
            };

        // If we have enough claims, wait before checking again
        if current_claims >= target_claims {
            if has_demand {
                println!(
                    "📦 {} pending withdrawals, but sufficient claims available",
                    pending_count
                );
            }
            println!("Waiting {}s before next check...\n", interval_secs);
            if running.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
            continue;
        }

        // Decision logic with clear reasoning
        if !has_demand && current_claims >= min_buffer {
            println!(
                "⚡ No demand detected ({} pending) and buffer sufficient ({} claims)",
                pending_count, current_claims
            );
            println!(
                "Skipping mining to avoid waste. Waiting {}s...\n",
                interval_secs
            );
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            continue;
        } else if has_demand {
            println!(
                "📦 Mining triggered by demand: {} pending withdrawals",
                pending_count
            );
        } else {
            println!(
                "🔄 Mining to maintain minimum buffer: {} claims < {} buffer",
                current_claims, min_buffer
            );
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

        // Execute automatic decoy transaction
        // Get current slot for rate limiting
        let current_slot = client.get_slot().unwrap_or(0);

        match decoy_manager.maybe_execute_decoy(current_slot).await {
            DecoyResult::Deposited { amount, commitment } => {
                info!(
                    "🎭 Decoy deposit: {:.4} SOL (commitment: {}...)",
                    amount as f64 / LAMPORTS_PER_SOL as f64,
                    hex::encode(&commitment[0..8])
                );
            }
            DecoyResult::Skipped(reason) => {
                debug!("Decoy skipped: {}", reason);
            }
            DecoyResult::Failed(err) => {
                warn!("Decoy failed: {}", err);
            }
        }

        // Check if escrow is depleted - miner must re-register with new address
        let current_escrow = decoy_manager.get_escrow_balance_pub().unwrap_or(0);
        if current_escrow < MIN_ESCROW_BALANCE {
            println!();
            error!("╔══════════════════════════════════════════════════════════╗");
            error!("║           💀 ESCROW DEPLETED - MINER RETIRING 💀          ║");
            error!("╠══════════════════════════════════════════════════════════╣");
            error!("║ Your miner escrow has been depleted through decoy        ║");
            error!("║ transactions. This is normal and expected behavior.      ║");
            error!("║                                                          ║");
            error!("║ To continue mining, create a NEW miner with fresh funds: ║");
            error!("║                                                          ║");
            error!("║   1. Generate new keypair: solana-keygen new -o new.json ║");
            error!("║   2. Fund the new wallet                                 ║");
            error!("║   3. Register: cloak-miner -k new.json register \\        ║");
            error!("║                  --initial-escrow 1.0                    ║");
            error!("║   4. Start mining: cloak-miner -k new.json mine          ║");
            error!("║                                                          ║");
            error!("║ Using a new address improves privacy by making it        ║");
            error!("║ harder to track which addresses belong to miners.        ║");
            error!("╚══════════════════════════════════════════════════════════╝");
            break;
        }

        // Wait before next mining round
        if running.load(Ordering::SeqCst) {
            println!("Waiting {}s before next mining round...\n", interval_secs);
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    println!("\n🛑 Miner stopped");
    stats.print_summary();
    decoy_manager.print_status();
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

    // Check escrow balance
    let (escrow_pda, _) = derive_miner_escrow_pda(program_id, &keypair.pubkey());
    match client.get_balance(&escrow_pda) {
        Ok(balance) => {
            info!("\n✓ Escrow status:");
            info!("  PDA: {}", escrow_pda);
            info!(
                "  Balance: {} SOL",
                balance as f64 / LAMPORTS_PER_SOL as f64
            );
        }
        Err(_) => {
            info!("\n✗ Escrow not found (register with --initial-escrow to create)");
        }
    }

    // TODO: Query and display active claims for this miner
    info!("\nNote: Active claim enumeration not yet implemented");

    Ok(())
}

// ============================================================================
// DECOY OPERATIONS
// ============================================================================

/// Get the path to the note storage file
fn get_note_storage_path(miner_pubkey: &solana_sdk::pubkey::Pubkey) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".cloak-miner")
        .join(format!("notes-{}.json", miner_pubkey))
}

/// Decoy deposit - generate note and deposit to shield-pool
async fn decoy_deposit(
    rpc_url: &str,
    _program_id: &solana_sdk::pubkey::Pubkey, // Reserved for future use
    keypair: Keypair,
    amount_sol: f64,
) -> Result<()> {
    use cloak_miner::build_deposit_ix;

    info!("Creating decoy deposit...");

    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    // Load or create note storage
    let storage_path = get_note_storage_path(&keypair.pubkey());
    let mut storage =
        NoteStorage::load_or_create(storage_path.clone()).context("Failed to load note storage")?;

    // Generate new note
    let note = DecoyNote::generate(amount_lamports);
    info!("Generated note:");
    info!("  Commitment: {}", hex::encode(note.commitment));
    info!("  Amount: {} SOL", amount_sol);

    // Get shield-pool PDAs (these are typically initialized accounts)
    // For now, we'll use hardcoded PDAs - in production these should be derived/configured
    let shield_pool_program_id: solana_sdk::pubkey::Pubkey =
        "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".parse()?;

    // Derive pool and commitments PDAs
    let (pool_pda, _) =
        solana_sdk::pubkey::Pubkey::find_program_address(&[b"pool"], &shield_pool_program_id);
    let (commitments_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"commitments"],
        &shield_pool_program_id,
    );

    // Build deposit instruction
    let deposit_ix = build_deposit_ix(
        &keypair.pubkey(),
        &pool_pda,
        &commitments_pda,
        amount_lamports,
        note.commitment,
    )?;

    // Submit transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    let signature = client
        .send_and_confirm_transaction(&tx)
        .context("Failed to submit deposit transaction")?;

    info!("✓ Deposit submitted!");
    info!("  Signature: {}", signature);

    // Save note to storage
    let mut note = note;
    note.deposit_signature = Some(signature.to_string());
    storage.add_note(note)?;

    info!("✓ Note saved to {}", storage_path.display());
    info!("\nNote: The leaf_index will be set after indexer processes the deposit.");
    info!("Run 'cloak-miner decoy status' to check pending notes.");

    Ok(())
}

/// Decoy status - show note storage info
async fn decoy_status(
    rpc_url: &str,
    program_id: &solana_sdk::pubkey::Pubkey,
    keypair: Keypair,
) -> Result<()> {
    info!("Decoy note status...");

    // Load note storage
    let storage_path = get_note_storage_path(&keypair.pubkey());
    let storage =
        NoteStorage::load_or_create(storage_path.clone()).context("Failed to load note storage")?;

    info!("Storage file: {}", storage_path.display());
    info!("Total notes: {}", storage.count());
    info!("Unspent notes: {}", storage.unspent_count());
    info!(
        "Total deposited: {} SOL",
        storage.get_total_deposited() as f64 / LAMPORTS_PER_SOL as f64
    );

    let pending = storage.get_pending_notes();
    if !pending.is_empty() {
        info!("\nPending deposits (awaiting leaf_index):");
        for note in pending {
            info!(
                "  - {} SOL (commitment: {}...)",
                note.amount as f64 / LAMPORTS_PER_SOL as f64,
                &hex::encode(note.commitment)[0..16]
            );
        }
    }

    let withdrawable = storage.get_withdrawable_notes();
    if !withdrawable.is_empty() {
        info!("\nWithdrawable notes:");
        for note in withdrawable {
            info!(
                "  - {} SOL (leaf: {}, commitment: {}...)",
                note.amount as f64 / LAMPORTS_PER_SOL as f64,
                note.leaf_index.unwrap(),
                &hex::encode(note.commitment)[0..16]
            );
        }
    }

    // Show escrow balance
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let (escrow_pda, _) = derive_miner_escrow_pda(program_id, &keypair.pubkey());

    match client.get_balance(&escrow_pda) {
        Ok(balance) => {
            info!(
                "\nEscrow balance: {} SOL",
                balance as f64 / LAMPORTS_PER_SOL as f64
            );
        }
        Err(_) => {
            info!("\nEscrow not found");
        }
    }

    Ok(())
}

/// Top up escrow with SOL
async fn decoy_top_up(
    rpc_url: &str,
    program_id: &solana_sdk::pubkey::Pubkey,
    keypair: Keypair,
    amount_sol: f64,
) -> Result<()> {
    use cloak_miner::build_top_up_escrow_ix;

    info!("Topping up escrow...");

    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    let (escrow_pda, _) = derive_miner_escrow_pda(program_id, &keypair.pubkey());

    // Check current balance
    let current_balance = client.get_balance(&escrow_pda).unwrap_or(0);
    info!(
        "Current escrow balance: {} SOL",
        current_balance as f64 / LAMPORTS_PER_SOL as f64
    );

    // Build top-up instruction
    let top_up_ix =
        build_top_up_escrow_ix(program_id, &keypair.pubkey(), &escrow_pda, amount_lamports);

    // Submit transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[top_up_ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    let signature = client
        .send_and_confirm_transaction(&tx)
        .context("Failed to top up escrow")?;

    info!("✓ Escrow topped up!");
    info!("  Added: {} SOL", amount_sol);
    info!("  Signature: {}", signature);

    // Show new balance
    let new_balance = client.get_balance(&escrow_pda)?;
    info!(
        "  New balance: {} SOL",
        new_balance as f64 / LAMPORTS_PER_SOL as f64
    );

    Ok(())
}
