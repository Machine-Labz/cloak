use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::types::InputLogEvent;
use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

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
    // Try to create log group first (ignore error if it already exists)
    if let Err(e) = client
        .create_log_group()
        .log_group_name(&log_group_name)
        .send()
        .await
    {
        // Only log if it's not an "already exists" error
        if !e.to_string().contains("ResourceAlreadyExistsException") {
            eprintln!("Warning: Could not create log group '{}': {:?}", log_group_name, e);
        }
    }

    // Try to create log stream (ignore error if it already exists)
    if let Err(e) = client
        .create_log_stream()
        .log_group_name(&log_group_name)
        .log_stream_name(&log_stream_name)
        .send()
        .await
    {
        // Only log if it's not an "already exists" error
        if !e.to_string().contains("ResourceAlreadyExistsException") {
            eprintln!("Warning: Could not create log stream '{}': {:?}", log_stream_name, e);
        }
    }

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

/// Custom tracing layer that sends logs to CloudWatch
struct CloudWatchLayer {
    tx: mpsc::UnboundedSender<String>,
}

impl CloudWatchLayer {
    fn new(tx: mpsc::UnboundedSender<String>) -> Self {
        Self { tx }
    }
}

impl<S> Layer<S> for CloudWatchLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Format the event as a string
        let mut message = String::new();

        // Extract level
        let level = event.metadata().level();
        message.push_str(&format!("[{}] ", level));

        // Extract target
        let target = event.metadata().target();
        message.push_str(&format!("{}: ", target));

        // Extract fields using a visitor
        struct MessageVisitor<'a>(&'a mut String);

        impl<'a> tracing::field::Visit for MessageVisitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0.push_str(&format!("{:?}", value));
                } else {
                    self.0.push_str(&format!(" {}={:?}", field.name(), value));
                }
            }
        }

        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        // Send to CloudWatch (ignore send errors as we don't want logging to fail the app)
        let _ = self.tx.send(message);
    }
}

/// Verify CloudWatch connectivity by attempting to describe the log group
async fn verify_cloudwatch_connectivity(
    client: &CloudWatchLogsClient,
    log_group_name: &str,
) -> Result<bool> {
    match client
        .describe_log_groups()
        .log_group_name_prefix(log_group_name)
        .send()
        .await
    {
        Ok(response) => {
            let group_exists = response
                .log_groups()
                .iter()
                .any(|g| g.log_group_name() == Some(log_group_name));

            if group_exists {
                Ok(true)
            } else {
                eprintln!(
                    "⚠️  CloudWatch log group '{}' does not exist. It will be created automatically when first log is sent.",
                    log_group_name
                );
                Ok(false)
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to CloudWatch");
            eprintln!("   Full error: {:#?}", e);
            eprintln!();
            eprintln!("   Common causes:");
            eprintln!("   1. Invalid AWS credentials (check AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY)");
            eprintln!("   2. Incorrect AWS region (current region: set in config, log group: {})", log_group_name);
            eprintln!("   3. Missing IAM permissions: logs:DescribeLogGroups, logs:CreateLogGroup, logs:CreateLogStream, logs:PutLogEvents");
            eprintln!("   4. Network connectivity issues");
            eprintln!("   5. AWS service temporarily unavailable");
            Err(anyhow::anyhow!("CloudWatch connectivity check failed: {:?}", e))
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

    // Verify CloudWatch connectivity
    eprintln!("🔍 Verifying CloudWatch connectivity...");
    match verify_cloudwatch_connectivity(&cloudwatch_client, log_group_name).await {
        Ok(true) => eprintln!("✅ CloudWatch connected successfully - log group '{}' exists", log_group_name),
        Ok(false) => eprintln!("✅ CloudWatch connected - log group will be auto-created"),
        Err(e) => {
            eprintln!("⚠️  CloudWatch connectivity check failed, continuing with console-only logging");
            eprintln!("   Error: {}", e);
            // Fall back to console-only logging
            let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
                format!(
                    "{},indexer={},cloak_indexer={},sqlx=warn",
                    log_level, log_level, log_level
                )
            });
            let env_filter = EnvFilter::new(rust_log);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_span_events(FmtSpan::CLOSE)
                        .json(),
                )
                .init();
            return Ok(());
        }
    }

    // Create channel for sending logs
    let (tx, rx) = mpsc::unbounded_channel();

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

    // Initialize subscriber with both console logging and CloudWatch layer
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .json(),
        )
        .with(CloudWatchLayer::new(tx))
        .init();

    tracing::info!(
        stream = %log_stream_name,
        group = %log_group_name,
        region = %aws_region,
        "CloudWatch logging initialized successfully"
    );

    Ok(())
}
