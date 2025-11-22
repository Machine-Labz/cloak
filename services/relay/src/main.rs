mod api;
mod claim_manager;
mod cloudwatch;
mod config;
mod db;
mod error;
mod planner;
mod solana;
mod worker;

use planner::orchestrator;

use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use std::num::NonZeroU32;
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::claim_manager::ClaimFinder;
use crate::config::Config as RelayConfig;
use crate::db::repository::{PostgresJobRepository, PostgresNullifierRepository};
use crate::solana::SolanaService;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: db::DatabasePool,
    pub job_repo: Arc<PostgresJobRepository>,
    pub nullifier_repo: Arc<PostgresNullifierRepository>,
    pub solana: Arc<SolanaService>,
    pub claim_finder: Option<Arc<ClaimFinder>>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load config
        let relay_config = RelayConfig::load()?;

        // Database connection
        let database_url = relay_config.database.url.clone();

        let db_pool = db::connect(&database_url, relay_config.database.max_connections).await?;
        db::run_migrations(&db_pool).await?;

        // Create repositories
        let job_repo = Arc::new(PostgresJobRepository::new(db_pool.clone()));
        let nullifier_repo = Arc::new(PostgresNullifierRepository::new(db_pool.clone()));

        // Solana service
        let mut solana_service = SolanaService::new(relay_config.solana.clone()).await?;

        // Initialize ClaimFinder if PoW is enabled
        let claim_finder =
            if let Some(ref registry_id) = relay_config.solana.scramble_registry_program_id {
                info!(
                    "Initializing PoW ClaimFinder with registry: {}",
                    registry_id
                );

                let registry_program_id = Pubkey::from_str(registry_id)
                    .map_err(|e| format!("Invalid scramble registry program ID: {}", e))?;

                let finder = Some(Arc::new(ClaimFinder::new(
                    relay_config.solana.rpc_url.clone(),
                    registry_program_id,
                )));

                // Configure solana service with claim finder
                solana_service.set_claim_finder(finder.clone());
                info!("✓ PoW ClaimFinder initialized successfully");

                finder
            } else {
                info!("PoW disabled - no scramble_registry_program_id configured");
                None
            };

        let solana = Arc::new(solana_service);

        Ok(Self {
            db_pool,
            job_repo,
            nullifier_repo,
            solana,
            claim_finder,
        })
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        // This would need proper mock implementations for testing
        panic!("Mock implementation not available yet")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (ignore error if file doesn't exist)
    let _ = dotenvy::dotenv();

    // Check if CloudWatch is enabled via environment variables
    let cloudwatch_enabled = std::env::var("CLOUDWATCH_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if cloudwatch_enabled {
        // Get CloudWatch configuration from environment
        let aws_access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
            .expect("AWS_ACCESS_KEY_ID must be set when CLOUDWATCH_ENABLED=true");
        let aws_secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .expect("AWS_SECRET_ACCESS_KEY must be set when CLOUDWATCH_ENABLED=true");
        let aws_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let log_group =
            std::env::var("CLOUDWATCH_LOG_GROUP").unwrap_or_else(|_| "Cloak".to_string());

        // Initialize CloudWatch logging
        cloudwatch::init_logging_with_cloudwatch(
            &aws_access_key_id,
            &aws_secret_access_key,
            &aws_region,
            &log_group,
        )
        .await?;
    } else {
        // Initialize standard tracing
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    info!("Starting Cloak Relay Service");

    // Load config for CORS settings
    let relay_config = RelayConfig::load()?;

    // Create application state with real connections
    let app_state = AppState::new().await?;

    // Configure CORS based on environment
    let cors = create_cors_layer(&relay_config.server.cors_origins);

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        // Withdraw endpoint with stricter rate limiting
        .route("/withdraw", post(api::withdraw::handle_withdraw).layer(rate_limit_withdraw()))
        .route("/status/:id", get(api::status::get_status))
        // Miners API - backlog status
        .route("/backlog", get(api::backlog::get_backlog_status))
        // Validator Agent API
        .route(
            "/jobs/withdraw",
            post(api::validator_agent::create_withdraw_job),
        )
        .route("/jobs/:job_id", get(api::validator_agent::get_job))
        .route("/submit", post(api::validator_agent::submit_tx))
        // Orchestration endpoint (planner-driven)
        .route(
            "/orchestrate/withdraw",
            post(orchestrator::orchestrate_withdraw),
        )
        // Apply general rate limiting to all routes
        .layer(rate_limit_general())
        .layer(cors)
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_XSS_PROTECTION,
            axum::http::HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    // Spawn the window scheduler task to process jobs in batched windows
    let scheduler_state = app_state.clone();
    tokio::spawn(async move {
        // Configure windowing: process when slot ends in 0 or 5
        let window_config = worker::window_scheduler::WindowConfig {
            slot_patterns: vec![0, 5],  // Every ~5 slots (~2.5s)
            min_batch_size: None,        // No minimum - process whatever is ready
            max_batch_size: 50,          // Safety limit
            poll_interval_secs: 1,       // Check slot every second
        };

        let scheduler = Arc::new(worker::window_scheduler::WindowScheduler::new(
            scheduler_state,
            window_config,
        ));

        scheduler.run().await;
    });

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    info!("Relay service listening on {}", addr);
    info!("Window scheduler spawned and running (processing on slot patterns: 0, 5)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Rate limiting middleware for withdraw endpoint
/// 10 requests per minute per IP (withdraw is critical)
fn rate_limit_withdraw() -> GovernorLayer {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_millisecond(NonZeroU32::new(6000).unwrap()) // 10 per minute = 1 per 6000ms
            .burst_size(NonZeroU32::new(20).unwrap())
            .finish()
            .unwrap(),
    );
    GovernorLayer::new(governor_conf)
}

/// Rate limiting middleware for general endpoints
/// 100 requests per minute per IP
fn rate_limit_general() -> GovernorLayer {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_millisecond(NonZeroU32::new(600).unwrap()) // 100 per minute = 1 per 600ms
            .burst_size(NonZeroU32::new(200).unwrap())
            .finish()
            .unwrap(),
    );
    GovernorLayer::new(governor_conf)
}

/// Create CORS layer based on configured origins
fn create_cors_layer(cors_origins: &[String]) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .max_age(std::time::Duration::from_secs(86400)); // 24 hours

    // Configure origins
    if cors_origins.len() == 1 && cors_origins[0] == "*" {
        // Allow all origins in development (without credentials)
        cors = cors.allow_origin(Any);
    } else {
        // Specific origins for production (with credentials)
        cors = cors.allow_credentials(true);
        for origin in cors_origins {
            if let Ok(origin_header) = origin.parse::<axum::http::HeaderValue>() {
                cors = cors.allow_origin(origin_header);
            }
        }
    }

    cors
}

async fn root() -> Json<Value> {
    Json(json!({
        "service": "Cloak Relay",
        "version": "0.1.0",
        "status": "running",
        "endpoints": {
            "health": "GET /health",
            "withdraw": "POST /withdraw",
            "status": "GET /status/:id",
            "jobs_withdraw": "POST /jobs/withdraw",
            "get_job": "GET /jobs/:job_id",
            "submit": "POST /submit"
        }
    }))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Relay service is healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
