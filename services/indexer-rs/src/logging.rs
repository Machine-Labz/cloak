use crate::config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init_logging(config: &Config) -> anyhow::Result<()> {
    let log_level = &config.server.log_level;
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("cloak_indexer={}", log_level)));

    if config.is_production() {
        // JSON logging for production
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_level(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_filter(env_filter),
            )
            .init();
    } else {
        // Pretty logging for development
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_level(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_filter(env_filter),
            )
            .init();
    }

    tracing::info!(
        environment = config.server.node_env,
        log_level = log_level,
        "Logging initialized"
    );

    Ok(())
}

// Custom tracing layer for request logging
pub struct RequestLoggingLayer;

impl<S> Layer<S> for RequestLoggingLayer
where
    S: tracing::Subscriber,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Custom span creation logic if needed
        let _ = (attrs, id, ctx);
    }

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Custom event handling if needed
        let _ = event;
    }
}

// Structured logging macros for common patterns
#[macro_export]
macro_rules! log_request {
    ($method:expr, $uri:expr, $status:expr, $duration_ms:expr) => {
        tracing::info!(
            method = $method,
            uri = $uri,
            status = $status,
            duration_ms = $duration_ms,
            "Request processed"
        );
    };
}

#[macro_export]
macro_rules! log_deposit {
    ($leaf_commit:expr, $leaf_index:expr, $tx_signature:expr) => {
        tracing::info!(
            leaf_commit = $leaf_commit,
            leaf_index = $leaf_index,
            tx_signature = $tx_signature,
            "Deposit processed"
        );
    };
}

#[macro_export]
macro_rules! log_merkle_operation {
    ($operation:expr, $leaf_index:expr, $root:expr) => {
        tracing::info!(
            operation = $operation,
            leaf_index = $leaf_index,
            root = $root,
            "Merkle tree operation completed"
        );
    };
}

#[macro_export]
macro_rules! log_database_operation {
    ($operation:expr, $table:expr, $duration_ms:expr) => {
        tracing::debug!(
            operation = $operation,
            table = $table,
            duration_ms = $duration_ms,
            "Database operation completed"
        );
    };
}

// Health check logging
pub fn log_health_check(healthy: bool, details: &serde_json::Value) {
    if healthy {
        tracing::info!(
            healthy = true,
            details = %details,
            "Health check passed"
        );
    } else {
        tracing::warn!(
            healthy = false,
            details = %details,
            "Health check failed"
        );
    }
}

// Startup logging
pub fn log_startup_info(config: &Config) {
    tracing::info!(
        service = "cloak-indexer",
        version = env!("CARGO_PKG_VERSION"),
        port = config.server.port,
        environment = config.server.node_env,
        tree_height = config.merkle.tree_height,
        database_host = config.database.host,
        database_port = config.database.port,
        database_name = config.database.name,
        solana_rpc = config.solana.rpc_url,
        "Cloak Indexer starting up"
    );
}

// Shutdown logging
pub fn log_shutdown() {
    tracing::info!("Cloak Indexer shutting down gracefully");
}

// Error context logging
pub fn log_error_context(operation: &str, error: &dyn std::error::Error) {
    tracing::error!(
        operation = operation,
        error = %error,
        error_chain = ?error,
        "Operation failed with error"
    );
}

// Tests can be added here when needed
