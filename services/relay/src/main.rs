mod api;
mod db;
mod error;
mod queue;

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::db::repository::{PostgresJobRepository, PostgresNullifierRepository};
use crate::queue::{redis_queue::RedisJobQueue, JobQueue, QueueConfig};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: db::DatabasePool,
    pub job_repo: Arc<PostgresJobRepository>,
    pub nullifier_repo: Arc<PostgresNullifierRepository>,
    pub queue: Arc<dyn JobQueue>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Database connection
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/relay".to_string());
        
        let db_pool = db::connect(&database_url).await?;
        db::run_migrations(&db_pool).await?;

        // Create repositories
        let job_repo = Arc::new(PostgresJobRepository::new(db_pool.clone()));
        let nullifier_repo = Arc::new(PostgresNullifierRepository::new(db_pool.clone()));

        // Redis connection
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let queue_config = QueueConfig::default();
        let queue: Arc<dyn JobQueue> = Arc::new(
            RedisJobQueue::new(&redis_url, queue_config).await?
        );

        Ok(Self {
            db_pool,
            job_repo,
            nullifier_repo,
            queue,
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Cloak Relay Service");

    // Create application state with real connections
    let app_state = AppState::new().await?;

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/withdraw", post(api::withdraw::handle_withdraw))
        .route("/status/:id", get(api::status::get_status))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    info!("Relay service listening on {}", addr);

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
            "status": "GET /status/:id"
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
