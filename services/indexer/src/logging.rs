use crate::config::Config;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

pub fn init_logging(config: &Config) -> anyhow::Result<()> {
    let level = parse_level(&config.server.log_level);
    let level_str = level_to_str(level);

    let fallback_directives = format!("indexer={level},cloak_indexer={level}", level = level_str);

    let combined_directives = match std::env::var("RUST_LOG") {
        Ok(existing) if !existing.trim().is_empty() => {
            if has_indexer_directive(&existing) {
                existing
            } else {
                format!("{existing},{fallback_directives}")
            }
        }
        _ => fallback_directives.clone(),
    };

    let env_filter = EnvFilter::try_new(combined_directives.as_str()).or_else(|err| {
        eprintln!(
            "Invalid RUST_LOG directives '{}': {}. Falling back to '{}'",
            combined_directives, err, fallback_directives
        );
        EnvFilter::try_new(fallback_directives.as_str()).map_err(|fallback_err| {
            anyhow::anyhow!(
                "Failed to parse logging directives. Provided: '{}' ({}), fallback ('{}') error: {}",
                combined_directives, err, fallback_directives, fallback_err
            )
        })
    })?;

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_level(true)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing subscriber: {}", e))?;

    tracing::info!(
        log_level = level_str,
        directives = %combined_directives,
        "Logging initialized"
    );

    Ok(())
}

fn parse_level(value: &str) -> LevelFilter {
    value
        .trim()
        .parse::<LevelFilter>()
        .unwrap_or(LevelFilter::INFO)
}

fn level_to_str(level: LevelFilter) -> &'static str {
    match level {
        LevelFilter::OFF => "off",
        LevelFilter::ERROR => "error",
        LevelFilter::WARN => "warn",
        LevelFilter::INFO => "info",
        LevelFilter::DEBUG => "debug",
        LevelFilter::TRACE => "trace",
    }
}

fn has_indexer_directive(value: &str) -> bool {
    value.split(',').any(|directive| {
        let directive = directive.trim();
        directive.starts_with("indexer") || directive.starts_with("cloak_indexer")
    })
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

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
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

// Enhanced startup logging similar to relay service
pub fn log_service_startup(config: &Config) {
    tracing::info!("Starting Cloak Indexer Service");

    tracing::info!(
        service = "cloak-indexer",
        version = env!("CARGO_PKG_VERSION"),
        tree_height = config.merkle.tree_height,
        zero_value = config.merkle.zero_value,
        solana_rpc = config.solana.rpc_url,
        shield_pool_program_id = config.solana.shield_pool_program_id,
        "Indexer service configuration loaded"
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
