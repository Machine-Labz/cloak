pub mod client;
pub mod submit;
pub mod transaction_builder;

use async_trait::async_trait;
use hex;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
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

// Manual implementation of associated token account derivation
// This avoids dependency conflicts with spl-associated-token-account
fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    // Associated Token Account Program ID
    const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
    
    // Token Program ID
    const TOKEN_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    
    // Find the associated token account address
    let (ata, _) = Pubkey::find_program_address(
        &[
            wallet.as_ref(),
            TOKEN_PROGRAM_ID.as_ref(),
            mint.as_ref(),
        ],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    
    ata
}
// Removed external TransactionResult dependency; we return Signature to callers.

#[async_trait]
pub trait SolanaClient: Send + Sync {
    async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash, Error>;
    async fn send_and_confirm_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, Error>;
    async fn get_block_height(&self) -> Result<u64, Error>;
    async fn get_account_balance(&self, pubkey: &Pubkey) -> Result<u64, Error>;
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
        let fee_payer = if let Some(ref path) = config.withdraw_keypair_path {
            match read_keypair_file(path) {
                Ok(kp) => Some(kp),
                Err(e) => {
                    warn!("Failed to read withdraw keypair from {}: {}", path, e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            client,
            program_id,
            config,
            fee_payer,
            claim_finder: None, // Will be set later
        })
    }

    /// Set the ClaimFinder (for PoW support)
    pub fn set_claim_finder(&mut self, claim_finder: Option<Arc<ClaimFinder>>) {
        if claim_finder.is_some() {
            info!("SolanaService: PoW ClaimFinder configured");
        }
        self.claim_finder = claim_finder;
    }

    /// Submit a withdraw transaction to Solana
    pub async fn submit_withdraw(&self, job: &Job) -> Result<Signature, Error> {
        info!(
            "Submitting withdraw transaction for job: {}",
            job.request_id
        );

        // 1. Parse outputs from JSON
        let outputs = self.parse_outputs(&job.outputs_json)?;

        // 2. Build transaction
        let transaction = self.build_withdraw_transaction(job, &outputs).await?;

        // 3. Submit and confirm transaction
        let signature = self.submit_and_confirm(&transaction, job, &outputs).await?;

        info!("Withdraw transaction confirmed: {}", signature);
        Ok(signature)
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

    /// Build withdraw transaction using the canonical shield-pool layout and PDAs
    /// If PoW is enabled (claim_finder present), will query for wildcard claims
    /// and use the PoW-enabled transaction builder
    async fn build_withdraw_transaction(
        &self,
        job: &Job,
        outputs: &[Output],
    ) -> Result<Transaction, Error> {
        let recent_blockhash = self.client.get_latest_blockhash().await?;

        // Enforce MVP single output
        if outputs.len() != 1 {
            return Err(Error::ValidationError(
                "exactly 1 output required in MVP".into(),
            ));
        }
        let recipient_pubkey = outputs[0].to_pubkey()?;
        let recipient_amount = outputs[0].amount;
        let recipient_addr_32: [u8; 32] = recipient_pubkey.to_bytes();

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

        // Check if PoW is enabled
        let tx = if let Some(ref claim_finder) = self.claim_finder {
            // PoW path: find specific claim and use PoW transaction builder
            info!("PoW enabled: searching for available claim...");

            // Compute batch_hash from job ID to match what the miner creates
            let job_id = job.request_id.to_string();
            let batch_hash = compute_batch_hash(&job_id);

            debug!(
                "Computed batch_hash for job {}: {:x?}...",
                job_id,
                &batch_hash[0..8]
            );

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

                    // Derive token accounts if mint is not native SOL
                    let (pool_token_account, recipient_token_account, treasury_token_account, miner_token_account) = 
                        if mint != Pubkey::default() {
                            // Derive token accounts for SPL tokens
                            let pool_token_account = get_associated_token_address(
                                &pool_pda,
                                &mint,
                            );
                            let recipient_token_account = get_associated_token_address(
                                &recipient_pubkey,
                                &mint,
                            );
                            let treasury_token_account = get_associated_token_address(
                                &treasury_pda,
                                &mint,
                            );
                            let miner_token_account = get_associated_token_address(
                                &claim.miner_authority,
                                &mint,
                            );
                            
                            (Some(pool_token_account), Some(recipient_token_account), Some(treasury_token_account), Some(miner_token_account))
                        } else {
                            (None, None, None, None)
                        };

                    // Build PoW-enabled transaction
                    transaction_builder::build_withdraw_transaction_with_pow(
                        proof_bytes.clone(),
                        public_104,
                        recipient_addr_32,
                        recipient_amount,
                        batch_hash,
                        self.program_id,
                        pool_pda,
                        roots_ring_pda,
                        nullifier_shard_pda,
                        treasury_pda,
                        recipient_pubkey,
                        scramble_registry_program_id,
                        claim.claim_pda,
                        claim.miner_pda,
                        claim.registry_pda,
                        claim.miner_authority,
                        fee_payer_pubkey,
                        recent_blockhash,
                        priority_micro_lamports,
                        Some(mint),
                        pool_token_account,
                        recipient_token_account,
                        treasury_token_account,
                        miner_token_account,
                    )?
                }
                Ok(None) => {
                    // PoW mode is enabled but no claims are available yet
                    // Return a retryable error so the worker will requeue the job
                    // and try again when miners have produced claims
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
            // Legacy path (no PoW)
            debug!("PoW disabled: using legacy transaction builder");

                // Derive token accounts if mint is not native SOL
                let (pool_token_account, recipient_token_account) = 
                    if mint != Pubkey::default() {
                        // Derive token accounts for SPL tokens
                        let pool_token_account = get_associated_token_address(
                            &pool_pda,
                            &mint,
                        );
                        let recipient_token_account = get_associated_token_address(
                            &recipient_pubkey,
                            &mint,
                        );
                        
                        (Some(pool_token_account), Some(recipient_token_account))
                    } else {
                        (None, None)
                    };

            transaction_builder::build_withdraw_transaction(
                proof_bytes.clone(),
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
                priority_micro_lamports,
                Some(mint),
                pool_token_account,
                recipient_token_account,
            )?
        };

        debug!("Built withdraw transaction for job: {}", job.request_id);
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
                        Pubkey::from_str(mint_str)
                            .map_err(|e| Error::ValidationError(format!("Invalid mint address: {}", e)))?
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
                            debug!(
                                "Jito bundle submitted: {} (attempt {})",
                                signature,
                                retries + 1
                            );
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
            withdraw_keypair_path: None,
            priority_micro_lamports: 1000,
            jito_tip_lamports: 0,
            max_retries: 3,
            retry_delay_ms: 1000,
            scramble_registry_program_id: Some(
                "EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6".to_string(),
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
