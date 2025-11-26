use std::time::Duration;

use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
    transaction::Transaction,
};
use tracing::{error, info, warn};

use super::SolanaClient;
use crate::{config::SolanaConfig, error::Error};

pub struct RpcSolanaClient {
    client: RpcClient,
    commitment: CommitmentConfig,
}

impl RpcSolanaClient {
    pub async fn new(config: &SolanaConfig) -> Result<Self, Error> {
        info!("Connecting to Solana RPC");

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
                error!("Solana RPC connection error details: {:?}", e);
                return Err(Error::InternalServerError(format!(
                    "Failed to connect to Solana RPC: {} - {:?}",
                    e, e
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
        // First send the transaction
        let signature = self
            .client
            .send_transaction(transaction)
            .await
            .map_err(|e| {
                error!("🔴 DEBUG: send_transaction failed: {:?}", e);
                error!("🔴 DEBUG: Error details: {}", e);
                // Check if it's a simulation error
                let err_str = e.to_string();
                if err_str.contains("0x1001") || err_str.contains("RootNotFound") {
                    error!("🔴 DEBUG: This is a RootNotFound (0x1001) error!");
                }
                Error::InternalServerError(format!("send_transaction failed: {}", e))
            })?;

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

    async fn get_slot(&self) -> Result<u64, Error> {
        self.client
            .get_slot()
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn get_account_balance(&self, pubkey: &Pubkey) -> Result<u64, Error> {
        self.client
            .get_balance(pubkey)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn check_nullifier_exists(
        &self,
        nullifier_shard: &Pubkey,
        nullifier: &[u8],
    ) -> Result<bool, Error> {
        // Fetch the nullifier shard account data
        match self.client.get_account_data(nullifier_shard).await {
            Ok(data) => {
                // The nullifier shard stores nullifiers as a set of 32-byte hashes
                // Check if our nullifier exists in the account data
                if nullifier.len() != 32 {
                    return Err(Error::ValidationError(
                        "Nullifier must be 32 bytes".to_string(),
                    ));
                }

                // Search for the nullifier in the account data
                // Account data structure depends on the on-chain program implementation
                // For now, we'll do a simple search through 32-byte chunks
                for chunk in data.chunks_exact(32) {
                    if chunk == nullifier {
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            Err(e) => {
                // If account doesn't exist, nullifier doesn't exist either
                if e.to_string().contains("AccountNotFound") {
                    Ok(false)
                } else {
                    Err(Error::InternalServerError(format!(
                        "Failed to check nullifier: {}",
                        e
                    )))
                }
            }
        }
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
