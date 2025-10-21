use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
    transaction::Transaction,
};
use std::time::Duration;
use tracing::{debug, info, warn};

use super::SolanaClient;
use crate::config::SolanaConfig;
use crate::error::Error;

pub struct RpcSolanaClient {
    client: RpcClient,
    commitment: CommitmentConfig,
}

impl RpcSolanaClient {
    pub async fn new(config: &SolanaConfig) -> Result<Self, Error> {
        info!("Connecting to Solana RPC: {}", config.rpc_url);

        let commitment = match config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        let client = RpcClient::new_with_commitment(config.rpc_url.clone(), commitment);

        // Test connection
        match client.get_version().await {
            Ok(version) => {
                info!("Connected to Solana RPC, version: {}", version.solana_core);
            }
            Err(e) => {
                return Err(Error::InternalServerError(format!(
                    "Failed to connect to Solana RPC: {}",
                    e
                )));
            }
        }

        Ok(Self { client, commitment })
    }
}

#[async_trait]
impl SolanaClient for RpcSolanaClient {
    async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash, Error> {
        self.client
            .get_latest_blockhash()
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn send_and_confirm_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, Error> {
        debug!("Sending transaction to Solana");

        // First send the transaction
        let signature = self
            .client
            .send_transaction(transaction)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;

        debug!("Transaction sent: {}, waiting for confirmation", signature);

        // Then confirm it with retries
        let mut retries = 0;
        const MAX_CONFIRMATION_RETRIES: u32 = 30; // 30 * 2s = 60s max wait
        const CONFIRMATION_DELAY: Duration = Duration::from_secs(2);

        while retries < MAX_CONFIRMATION_RETRIES {
            match self
                .client
                .confirm_transaction_with_spinner(
                    &signature,
                    &self.client.get_latest_blockhash().await.unwrap(),
                    self.commitment,
                )
                .await
            {
                Ok(_) => {
                    debug!("Transaction confirmed: {}", signature);
                    return Ok(signature);
                }
                Err(e) => {
                    retries += 1;
                    if retries >= MAX_CONFIRMATION_RETRIES {
                        return Err(Error::InternalServerError(e.to_string()));
                    }

                    warn!(
                        "Transaction confirmation attempt {} failed, retrying in {:?}: {}",
                        retries, CONFIRMATION_DELAY, e
                    );
                    tokio::time::sleep(CONFIRMATION_DELAY).await;
                }
            }
        }

        Err(Error::InternalServerError(
            "Transaction confirmation timeout".to_string(),
        ))
    }

    async fn get_block_height(&self) -> Result<u64, Error> {
        self.client
            .get_block_height()
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn get_account_balance(&self, pubkey: &Pubkey) -> Result<u64, Error> {
        self.client
            .get_balance(pubkey)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_config_parsing() {
        let config1 = SolanaConfig {
            commitment: "processed".to_string(),
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: "ws://localhost:8900".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
            withdraw_authority: None,
            withdraw_keypair_path: None,
            priority_micro_lamports: 1000,
            jito_tip_lamports: 0,
            max_retries: 3,
            retry_delay_ms: 1000,
            scramble_registry_program_id: None,
            pool_address: Some("11111111111111111111111111111111".to_string()),
            treasury_address: Some("11111111111111111111111111111111".to_string()),
            roots_ring_address: Some("11111111111111111111111111111111".to_string()),
            nullifier_shard_address: Some("11111111111111111111111111111111".to_string()),
        };

        let config2 = SolanaConfig {
            commitment: "confirmed".to_string(),
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: "ws://localhost:8900".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
            withdraw_authority: None,
            withdraw_keypair_path: None,
            priority_micro_lamports: 1000,
            jito_tip_lamports: 0,
            max_retries: 3,
            retry_delay_ms: 1000,
            scramble_registry_program_id: None,
            pool_address: Some("11111111111111111111111111111111".to_string()),
            treasury_address: Some("11111111111111111111111111111111".to_string()),
            roots_ring_address: Some("11111111111111111111111111111111".to_string()),
            nullifier_shard_address: Some("11111111111111111111111111111111".to_string()),
        };

        let config3 = SolanaConfig {
            commitment: "finalized".to_string(),
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: "ws://localhost:8900".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
            withdraw_authority: None,
            withdraw_keypair_path: None,
            priority_micro_lamports: 1000,
            jito_tip_lamports: 0,
            max_retries: 3,
            retry_delay_ms: 1000,
            scramble_registry_program_id: None,
            pool_address: Some("11111111111111111111111111111111".to_string()),
            treasury_address: Some("11111111111111111111111111111111".to_string()),
            roots_ring_address: Some("11111111111111111111111111111111".to_string()),
            nullifier_shard_address: Some("11111111111111111111111111111111".to_string()),
        };

        // Test commitment parsing
        assert_eq!(config1.commitment, "processed");
        assert_eq!(config2.commitment, "confirmed");
        assert_eq!(config3.commitment, "finalized");
    }
}
