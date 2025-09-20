mod api;
mod error;

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    // For now, this is empty since we're using mock implementations
    // In the future, this would contain database pools, Redis connections, etc.
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Cloak Relay Service");

    // Create application state
    let app_state = AppState::new();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/withdraw", post(api::withdraw::handle_withdraw))
        .route("/status/:id", get(api::status::get_status))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

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
