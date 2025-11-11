use crate::artifacts::ArtifactManager;
use crate::config::Config;
use crate::database::{Database, PostgresTreeStorage};
use crate::error::{not_found_with_endpoints, IndexerError};
use crate::merkle::{MerkleTree, TreeStorage};
use crate::server::final_handlers::{AppState, *};
use crate::server::middleware::{
    cors_layer, logging_middleware, request_size_limit, timeout_middleware,
};
use crate::server::prover_handler::generate_proof;
use crate::sp1_tee_client::create_tee_client;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub async fn create_app(config: Config) -> Result<(Router, AppState), IndexerError> {
    tracing::info!("Initializing Cloak Indexer application components");

    // Initialize database
    let database = Database::new(&config).await?;

    // Run migrations
    database.migrate().await?;

    // Perform initial database health check and log pool stats
    match database.health_check().await {
        Ok(health) => {
            tracing::info!(
                current_time = %health.current_time,
                pool_size = health.pool_stats.size,
                pool_idle = health.pool_stats.idle,
                response_time_ms = health.response_time_ms,
                tables = ?health.tables_found,
                stats = ?health.stats,
                "Database health check passed at startup"
            );
        }
        Err(e) => {
            tracing::warn!("Database health check failed at startup: {}", e);
        }
    }

    // Initialize storage
    tracing::info!("Initializing Merkle tree storage");
    let storage = PostgresTreeStorage::new(database.pool().clone());

    // Initialize Merkle tree
    tracing::info!(
        tree_height = config.merkle.tree_height,
        zero_value = config.merkle.zero_value,
        "Creating Merkle tree"
    );
    let mut merkle_tree = MerkleTree::new(config.merkle.tree_height, &config.merkle.zero_value)?;

    // Set next index from storage
    let next_index = storage.get_max_leaf_index().await?;
    tracing::info!(next_index = next_index, "Setting Merkle tree next index");
    merkle_tree.set_next_index(next_index);

    // Initialize artifact manager
    tracing::info!("Initializing artifact manager");
    let artifact_manager = ArtifactManager::new(&config);

    // Initialize TEE client if enabled
    let tee_client = if config.sp1_tee.enabled {
        tracing::info!("Initializing SP1 TEE client at startup");
        match create_tee_client(config.sp1_tee.clone()) {
            Ok(client) => {
                tracing::info!("✅ SP1 TEE client initialized successfully at startup");
                Some(Arc::new(client))
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to initialize SP1 TEE client: {}", e);
                tracing::warn!("🔄 TEE proving will not be available");
                None
            }
        }
    } else {
        tracing::info!("SP1 TEE is disabled");
        None
    };

    // Create shared state
    tracing::info!("Application components initialized successfully");
    let state = AppState {
        storage,
        merkle_tree: Arc::new(Mutex::new(merkle_tree)),
        artifact_manager,
        config: config.clone(),
        tee_client,
    };

    // Create the router
    tracing::info!("Setting up HTTP routes and middleware");
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

    tracing::info!("HTTP server configuration completed");

    Ok((app, state))
}

fn create_api_v1_routes() -> Router<AppState> {
    Router::new()
        // Core endpoints
        .route("/deposit", post(deposit))
        .route("/merkle/root", get(get_merkle_root))
        .route("/merkle/proof/:index", get(get_merkle_proof))
        .route("/notes/range", get(get_notes_range))
        // Proof generation endpoint
        .route("/prove", post(generate_proof))
        // Admin endpoints
        .route("/admin/reset", post(reset_database))
        .route("/admin/push-root", post(admin_push_root))
        .route("/admin/push-specific-root", post(admin_push_specific_root))
}

/// Start the HTTP server
pub async fn start_server(config: Config) -> Result<(), IndexerError> {
    let port = config.server.port;

    // Create application and initialize components
    let (app, _state) = create_app(config).await?;
    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| {
            tracing::error!("Failed to bind to port {}: {}", port, e);
            IndexerError::internal(format!("Failed to bind to port {}", port))
        })?;

    tracing::info!("Cloak Indexer Service listening on 0.0.0.0:{}", port);
    tracing::info!("Indexer API started");

    axum::serve(listener, make_service)
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
