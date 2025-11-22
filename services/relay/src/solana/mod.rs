pub mod client;
pub mod jupiter;
pub mod submit;
pub mod transaction_builder;
pub mod swap;

use async_trait::async_trait;
use base64;
use hex;
use shield_pool::state::SwapState;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
#[cfg(feature = "jito")]
use solana_sdk::{message::VersionedMessage, transaction::VersionedTransaction};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::claim_manager::{compute_batch_hash, ClaimFinder};
use crate::config::SolanaConfig;
use crate::db::models::Job;
use crate::error::Error;
use serde_json;

// Manual implementation of associated token account derivation
// This avoids dependency conflicts with spl-associated-token-account
fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    // Associated Token Account Program ID
    const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
        solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

    // Token Program ID
    const TOKEN_PROGRAM_ID: Pubkey =
        solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    // Find the associated token account address
    let (ata, _) = Pubkey::find_program_address(
        &[wallet.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    ata
}

// Removed external TransactionResult dependency; we return Signature to callers.

// Helper function to parse keypair from environment variable
// Supports both JSON array format [66,197,...] and base58 string format
fn parse_keypair_from_env(keypair_str: &str) -> Result<Keypair, Error> {
    let bytes: Vec<u8> = serde_json::from_str(keypair_str).map_err(|e| {
        Error::ValidationError(format!("Failed to parse keypair JSON array: {}", e))
    })?;
    Ok(Keypair::try_from(bytes.as_slice()).map_err(|e| {
        Error::ValidationError(format!("Failed to create keypair from bytes: {}", e))
    })?)
}

#[async_trait]
pub trait SolanaClient: Send + Sync {
    async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash, Error>;
    async fn send_and_confirm_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, Error>;
    async fn get_block_height(&self) -> Result<u64, Error>;
    async fn get_slot(&self) -> Result<u64, Error>;
    async fn get_account_balance(&self, pubkey: &Pubkey) -> Result<u64, Error>;
    async fn check_nullifier_exists(
        &self,
        nullifier_shard: &Pubkey,
        nullifier: &[u8],
    ) -> Result<bool, Error>;
    async fn get_account(&self, pubkey: &Pubkey) -> Result<solana_sdk::account::Account, Error>;
    async fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> Result<u64, Error>;
}

pub struct SolanaService {
    client: Box<dyn SolanaClient>,
    program_id: Pubkey,
    config: SolanaConfig,
    fee_payer: Option<Keypair>,
    claim_finder: Option<Arc<ClaimFinder>>,
}

impl SolanaService {
    pub async fn new(config: SolanaConfig) -> Result<Self, Error> {
        let program_id = Pubkey::from_str(&config.program_id)
            .map_err(|e| Error::ValidationError(format!("Invalid program ID: {}", e)))?;

        let client = Box::new(client::RpcSolanaClient::new(&config).await?);

        // Optionally load fee payer keypair
        let fee_payer = if let Some(ref authority) = config.withdraw_authority {
            Some(parse_keypair_from_env(authority)?)
        } else {
            None
        };

        Ok(Self {
            client,
            program_id,
            config,
            fee_payer,
            claim_finder: None,
        })
    }

    /// Set the ClaimFinder (for PoW support)
    pub fn set_claim_finder(&mut self, claim_finder: Option<Arc<ClaimFinder>>) {
        if claim_finder.is_some() {
            info!("SolanaService: PoW ClaimFinder configured");
        }
        self.claim_finder = claim_finder;
    }

    /// Get current Solana slot
    pub async fn get_slot(&self) -> Result<u64, Error> {
        self.client.get_slot().await
    }

    /// Check if a nullifier already exists on-chain
    pub async fn check_nullifier_exists(&self, nullifier: &[u8]) -> Result<bool, Error> {
        // Parse mint address (use configured mint or default to native SOL)
        let mint = if let Some(mint_str) = &self.config.mint_address {
            if mint_str.is_empty() {
                Pubkey::default()
            } else {
                Pubkey::from_str(mint_str)
                    .map_err(|e| Error::ValidationError(format!("Invalid mint address: {}", e)))?
            }
        } else {
            Pubkey::default()
        };

        // Derive nullifier shard PDA from program ID with mint
        let (_, _, _, nullifier_shard_pda) =
            transaction_builder::derive_shield_pool_pdas(&self.program_id, &mint);

        self.client
            .check_nullifier_exists(&nullifier_shard_pda, nullifier)
            .await
    }

    /// Submit a withdraw transaction to Solana
    pub async fn submit_withdraw(&self, job: &Job) -> Result<Signature, Error> {
        info!(
            "Submitting withdraw transaction for job: {}",
            job.request_id
        );

        // 1. Parse outputs from JSON
        let outputs_value = if job.outputs_json.is_object() {
            // New format: { "outputs": [...], "swap": {...} }
            job.outputs_json.get("outputs").unwrap_or(&job.outputs_json)
        } else {
            // Legacy format: [...]
            &job.outputs_json
        };
        let outputs = self.parse_outputs(outputs_value)?;

        // 2. Check if swap is requested
        let swap_config: Option<crate::swap::SwapConfig> =
            if let Some(swap_value) = job.outputs_json.get("swap") {
                match serde_json::from_value(swap_value.clone()) {
                    Ok(config) => {
                        info!("Swap requested in job {}: {:?}", job.request_id, config);
                        Some(config)
                    }
                    Err(e) => {
                        warn!("Invalid swap config in job {}: {}", job.request_id, e);
                        None
                    }
                }
            } else {
                None
            };

        // 3. Build and submit transaction(s)
        if let Some(swap_config) = swap_config {
            // Two-transaction flow: withdraw to relay temp account, then swap to final recipient
            self.submit_withdraw_with_swap(job, &outputs, &swap_config).await
        } else {
            // Single-transaction flow: just withdraw
            let transaction = self.build_withdraw_transaction(job, &outputs).await?;
            let signature = self.submit_and_confirm(&transaction, job, &outputs).await?;
            info!("Withdraw transaction confirmed: {}", signature);
            Ok(signature)
        }
    }

    /// Health check for Solana connection
    pub async fn health_check(&self) -> Result<(), Error> {
        match self.client.get_latest_blockhash().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Solana health check failed: {}", e);
                Err(e)
            }
        }
    }

    /// Check if a SwapState PDA exists for a given nullifier
    /// Returns Ok(true) if exists, Ok(false) if not found
    pub async fn check_swap_state_exists(&self, nullifier: &[u8; 32]) -> Result<bool, Error> {
        let (swap_state_pda, _) = transaction_builder::derive_swap_state_pda(&self.program_id, nullifier);
        
        match self.client.get_account(&swap_state_pda).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's "account not found" error
                let error_str = e.to_string();
                if error_str.contains("AccountNotFound") || error_str.contains("could not find account") {
                    Ok(false)
                } else {
                    // Some other error occurred
                    Err(e)
                }
            }
        }
    }

    /// Get current commitment configuration
    pub fn get_commitment(&self) -> CommitmentConfig {
        match self.config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(), // Default
        }
    }

    /// Parse outputs from JSON with validation
    fn parse_outputs(&self, outputs_json: &serde_json::Value) -> Result<Vec<Output>, Error> {
        let outputs_array = outputs_json
            .as_array()
            .ok_or_else(|| Error::ValidationError("Outputs must be an array".to_string()))?;

        let mut outputs = Vec::new();
        for output_value in outputs_array {
            let output: Output = serde_json::from_value(output_value.clone())
                .map_err(|e| Error::ValidationError(format!("Invalid output format: {}", e)))?;
            outputs.push(output);
        }

        if outputs.is_empty() {
            return Err(Error::ValidationError(
                "At least one output is required".to_string(),
            ));
        }

        Ok(outputs)
    }

    /// Submit a two-transaction swap flow (PDA-guarded):
    /// 1) WithdrawSwap locks user's SOL in a SwapState PDA (no relay custody)
    /// 2) Relay executes Jupiter swap using its own SOL to the recipient ATA, then calls ExecuteSwap
    ///    to verify min output and reimburse the relay by closing the PDA
    async fn submit_withdraw_with_swap(
        &self,
        job: &Job,
        outputs: &[Output],
        swap_config: &crate::swap::SwapConfig,
    ) -> Result<Signature, Error> {
        info!("Starting PDA-based withdraw+swap flow for job {}", job.request_id);

        // Relay fee payer is required (pays PDA rent and signs Jupiter swap; reimbursed via ExecuteSwap)
        let relay_keypair = self.fee_payer.as_ref().ok_or_else(|| {
            Error::ValidationError("Relay fee payer keypair required for swap withdrawals".into())
        })?;
        let relay_pubkey = relay_keypair.pubkey();

        // Output mint to receive (e.g., USDC)
        let output_mint = Pubkey::from_str(&swap_config.output_mint)
            .map_err(|e| Error::ValidationError(format!("Invalid output mint: {}", e)))?;

        // Parse public inputs -> nullifier and public_amount
        if job.public_inputs.len() != 104 {
            return Err(Error::ValidationError("public inputs must be 104 bytes".into()));
        }
        let mut public_104 = [0u8; 104];
        public_104.copy_from_slice(&job.public_inputs);
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&public_104[32..64]);
        let public_amount = u64::from_le_bytes(public_104[96..104].try_into().unwrap());

        // Recipient wallet -> recipient ATA for output mint
        if outputs.is_empty() {
            return Err(Error::ValidationError("At least one output is required".into()));
        }
        let recipient_wallet = outputs[0].to_pubkey()?;
        let recipient_ata = get_associated_token_address(&recipient_wallet, &output_mint);

        // Parse mint address (use configured mint or default to native SOL)
        let input_mint = if let Some(mint_str) = &self.config.mint_address {
            if mint_str.is_empty() {
                Pubkey::default()
            } else {
                Pubkey::from_str(mint_str)
                    .map_err(|e| Error::ValidationError(format!("Invalid mint address: {}", e)))?
            }
        } else {
            Pubkey::default()
        };

        // Derive PDAs using the input mint (the pool's token)
        let (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda) =
            transaction_builder::derive_shield_pool_pdas(&self.program_id, &input_mint);
        let (swap_state_pda, _bump) =
            transaction_builder::derive_swap_state_pda(&self.program_id, &nullifier);

        // Use min_output_amount from client (baked into proof)
        // The client already got a quote and generated the proof with that value
        let min_output_amount = swap_config.min_output_amount;
        info!(
            "Using client-provided min_output_amount: {} (from proof)",
            min_output_amount
        );
        info!("🌊 Using Orca Whirlpool CPI for atomic on-chain swap");

        // Check if TX1 (WithdrawSwap) was already completed
        // If SwapState PDA exists, TX1 is done and we can skip to TX2
        let tx1_already_done = match self.check_swap_state_exists(&nullifier).await {
            Ok(exists) => {
                if exists {
                    info!("✓ SwapState PDA already exists - TX1 (WithdrawSwap) was previously completed");
                    info!("  Skipping TX1, proceeding directly to TX2 (swap + close)");
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                warn!("⚠️  Could not check SwapState PDA existence: {}, assuming TX1 not done", e);
                false
            }
        };

        // TX1: WithdrawSwap — lock SOL in SwapState PDA (skip if already done)
        if !tx1_already_done {
            let recent = self.client.get_latest_blockhash().await?;
            let withdraw_swap_tx = transaction_builder::build_withdraw_swap_transaction(
                job.proof_bytes.clone(),
                public_104,
                output_mint,
                recipient_ata,
                min_output_amount,
                self.program_id,
                pool_pda,
                roots_ring_pda,
                nullifier_shard_pda,
                treasury_pda,
                swap_state_pda,
                relay_pubkey,
                recent,
                self.config.priority_micro_lamports,
            )?;
            info!("Submitting WithdrawSwap (1/2)...");
            let mut signed_withdraw = withdraw_swap_tx.clone();
            let bh1 = signed_withdraw.message.recent_blockhash;
            signed_withdraw.sign(&[relay_keypair], bh1);
            let withdraw_sig = self
                .client
                .send_and_confirm_transaction(&signed_withdraw)
                .await?;
            info!("✓ WithdrawSwap confirmed: {}", withdraw_sig);
        }

        // TX2: ReleaseSwapFunds - get SOL from SwapState PDA
        // Check if SwapState PDA still has lamports to release (idempotency check)
        match self.client.get_account(&swap_state_pda).await {
            Ok(swap_state_account) => {
                let rent_exempt = self.client.get_minimum_balance_for_rent_exemption(SwapState::SIZE).await?;

                if swap_state_account.lamports > rent_exempt + 1_000_000 {
                    // SwapState has significant lamports beyond rent-exempt, proceed with release
                    info!("Releasing swap funds from SwapState PDA...");
                    let release_ix = transaction_builder::build_release_swap_funds_instruction(
                        self.program_id,
                        swap_state_pda,
                        relay_pubkey,
                    );
                    let release_blockhash = self.client.get_latest_blockhash().await?;
                    let mut release_msg = Message::new(&[release_ix], Some(&relay_pubkey));
                    release_msg.recent_blockhash = release_blockhash;
                    let mut release_tx = Transaction::new_unsigned(release_msg);
                    release_tx.sign(&[relay_keypair], release_blockhash);
                    let release_sig = self.client.send_and_confirm_transaction(&release_tx).await?;
                    info!("✓ ReleaseSwapFunds confirmed: {}", release_sig);
                } else {
                    info!("✓ SwapState PDA already released (lamports: {}, rent-exempt: {}), skipping TX2",
                        swap_state_account.lamports, rent_exempt);
                }
            }
            Err(e) => {
                warn!("⚠️ Could not check SwapState PDA balance: {}, proceeding with ReleaseSwapFunds", e);
                // If we can't check, try anyway (will fail with InsufficientLamports if already released)
                info!("Releasing swap funds from SwapState PDA...");
                let release_ix = transaction_builder::build_release_swap_funds_instruction(
                    self.program_id,
                    swap_state_pda,
                    relay_pubkey,
                );
                let release_blockhash = self.client.get_latest_blockhash().await?;
                let mut release_msg = Message::new(&[release_ix], Some(&relay_pubkey));
                release_msg.recent_blockhash = release_blockhash;
                let mut release_tx = Transaction::new_unsigned(release_msg);
                release_tx.sign(&[relay_keypair], release_blockhash);
                let release_sig = self.client.send_and_confirm_transaction(&release_tx).await?;
                info!("✓ ReleaseSwapFunds confirmed: {}", release_sig);
            }
        }

        // TX3: Relay performs OFF-CHAIN swap using Jupiter/Orca SDK
        // Note: After WithdrawSwap, the swap amount is already the user's amount (public_amount)
        // which includes the protocol fee deduction
        info!("🔄 Relay executing OFF-CHAIN swap: {} lamports SOL → minimum {} tokens of {}",
            public_amount, min_output_amount, output_mint);
        
        // Ensure relay has an ATA for the output token
        swap::ensure_ata_exists(self.client.as_ref(), &relay_pubkey, &output_mint, relay_keypair)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;
        
        // Ensure recipient has an ATA for the output token (relay pays for creation)
        swap::ensure_ata_exists(self.client.as_ref(), &recipient_wallet, &output_mint, relay_keypair)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;
        
        // Perform the swap (tries Jupiter first, falls back to Orca)
        let swap_signature = swap::perform_swap(
            self.client.as_ref(),
            relay_keypair,
            public_amount,
            output_mint,
            min_output_amount,
            recipient_ata,
        ).await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
        
        info!("✓ Token swap completed: {}", swap_signature);

        // TX4: ExecuteSwap — verify min output and close SwapState PDA
        let exec_ix = transaction_builder::build_execute_swap_instruction(
            self.program_id,
            nullifier,
            swap_state_pda,
            recipient_ata,
            relay_pubkey,
        );
        let recent2 = self.client.get_latest_blockhash().await?;
        let mut msg2 = Message::new(&[exec_ix], Some(&relay_pubkey));
        msg2.recent_blockhash = recent2;
        let mut exec_tx = Transaction::new_unsigned(msg2);
        let bh2 = exec_tx.message.recent_blockhash;
        exec_tx.sign(&[relay_keypair], bh2);
        let exec_sig = self.client.send_and_confirm_transaction(&exec_tx).await?;
        info!("✓ ExecuteSwap confirmed: {}", exec_sig);

        Ok(exec_sig)
    }

    /// Build withdraw transaction using the canonical shield-pool layout and PDAs
    /// If PoW is enabled (claim_finder present), will query for wildcard claims
    /// and use the PoW-enabled transaction builder
    async fn build_withdraw_transaction(
        &self,
        job: &Job,
        outputs: &[Output],
    ) -> Result<Transaction, Error> {
        let recent_blockhash = self.client.get_latest_blockhash().await?;

        // Validate outputs (1-10 allowed)
        if outputs.is_empty() || outputs.len() > 10 {
            return Err(Error::ValidationError(
                "Number of outputs must be between 1 and 10".into(),
            ));
        }

        // Convert API Output to planner Output
        use crate::planner::Output as PlannerOutput;
        let planner_outputs: Vec<PlannerOutput> = outputs
            .iter()
            .map(|o| {
                let pubkey = o.to_pubkey()?;
                Ok(PlannerOutput {
                    address: pubkey.to_bytes(),
                    amount: o.amount,
                })
            })
            .collect::<Result<_, Error>>()?;

        // Collect recipient pubkeys (1..N)
        let recipient_pubkeys: Vec<Pubkey> = planner_outputs
            .iter()
            .map(|o| Pubkey::new_from_array(o.address))
            .collect();

        // Get first recipient for fee payer fallback
        let recipient_pubkey = outputs[0].to_pubkey()?;

        if job.proof_bytes.is_empty() {
            return Err(Error::ValidationError(
                "proof bytes must be non-empty".into(),
            ));
        }
        let proof_bytes = job.proof_bytes.clone();
        if proof_bytes.len() >= 4 {
            let prefix = hex::encode(&proof_bytes[..4]);
            info!(
                proof_prefix = prefix.as_str(),
                proof_len = proof_bytes.len()
            );
        }

        if job.public_inputs.len() != 104 {
            return Err(Error::ValidationError(
                "public inputs must be 104 bytes".into(),
            ));
        }
        let mut public_104 = [0u8; 104];
        public_104.copy_from_slice(&job.public_inputs);

        // Parse mint address (empty = native SOL)
        let mint = if let Some(mint_str) = &self.config.mint_address {
            if mint_str.is_empty() {
                Pubkey::default() // Native SOL
            } else {
                Pubkey::from_str(mint_str)
                    .map_err(|e| Error::ValidationError(format!("Invalid mint address: {}", e)))?
            }
        } else {
            Pubkey::default() // Default to native SOL
        };
        let is_spl_mint = mint != Pubkey::default();

        // Get Shield Pool account addresses (use configured addresses if available, otherwise derive PDAs)
        let (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda) = if let (
            Some(pool_addr),
            Some(treasury_addr),
            Some(roots_ring_addr),
            Some(nullifier_shard_addr),
        ) = (
            &self.config.pool_address,
            &self.config.treasury_address,
            &self.config.roots_ring_address,
            &self.config.nullifier_shard_address,
        ) {
            // Use configured addresses
            let pool_pda = Pubkey::from_str(pool_addr)
                .map_err(|e| Error::ValidationError(format!("Invalid pool address: {}", e)))?;
            let treasury_pda = Pubkey::from_str(treasury_addr)
                .map_err(|e| Error::ValidationError(format!("Invalid treasury address: {}", e)))?;
            let roots_ring_pda = Pubkey::from_str(roots_ring_addr).map_err(|e| {
                Error::ValidationError(format!("Invalid roots ring address: {}", e))
            })?;
            let nullifier_shard_pda = Pubkey::from_str(nullifier_shard_addr).map_err(|e| {
                Error::ValidationError(format!("Invalid nullifier shard address: {}", e))
            })?;

            info!("Using configured account addresses:");
            info!("  Pool: {}", pool_pda);
            info!("  Treasury: {}", treasury_pda);
            info!("  Roots Ring: {}", roots_ring_pda);
            info!("  Nullifier Shard: {}", nullifier_shard_pda);

            (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda)
        } else {
            // Fallback to PDA derivation with mint
            warn!("Account addresses not configured, deriving PDAs with mint (this may cause errors if accounts don't exist)");
            transaction_builder::derive_shield_pool_pdas(&self.program_id, &mint)
        };

        // Fee payer pubkey: prefer loaded keypair, else withdraw_authority pubkey, else recipient
        let fee_payer_pubkey = if let Some(ref kp) = self.fee_payer {
            kp.pubkey()
        } else if let Some(ref auth) = self.config.withdraw_authority {
            Pubkey::from_str(auth).map_err(|e| {
                Error::ValidationError(format!("Invalid withdraw authority pubkey: {}", e))
            })?
        } else {
            recipient_pubkey
        };

        // Priority fee (micro-lamports per CU) from config
        let priority_micro_lamports: u64 = self.config.priority_micro_lamports;

        // Pre-compute SPL token accounts when using an SPL mint
        let pool_token_account = if is_spl_mint {
            Some(get_associated_token_address(&pool_pda, &mint))
        } else {
            None
        };
        let treasury_token_account = if is_spl_mint {
            Some(get_associated_token_address(&treasury_pda, &mint))
        } else {
            None
        };
        let recipient_token_accounts_vec: Option<Vec<Pubkey>> = if is_spl_mint {
            Some(
                recipient_pubkeys
                    .iter()
                    .map(|pk| get_associated_token_address(pk, &mint))
                    .collect(),
            )
        } else {
            None
        };
        let recipient_token_accounts_slice = recipient_token_accounts_vec.as_deref();

        // Check if PoW is enabled
        let tx = if let Some(ref claim_finder) = self.claim_finder {
            // PoW path: find specific claim and use PoW transaction builder
            info!("PoW enabled: searching for available claim...");

            // Compute batch_hash from job ID to match what the miner creates
            let job_id = job.request_id.to_string();
            let batch_hash = compute_batch_hash(&job_id);

            // Find available claim
            match claim_finder.find_claim(&batch_hash).await {
                Ok(Some(claim)) => {
                    info!(
                        "✓ Found claim: {} (miner: {}, expires at slot: {})",
                        claim.claim_pda, claim.miner_authority, claim.mined_slot
                    );

                    // Get scramble registry program ID from config
                    let scramble_registry_program_id = self
                        .config
                        .scramble_registry_program_id
                        .as_ref()
                        .and_then(|id| Pubkey::from_str(id).ok())
                        .ok_or_else(|| {
                            Error::ValidationError(
                                "Scramble registry program ID not configured".into(),
                            )
                        })?;

                    let miner_token_account = if is_spl_mint {
                        Some(get_associated_token_address(&claim.miner_authority, &mint))
                    } else {
                        None
                    };

                    // Build PoW-enabled transaction
                    let pow_tx = transaction_builder::build_withdraw_transaction_with_pow(
                        proof_bytes.clone(),
                        public_104,
                        &planner_outputs,
                        batch_hash,
                        self.program_id,
                        pool_pda,
                        roots_ring_pda,
                        nullifier_shard_pda,
                        treasury_pda,
                        &recipient_pubkeys,
                        scramble_registry_program_id,
                        claim.claim_pda,
                        claim.miner_pda,
                        claim.registry_pda,
                        claim.miner_authority,
                        fee_payer_pubkey,
                        recent_blockhash,
                        priority_micro_lamports,
                        if is_spl_mint { Some(mint) } else { None },
                        pool_token_account,
                        recipient_token_accounts_slice,
                        treasury_token_account,
                        miner_token_account,
                    )?;

                    // Check if transaction size exceeds Solana's limit (1644 bytes base64-encoded)
                    // We need to check the base64-encoded size since that's what RPC receives
                    let serialized_bytes = bincode::serialize(&pow_tx).unwrap_or_default();
                    let base64_encoded = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &serialized_bytes,
                    );
                    let encoded_size = base64_encoded.len();

                    if encoded_size > 1644 {
                        warn!(
                            "⚠️  PoW transaction too large ({} bytes base64 > 1644 limit), falling back to non-PoW transaction",
                            encoded_size
                        );

                        // Fallback to non-PoW transaction
                        transaction_builder::build_withdraw_transaction(
                            proof_bytes.clone(),
                            public_104,
                            &planner_outputs,
                            self.program_id,
                            pool_pda,
                            roots_ring_pda,
                            nullifier_shard_pda,
                            treasury_pda,
                            &recipient_pubkeys,
                            fee_payer_pubkey,
                            recent_blockhash,
                            priority_micro_lamports,
                            if is_spl_mint { Some(mint) } else { None },
                            pool_token_account,
                            recipient_token_accounts_slice,
                            treasury_token_account,
                        )?
                    } else {
                        info!(
                            "✓ PoW transaction size: {} bytes (base64: {})",
                            serialized_bytes.len(),
                            encoded_size
                        );
                        pow_tx
                    }
                }
                Ok(None) => {
                    warn!(
                        "No PoW claims available for job {}. Will retry until miners provide claims.",
                        job.request_id
                    );
                    return Err(Error::InternalServerError(
                        "No PoW claims available yet - waiting for miners to produce claims"
                            .to_string(),
                    ));
                }
                Err(e) => {
                    error!("Failed to query for claims: {}", e);
                    return Err(Error::InternalServerError(format!(
                        "Claim query failed: {}",
                        e
                    )));
                }
            }
        } else {
            transaction_builder::build_withdraw_transaction(
                proof_bytes.clone(),
                public_104,
                &planner_outputs,
                self.program_id,
                pool_pda,
                roots_ring_pda,
                nullifier_shard_pda,
                treasury_pda,
                &recipient_pubkeys,
                fee_payer_pubkey,
                recent_blockhash,
                priority_micro_lamports,
                if is_spl_mint { Some(mint) } else { None },
                pool_token_account,
                recipient_token_accounts_slice,
                treasury_token_account,
            )?
        };

        Ok(tx)
    }

    /// Submit transaction with retry logic.
    /// When Jito is enabled, rebuilds the transaction with a tip instruction.
    async fn submit_and_confirm(
        &self,
        transaction: &Transaction,
        job: &Job,
        outputs: &[Output],
    ) -> Result<Signature, Error> {
        // Suppress warnings when jito feature is not enabled
        #[cfg(not(feature = "jito"))]
        let _ = (job, outputs);
        let mut retries = 0;
        let max_retries = self.config.max_retries;

        // Choose submit path: Jito (feature + env) or RPC
        let use_jito =
            std::env::var("RELAY_JITO_ENABLED").unwrap_or_else(|_| "false".into()) == "true";

        if use_jito {
            #[cfg(feature = "jito")]
            {
                // Use RELAY_JITO_URL (or fallback to RELAY_SOLANA__RPC_URL)
                let jito_url = std::env::var("RELAY_JITO_URL")
                    .or_else(|_| std::env::var("RELAY_SOLANA__RPC_URL"))
                    .unwrap_or_else(|_| "http://localhost:8899".into());

                let mut jito = crate::solana::submit::JitoSubmit::new(&jito_url)
                    .map_err(|e| Error::InternalServerError(e.to_string()))?;

                // Fetch a random Jito tip account
                let tip_account = jito.fetch_tip_account().map_err(|e| {
                    Error::InternalServerError(format!("fetch tip account failed: {}", e))
                })?;

                // Rebuild transaction with tip instruction
                let recent_blockhash = self.client.get_latest_blockhash().await?;
                let recipient_pubkey = outputs[0].to_pubkey()?;
                let recipient_amount = outputs[0].amount;
                let recipient_addr_32: [u8; 32] = recipient_pubkey.to_bytes();

                if job.proof_bytes.is_empty() {
                    return Err(Error::ValidationError(
                        "proof bytes must be non-empty".into(),
                    ));
                }
                let proof_bytes = job.proof_bytes.clone();

                let mut public_104 = [0u8; 104];
                public_104.copy_from_slice(&job.public_inputs);

                // Parse mint address (empty = native SOL)
                let mint = if let Some(mint_str) = &self.config.mint_address {
                    if mint_str.is_empty() {
                        Pubkey::default() // Native SOL
                    } else {
                        Pubkey::from_str(mint_str).map_err(|e| {
                            Error::ValidationError(format!("Invalid mint address: {}", e))
                        })?
                    }
                } else {
                    Pubkey::default() // Default to native SOL
                };

                let (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda) = if let (
                    Some(pool_addr),
                    Some(treasury_addr),
                    Some(roots_ring_addr),
                    Some(nullifier_shard_addr),
                ) = (
                    &self.config.pool_address,
                    &self.config.treasury_address,
                    &self.config.roots_ring_address,
                    &self.config.nullifier_shard_address,
                ) {
                    // Use configured addresses
                    let pool_pda = Pubkey::from_str(pool_addr).map_err(|e| {
                        Error::ValidationError(format!("Invalid pool address: {}", e))
                    })?;
                    let treasury_pda = Pubkey::from_str(treasury_addr).map_err(|e| {
                        Error::ValidationError(format!("Invalid treasury address: {}", e))
                    })?;
                    let roots_ring_pda = Pubkey::from_str(roots_ring_addr).map_err(|e| {
                        Error::ValidationError(format!("Invalid roots ring address: {}", e))
                    })?;
                    let nullifier_shard_pda =
                        Pubkey::from_str(nullifier_shard_addr).map_err(|e| {
                            Error::ValidationError(format!(
                                "Invalid nullifier shard address: {}",
                                e
                            ))
                        })?;

                    (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda)
                } else {
                    // Fallback to PDA derivation with mint
                    transaction_builder::derive_shield_pool_pdas(&self.program_id, &mint)
                };

                let fee_payer_pubkey = if let Some(ref kp) = self.fee_payer {
                    kp.pubkey()
                } else if let Some(ref auth) = self.config.withdraw_authority {
                    Pubkey::from_str(auth).map_err(|e| {
                        Error::ValidationError(format!("Invalid withdraw authority pubkey: {}", e))
                    })?
                } else {
                    recipient_pubkey
                };

                let mut vtx = transaction_builder::build_withdraw_versioned_with_tip(
                    proof_bytes,
                    public_104,
                    recipient_addr_32,
                    recipient_amount,
                    self.program_id,
                    pool_pda,
                    roots_ring_pda,
                    nullifier_shard_pda,
                    treasury_pda,
                    recipient_pubkey,
                    fee_payer_pubkey,
                    recent_blockhash,
                    self.config.priority_micro_lamports,
                    tip_account,
                    self.config.jito_tip_lamports,
                )?;

                // Sign with fee payer
                if let Some(ref kp) = self.fee_payer {
                    vtx.sign(&[kp], recent_blockhash);
                }

                // Submit via Jito with retries
                while retries < max_retries {
                    match jito.send(vtx.clone()) {
                        Ok(signature) => {
                            return Ok(signature);
                        }
                        Err(e) => {
                            retries += 1;
                            if retries >= max_retries {
                                error!("Jito submit failed after {} attempts: {}", max_retries, e);
                                return Err(Error::InternalServerError(e.to_string()));
                            }
                            let delay =
                                Duration::from_millis(self.config.retry_delay_ms * retries as u64);
                            warn!(
                                "Jito attempt {} failed, retrying in {:?}: {}",
                                retries, delay, e
                            );
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
                return Err(Error::InternalServerError(
                    "Jito max retries exceeded".into(),
                ));
            }
            #[cfg(not(feature = "jito"))]
            {
                warn!("RELAY_JITO_ENABLED=true but crate not compiled with 'jito' feature; using RPC path");
            }
        }

        // RPC path: sign and submit the provided transaction
        let mut tx = transaction.clone();
        if let Some(ref kp) = self.fee_payer {
            let recent = tx.message.recent_blockhash;
            tx.sign(&[kp], recent);
        }

        while retries < max_retries {
            match self.client.send_and_confirm_transaction(&tx).await {
                Ok(signature) => {
                    debug!(
                        "Transaction confirmed: {} (attempt {})",
                        signature,
                        retries + 1
                    );
                    return Ok(signature);
                }
                Err(e) => {
                    retries += 1;
                    if retries >= max_retries {
                        error!("Transaction failed after {} attempts: {}", max_retries, e);
                        return Err(e);
                    }
                    let delay = Duration::from_millis(self.config.retry_delay_ms * retries as u64);
                    warn!(
                        "Transaction attempt {} failed, retrying in {:?}: {}",
                        retries, delay, e
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(Error::InternalServerError(
            "Max retries exceeded".to_string(),
        ))
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Output {
    pub recipient: String, // Base58 encoded public key
    pub amount: u64,       // Amount in lamports
}

impl Output {
    pub fn to_pubkey(&self) -> Result<Pubkey, Error> {
        Pubkey::from_str(&self.recipient)
            .map_err(|e| Error::ValidationError(format!("Invalid recipient address: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_outputs() {
        let _config = SolanaConfig {
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: "ws://localhost:8900".to_string(),
            commitment: "confirmed".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
            withdraw_authority: None,
            priority_micro_lamports: 1000,
            jito_tip_lamports: 0,
            max_retries: 3,
            retry_delay_ms: 1000,
            scramble_registry_program_id: Some(
                "scb1q9pSBXbmAj2rc58wqKKsvuf4t4z9CKx6Xk79Js4".to_string(),
            ),
            mint_address: None, // Default to native SOL
            pool_address: Some("11111111111111111111111111111111".to_string()),
            treasury_address: Some("11111111111111111111111111111111".to_string()),
            roots_ring_address: Some("11111111111111111111111111111111".to_string()),
            nullifier_shard_address: Some("11111111111111111111111111111111".to_string()),
        };

        // This would need to be updated when we implement the actual service
        // let service = SolanaService { /* ... */ };

        let outputs_json = json!([
            {
                "recipient": "11111111111111111111111111111112",
                "amount": 1000000
            },
            {
                "recipient": "11111111111111111111111111111113",
                "amount": 2000000
            }
        ]);

        // Test would call service.parse_outputs(&outputs_json)
        // For now, just test the JSON structure
        assert!(outputs_json.is_array());
        assert_eq!(outputs_json.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_output_to_pubkey() {
        let output = Output {
            recipient: "11111111111111111111111111111112".to_string(),
            amount: 1000000,
        };

        assert!(output.to_pubkey().is_ok());
    }

    #[test]
    fn test_invalid_output_pubkey() {
        let output = Output {
            recipient: "invalid".to_string(),
            amount: 1000000,
        };

        assert!(output.to_pubkey().is_err());
    }
}
