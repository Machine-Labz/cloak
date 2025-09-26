use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub solana: SolanaConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub metrics: MetricsConfig,
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
    pub max_retries: u8,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    pub route: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // Initialize configuration reader
        let settings = config::Config::builder()
            // Start with default settings
            .set_default("server.port", 3001)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.request_timeout_seconds", 30)?
            
            // Solana defaults
            .set_default("solana.rpc_url", "http://localhost:8899")?
            .set_default("solana.ws_url", "ws://localhost:8900")?
            .set_default("solana.commitment", "confirmed")?
            .set_default("solana.program_id", "11111111111111111111111111111111")?  // Default to system program ID
            .set_default("solana.max_retries", 3)?
            .set_default("solana.retry_delay_ms", 1000)?
            
            // Database defaults
            .set_default("database.url", "postgres://postgres:postgres@localhost:5432/relay")?
            .set_default("database.max_connections", 5)?
            
            // Redis defaults
            .set_default("redis.url", "redis://localhost:6379")?
            .set_default("redis.max_connections", 10)?
            .set_default("redis.connection_timeout_seconds", 5)?
            
            // Metrics defaults
            .set_default("metrics.enabled", true)?
            .set_default("metrics.port", 9090)?
            .set_default("metrics.route", "/metrics")?
            
            // Add in settings from environment variables (with a prefix of RELAY and '__' as separator)
            // E.g. `RELAY_SERVER__PORT=5000` would set `server.port`
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
        std::env::set_var("RELAY_SERVER__PORT", "4000");
        let config = Config::load().unwrap();
        assert_eq!(config.server.port, 4000);
        std::env::remove_var("RELAY_SERVER__PORT");
    }

    #[test]
    fn test_redis_config_defaults() {
        let config = Config::load().unwrap();
        assert_eq!(config.redis.url, "redis://localhost:6379");
        assert_eq!(config.redis.max_connections, 10);
    }
}
