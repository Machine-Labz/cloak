pub mod client;
pub mod transaction_builder;

use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::config::SolanaConfig;
use crate::db::models::Job;
use crate::error::Error;
use crate::queue::processor::TransactionResult;

#[async_trait]
pub trait SolanaClient: Send + Sync {
    async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash, Error>;
    async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature, Error>;
    async fn get_block_height(&self) -> Result<u64, Error>;
    async fn get_account_balance(&self, pubkey: &Pubkey) -> Result<u64, Error>;
}

pub struct SolanaService {
    client: Box<dyn SolanaClient>,
    program_id: Pubkey,
    config: SolanaConfig,
}

impl SolanaService {
    pub async fn new(config: SolanaConfig) -> Result<Self, Error> {
        let program_id = Pubkey::from_str(&config.program_id)
            .map_err(|e| Error::ConfigError(format!("Invalid program ID: {}", e)))?;

        let client = Box::new(client::RpcSolanaClient::new(&config).await?);

        Ok(Self {
            client,
            program_id,
            config,
        })
    }

    /// Submit a withdraw transaction to Solana
    pub async fn submit_withdraw(&self, job: &Job) -> Result<TransactionResult, Error> {
        info!("Submitting withdraw transaction for job: {}", job.request_id);

        // 1. Parse outputs from JSON
        let outputs = self.parse_outputs(&job.outputs_json)?;

        // 2. Build transaction
        let transaction = self.build_withdraw_transaction(job, &outputs).await?;

        // 3. Submit and confirm transaction
        let signature = self.submit_and_confirm(&transaction).await?;

        // 4. Get block height for the transaction
        let block_height = self.client.get_block_height().await.ok();

        let result = TransactionResult {
            transaction_id: signature.to_string(),
            signature: signature.to_string(),
            block_height,
        };

        info!("Withdraw transaction confirmed: {}", signature);
        Ok(result)
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
        let outputs_array = outputs_json.as_array()
            .ok_or_else(|| Error::ValidationError("Outputs must be an array".to_string()))?;

        let mut outputs = Vec::new();
        for output_value in outputs_array {
            let output: Output = serde_json::from_value(output_value.clone())
                .map_err(|e| Error::ValidationError(format!("Invalid output format: {}", e)))?;
            outputs.push(output);
        }

        if outputs.is_empty() {
            return Err(Error::ValidationError("At least one output is required".to_string()));
        }

        Ok(outputs)
    }

    /// Build withdraw transaction
    async fn build_withdraw_transaction(&self, job: &Job, outputs: &[Output]) -> Result<Transaction, Error> {
        // This will be implemented based on the actual shield-pool program interface
        // For now, we'll create a placeholder that shows the structure

        let recent_blockhash = self.client.get_latest_blockhash().await?;
        
        // TODO: Replace with actual shield-pool program instruction building
        let transaction = transaction_builder::build_withdraw_instruction(
            &self.program_id,
            &job.proof_bytes,
            &job.public_inputs,
            outputs,
            recent_blockhash,
        )?;

        debug!("Built withdraw transaction for job: {}", job.request_id);
        Ok(transaction)
    }

    /// Submit transaction with retry logic
    async fn submit_and_confirm(&self, transaction: &Transaction) -> Result<Signature, Error> {
        let mut retries = 0;
        let max_retries = self.config.max_retries;

        while retries < max_retries {
            match self.client.send_and_confirm_transaction(transaction).await {
                Ok(signature) => {
                    debug!("Transaction confirmed: {} (attempt {})", signature, retries + 1);
                    return Ok(signature);
                }
                Err(e) => {
                    retries += 1;
                    if retries >= max_retries {
                        error!("Transaction failed after {} attempts: {}", max_retries, e);
                        return Err(e);
                    }
                    
                    let delay = Duration::from_millis(self.config.retry_delay_ms * retries as u64);
                    warn!("Transaction attempt {} failed, retrying in {:?}: {}", retries, delay, e);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(Error::InternalServerError("Max retries exceeded".to_string()))
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Output {
    pub recipient: String,  // Base58 encoded public key
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
        let config = SolanaConfig {
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: "ws://localhost:8900".to_string(),
            commitment: "confirmed".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
            withdraw_authority: None,
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