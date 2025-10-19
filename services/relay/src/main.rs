mod api;
mod config;
mod db;
mod error;
mod miner;
mod planner;
mod queue;
mod solana;
mod worker;

use planner::orchestrator;

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::db::repository::{PostgresJobRepository, PostgresNullifierRepository};
use crate::queue::{redis_queue::RedisJobQueue, JobQueue, QueueConfig};
use crate::solana::SolanaService;
use crate::config::Config as RelayConfig;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: db::DatabasePool,
    pub job_repo: Arc<PostgresJobRepository>,
    pub nullifier_repo: Arc<PostgresNullifierRepository>,
    pub queue: Arc<dyn JobQueue>,
    pub solana: Arc<SolanaService>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load config
        let relay_config = RelayConfig::load()?;

        // Database connection
        let database_url = relay_config.database.url.clone();

        let db_pool = db::connect(&database_url).await?;
        db::run_migrations(&db_pool).await?;

        // Create repositories
        let job_repo = Arc::new(PostgresJobRepository::new(db_pool.clone()));
        let nullifier_repo = Arc::new(PostgresNullifierRepository::new(db_pool.clone()));

        // Redis connection
        let redis_url = relay_config.redis.url.clone();

        let queue_config = QueueConfig::default();
        let queue: Arc<dyn JobQueue> =
            Arc::new(RedisJobQueue::new(&redis_url, queue_config).await?);

        // Solana service
        let solana = Arc::new(SolanaService::new(relay_config.solana.clone()).await?);

        Ok(Self {
            db_pool,
            job_repo,
            nullifier_repo,
            queue,
            solana,
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

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Cloak Relay Service");

    // Create application state with real connections
    let app_state = AppState::new().await?;

    // Configure CORS to allow requests from the frontend
    let cors = CorsLayer::permissive();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/withdraw", post(api::withdraw::handle_withdraw))
        .route("/status/:id", get(api::status::get_status))
        // Validator Agent API
        .route("/jobs/withdraw", post(api::validator_agent::create_withdraw_job))
        .route("/jobs/:job_id", get(api::validator_agent::get_job))
        .route("/submit", post(api::validator_agent::submit_tx))
        // Orchestration endpoint (planner-driven)
        .route("/orchestrate/withdraw", post(orchestrator::orchestrate_withdraw))
        // Local proving endpoint (temporary until full worker proving pipeline)
        .route("/jobs/:job_id/prove-local", post(api::prove_local::prove_local))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    // Spawn the worker task to process jobs
    let worker_state = app_state.clone();
    tokio::spawn(async move {
        let worker = worker::Worker::new(worker_state)
            .with_poll_interval(std::time::Duration::from_secs(1))
            .with_max_concurrent_jobs(10);

        worker.run().await;
    });

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    info!("Relay service listening on {}", addr);
    info!("Worker task spawned and running");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
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
