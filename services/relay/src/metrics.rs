use std::time::Instant;

use axum::{
    extract::MatchedPath,
    http::Request,
    middleware::Next,
    response::Response,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};

const EXPONENTIAL_SECONDS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

pub fn setup_metrics() -> anyhow::Result<()> {
    let builder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )?;
    
    let (recorder, _handle) = builder.build()?;
    metrics::set_boxed_recorder(Box::new(recorder))?;
    
    register_metrics();
    Ok(())
}

fn register_metrics() {
    metrics::describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests made."
    );
    
    metrics::describe_histogram!(
        "http_requests_duration_seconds",
        "HTTP request duration in seconds."
    );
    
    metrics::describe_counter!(
        "withdraw_requests_total",
        "Total number of withdraw requests received."
    );
    
    metrics::describe_counter!(
        "withdraw_requests_success_total",
        "Total number of successful withdraw requests."
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

pub async fn track_metrics(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    let method = req.method().clone();
    let response = next.run(req).await;
    let status = response.status().as_u16().to_string();
    let latency = start.elapsed();

    let method_str = method.to_string();
    let path_clone = path.clone();
    let status_clone = status.clone();
    
    metrics::increment_counter!(
        "http_requests_total",
        "method" => method_str.clone(),
        "path" => path_clone,
        "status" => status_clone
    );

    metrics::histogram!(
        "http_requests_duration_seconds",
        latency.as_secs_f64(),
        "method" => method_str,
        "path" => path,
        "status" => status
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
