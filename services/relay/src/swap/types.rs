use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Request to get a quote for a token swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    /// Input token mint address
    pub input_mint: String,
    /// Output token mint address
    pub output_mint: String,
    /// Amount of input token (in smallest unit)
    pub amount: u64,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage_bps: u16,
}

/// Response from quote API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Input mint address
    pub input_mint: String,
    /// Input amount
    pub in_amount: String,
    /// Output mint address
    pub output_mint: String,
    /// Expected output amount
    pub out_amount: String,
    /// Minimum output amount (accounting for slippage)
    pub other_amount_threshold: String,
    /// Price impact percentage
    pub price_impact_pct: String,
    /// Route information (for swap API)
    #[serde(flatten)]
    pub route_plan: serde_json::Value,
}

/// Request to execute a swap
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    /// Quote response from Jupiter
    #[serde(flatten)]
    pub quote: QuoteResponse,
    /// User's public key (relay wallet in our case)
    pub user_public_key: String,
    /// Whether to wrap/unwrap SOL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_and_unwrap_sol: Option<bool>,
    /// Use shared accounts for better pricing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_shared_accounts: Option<bool>,
    /// Fee account to collect fees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<String>,
    /// Compute unit price in micro lamports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_unit_price_micro_lamports: Option<u64>,
}

/// Response from swap API containing serialized transaction
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    /// Base64 encoded serialized transaction
    pub swap_transaction: String,
    /// Last valid block height
    pub last_valid_block_height: u64,
}

/// Swap configuration from user's withdraw request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapConfig {
    /// Output token mint address
    pub output_mint: String,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage_bps: u16,
    /// Minimum output amount (from Jupiter quote, baked into ZK proof)
    /// This MUST match the value used when generating the proof
    pub min_output_amount: u64,
}

impl SwapConfig {
    /// Validate swap configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate mint address
        Pubkey::try_from(self.output_mint.as_str())
            .map_err(|_| "Invalid output mint address".to_string())?;

        // Validate slippage (max 10% = 1000 bps)
        if self.slippage_bps > 1000 {
            return Err("Slippage too high (max 10%)".to_string());
        }

        Ok(())
    }
}
