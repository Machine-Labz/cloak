use super::types::{QuoteRequest, QuoteResponse, SwapRequest, SwapResponse};
use crate::error::Error;
use reqwest::Client;
use solana_sdk::pubkey::Pubkey;
use std::net::SocketAddr;
use std::str::FromStr;
use tracing::{debug, error, info};

const JUPITER_QUOTE_API: &str = "https://quote-api.jup.ag/v6/quote";
const JUPITER_SWAP_API: &str = "https://quote-api.jup.ag/v6/swap";

/// Jupiter aggregator client for token swaps
#[derive(Clone)]
pub struct JupiterClient {
    http_client: Client,
}

impl JupiterClient {
    /// Create a new Jupiter client
    pub fn new() -> Self {
        // Allow DNS override for quote-api host if container DNS is blocked
        let mut builder = Client::builder();
        if let Ok(override_ip) = std::env::var("JUPITER_DNS_OVERRIDE_IP") {
            if !override_ip.trim().is_empty() {
                let addr_str = format!("{}:443", override_ip.trim());
                if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                    builder = builder.resolve("quote-api.jup.ag", addr);
                }
            }
        }

        Self {
            http_client: builder.build().unwrap_or_else(|_| Client::new()),
        }
    }

    /// Get a quote for swapping tokens
    ///
    /// # Arguments
    /// * `request` - Quote request parameters
    ///
    /// # Returns
    /// * `QuoteResponse` - Quote with route and expected amounts
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, Error> {
        debug!(
            "Requesting Jupiter quote: {} {} -> {}",
            request.amount, request.input_mint, request.output_mint
        );

        // Validate mint addresses
        Pubkey::from_str(&request.input_mint).map_err(|_| {
            Error::ValidationError(format!("Invalid input mint: {}", request.input_mint))
        })?;
        Pubkey::from_str(&request.output_mint).map_err(|_| {
            Error::ValidationError(format!("Invalid output mint: {}", request.output_mint))
        })?;

        // Build query parameters
        let url = format!(
            "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_QUOTE_API,
            request.input_mint,
            request.output_mint,
            request.amount,
            request.slippage_bps
        );

        // Make request
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("Jupiter quote request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Jupiter quote failed: {} - {}", status, error_text);
            return Err(Error::NetworkError(format!(
                "Jupiter quote failed: {} - {}",
                status, error_text
            )));
        }

        let quote: QuoteResponse = response.json().await.map_err(|e| {
            Error::SerializationError(format!("Failed to parse Jupiter quote: {}", e))
        })?;

        info!(
            "Jupiter quote: {} {} -> {} {} (price impact: {}%)",
            quote.in_amount,
            request.input_mint,
            quote.out_amount,
            request.output_mint,
            quote.price_impact_pct
        );

        Ok(quote)
    }

    /// Get swap transaction from Jupiter
    ///
    /// # Arguments
    /// * `request` - Swap request with quote and user public key
    ///
    /// # Returns
    /// * `SwapResponse` - Serialized transaction ready to sign and send
    pub async fn get_swap_transaction(&self, request: &SwapRequest) -> Result<SwapResponse, Error> {
        debug!(
            "Requesting Jupiter swap transaction for user: {}",
            request.user_public_key
        );

        // Validate user public key
        Pubkey::from_str(&request.user_public_key).map_err(|_| {
            Error::ValidationError(format!(
                "Invalid user public key: {}",
                request.user_public_key
            ))
        })?;

        // Make request
        let response = self
            .http_client
            .post(JUPITER_SWAP_API)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("Jupiter swap request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Jupiter swap failed: {} - {}", status, error_text);
            return Err(Error::NetworkError(format!(
                "Jupiter swap failed: {} - {}",
                status, error_text
            )));
        }

        let swap_response: SwapResponse = response.json().await.map_err(|e| {
            Error::SerializationError(format!("Failed to parse Jupiter swap response: {}", e))
        })?;

        info!(
            "Jupiter swap transaction received (valid until block height: {})",
            swap_response.last_valid_block_height
        );

        Ok(swap_response)
    }

    /// Get a quote and execute swap in one call (convenience method)
    ///
    /// # Arguments
    /// * `input_mint` - Input token mint address
    /// * `output_mint` - Output token mint address
    /// * `amount` - Amount to swap (in smallest unit)
    /// * `slippage_bps` - Slippage tolerance in basis points
    /// * `user_pubkey` - User's public key (relay wallet)
    ///
    /// # Returns
    /// * `(QuoteResponse, SwapResponse)` - Quote and swap transaction
    pub async fn quote_and_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &Pubkey,
    ) -> Result<(QuoteResponse, SwapResponse), Error> {
        // Get quote
        let quote_request = QuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount,
            slippage_bps,
        };

        let quote = self.get_quote(&quote_request).await?;

        // Get swap transaction
        let swap_request = SwapRequest {
            quote: quote.clone(),
            user_public_key: user_pubkey.to_string(),
            wrap_and_unwrap_sol: Some(true), // Auto wrap/unwrap SOL
            use_shared_accounts: Some(true), // Better pricing
            fee_account: None,
            compute_unit_price_micro_lamports: None, // Use default
        };

        let swap = self.get_swap_transaction(&swap_request).await?;

        Ok((quote, swap))
    }
}

impl Default for JupiterClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
    const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_quote() {
        let client = JupiterClient::new();
        let request = QuoteRequest {
            input_mint: SOL_MINT.to_string(),
            output_mint: USDC_MINT.to_string(),
            amount: 1_000_000_000, // 1 SOL
            slippage_bps: 50,      // 0.5%
        };

        let result = client.get_quote(&request).await;
        assert!(result.is_ok());

        let quote = result.unwrap();
        assert_eq!(quote.input_mint, SOL_MINT);
        assert_eq!(quote.output_mint, USDC_MINT);
    }
}
