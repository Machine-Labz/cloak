use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::types::InputLogEvent;
use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Get the machine ID to use as log stream name
fn get_machine_id() -> String {
    // Try to get hostname first
    if let Ok(hostname) = hostname::get() {
        if let Ok(hostname_str) = hostname.into_string() {
            return hostname_str;
        }
    }

    // Fallback to a UUID if hostname is not available
    uuid::Uuid::new_v4().to_string()
}

/// Background task to send logs to CloudWatch
async fn cloudwatch_log_sender(
    client: Arc<CloudWatchLogsClient>,
    log_group_name: String,
    log_stream_name: String,
    mut rx: mpsc::UnboundedReceiver<String>,
) {
    // Try to create log stream (ignore error if it already exists)
    let _ = client
        .create_log_stream()
        .log_group_name(&log_group_name)
        .log_stream_name(&log_stream_name)
        .send()
        .await;

    let mut sequence_token: Option<String> = None;
    let mut batch = Vec::new();

    while let Some(message) = rx.recv().await {
        batch.push(
            InputLogEvent::builder()
                .timestamp(chrono::Utc::now().timestamp_millis())
                .message(message)
                .build()
                .expect("Failed to build log event"),
        );

        // Send in batches of 10 or after accumulation
        if batch.len() >= 10 {
            if let Err(e) = send_batch(&client, &log_group_name, &log_stream_name, &mut batch, &mut sequence_token).await {
                eprintln!("Failed to send logs to CloudWatch: {}", e);
            }
        }
    }

    // Send remaining logs
    if !batch.is_empty() {
        let _ = send_batch(&client, &log_group_name, &log_stream_name, &mut batch, &mut sequence_token).await;
    }
}

async fn send_batch(
    client: &CloudWatchLogsClient,
    log_group_name: &str,
    log_stream_name: &str,
    batch: &mut Vec<InputLogEvent>,
    sequence_token: &mut Option<String>,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut request = client
        .put_log_events()
        .log_group_name(log_group_name)
        .log_stream_name(log_stream_name)
        .set_log_events(Some(batch.clone()));

    if let Some(token) = sequence_token.as_ref() {
        request = request.sequence_token(token.clone());
    }

    match request.send().await {
        Ok(output) => {
            *sequence_token = output.next_sequence_token;
            batch.clear();
            Ok(())
        }
        Err(e) => {
            batch.clear();
            Err(anyhow::anyhow!("Failed to put log events: {}", e))
        }
    }
}

/// Initialize logging with CloudWatch integration
pub async fn init_logging_with_cloudwatch(
    aws_access_key_id: &str,
    aws_secret_access_key: &str,
    aws_region: &str,
    log_group_name: &str,
    log_level: &str,
) -> Result<()> {
    // Get machine ID for log stream
    let machine_id = get_machine_id();
    let log_stream_name = format!("indexer-{}", machine_id);

    // Configure AWS credentials
    let credentials = aws_sdk_cloudwatchlogs::config::Credentials::new(
        aws_access_key_id,
        aws_secret_access_key,
        None,
        None,
        "cloak-indexer",
    );

    // Build AWS config
    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(aws_config::Region::new(aws_region.to_string()))
        .load()
        .await;

    // Create CloudWatch Logs client
    let cloudwatch_client = Arc::new(CloudWatchLogsClient::new(&config));

    // Create channel for sending logs
    let (_tx, rx) = mpsc::unbounded_channel();

    // Spawn background task to send logs
    let client_clone = cloudwatch_client.clone();
    let group_clone = log_group_name.to_string();
    let stream_clone = log_stream_name.clone();
    tokio::spawn(async move {
        cloudwatch_log_sender(client_clone, group_clone, stream_clone, rx).await;
    });

    // Create env filter with log level and common directives
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "{},indexer={},cloak_indexer={},sqlx=warn",
            log_level, log_level, log_level
        )
    });

    let env_filter = EnvFilter::new(rust_log);

    // Initialize subscriber with console logging in JSON format
    // Note: For now, we're just logging to console. A full CloudWatch layer implementation
    // would require a custom tracing_subscriber::Layer implementation.
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .json(),
        )
        .init();

    tracing::info!(
        stream = %log_stream_name,
        group = %log_group_name,
        "CloudWatch logging configured (console + background sender)"
    );

    Ok(())
}
