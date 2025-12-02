use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub solana: SolanaConfig,
    pub server: ServerConfig,
    pub merkle: MerkleConfig,
    pub artifacts: ArtifactsConfig,
    pub sp1_tee: Sp1TeeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub shield_pool_program_id: String,
    pub admin_keypair: Option<Vec<u8>>,
    pub mint_address: String, // Token mint address (empty = native SOL)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub node_env: String,
    pub log_level: String,
    pub request_timeout_seconds: u64,
    pub cors_origins: Vec<String>,
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
    pub fn from_env() -> Result<Self> {
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
                url: std::env::var("DATABASE_URL")
                    .or_else(|_| std::env::var("DATABASE_URL"))
                    .ok(),
                max_connections: get_env_var_as_number("DB_MAX_CONNECTIONS", 20)?,
                min_connections: get_env_var_as_number("DB_MIN_CONNECTIONS", 2)?,
            },
            solana: SolanaConfig {
                rpc_url: get_env_var("SOLANA_RPC_URL", "http://127.0.0.1:8899"),
                shield_pool_program_id: get_env_var("CLOAK_PROGRAM_ID", ""),
                admin_keypair: get_admin_keypair_from_env(),
                mint_address: get_env_var("MINT_ADDRESS", ""), // Empty = native SOL
            },
            server: ServerConfig {
                port: get_env_var_as_number("PORT", 3001)?,
                node_env: get_env_var("NODE_ENV", "development"),
                log_level: get_env_var("LOG_LEVEL", "info"),
                request_timeout_seconds: get_env_var_as_number("REQUEST_TIMEOUT_SECONDS", 30)?,
                cors_origins: get_cors_origins(),
            },
            merkle: MerkleConfig {
                tree_height: get_env_var_as_number("TREE_HEIGHT", 32)?,
                zero_value: get_env_var(
                    "TREE_ZERO_VALUE",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                ),
            },
            artifacts: ArtifactsConfig {
                base_path: PathBuf::from(get_env_var("ARTIFACTS_BASE_PATH", "./artifacts")),
                sp1_version: get_env_var("SP1_VERSION", "v2.0.0"),
            },
            sp1_tee: Sp1TeeConfig {
                enabled: get_env_var("SP1_TEE_ENABLED", "false")
                    .parse()
                    .unwrap_or(false),
                wallet_address: get_env_var("SP1_TEE_WALLET_ADDRESS", ""),
                rpc_url: get_env_var("SP1_TEE_RPC_URL", "https://rpc.sp1-lumiere.xyz"),
                timeout_seconds: get_env_var_as_number("SP1_TEE_TIMEOUT_SECONDS", 300)?,
                private_key: std::env::var("NETWORK_PRIVATE_KEY").ok(),
            },
        };

        Ok(config)
    }

    /// Validate configuration and print helpful error messages
    pub fn log_summary(&self) {
        tracing::info!("Configuration loaded:");
        tracing::info!(
            "  Server: port {}, env: {}",
            self.server.port,
            self.server.node_env
        );
        tracing::info!("  Merkle tree: height {}", self.merkle.tree_height);
        tracing::info!(
            "  SP1 TEE: enabled={}, wallet={}, private_key_present={}",
            self.sp1_tee.enabled,
            self.sp1_tee.wallet_address,
            self.sp1_tee.private_key.is_some()
        );
    }

    pub fn database_url(&self) -> String {
        self.database.url.clone().unwrap_or_default()
    }

    pub fn is_production(&self) -> bool {
        self.server.node_env == "production"
    }

    pub fn is_development(&self) -> bool {
        self.server.node_env == "development"
    }

    pub fn is_test(&self) -> bool {
        self.server.node_env == "test"
    }
}

fn ensure_required_env_vars() -> Result<()> {
    let mut missing: Vec<String> = Vec::new();

    if !has_non_empty_env(["DATABASE_URL", "DATABASE_URL"]) {
        missing.push("DATABASE_URL".to_string());
    }

    if !has_non_empty_env(["SOLANA_RPC_URL", "CLOAK_PROGRAM_ID"]) {
        missing.push("SOLANA_RPC_URL".to_string());
        missing.push("CLOAK_PROGRAM_ID".to_string());
    }

    let tee_enabled = std::env::var("SP1_TEE_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .eq_ignore_ascii_case("true");

    if tee_enabled {
        for key in [
            "SP1_TEE_WALLET_ADDRESS",
            "SP1_TEE_RPC_URL",
            "SP1_TEE_TIMEOUT_SECONDS",
            "NETWORK_PRIVATE_KEY",
        ] {
            if !has_non_empty_env([key]) {
                missing.push(key.to_string());
            }
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        missing.sort();
        missing.dedup();
        Err(anyhow!(
            "Missing required environment variables: {}",
            missing.join(", ")
        ))
    }
}

fn has_non_empty_env<const N: usize>(keys: [&str; N]) -> bool {
    keys.iter().any(|key| {
        std::env::var(key)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}

fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_var_as_number<T>(key: &str, default: T) -> Result<T>
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

fn get_admin_keypair_from_env() -> Option<Vec<u8>> {
    // Try to get admin keypair from environment variable as JSON array
    if let Ok(mut keypair_json) = std::env::var("ADMIN_KEYPAIR") {
        // Strip surrounding single or double quotes if present
        keypair_json = keypair_json
            .trim()
            .trim_matches('\'')
            .trim_matches('"')
            .to_string();

        match serde_json::from_str::<Vec<u8>>(&keypair_json) {
            Ok(keypair_bytes) => {
                if keypair_bytes.len() == 64 {
                    tracing::info!(
                        "✅ Loaded admin keypair from ADMIN_KEYPAIR environment variable"
                    );
                    return Some(keypair_bytes);
                } else {
                    tracing::warn!(
                        "⚠️ ADMIN_KEYPAIR has invalid length: {} (expected 64)",
                        keypair_bytes.len()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to parse ADMIN_KEYPAIR as JSON: {}", e);
            }
        }
    }

    // Try to get from file path
    if let Ok(keypair_path) = std::env::var("ADMIN_KEYPAIR_PATH") {
        match std::fs::read_to_string(&keypair_path) {
            Ok(keypair_data) => match serde_json::from_str::<Vec<u8>>(&keypair_data) {
                Ok(keypair_bytes) => {
                    if keypair_bytes.len() == 64 {
                        tracing::info!("✅ Loaded admin keypair from file: {}", keypair_path);
                        return Some(keypair_bytes);
                    } else {
                        tracing::warn!(
                            "⚠️ Admin keypair file has invalid length: {} (expected 64)",
                            keypair_bytes.len()
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ Failed to parse admin keypair file as JSON: {}", e);
                }
            },
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read admin keypair file {}: {}",
                    keypair_path,
                    e
                );
            }
        }
    }

    tracing::warn!("⚠️ No admin keypair configured. Set ADMIN_KEYPAIR or ADMIN_KEYPAIR_PATH to enable automatic root pushing.");
    None
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
                    "https://cloak.network".to_string(),
                    "https://app.cloak.network".to_string(),
                ]
            } else {
                vec!["*".to_string()] // Allow all origins in development
            }
        }
    }
}