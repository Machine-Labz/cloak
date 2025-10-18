use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::error::Error;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

/// Transaction submission interface.
pub trait Submit {
    /// Send a fully signed VersionedTransaction and return signature on success.
    fn send(&self, tx: VersionedTransaction) -> Result<Signature, Error>;
}

/// Backoff configuration for retries.
#[derive(Clone, Debug)]
pub struct BackoffConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(2),
        }
    }
}

fn backoff_sleep(cfg: &BackoffConfig, attempt: u32) {
    // Exponential backoff with full jitter.
    let base = cfg.initial_delay.as_millis() as u64;
    let mut delay = base.saturating_mul(1u64 << attempt.min(10));
    delay = delay.min(cfg.max_delay.as_millis() as u64);
    let jitter = fastrand::u64(0..=delay);
    sleep(Duration::from_millis(jitter));
}

/// Helper to confirm a signature by polling RPC until an optional min_slot is reached or timeout elapses.
pub fn confirm(
    rpc: &RpcClient,
    signature: &Signature,
    min_slot: Option<u64>,
    timeout: Duration,
) -> Result<(), Error> {
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err(Error::InternalServerError("confirmation timeout".into()));
        }

        let statuses = rpc
            .get_signature_statuses(&[*signature])
            .map_err(|e| Error::InternalServerError(format!("status fetch failed: {}", e)))?;
        if let Some(Some(status)) = statuses.value.get(0) {
            if let Some(err) = &status.err {
                return Err(Error::InternalServerError(format!(
                    "transaction failed: {:?}",
                    err
                )));
            }
            if let Some(min) = min_slot {
                if status.slot >= min {
                    return Ok(());
                }
            } else {
                // No slot target, any confirmation is fine
                return Ok(());
            }
        }
        sleep(Duration::from_millis(500));
    }
}

/// RPC-based submitter. Assumes tx carries compute budget ixs (including compute unit price) already.
pub struct RpcSubmit {
    pub rpc: RpcClient,
    pub commitment: CommitmentConfig,
    pub backoff: BackoffConfig,
}

impl RpcSubmit {
    pub fn new(rpc_url: &str) -> Result<Self, Error> {
        Ok(Self {
            rpc: RpcClient::new(rpc_url.to_string()),
            commitment: CommitmentConfig::confirmed(),
            backoff: BackoffConfig::default(),
        })
    }
}

impl Submit for RpcSubmit {
    fn send(&self, tx: VersionedTransaction) -> Result<Signature, Error> {
        let cfg = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentLevel::Processed),
            max_retries: Some(self.backoff.max_retries as usize),
            ..Default::default()
        };
        let mut last_err: Option<Error> = None;
        for attempt in 0..=self.backoff.max_retries {
            match self.rpc.send_transaction_with_config(&tx, cfg) {
                Ok(sig) => return Ok(sig),
                Err(e) => {
                    last_err = Some(Error::InternalServerError(format!(
                        "rpc send failed (attempt {}): {}",
                        attempt + 1,
                        e
                    )));
                    if attempt < self.backoff.max_retries {
                        backoff_sleep(&self.backoff, attempt);
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| Error::InternalServerError("send failed".into())))
    }
}

/// Jito-based submitter using the official jito-sdk-rust.
#[cfg(feature = "jito")]
pub struct JitoSubmit {
    pub sdk: jito_sdk_rust::JitoJsonRpcSDK,
    pub backoff: BackoffConfig,
    pub tip_account: Option<solana_sdk::pubkey::Pubkey>,
}

#[cfg(feature = "jito")]
impl JitoSubmit {
    /// Create a new Jito submitter. `jito_url` is your QuickNode/Jito endpoint.
    pub fn new(jito_url: &str) -> Result<Self, Error> {
        Ok(Self {
            sdk: jito_sdk_rust::JitoJsonRpcSDK::new(jito_url, None),
            backoff: BackoffConfig::default(),
            tip_account: None,
        })
    }

    /// Fetch a random Jito tip account and store it for use in tip instructions.
    pub fn fetch_tip_account(&mut self) -> Result<solana_sdk::pubkey::Pubkey, Error> {
        use std::str::FromStr;

        let tip_account_str = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.sdk.get_random_tip_account().await
            })
        }).map_err(|e| Error::InternalServerError(format!("failed to fetch Jito tip account: {}", e)))?;

        let pubkey = solana_sdk::pubkey::Pubkey::from_str(&tip_account_str)
            .map_err(|e| Error::InternalServerError(format!("invalid tip account pubkey: {}", e)))?;

        self.tip_account = Some(pubkey);
        tracing::info!("Selected Jito tip account: {}", tip_account_str);
        Ok(pubkey)
    }

    /// Get the stored tip account, or fetch a new one if not yet set.
    pub fn get_or_fetch_tip_account(&mut self) -> Result<solana_sdk::pubkey::Pubkey, Error> {
        if let Some(tip) = self.tip_account {
            Ok(tip)
        } else {
            self.fetch_tip_account()
        }
    }
}

#[cfg(feature = "jito")]
impl Submit for JitoSubmit {
    fn send(&self, tx: VersionedTransaction) -> Result<Signature, Error> {
        // Serialize tx to base64
        let bytes = bincode::serialize(&tx)
            .map_err(|e| Error::InternalServerError(format!("serialize tx failed: {}", e)))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);

        // Prepare bundle with single transaction
        let encoded_txs = vec![b64];
        let params = serde_json::json!([
            encoded_txs,
            { "encoding": "base64" }
        ]);

        // Send bundle with retries
        for attempt in 0..=self.backoff.max_retries {
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.sdk.send_bundle(Some(params.clone()), None).await
                })
            }) {
                Ok(resp) => {
                    let bundle_id = resp["result"]
                        .as_str()
                        .ok_or_else(|| Error::InternalServerError("no bundle_id in Jito response".into()))?;

                    // Return the transaction's signature (first sig in the tx)
                    let sig = tx
                        .signatures
                        .get(0)
                        .cloned()
                        .ok_or_else(|| Error::InternalServerError("missing tx signature".into()))?;

                    tracing::info!("Jito bundle submitted: {}", bundle_id);
                    return Ok(sig);
                }
                Err(e) => {
                    if attempt >= self.backoff.max_retries {
                        return Err(Error::InternalServerError(format!(
                            "Jito bundle send failed after {} attempts: {}",
                            attempt + 1,
                            e
                        )));
                    }
                    tracing::warn!("Jito send attempt {} failed: {}", attempt + 1, e);
                    backoff_sleep(&self.backoff, attempt);
                }
            }
        }
        Err(Error::InternalServerError("Jito send failed".into()))
    }
}
