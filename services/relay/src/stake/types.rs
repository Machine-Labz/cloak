use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Staking configuration from user's withdraw request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeConfig {
    /// Stake account address (where SOL will be staked)
    pub stake_account: String,
    /// Stake authority (who controls the stake account)
    pub stake_authority: String,
    /// Validator vote account to delegate to
    pub validator_vote_account: String,
}

impl StakeConfig {
    /// Validate stake configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate stake account address
        Pubkey::try_from(self.stake_account.as_str())
            .map_err(|_| "Invalid stake account address".to_string())?;

        // Validate stake authority address
        Pubkey::try_from(self.stake_authority.as_str())
            .map_err(|_| "Invalid stake authority address".to_string())?;

        // Validate validator vote account address
        Pubkey::try_from(self.validator_vote_account.as_str())
            .map_err(|_| "Invalid validator vote account address".to_string())?;

        Ok(())
    }
}

/// Unstaking configuration for private unstake-to-pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeConfig {
    /// Stake account address (where SOL is being unstaked from)
    pub stake_account: String,
    /// Stake authority (who controls the stake account - must sign)
    pub stake_authority: String,
    /// The new commitment to add to the shield pool
    pub commitment: String,
    /// Amount being unstaked (in lamports)
    pub amount: u64,
}

impl UnstakeConfig {
    /// Validate unstake configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate stake account address
        Pubkey::try_from(self.stake_account.as_str())
            .map_err(|_| "Invalid stake account address".to_string())?;

        // Validate stake authority address
        Pubkey::try_from(self.stake_authority.as_str())
            .map_err(|_| "Invalid stake authority address".to_string())?;

        // Validate commitment (should be 32 bytes hex = 64 chars)
        if self.commitment.len() != 64 {
            return Err("Commitment must be 64 hex characters (32 bytes)".to_string());
        }
        hex::decode(&self.commitment)
            .map_err(|_| "Invalid commitment hex".to_string())?;

        // Validate amount
        if self.amount == 0 {
            return Err("Amount cannot be zero".to_string());
        }

        Ok(())
    }
}

