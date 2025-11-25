use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};
use governor::middleware::NoOpMiddleware;
use tower_http::cors::{Any, CorsLayer};

/// Request logging middleware
pub async fn logging_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    tracing::info!(
        method = %method,
        uri = %uri,
        user_agent = %user_agent,
        client_ip = %client_ip,
        "📥 Incoming request"
    );

    // Process the request
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log completion with detailed information
    if status.is_success() {
        tracing::info!(
            method = %method,
            uri = %uri,
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "✅ Request completed successfully"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            method = %method,
            uri = %uri,
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "⚠️ Client error response"
        );
    } else if status.is_server_error() {
        tracing::error!(
            method = %method,
            uri = %uri,
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "❌ Server error response"
        );
    } else {
        tracing::info!(
            method = %method,
            uri = %uri,
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "📤 Request completed"
        );
    }

    response
}

/// CORS middleware configuration
pub fn cors_layer(cors_origins: &[String]) -> CorsLayer {
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

/// Request timeout middleware
/// Uses different timeouts based on endpoint:
/// - /api/v1/deposit: 60 seconds - Merkle tree insertion can be slow
/// - Other endpoints: 10 seconds - faster timeout for regular requests
pub async fn timeout_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    
    // Use longer timeout for slow endpoints
    let timeout_duration = if path == "/api/v1/deposit" {
        std::time::Duration::from_secs(60) // 60 seconds for deposit (Merkle tree insertion)
    } else {
        std::time::Duration::from_secs(10) // 10 seconds for other endpoints
    };

    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            tracing::warn!(
                path = %path,
                timeout_seconds = timeout_duration.as_secs(),
                "Request timed out"
            );
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Request size limit middleware (handled by axum's built-in DefaultBodyLimit)
pub fn request_size_limit() -> axum::extract::DefaultBodyLimit {
    // Limit request body to 1MB
    axum::extract::DefaultBodyLimit::max(1024 * 1024)
}

/// Rate limiting middleware for general endpoints
/// 100 requests per minute per IP
pub fn rate_limit_general() -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_millisecond(600u64) // 100 per minute = 1 per 600ms
            .burst_size(200u32)
            .finish()
            .unwrap(),
    );
    GovernorLayer {
        config: governor_conf,
    }
}


// Tests can be added here when needed
