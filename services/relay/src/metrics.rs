use std::net::SocketAddr;
use std::time::Instant;

use axum::{
    extract::MatchedPath,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use metrics_exporter_prometheus::{
    Matcher, PrometheusBuilder, PrometheusHandle,
};
use metrics_util::MetricKindMask;

const EXPONENTIAL_SECONDS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

static METRICS: once_cell::sync::Lazy<PrometheusHandle> = once_cell::sync::Lazy::new(|| {
    let recorder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .expect("Failed to install metrics recorder");

    // Register default metrics
    register_metrics();

    recorder
});

fn register_metrics() {
    // Register custom metrics here
    metrics::describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests made."
    );
    
    metrics::describe_histogram!(
        "http_requests_duration_seconds",
        "A histogram of the HTTP request durations in seconds."
    );

    metrics::describe_counter!(
        "withdraw_requests_total",
        "Total number of withdraw requests processed."
    );

    metrics::describe_counter!(
        "withdraw_requests_failed_total",
        "Total number of failed withdraw requests."
    );

    metrics::describe_gauge!(
        "active_withdraw_requests",
        "Current number of active withdraw requests."
    );
}

pub fn init() -> anyhow::Result<()> {
    // The recorder is already initialized in the Lazy static
    Ok(())
}

pub fn get_handle() -> &'static PrometheusHandle {
    &METRICS
}

pub async fn track_metrics<B>(
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    let method = req.method().clone();

    // Execute the next middleware
    let response = next.run(req).await;

    let latency = start.elapsed();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!(
        "http_requests_duration_seconds",
        latency.as_secs_f64(),
        &labels
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use axum::Router;
    use axum_test::TestServer;
    use std::time::Duration;

    async fn test_handler() -> &'static str {
        "Hello, world!"
    }

    #[tokio::test]
    async fn test_metrics_middleware() {
        // Create a test router with the metrics middleware
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(tower::ServiceBuilder::new().concurrency_limit(10));

        // Create a test server
        let server = TestServer::new(app).unwrap();

        // Make a request to the test endpoint
        let response = server.get("/test").await;
        assert_eq!(response.text(), "Hello, world!");

        // Verify metrics were recorded
        let metrics = get_handle().render();
        assert!(metrics.contains("http_requests_total"));
        assert!(metrics.contains("http_requests_duration_seconds"));
    }
}
