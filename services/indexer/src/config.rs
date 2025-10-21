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
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub shield_pool_program_id: String,
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
                host: get_env_var("DB_HOST", "localhost"),
                port: get_env_var_as_number("DB_PORT", 5434)?, // Changed from 5432 to 5434 (Docker default)
                name: get_env_var("DB_NAME", "cloak_indexer"),
                user: get_env_var("DB_USER", "cloak"), // Changed from postgres to cloak (Docker default)
                password: get_env_var("DB_PASSWORD", "development_password_change_in_production"), // Added default password
                url: std::env::var("INDEXER_DATABASE_URL")
                    .or_else(|_| std::env::var("DATABASE_URL"))
                    .ok(),
                max_connections: get_env_var_as_number("DB_MAX_CONNECTIONS", 20)?,
                min_connections: get_env_var_as_number("DB_MIN_CONNECTIONS", 2)?,
            },
            solana: SolanaConfig {
                rpc_url: get_env_var("SOLANA_RPC_URL", "http://127.0.0.1:8899"),
                shield_pool_program_id: get_env_var("SHIELD_POOL_PROGRAM_ID", ""),
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

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration and print helpful error messages
    pub fn validate(&self) -> Result<()> {
        // Check database password
        if self.database.password.is_empty() {
            eprintln!("⚠️  WARNING: Database password is empty!");
            eprintln!("   Set DB_PASSWORD environment variable or use default: development_password_change_in_production");
        }

        Ok(())
    }

    pub fn log_summary(&self) {
        tracing::info!("Configuration loaded:");
        tracing::info!(
            "  Database: {}@{}:{}/{}",
            self.database.user,
            self.database.host,
            self.database.port,
            self.database.name
        );
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
        if let Some(ref url) = self.database.url {
            url.clone()
        } else {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.database.user,
                self.database.password,
                self.database.host,
                self.database.port,
                self.database.name
            )
        }
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

    if !has_non_empty_env(["INDEXER_DATABASE_URL", "DATABASE_URL"]) {
        for key in ["DB_HOST", "DB_PORT", "DB_NAME", "DB_USER", "DB_PASSWORD"] {
            if !has_non_empty_env([key]) {
                missing.push(key.to_string());
            }
        }
    }

    for key in ["SOLANA_RPC_URL", "SHIELD_POOL_PROGRAM_ID"] {
        if !has_non_empty_env([key]) {
            missing.push(key.to_string());
        }
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

// Tests can be added here when needed
