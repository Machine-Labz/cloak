use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub solana: SolanaConfig,
    pub server: ServerConfig,
    pub merkle: MerkleConfig,
    pub artifacts: ArtifactsConfig,
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

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if it exists

        let config = Config {
            database: DatabaseConfig {
                host: get_env_var("DB_HOST", "localhost"),
                port: get_env_var_as_number("DB_PORT", 5432)?,
                name: get_env_var("DB_NAME", "cloak_indexer"),
                user: get_env_var("DB_USER", "postgres"),
                password: get_env_var("DB_PASSWORD", ""),
                url: std::env::var("DATABASE_URL").ok(),
                max_connections: get_env_var_as_number("DB_MAX_CONNECTIONS", 20)?,
                min_connections: get_env_var_as_number("DB_MIN_CONNECTIONS", 2)?,
            },
            solana: SolanaConfig {
                rpc_url: get_env_var("SOLANA_RPC_URL", "https://api.devnet.solana.com"),
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
        };

        Ok(config)
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
