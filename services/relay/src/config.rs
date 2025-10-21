use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub solana: SolanaConfig,
    pub database: DatabaseConfig,
    pub metrics: MetricsConfig,
    // Note: No miner config - relay queries on-chain for claims from independent miners
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
    pub program_id: String,
    pub withdraw_authority: Option<String>,
    pub withdraw_keypair_path: Option<String>,
    pub priority_micro_lamports: u64,
    pub jito_tip_lamports: u64,
    pub max_retries: u8,
    pub retry_delay_ms: u64,

    // PoW Scrambler Registry (optional - if not set, PoW is disabled)
    // Relay queries on-chain for available claims from independent miners
    pub scramble_registry_program_id: Option<String>,

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
        // Load configuration from environment variables only
        let settings = config::Config::builder()
            // Start with default settings
            .set_default("server.port", 3002)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.request_timeout_seconds", 30)?
            // Solana defaults
            .set_default("solana.rpc_url", "http://localhost:8899")?
            .set_default("solana.ws_url", "ws://localhost:8900")?
            .set_default("solana.commitment", "confirmed")?
            .set_default("solana.program_id", "11111111111111111111111111111111")? // Default to system program ID
            .set_default("solana.priority_micro_lamports", 10000u64)?
            .set_default("solana.jito_tip_lamports", 100000u64)? // 0.0001 SOL tip
            .set_default("solana.max_retries", 5)?
            .set_default("solana.retry_delay_ms", 2000)?
            // Database defaults
            .set_default(
                "database.url",
                "postgres://postgres:postgres@localhost:5432/relay",
            )?
            .set_default("database.max_connections", 5)?
            // Metrics defaults
            .set_default("metrics.enabled", true)?
            .set_default("metrics.port", 9090)?
            .set_default("metrics.route", "/metrics")?
            .add_source(
                config::Environment::with_prefix("RELAY")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        // Deserialize and return the configuration
        settings.try_deserialize().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        std::env::set_var("RELAY__SERVER__PORT", "4000");
        let config = Config::load().unwrap();
        assert_eq!(config.server.port, 4000);
        std::env::remove_var("RELAY__SERVER__PORT");
    }
}
