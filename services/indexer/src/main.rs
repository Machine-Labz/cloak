mod artifacts;
mod config;
mod database;
mod error;
mod logging;
mod merkle;
mod server;

use crate::config::Config;
use crate::logging::init_logging;
use crate::server::routes::start_server;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize logging
    init_logging(&config)?;

    // Start the server
    if let Err(e) = start_server(config).await {
        tracing::error!("Failed to start Cloak Indexer Service: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
