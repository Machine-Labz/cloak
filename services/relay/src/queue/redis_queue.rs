use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use serde_json;
use std::time::Duration;
use tracing::{debug, warn};

use super::{JobMessage, JobQueue, QueueConfig};
use crate::error::Error;

const MAIN_QUEUE_KEY: &str = "cloak:jobs:main";
const RETRY_QUEUE_KEY: &str = "cloak:jobs:retry";
const PROCESSING_KEY: &str = "cloak:jobs:processing";
const DEAD_LETTER_KEY: &str = "cloak:jobs:dead";

pub struct RedisJobQueue {
    client: Client,
    config: QueueConfig,
}

impl RedisJobQueue {
    pub async fn new(redis_url: &str, config: QueueConfig) -> Result<Self, Error> {
        let client = Client::open(redis_url)
            .map_err(|e| Error::RedisError(format!("Failed to create Redis client: {}", e)))?;
        
        // Test connection
        let mut conn = client.get_async_connection().await
            .map_err(|e| Error::RedisError(format!("Failed to connect to Redis: {}", e)))?;
        
        // Test the connection with a simple command
        let _: String = redis::cmd("PING").query_async(&mut conn).await
            .map_err(|e| Error::RedisError(format!("Redis ping failed: {}", e)))?;
        
        debug!("Redis connection established");
        
        Ok(Self { client, config })
    }

    async fn get_connection(&self) -> Result<redis::aio::Connection, Error> {
        self.client.get_async_connection().await
            .map_err(|e| Error::RedisError(format!("Failed to get Redis connection: {}", e)))
    }
}

#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn enqueue(&self, message: JobMessage) -> Result<(), Error> {
        let mut conn = self.get_connection().await?;
        let job_json = serde_json::to_string(&message)
            .map_err(|e| Error::RedisError(format!("Failed to serialize job message: {}", e)))?;
        
        // Use priority as score for sorted set (lower score = higher priority)
        let score = message.priority as f64;
        
        let _: () = conn.zadd(MAIN_QUEUE_KEY, job_json, score).await
            .map_err(|e| Error::RedisError(format!("Failed to enqueue job: {}", e)))?;
        
        debug!("Job enqueued: {} (priority: {})", message.job_id, message.priority);
        Ok(())
    }

    async fn dequeue(&self, timeout: Duration) -> Result<Option<JobMessage>, Error> {
        let mut conn = self.get_connection().await?;
        
        // Use ZPOPMIN instead of BZPOPMIN for simplicity
        let result: Option<(String, f64)> = conn.zpopmin(MAIN_QUEUE_KEY, 1).await
            .map_err(|e| Error::RedisError(format!("Failed to dequeue job: {}", e)))?;
        
        if let Some((job_json, _score)) = result {
            let message: JobMessage = serde_json::from_str(&job_json)
                .map_err(|e| Error::RedisError(format!("Failed to deserialize job message: {}", e)))?;
            
            // Move to processing queue with timestamp
            let processing_data = serde_json::json!({
                "job": message,
                "started_at": chrono::Utc::now().timestamp()
            });
            let processing_json = serde_json::to_string(&processing_data)
                .map_err(|e| Error::RedisError(format!("Failed to serialize processing data: {}", e)))?;
            
            let _: () = conn.zadd(PROCESSING_KEY, processing_json, chrono::Utc::now().timestamp()).await
                .map_err(|e| Error::RedisError(format!("Failed to add to processing queue: {}", e)))?;
            
            debug!("Job dequeued: {}", message.job_id);
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    async fn requeue_with_delay(&self, message: JobMessage, delay: Duration) -> Result<(), Error> {
        let mut conn = self.get_connection().await?;
        let retry_count = message.retry_count + 1;
        let retry_message = message.with_retry_count(retry_count);
        let job_json = serde_json::to_string(&retry_message)
            .map_err(|e| Error::RedisError(format!("Failed to serialize retry message: {}", e)))?;
        
        // Schedule for retry at current time + delay
        let retry_at = chrono::Utc::now().timestamp() + delay.as_secs() as i64;
        
        let _: () = conn.zadd(RETRY_QUEUE_KEY, job_json, retry_at).await
            .map_err(|e| Error::RedisError(format!("Failed to requeue job: {}", e)))?;
        
        debug!("Job requeued for retry: {} (attempt: {}, delay: {}s)", 
               retry_message.job_id, retry_message.retry_count, delay.as_secs());
        Ok(())
    }

    async fn dead_letter(&self, message: JobMessage, reason: String) -> Result<(), Error> {
        let mut conn = self.get_connection().await?;
        let dead_letter_data = serde_json::json!({
            "job": message,
            "reason": reason,
            "dead_lettered_at": chrono::Utc::now().timestamp()
        });
        let dead_letter_json = serde_json::to_string(&dead_letter_data)
            .map_err(|e| Error::RedisError(format!("Failed to serialize dead letter data: {}", e)))?;
        
        let _: () = conn.zadd(DEAD_LETTER_KEY, dead_letter_json, chrono::Utc::now().timestamp()).await
            .map_err(|e| Error::RedisError(format!("Failed to add to dead letter queue: {}", e)))?;
        
        warn!("Job dead lettered: {} (reason: {})", message.job_id, reason);
        Ok(())
    }

    async fn queue_size(&self) -> Result<u64, Error> {
        let mut conn = self.get_connection().await?;
        let main_size: u64 = conn.zcard(MAIN_QUEUE_KEY).await
            .map_err(|e| Error::RedisError(format!("Failed to get main queue size: {}", e)))?;
        let retry_size: u64 = conn.zcard(RETRY_QUEUE_KEY).await
            .map_err(|e| Error::RedisError(format!("Failed to get retry queue size: {}", e)))?;
        let processing_size: u64 = conn.zcard(PROCESSING_KEY).await
            .map_err(|e| Error::RedisError(format!("Failed to get processing queue size: {}", e)))?;
        
        Ok(main_size + retry_size + processing_size)
    }

    async fn health_check(&self) -> Result<(), Error> {
        let mut conn = self.get_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await
            .map_err(|e| Error::RedisError(format!("Redis health check failed: {}", e)))?;
        Ok(())
    }
}

impl RedisJobQueue {
    /// Process retry queue - move ready jobs back to main queue
    pub async fn process_retry_queue(&self) -> Result<u32, Error> {
        let mut conn = self.get_connection().await?;
        let now = chrono::Utc::now().timestamp();
        
        // Get jobs ready for retry
        let ready_jobs: Vec<String> = conn
            .zrangebyscore_limit(RETRY_QUEUE_KEY, 0, now, 0, 100)
            .await
            .map_err(|e| Error::RedisError(format!("Failed to get retry jobs: {}", e)))?;

        let mut processed = 0;
        for job_json in ready_jobs {
            if let Ok(message) = serde_json::from_str::<JobMessage>(&job_json) {
                // Remove from retry queue
                let _: () = conn.zrem(RETRY_QUEUE_KEY, &job_json).await.unwrap_or(());
                
                // Add back to main queue
                if self.enqueue(message).await.is_ok() {
                    processed += 1;
                }
            }
        }

        if processed > 0 {
            debug!("Processed {} retry jobs", processed);
        }

        Ok(processed)
    }

    /// Clean up processing queue - move stale jobs back to main queue
    pub async fn cleanup_processing_queue(&self) -> Result<u32, Error> {
        let mut conn = self.get_connection().await?;
        let stale_threshold = chrono::Utc::now().timestamp() - self.config.processing_timeout.as_secs() as i64;
        
        // Get stale processing jobs
        let stale_jobs: Vec<String> = conn
            .zrangebyscore_limit(PROCESSING_KEY, 0, stale_threshold, 0, 100)
            .await
            .map_err(|e| Error::RedisError(format!("Failed to get stale jobs: {}", e)))?;

        let mut cleaned = 0;
        for job_json in stale_jobs {
            if let Ok(processing_data) = serde_json::from_str::<serde_json::Value>(&job_json) {
                if let Ok(message) = serde_json::from_value::<JobMessage>(processing_data["job"].clone()) {
                    // Remove from processing queue
                    let _: () = conn.zrem(PROCESSING_KEY, &job_json).await.unwrap_or(());
                    
                    // Requeue with delay
                    let delay = self.config.calculate_retry_delay(message.retry_count);
                    if self.requeue_with_delay(message, delay).await.is_ok() {
                        cleaned += 1;
                    }
                }
            }
        }

        if cleaned > 0 {
            warn!("Cleaned up {} stale processing jobs", cleaned);
        }

        Ok(cleaned)
    }

    /// Mark job as completed - remove from processing queue
    pub async fn mark_completed(&self, job_id: uuid::Uuid) -> Result<(), Error> {
        let mut conn = self.get_connection().await?;
        
        // Find and remove the job from processing queue
        let processing_jobs: Vec<String> = conn
            .zrange(PROCESSING_KEY, 0, -1)
            .await
            .map_err(|e| Error::RedisError(format!("Failed to get processing jobs: {}", e)))?;

        for job_json in processing_jobs {
            if let Ok(processing_data) = serde_json::from_str::<serde_json::Value>(&job_json) {
                if let Ok(message) = serde_json::from_value::<JobMessage>(processing_data["job"].clone()) {
                    if message.job_id == job_id {
                        let _: () = conn.zrem(PROCESSING_KEY, &job_json).await.unwrap_or(());
                        debug!("Job marked as completed: {}", job_id);
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }
} 