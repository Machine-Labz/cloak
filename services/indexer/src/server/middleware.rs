use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
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

    // Process the request
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    crate::log_request!(
        method.as_str(),
        uri.path(),
        status.as_u16(),
        duration.as_millis() as u64
    );

    tracing::info!(
        method = %method,
        uri = %uri,
        status = status.as_u16(),
        duration_ms = duration.as_millis(),
        user_agent = %user_agent,
        "Request processed"
    );

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
pub async fn timeout_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let timeout_duration = std::time::Duration::from_secs(30); // Default 30 seconds

    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            tracing::warn!("Request timed out after {:?}", timeout_duration);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Request size limit middleware (handled by axum's built-in DefaultBodyLimit)
pub fn request_size_limit() -> axum::extract::DefaultBodyLimit {
    // Limit request body to 1MB
    axum::extract::DefaultBodyLimit::max(1024 * 1024)
}

// Tests can be added here when needed
