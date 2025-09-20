use async_trait::async_trait;
use redis::{AsyncCommands, RedisResult};
use serde_json;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::{JobMessage, JobQueue, QueueConfig};
use crate::error::Error;

const QUEUE_KEY: &str = "relay:queue:jobs";
const PROCESSING_KEY: &str = "relay:queue:processing";
const DEAD_LETTER_KEY: &str = "relay:queue:dead_letter";
const RETRY_QUEUE_KEY: &str = "relay:queue:retry";

pub struct RedisJobQueue {
    connection: redis::aio::ConnectionManager,
    config: QueueConfig,
}

impl RedisJobQueue {
    pub async fn new(redis_url: &str, config: QueueConfig) -> Result<Self, Error> {
        info!("Connecting to Redis: {}", mask_redis_url(redis_url));
        
        let client = redis::Client::open(redis_url)
            .map_err(|e| Error::InternalServerError(format!("Failed to create Redis client: {}", e)))?;
        
        let connection = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to connect to Redis: {}", e)))?;

        info!("Redis connection established");
        Ok(Self { connection, config })
    }

    async fn lua_dequeue_script(&self) -> RedisResult<String> {
        // Lua script for atomic dequeue operation
        let script = r#"
            local queue_key = KEYS[1]
            local processing_key = KEYS[2]
            local timeout = tonumber(ARGV[1])
            
            -- Try to pop from main queue
            local result = redis.call('BLPOP', queue_key, timeout)
            if result then
                local job_data = result[2]
                local job = cjson.decode(job_data)
                
                -- Add to processing set with timestamp
                local processing_item = {
                    job = job,
                    started_at = redis.call('TIME')[1]
                }
                redis.call('ZADD', processing_key, redis.call('TIME')[1], cjson.encode(processing_item))
                
                return job_data
            end
            
            return nil
        "#;

        self.connection.clone().query_async(redis::Script::new(script)).await
    }
}

#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn enqueue(&self, message: JobMessage) -> Result<(), Error> {
        let mut conn = self.connection.clone();
        
        let serialized = serde_json::to_string(&message)
            .map_err(|e| Error::InternalServerError(format!("Failed to serialize job: {}", e)))?;

        // Use priority queue (sorted set) for better ordering
        let score = calculate_priority_score(&message);
        
        let _: () = conn
            .zadd(QUEUE_KEY, serialized, score)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to enqueue job: {}", e)))?;

        debug!("Job enqueued: {} with score {}", message.request_id, score);
        Ok(())
    }

    async fn dequeue(&self, timeout: Duration) -> Result<Option<JobMessage>, Error> {
        let mut conn = self.connection.clone();
        let timeout_secs = timeout.as_secs().max(1) as usize;

        // Get highest priority job (lowest score)
        let result: Option<Vec<String>> = conn
            .bzpopmin(QUEUE_KEY, timeout_secs)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to dequeue job: {}", e)))?;

        if let Some(job_data) = result {
            if job_data.len() >= 2 {
                let job_json = &job_data[1]; // [queue_name, value, score]
                let message: JobMessage = serde_json::from_str(job_json)
                    .map_err(|e| Error::InternalServerError(format!("Failed to deserialize job: {}", e)))?;

                // Add to processing set
                let processing_data = serde_json::json!({
                    "job": message,
                    "started_at": chrono::Utc::now().timestamp()
                });

                let _: () = conn
                    .zadd(PROCESSING_KEY, serde_json::to_string(&processing_data).unwrap(), chrono::Utc::now().timestamp())
                    .await
                    .map_err(|e| warn!("Failed to add job to processing set: {}", e))
                    .unwrap_or(());

                debug!("Job dequeued: {}", message.request_id);
                return Ok(Some(message));
            }
        }

        Ok(None)
    }

    async fn requeue_with_delay(&self, mut message: JobMessage, delay: Duration) -> Result<(), Error> {
        let mut conn = self.connection.clone();
        
        message.increment_retry();

        if message.retry_count > self.config.max_retries {
            return self.dead_letter(message, "Max retries exceeded".to_string()).await;
        }

        let scheduled_time = chrono::Utc::now().timestamp() + delay.as_secs() as i64;
        let serialized = serde_json::to_string(&message)
            .map_err(|e| Error::InternalServerError(format!("Failed to serialize job for retry: {}", e)))?;

        // Add to retry queue with scheduled time as score
        let _: () = conn
            .zadd(RETRY_QUEUE_KEY, serialized, scheduled_time)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to schedule retry: {}", e)))?;

        debug!("Job scheduled for retry: {} at {}", message.request_id, scheduled_time);
        Ok(())
    }

    async fn dead_letter(&self, message: JobMessage, reason: String) -> Result<(), Error> {
        let mut conn = self.connection.clone();
        
        let dead_letter_data = serde_json::json!({
            "job": message,
            "reason": reason,
            "dead_lettered_at": chrono::Utc::now().timestamp()
        });

        let serialized = serde_json::to_string(&dead_letter_data)
            .map_err(|e| Error::InternalServerError(format!("Failed to serialize dead letter: {}", e)))?;

        let expiry = chrono::Utc::now().timestamp() + self.config.dead_letter_ttl.as_secs() as i64;

        let _: () = conn
            .zadd(DEAD_LETTER_KEY, serialized, expiry)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to add to dead letter queue: {}", e)))?;

        error!("Job dead lettered: {} - {}", message.request_id, reason);
        Ok(())
    }

    async fn queue_size(&self) -> Result<u64, Error> {
        let mut conn = self.connection.clone();
        
        let size: u64 = conn
            .zcard(QUEUE_KEY)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to get queue size: {}", e)))?;

        Ok(size)
    }

    async fn health_check(&self) -> Result<(), Error> {
        let mut conn = self.connection.clone();
        
        let _: String = conn
            .ping()
            .await
            .map_err(|e| Error::InternalServerError(format!("Redis health check failed: {}", e)))?;

        Ok(())
    }
}

impl RedisJobQueue {
    /// Process retry queue - move ready jobs back to main queue
    pub async fn process_retry_queue(&self) -> Result<u32, Error> {
        let mut conn = self.connection.clone();
        let now = chrono::Utc::now().timestamp();
        
        // Get jobs ready for retry
        let ready_jobs: Vec<String> = conn
            .zrangebyscore_limit(RETRY_QUEUE_KEY, 0, now, 0, 100)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to get retry jobs: {}", e)))?;

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
        let mut conn = self.connection.clone();
        let stale_threshold = chrono::Utc::now().timestamp() - self.config.processing_timeout.as_secs() as i64;
        
        // Get stale processing jobs
        let stale_jobs: Vec<String> = conn
            .zrangebyscore_limit(PROCESSING_KEY, 0, stale_threshold, 0, 100)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to get stale jobs: {}", e)))?;

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
        let mut conn = self.connection.clone();
        
        // Find and remove the job from processing queue
        let processing_jobs: Vec<String> = conn
            .zrange(PROCESSING_KEY, 0, -1)
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to get processing jobs: {}", e)))?;

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

fn calculate_priority_score(message: &JobMessage) -> f64 {
    // Lower score = higher priority
    // Combine priority with age for FIFO within same priority
    let priority_component = message.priority as f64 * 1000000.0; // Large multiplier for priority
    let age_component = message.created_at as f64; // Older jobs get processed first
    priority_component + age_component
}

fn mask_redis_url(url: &str) -> String {
    if url.contains('@') {
        if let Some(at_pos) = url.find('@') {
            format!("redis://***:***@{}", &url[at_pos + 1..])
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_calculate_priority_score() {
        let job1 = JobMessage::new(Uuid::new_v4(), Uuid::new_v4()).with_priority(0); // High priority
        let job2 = JobMessage::new(Uuid::new_v4(), Uuid::new_v4()).with_priority(255); // Low priority
        
        let score1 = calculate_priority_score(&job1);
        let score2 = calculate_priority_score(&job2);
        
        assert!(score1 < score2); // Lower score = higher priority
    }

    #[test]
    fn test_mask_redis_url() {
        let url = "redis://user:pass@localhost:6379";
        let masked = mask_redis_url(url);
        assert_eq!(masked, "redis://***:***@localhost:6379");
        
        let url_no_auth = "redis://localhost:6379";
        let masked_no_auth = mask_redis_url(url_no_auth);
        assert_eq!(masked_no_auth, "redis://localhost:6379");
    }
} 