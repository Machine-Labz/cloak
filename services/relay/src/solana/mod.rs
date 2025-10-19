pub mod client;
pub mod submit;
pub mod transaction_builder;

use async_trait::async_trait;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    transaction::Transaction,
};
#[cfg(feature = "jito")]
use solana_sdk::{message::VersionedMessage, transaction::VersionedTransaction};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::config::SolanaConfig;
use crate::db::models::Job;
use crate::error::Error;
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
        })
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

    /// Build withdraw transaction using canonical 437-byte layout and PDAs
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

        // Extract proof fragment (260) and validate public inputs (104)
        let groth16_260 = cloak_proof_extract::extract_groth16_260(&job.proof_bytes)
            .map_err(|_| Error::ValidationError("failed to extract 260-byte proof".into()))?;
        if job.public_inputs.len() != 104 {
            return Err(Error::ValidationError(
                "public inputs must be 104 bytes".into(),
            ));
        }
        let mut public_104 = [0u8; 104];
        public_104.copy_from_slice(&job.public_inputs);

        // Derive Shield Pool PDAs
        let (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda) =
            transaction_builder::derive_shield_pool_pdas(&self.program_id);

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

        let tx = transaction_builder::build_withdraw_transaction(
            groth16_260,
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
        )?;

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

                let groth16_260 = cloak_proof_extract::extract_groth16_260(&job.proof_bytes)
                    .map_err(|_| {
                        Error::ValidationError("failed to extract 260-byte proof".into())
                    })?;
                let mut public_104 = [0u8; 104];
                public_104.copy_from_slice(&job.public_inputs);

                let (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda) =
                    transaction_builder::derive_shield_pool_pdas(&self.program_id);

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
                    groth16_260,
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
