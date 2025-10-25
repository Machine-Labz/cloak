mod artifacts;
mod cloudwatch;
mod config;
mod database;
mod error;
mod logging;
mod merkle;
mod server;
pub mod solana;
mod sp1_tee_client;

use crate::config::Config;
use crate::logging::init_logging;
use crate::server::routes::start_server;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize logging (now async to support CloudWatch)
    init_logging(&config).await?;

    // Log configuration summary now that logging is ready
    config.log_summary();

    // Log service startup information
    crate::logging::log_service_startup(&config);

    // Start the server
    if let Err(e) = start_server(config).await {
        tracing::error!("Failed to start Cloak Indexer Service: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
