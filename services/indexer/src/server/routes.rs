use crate::artifacts::ArtifactManager;
use crate::config::Config;
use crate::database::{Database, PostgresTreeStorage};
use crate::error::{not_found_with_endpoints, IndexerError};
use crate::merkle::{MerkleTree, TreeStorage};
use crate::server::final_handlers::{AppState, *};
use crate::server::middleware::{
    cors_layer, logging_middleware, request_size_limit, timeout_middleware,
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub async fn create_app(config: Config) -> Result<(Router, AppState), IndexerError> {
    // Initialize database
    let database = Database::new(&config).await?;

    // Run migrations
    database.migrate().await?;

    // Initialize storage
    let storage = PostgresTreeStorage::new(database.pool().clone());

    // Initialize Merkle tree
    let mut merkle_tree = MerkleTree::new(config.merkle.tree_height, &config.merkle.zero_value)?;

    // Set next index from storage
    let next_index = storage.get_max_leaf_index().await?;
    merkle_tree.set_next_index(next_index);

    // Initialize artifact manager
    let artifact_manager = ArtifactManager::new(&config);

    // Create shared state
    let state = AppState {
        storage,
        merkle_tree: Arc::new(Mutex::new(merkle_tree)),
        artifact_manager,
        config: config.clone(),
    };

    // Create the router
    let app = Router::new()
        // Root endpoint
        .route("/", get(api_info))
        // Health endpoint (outside API versioning)
        .route("/health", get(health_check))
        // API v1 routes
        .nest("/api/v1", create_api_v1_routes())
        // Global 404 handler
        .fallback(|| async { not_found_with_endpoints() })
        // Add shared state
        .with_state(state.clone())
        // Add middleware layers (applied in reverse order)
        .layer(
            ServiceBuilder::new()
                // Outermost layer - compression
                .layer(CompressionLayer::new())
                // CORS
                .layer(cors_layer(&config.server.cors_origins))
                // Request timeout
                .layer(middleware::from_fn(timeout_middleware))
                // Request size limit
                .layer(request_size_limit())
                // Request logging
                .layer(middleware::from_fn(logging_middleware))
                // Tracing (innermost layer)
                .layer(TraceLayer::new_for_http()),
        );

    Ok((app, state))
}

fn create_api_v1_routes() -> Router<AppState> {
    Router::new()
        // Core endpoints
        .route("/deposit", post(deposit))
        .route("/merkle/root", get(get_merkle_root))
        .route("/merkle/proof/:index", get(get_merkle_proof))
        .route("/notes/range", get(get_notes_range))
        // Artifact endpoints
        .route("/artifacts/withdraw/:version", get(get_withdraw_artifacts))
        .route(
            "/artifacts/files/:version/:filename",
            get(serve_artifact_file),
        )
        // Admin endpoints (for development)
        .route("/admin/push-root", post(admin_push_root))
        .route("/admin/insert-leaf", post(admin_insert_leaf))
        .route("/admin/reset", post(reset_database))
}

/// Start the HTTP server
pub async fn start_server(config: Config) -> Result<(), IndexerError> {
    let port = config.server.port;

    crate::logging::log_startup_info(&config);

    let (app, _state) = create_app(config).await?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| {
            tracing::error!("Failed to bind to port {}: {}", port, e);
            IndexerError::internal(format!("Failed to bind to port {}", port))
        })?;

    tracing::info!(
        port = port,
        pid = std::process::id(),
        "Cloak Indexer API started"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            tracing::error!("Server error: {}", e);
            IndexerError::internal("Server failed to start")
        })?;

    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        },
    }

    crate::logging::log_shutdown();
}

// Integration tests would require a test database
