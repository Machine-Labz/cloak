//! Decoy Manager - Automatic decoy deposit orchestration
//!
//! Handles automatic decoy deposits during mining.
//! Integrates with the mining loop to create indistinguishable transactions.

use std::path::PathBuf;

use anyhow::{Context, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use tracing::{debug, error, info, warn};

use crate::{build_deposit_ix, decoy::NoteStorage, DecoyNote};

/// Minimum interval between decoy operations (in slots)
/// ~100 slots ≈ 40 seconds at 400ms/slot
const DECOY_MIN_INTERVAL_SLOTS: u64 = 100;

/// Amount distribution parameters (in lamports)
const MIN_DECOY_AMOUNT: u64 = 10_000_000; // 0.01 SOL
const MAX_DECOY_AMOUNT: u64 = 500_000_000; // 0.5 SOL

/// Shield Pool program ID
const SHIELD_POOL_PROGRAM_ID: &str = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp";

/// Default indexer URL
const DEFAULT_INDEXER_URL: &str = "http://localhost:3001";

// ============ Indexer API Types ============

#[derive(Debug, Serialize)]
struct IndexerDepositRequest {
    leaf_commit: String,
    encrypted_output: String,
    tx_signature: String,
    slot: i64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IndexerDepositResponse {
    success: bool,
    #[serde(rename = "leafIndex")]
    leaf_index: u64,
    root: String,
}

/// Decoy operation result
#[derive(Debug)]
pub enum DecoyResult {
    /// No operation performed (rate limited or insufficient funds)
    Skipped(String),
    /// Deposit executed successfully
    Deposited { amount: u64, commitment: [u8; 32] },
    /// Operation failed
    Failed(String),
}

/// Manages automatic decoy deposits
pub struct DecoyManager {
    /// RPC client
    rpc_client: RpcClient,
    /// HTTP client for indexer API
    http_client: reqwest::Client,
    /// Indexer URL
    indexer_url: String,
    /// Miner keypair
    miner_keypair: Keypair,
    /// Scramble registry program ID
    scramble_program_id: Pubkey,
    /// Shield pool program ID
    shield_pool_program_id: Pubkey,
    /// Note storage
    note_storage: NoteStorage,
    /// Last slot when a decoy was executed
    last_decoy_slot: u64,
    /// Decoy statistics
    stats: DecoyStats,
}

/// Statistics for decoy operations
#[derive(Debug, Default)]
pub struct DecoyStats {
    pub total_deposits: u64,
    pub total_deposited_lamports: u64,
    pub failed_deposits: u64,
}

impl DecoyStats {
    pub fn print_summary(&self) {
        println!("Decoy Statistics:");
        println!(
            "  Deposits: {} ({:.4} SOL)",
            self.total_deposits,
            self.total_deposited_lamports as f64 / LAMPORTS_PER_SOL as f64
        );
        println!("  Failed: {} deposits", self.failed_deposits);
    }
}

impl DecoyManager {
    /// Create a new decoy manager
    pub fn new(
        rpc_url: &str,
        miner_keypair: Keypair,
        scramble_program_id: &Pubkey,
    ) -> Result<Self> {
        let storage_path = Self::get_storage_path(&miner_keypair.pubkey());
        let note_storage =
            NoteStorage::load_or_create(storage_path).context("Failed to load note storage")?;

        let shield_pool_program_id: Pubkey = SHIELD_POOL_PROGRAM_ID
            .parse()
            .context("Invalid shield pool program ID")?;

        let indexer_url =
            std::env::var("INDEXER_URL").unwrap_or_else(|_| DEFAULT_INDEXER_URL.to_string());

        Ok(Self {
            rpc_client: RpcClient::new_with_commitment(
                rpc_url.to_string(),
                CommitmentConfig::confirmed(),
            ),
            http_client: reqwest::Client::new(),
            indexer_url,
            miner_keypair,
            scramble_program_id: *scramble_program_id,
            shield_pool_program_id,
            note_storage,
            last_decoy_slot: 0,
            stats: DecoyStats::default(),
        })
    }

    /// Get storage path for notes
    fn get_storage_path(miner_pubkey: &Pubkey) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".cloak-miner")
            .join(format!("notes-{}.json", miner_pubkey))
    }

    /// Get decoy statistics
    pub fn stats(&self) -> &DecoyStats {
        &self.stats
    }

    /// Get escrow balance (public method for main loop checks)
    pub fn get_escrow_balance_pub(&self) -> Result<u64> {
        self.get_escrow_balance()
    }

    /// Maybe execute a decoy deposit
    ///
    /// Called during each mining round. Will:
    /// 1. Check rate limiting (minimum slots between decoys)
    /// 2. Check if funds are available
    /// 3. Execute deposit if possible
    pub async fn maybe_execute_decoy(&mut self, current_slot: u64) -> DecoyResult {
        // Rate limiting check
        if current_slot < self.last_decoy_slot + DECOY_MIN_INTERVAL_SLOTS {
            let slots_remaining = (self.last_decoy_slot + DECOY_MIN_INTERVAL_SLOTS) - current_slot;
            return DecoyResult::Skipped(format!(
                "Rate limited ({} slots remaining)",
                slots_remaining
            ));
        }

        let wallet_balance = self.get_wallet_balance().unwrap_or(0);

        // Calculate how much we can deposit (leave some for fees)
        let min_fee_reserve = 10_000_000; // 0.01 SOL for fees
        let can_deposit = wallet_balance > min_fee_reserve + MIN_DECOY_AMOUNT;

        debug!(
            "Decoy state: wallet={:.4} SOL, can_deposit={}",
            wallet_balance as f64 / LAMPORTS_PER_SOL as f64,
            can_deposit,
        );

        if !can_deposit {
            return DecoyResult::Skipped("Insufficient wallet balance".to_string());
        }

        self.execute_deposit(wallet_balance - min_fee_reserve, current_slot)
            .await
    }

    /// Sample a random amount for decoy transactions
    fn sample_amount(&self, max_available: u64) -> u64 {
        let mut rng = rand::thread_rng();

        let max = max_available.min(MAX_DECOY_AMOUNT);
        let min = MIN_DECOY_AMOUNT.min(max);

        if min >= max {
            return min;
        }

        // Use log-normal distribution to favor smaller amounts
        let log_min = (min as f64).ln();
        let log_max = (max as f64).ln();
        let log_amount = rng.gen_range(log_min..log_max);

        log_amount.exp() as u64
    }

    /// Execute a decoy deposit
    async fn execute_deposit(&mut self, max_amount: u64, current_slot: u64) -> DecoyResult {
        let amount = self.sample_amount(max_amount);

        info!(
            "Executing decoy deposit: {:.4} SOL",
            amount as f64 / LAMPORTS_PER_SOL as f64
        );

        // Generate note
        let mut note = DecoyNote::generate(amount);
        let commitment = note.commitment;

        // Derive PDAs
        let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], &self.shield_pool_program_id);
        let (commitments_pda, _) =
            Pubkey::find_program_address(&[b"commitments"], &self.shield_pool_program_id);

        // Build and submit transaction
        let deposit_ix = match build_deposit_ix(
            &self.miner_keypair.pubkey(),
            &pool_pda,
            &commitments_pda,
            amount,
            commitment,
        ) {
            Ok(ix) => ix,
            Err(e) => {
                self.stats.failed_deposits += 1;
                return DecoyResult::Failed(format!("Failed to build deposit ix: {}", e));
            }
        };

        let recent_blockhash = match self.rpc_client.get_latest_blockhash() {
            Ok(bh) => bh,
            Err(e) => {
                self.stats.failed_deposits += 1;
                return DecoyResult::Failed(format!("Failed to get blockhash: {}", e));
            }
        };

        let tx = Transaction::new_signed_with_payer(
            &[deposit_ix],
            Some(&self.miner_keypair.pubkey()),
            &[&self.miner_keypair],
            recent_blockhash,
        );

        // Submit on-chain transaction
        let tx_signature = match self.rpc_client.send_and_confirm_transaction(&tx) {
            Ok(sig) => {
                info!("✅ Decoy deposit confirmed: {}", sig);
                sig.to_string()
            }
            Err(e) => {
                self.stats.failed_deposits += 1;
                error!("❌ Decoy deposit failed: {}", e);
                return DecoyResult::Failed(format!("Transaction failed: {}", e));
            }
        };

        // Register with indexer to get leaf_index
        let leaf_index = match self
            .register_deposit_with_indexer(&commitment, &tx_signature, current_slot)
            .await
        {
            Ok(index) => {
                info!("📝 Deposit registered with indexer, leaf_index: {}", index);
                index
            }
            Err(e) => {
                warn!("⚠️ Failed to register with indexer: {}", e);
                note.deposit_signature = Some(tx_signature);
                if let Err(e) = self.note_storage.add_note(note) {
                    warn!("Failed to save note: {}", e);
                }

                self.stats.total_deposits += 1;
                self.stats.total_deposited_lamports += amount;
                self.last_decoy_slot = current_slot;

                return DecoyResult::Deposited { amount, commitment };
            }
        };

        // Save note with leaf_index
        note.deposit_signature = Some(tx_signature);
        note.leaf_index = Some(leaf_index as u32);
        if let Err(e) = self.note_storage.add_note(note) {
            warn!("Failed to save note: {}", e);
        }

        // Update stats
        self.stats.total_deposits += 1;
        self.stats.total_deposited_lamports += amount;
        self.last_decoy_slot = current_slot;

        DecoyResult::Deposited { amount, commitment }
    }

    /// Register a deposit with the indexer
    async fn register_deposit_with_indexer(
        &self,
        commitment: &[u8; 32],
        tx_signature: &str,
        slot: u64,
    ) -> Result<u64> {
        let url = format!("{}/api/v1/deposit", self.indexer_url);

        let encrypted_output = hex::encode(&[0u8; 64]); // Dummy for decoy

        let request = IndexerDepositRequest {
            leaf_commit: hex::encode(commitment),
            encrypted_output,
            tx_signature: tx_signature.to_string(),
            slot: slot as i64,
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send deposit to indexer")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Indexer returned {}: {}", status, body);
        }

        let deposit_response: IndexerDepositResponse = response
            .json()
            .await
            .context("Failed to parse indexer response")?;

        Ok(deposit_response.leaf_index)
    }

    /// Get miner wallet balance
    fn get_wallet_balance(&self) -> Result<u64> {
        Ok(self.rpc_client.get_balance(&self.miner_keypair.pubkey())?)
    }

    /// Get escrow balance
    fn get_escrow_balance(&self) -> Result<u64> {
        let (escrow_pda, _) = Pubkey::find_program_address(
            &[b"miner_escrow", self.miner_keypair.pubkey().as_ref()],
            &self.scramble_program_id,
        );

        Ok(self.rpc_client.get_balance(&escrow_pda)?)
    }

    /// Print status summary
    pub fn print_status(&self) {
        println!("  Notes stored: {}", self.note_storage.count());
        println!(
            "  Total deposited: {:.4} SOL",
            self.note_storage.get_total_deposited() as f64 / LAMPORTS_PER_SOL as f64
        );

        if let Ok(balance) = self.get_wallet_balance() {
            println!(
                "  Wallet balance: {:.4} SOL",
                balance as f64 / LAMPORTS_PER_SOL as f64
            );
        }
        if let Ok(balance) = self.get_escrow_balance() {
            println!(
                "  Escrow balance: {:.4} SOL",
                balance as f64 / LAMPORTS_PER_SOL as f64
            );
        }

        self.stats.print_summary();
    }
}
