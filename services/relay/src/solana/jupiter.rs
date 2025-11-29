/// Jupiter DEX Integration for SPL Token Swaps
///
/// This module provides integration with Jupiter's swap aggregator API
/// for devnet/testnet environments. Jupiter aggregates liquidity across
/// multiple Solana DEXes to provide optimal swap rates.
///
/// # Features
/// - Quote fetching for token swaps
/// - Swap transaction execution
/// - Slippage protection
/// - Rate limiting and retries
///
/// # Usage
/// ```ignore
/// let jupiter = JupiterService::new(config)?;
/// let quote = jupiter.get_quote(input_mint, output_mint, amount).await?;
/// let tx = jupiter.swap(&quote, user_pubkey).await?;
/// ```
use crate::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::net::SocketAddr;
use std::str::FromStr;
use tracing::{debug, info, warn};

const JUPITER_QUOTE_API_V6: &str = "https://quote-api.jup.ag/v6";
const DEFAULT_SLIPPAGE_BPS: u16 = 50; // 0.5%

/// Jupiter service configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    /// Jupiter API base URL (v6)
    pub api_url: String,
    /// Maximum slippage in basis points (default: 50 = 0.5%)
    pub slippage_bps: u16,
    /// Enable/disable Jupiter integration
    pub enabled: bool,
    /// Prefer stable routes by restricting intermediate tokens
    pub restrict_intermediate_tokens: bool,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_url: JUPITER_QUOTE_API_V6.to_string(),
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            enabled: false,
            restrict_intermediate_tokens: true,
        }
    }
}

/// Jupiter swap service
pub struct JupiterService {
    client: Client,
    config: JupiterConfig,
}

/// Quote request parameters
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRequest {
    input_mint: String,
    output_mint: String,
    amount: String,
    slippage_bps: u16,
    only_direct_routes: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    restrict_intermediate_tokens: Option<bool>,
}

/// Quote response from Jupiter API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

/// Swap request parameters
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequest {
    user_public_key: String,
    quote_response: QuoteResponse,
    wrap_and_unwrap_sol: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    as_legacy_transaction: Option<bool>,
    compute_unit_price_micro_lamports: Option<u64>,
}

/// Swap response containing the serialized transaction
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapResponse {
    swap_transaction: String,
}

impl JupiterService {
    /// Create a new Jupiter service instance
    pub fn new(config: JupiterConfig) -> Result<Self, Error> {
        if !config.enabled {
            info!("Jupiter swap integration is disabled");
        }

        // Build HTTP client with optional DNS override for environments where
        // container DNS cannot resolve external hosts (e.g., quote-api.jup.ag)
        let mut builder = Client::builder().timeout(std::time::Duration::from_secs(60));

        // If JUPITER_DNS_OVERRIDE_IP is set, force-resolve the API host to this IP.
        // This preserves TLS/SNI while bypassing container DNS.
        if let Ok(override_ip) = std::env::var("JUPITER_DNS_OVERRIDE_IP") {
            if !override_ip.trim().is_empty() {
                match url::Url::parse(&config.api_url) {
                    Ok(api_url) => {
                        if let Some(host) = api_url.host_str() {
                            let port = api_url.port_or_known_default().unwrap_or(443);
                            let addr_str = format!("{}:{}", override_ip.trim(), port);
                            match addr_str.parse::<SocketAddr>() {
                                Ok(addr) => {
                                    builder = builder.resolve(host, addr);
                                    info!("Jupiter DNS override active: {} -> {}", host, addr);
                                }
                                Err(e) => warn!(
                                    "Invalid JUPITER_DNS_OVERRIDE_IP '{}': {} (skipping override)",
                                    override_ip, e
                                ),
                            }
                        } else {
                            warn!(
                                "Could not extract host from JUPITER_API_URL='{}'. Skipping DNS override.",
                                config.api_url
                            );
                        }
                    }
                    Err(e) => warn!(
                        "Invalid JUPITER_API_URL '{}': {} (skipping DNS override)",
                        config.api_url, e
                    ),
                }
            }
        }

        let client = builder.build().map_err(|e| {
            Error::InternalServerError(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self { client, config })
    }

    /// Check if Jupiter integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get a swap quote from Jupiter
    ///
    /// # Arguments
    /// * `input_mint` - Input token mint address
    /// * `output_mint` - Output token mint address
    /// * `amount` - Amount of input tokens (in smallest units)
    ///
    /// # Returns
    /// Quote containing expected output amount and route information
    pub async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
    ) -> Result<QuoteResponse, Error> {
        if !self.config.enabled {
            return Err(Error::ValidationError(
                "Jupiter integration is not enabled".into(),
            ));
        }

        let request = QuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount: amount.to_string(),
            slippage_bps: self.config.slippage_bps,
            only_direct_routes: false,
            restrict_intermediate_tokens: Some(self.config.restrict_intermediate_tokens),
        };

        debug!(
            "Fetching Jupiter quote: {} {} -> {}",
            amount, input_mint, output_mint
        );

        let url = format!("{}/quote", self.config.api_url);

        // Simple 1-shot retry on 429 with small backoff
        let mut response = self
            .client
            .get(&url)
            .query(&request)
            .send()
            .await
            .map_err(|e| {
                Error::InternalServerError(format!("Jupiter quote request failed: {}", e))
            })?;
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_ms = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| s * 1000)
                .unwrap_or(400);
            warn!(
                "Jupiter quote rate limited (429), retrying in {}ms",
                retry_ms
            );
            tokio::time::sleep(std::time::Duration::from_millis(retry_ms)).await;
            response = self
                .client
                .get(&url)
                .query(&request)
                .send()
                .await
                .map_err(|e| {
                    Error::InternalServerError(format!("Jupiter quote request failed: {}", e))
                })?;
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Jupiter quote failed: {} - {}", status, body);
            return Err(Error::InternalServerError(format!(
                "Jupiter quote failed: {}",
                status
            )));
        }

        let quote: QuoteResponse = response.json().await.map_err(|e| {
            Error::InternalServerError(format!("Failed to parse quote response: {}", e))
        })?;

        info!(
            "Jupiter quote received: {} {} -> {} {} (impact: {}%)",
            quote.in_amount, input_mint, quote.out_amount, output_mint, quote.price_impact_pct
        );

        Ok(quote)
    }

    /// Execute a swap using a quote
    ///
    /// # Arguments
    /// * `quote` - Quote obtained from `get_quote()`
    /// * `user_pubkey` - User's wallet public key
    /// * `priority_fee` - Optional priority fee in micro-lamports
    ///
    /// # Returns
    /// Unsigned transaction ready to be signed and submitted
    pub async fn swap(
        &self,
        quote: &QuoteResponse,
        user_pubkey: &Pubkey,
        priority_fee: Option<u64>,
    ) -> Result<Transaction, Error> {
        if !self.config.enabled {
            return Err(Error::ValidationError(
                "Jupiter integration is not enabled".into(),
            ));
        }

        let request = SwapRequest {
            user_public_key: user_pubkey.to_string(),
            quote_response: quote.clone(),
            wrap_and_unwrap_sol: true,
            // Request a legacy (non-versioned) transaction so we can extract
            // instructions and compose with our withdraw easily.
            as_legacy_transaction: Some(true),
            compute_unit_price_micro_lamports: priority_fee,
        };

        debug!("Requesting swap transaction from Jupiter");

        let url = format!("{}/swap", self.config.api_url);
        // One retry on 429 for swap as well
        let mut response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                Error::InternalServerError(format!("Jupiter swap request failed: {}", e))
            })?;
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_ms = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| s * 1000)
                .unwrap_or(500);
            warn!(
                "Jupiter swap rate limited (429), retrying in {}ms",
                retry_ms
            );
            tokio::time::sleep(std::time::Duration::from_millis(retry_ms)).await;
            response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    Error::InternalServerError(format!("Jupiter swap request failed: {}", e))
                })?;
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Jupiter swap failed: {} - {}", status, body);
            return Err(Error::InternalServerError(format!(
                "Jupiter swap failed: {}",
                status
            )));
        }

        let swap_response: SwapResponse = response.json().await.map_err(|e| {
            Error::InternalServerError(format!("Failed to parse swap response: {}", e))
        })?;

        // Decode the base64-encoded transaction
        use base64::Engine;
        let tx_bytes = base64::engine::general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .map_err(|e| {
                Error::InternalServerError(format!("Failed to decode swap transaction: {}", e))
            })?;

        let transaction: Transaction = bincode::deserialize(&tx_bytes).map_err(|e| {
            Error::InternalServerError(format!("Failed to deserialize transaction: {}", e))
        })?;

        info!("Jupiter swap transaction created successfully");

        Ok(transaction)
    }

    /// Get price impact percentage for a potential swap
    pub async fn get_price_impact(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
    ) -> Result<f64, Error> {
        let quote = self.get_quote(input_mint, output_mint, amount).await?;

        let impact = quote.price_impact_pct.parse::<f64>().map_err(|e| {
            Error::InternalServerError(format!("Failed to parse price impact: {}", e))
        })?;

        Ok(impact)
    }

    /// Check if a swap would have acceptable price impact
    pub async fn is_swap_acceptable(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        max_impact_pct: f64,
    ) -> Result<bool, Error> {
        let impact = self
            .get_price_impact(input_mint, output_mint, amount)
            .await?;
        Ok(impact <= max_impact_pct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jupiter_config_default() {
        let config = JupiterConfig::default();
        assert_eq!(config.api_url, JUPITER_QUOTE_API_V6);
        assert_eq!(config.slippage_bps, DEFAULT_SLIPPAGE_BPS);
        assert!(!config.enabled);
    }

    #[test]
    fn test_service_disabled() {
        let config = JupiterConfig::default();
        let service = JupiterService::new(config).unwrap();
        assert!(!service.is_enabled());
    }
}
