use anyhow::{anyhow, Result};
use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use tracing::info;

pub async fn fetch_database_url() -> Result<String> {
    let secret_name = get_env_var("DATABASE_URL_SECRET_NAME", "");
    
    // Directly use DATABASE_URL for local development/testing - LOCAL DB (for @Arthur and @marcelofeitoza)
    if secret_name.is_empty() {
        let fallback_url = get_env_var("DATABASE_URL", "");
        if !fallback_url.is_empty() {
            info!("Using DATABASE_URL from environment (DATABASE_URL_SECRET_NAME not set)");
            return Ok(fallback_url);
        }
        return Err(anyhow::anyhow!("Either DATABASE_URL_SECRET_NAME or DATABASE_URL environment variable is required"));
    }

    info!("Fetching database URL from AWS Secrets Manager: {}", secret_name);

    // Get AWS credentials from environment variables
    let aws_access_key_id = get_env_var("AWS_ACCESS_KEY_ID", "");
    let aws_secret_access_key = get_env_var("AWS_SECRET_ACCESS_KEY", "");
    let aws_region = get_env_var("AWS_REGION", "us-east-1");

    if aws_access_key_id.is_empty() || aws_secret_access_key.is_empty() {
        return Err(anyhow!(
            "AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY must be set when using DATABASE_URL_SECRET_NAME"
        ));
    }

    // Configure AWS credentials
    let credentials = aws_sdk_secretsmanager::config::Credentials::new(
        &aws_access_key_id,
        &aws_secret_access_key,
        None,
        None,
        "cloak-indexer",
    );

    // Build AWS config with credentials and region
    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(aws_config::Region::new(aws_region))
        .load()
        .await;

    let client = SecretsManagerClient::new(&config);

    // Get secret value
    let response = client
        .get_secret_value()
        .secret_id(&secret_name)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch secret from AWS Secrets Manager: {}", e))?;

    let secret_string = response
        .secret_string()
        .ok_or_else(|| anyhow::anyhow!("Secret value is empty or not a string"))?;

    info!("Successfully fetched database URL from AWS Secrets Manager");
    
    Ok(secret_string.to_string())
}

fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

