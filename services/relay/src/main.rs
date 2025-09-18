mod api;
mod config;
mod error;
mod metrics;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "relay=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;
    let port = config.server.port;
    
    // Initialize metrics
    metrics::init()?;

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/withdraw", post(api::withdraw::handle_withdraw))
        .route("/status/:id", get(api::status::get_status))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(config));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("Not found")]
    NotFound,
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg).into_response(),
            Self::NotFound => (axum::http::StatusCode::NOT_FOUND, "Not found").into_response(),
            Self::Internal(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            )
                .into_response(),
        }
    }
}
