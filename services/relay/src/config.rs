use std::path::PathBuf;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub solana: SolanaConfig,
    pub database: DatabaseConfig,
    pub metrics: MetricsConfig,
    pub jupiter: JupiterConfig,
    // Note: No miner config - relay queries on-chain for claims from independent miners
}

#[derive(Debug, Deserialize, Clone)]
pub struct JupiterConfig {
    pub enabled: bool,
    pub api_url: String,
    pub slippage_bps: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub request_timeout_seconds: u64,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
    pub program_id: String,
    pub withdraw_authority: Option<String>,
    pub priority_micro_lamports: u64,
    pub jito_tip_lamports: u64,
    pub max_retries: u8,
    pub retry_delay_ms: u64,

    // PoW Scrambler Registry (optional - if not set, PoW is disabled)
    // Relay queries on-chain for available claims from independent miners
    pub scramble_registry_program_id: Option<String>,

    // Token mint address (empty = native SOL)
    pub mint_address: Option<String>,

    // Shield Pool Account Addresses (optional - if not set, will calculate PDAs)
    pub pool_address: Option<String>,
    pub treasury_address: Option<String>,
    pub roots_ring_address: Option<String>,
    pub nullifier_shard_address: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    pub route: String,
}

// Note: MinerConfig removed - relay doesn't mine, it queries on-chain for claims
// Miners run independently using cloak-miner CLI

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        ensure_required_env_vars()?;

        let config = Config {
            server: ServerConfig {
                port: get_env_var_as_number("RELAY_PORT", 3002).unwrap_or(3002),
                host: get_env_var("RELAY_HOST", "0.0.0.0").to_string(),
                request_timeout_seconds: get_env_var_as_number("RELAY_REQUEST_TIMEOUT_SECONDS", 60)
                    .unwrap_or(60),
                cors_origins: get_cors_origins(),
            },
            solana: SolanaConfig {
                rpc_url: get_env_var("SOLANA_RPC_URL", "http://localhost:8899").to_string(),
                ws_url: get_env_var("SOLANA_WS_URL", "ws://localhost:8900").to_string(),
                commitment: get_env_var("SOLANA_COMMITMENT", "confirmed").to_string(),
                program_id: get_env_var("CLOAK_PROGRAM_ID", "11111111111111111111111111111111")
                    .to_string(),
                withdraw_authority: {
                    let val = get_env_var("ADMIN_KEYPAIR", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                priority_micro_lamports: get_env_var_as_number(
                    "SOLANA_PRIORITY_MICROLAMPORTS",
                    10000,
                )
                .unwrap_or(10000),
                jito_tip_lamports: get_env_var_as_number("SOLANA_JITO_TIP_LAMPORTS", 100000)
                    .unwrap_or(100000),
                max_retries: get_env_var_as_number("SOLANA_MAX_RETRIES", 5).unwrap_or(5),
                retry_delay_ms: get_env_var_as_number("SOLANA_RETRY_DELAY_MS", 4000)
                    .unwrap_or(4000),
                scramble_registry_program_id: {
                    let val = get_env_var("SCRAMBLE_REGISTRY_PROGRAM_ID", "")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                mint_address: {
                    let val = get_env_var("MINT_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                pool_address: {
                    let val = get_env_var("CLOAK_POOL_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                treasury_address: {
                    let val = get_env_var("CLOAK_TREASURY_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                roots_ring_address: {
                    let val = get_env_var("ROOTS_RING_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                nullifier_shard_address: {
                    let val = get_env_var("CLOAK_NULLIFIER_SHARD_ADDRESS", "")
                        .trim()
                        .to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
            },
            database: DatabaseConfig {
                url: get_env_var("DATABASE_URL", "postgres://user:pass@localhost:5432/db")
                    .to_string(),
                max_connections: get_env_var_as_number("DB_MAX_CONNECTIONS", 20).unwrap_or(20),
            },
            metrics: MetricsConfig {
                enabled: get_env_var("RELAY_METRICS_ENABLED", "true")
                    .parse()
                    .unwrap_or(true),
                port: get_env_var_as_number("RELAY_METRICS_PORT", 9090).unwrap_or(9090),
                route: get_env_var("RELAY_METRICS_ROUTE", "/metrics").to_string(),
            },
            jupiter: JupiterConfig {
                enabled: get_env_var("JUPITER_ENABLED", "false")
                    .parse()
                    .unwrap_or(false),
                api_url: get_env_var("JUPITER_API_URL", "https://quote-api.jup.ag/v6").to_string(),
                slippage_bps: get_env_var_as_number("JUPITER_SLIPPAGE_BPS", 50).unwrap_or(50),
            },
        };

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let vars = [
            ("RELAY_PORT", "4000"),
            ("DATABASE_URL", "postgres://user:pass@localhost:5432/db"),
            ("SOLANA_RPC_URL", "https://api.testnet.solana.com"),
            (
                "CLOAK_PROGRAM_ID",
                "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp",
            ),
        ];

        for (key, value) in vars.iter() {
            std::env::set_var(key, value);
        }

        let config = Config::load().unwrap();
        assert_eq!(config.server.port, 4000);

        for (key, _) in vars.iter() {
            std::env::remove_var(key);
        }
    }
}

fn ensure_required_env_vars() -> anyhow::Result<()> {
    let mut missing = Vec::new();

    if !has_non_empty_env(&["DATABASE_URL", "DATABASE_URL"]) {
        missing.push("DATABASE_URL (or DATABASE_URL)".to_string());
    }

    if !has_non_empty_env(&["SOLANA_RPC_URL"]) {
        missing.push("SOLANA_RPC_URL".to_string());
    }

    if !has_non_empty_env(&["CLOAK_PROGRAM_ID"]) {
        missing.push("CLOAK_PROGRAM_ID".to_string());
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "Missing required environment variables: {}",
            missing.join(", ")
        ))
    }
}

fn has_non_empty_env(keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        std::env::var(key)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleConfig {
    pub tree_height: usize,
    pub zero_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactsConfig {
    pub base_path: PathBuf,
    pub sp1_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sp1TeeConfig {
    pub enabled: bool,
    pub wallet_address: String,
    pub rpc_url: String,
    pub timeout_seconds: u64,
    pub private_key: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        match dotenvy::dotenv() {
            Ok(path) => tracing::info!("Loading environment from: {:?}", path),
            Err(err) => tracing::warn!(
                "No .env file found ({}), using system environment variables only",
                err
            ),
        }

        ensure_required_env_vars()?;

        let config = Config {
            database: DatabaseConfig {
                url: get_env_var(
                    "DATABASE_URL",
                    "postgres://postgres:postgres@localhost:5432/relay",
                )
                .to_string(),
                max_connections: get_env_var_as_number("DB_MAX_CONNECTIONS", 20).unwrap_or(20),
            },
            solana: SolanaConfig {
                rpc_url: get_env_var("SOLANA_RPC_URL", "http://localhost:8899").to_string(),
                ws_url: get_env_var("SOLANA_WS_URL", "ws://localhost:8900").to_string(),
                commitment: get_env_var("SOLANA_COMMITMENT", "confirmed").to_string(),
                program_id: get_env_var("CLOAK_PROGRAM_ID", "").to_string(),
                mint_address: {
                    let val = get_env_var("MINT_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                withdraw_authority: {
                    let val = get_env_var("ADMIN_KEYPAIR", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                priority_micro_lamports: get_env_var_as_number(
                    "SOLANA_PRIORITY_MICROLAMPORTS",
                    10000,
                )
                .unwrap_or(10000),
                jito_tip_lamports: get_env_var_as_number("SOLANA_JITO_TIP_LAMPORTS", 100000)
                    .unwrap_or(100000),
                max_retries: get_env_var_as_number("SOLANA_MAX_RETRIES", 5).unwrap_or(5),
                retry_delay_ms: get_env_var_as_number("SOLANA_RETRY_DELAY_MS", 4000)
                    .unwrap_or(4000),
                scramble_registry_program_id: {
                    let val = get_env_var("SCRAMBLE_REGISTRY_PROGRAM_ID", "")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                pool_address: {
                    let val = get_env_var("CLOAK_POOL_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                treasury_address: {
                    let val = get_env_var("CLOAK_TREASURY_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                roots_ring_address: {
                    let val = get_env_var("ROOTS_RING_ADDRESS", "").trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                nullifier_shard_address: {
                    let val = get_env_var("CLOAK_NULLIFIER_SHARD_ADDRESS", "")
                        .trim()
                        .to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
            },
            metrics: MetricsConfig {
                enabled: get_env_var("RELAY_METRICS_ENABLED", "true")
                    .parse()
                    .unwrap_or(true),
                port: get_env_var_as_number("RELAY_METRICS_PORT", 9090).unwrap_or(9090),
                route: get_env_var("RELAY_METRICS_ROUTE", "/metrics").to_string(),
            },
            jupiter: JupiterConfig {
                enabled: get_env_var("JUPITER_ENABLED", "false")
                    .parse()
                    .unwrap_or(false),
                api_url: get_env_var("JUPITER_API_URL", "https://quote-api.jup.ag/v6").to_string(),
                slippage_bps: get_env_var_as_number("JUPITER_SLIPPAGE_BPS", 50).unwrap_or(50),
            },
            server: ServerConfig {
                port: get_env_var_as_number("RELAY_PORT", 3002).unwrap_or(3002),
                host: get_env_var("RELAY_HOST", "0.0.0.0").to_string(),
                request_timeout_seconds: get_env_var_as_number("RELAY_REQUEST_TIMEOUT_SECONDS", 60)
                    .unwrap_or(60),
                cors_origins: get_cors_origins(),
            },
        };

        Ok(config)
    }
}

fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_var_as_number<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr + Copy,
    T::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("Failed to parse environment variable {}: {}", key, e)),
        Err(_) => Ok(default),
    }
}

fn get_cors_origins() -> Vec<String> {
    match std::env::var("CORS_ORIGINS") {
        Ok(origins) => origins
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Err(_) => {
            // Default CORS origins based on environment
            let node_env = get_env_var("NODE_ENV", "development");
            if node_env == "production" {
                vec![
                    "https://cloaklabz.xyz".to_string(),
                    "https://www.cloaklabz.xyz".to_string(),
                ]
            } else {
                vec!["*".to_string()] // Allow all origins in development
            }
        }
    }
}
