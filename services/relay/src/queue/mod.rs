pub mod processor;
pub mod redis_queue;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMessage {
    pub job_id: Uuid,
    pub request_id: Uuid,
    pub priority: u8, // 0 = highest, 255 = lowest
    pub retry_count: u32,
    pub created_at: i64, // Unix timestamp
}

impl JobMessage {
    pub fn new(job_id: Uuid, request_id: Uuid) -> Self {
        Self {
            job_id,
            request_id,
            priority: 128, // Default priority
            retry_count: 0,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, message: JobMessage) -> Result<(), Error>;
    async fn dequeue(&self, timeout: Duration) -> Result<Option<JobMessage>, Error>;
    async fn requeue_with_delay(&self, message: JobMessage, delay: Duration) -> Result<(), Error>;
    async fn dead_letter(&self, message: JobMessage, reason: String) -> Result<(), Error>;
    async fn queue_size(&self) -> Result<u64, Error>;
    async fn health_check(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub max_retries: u32,
    pub retry_delay_base: Duration,
    pub retry_delay_max: Duration,
    pub processing_timeout: Duration,
    pub dead_letter_ttl: Duration,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_base: Duration::from_secs(1),
            retry_delay_max: Duration::from_secs(300), // 5 minutes
            processing_timeout: Duration::from_secs(300), // 5 minutes
            dead_letter_ttl: Duration::from_secs(86400), // 24 hours
        }
    }
}

impl QueueConfig {
    pub fn calculate_retry_delay(&self, retry_count: u32) -> Duration {
        // Exponential backoff with jitter
        let base_delay = self.retry_delay_base.as_secs() as f64;
        let exponential_delay = base_delay * 2_f64.powi(retry_count as i32);
        let max_delay = self.retry_delay_max.as_secs() as f64;
        
        // Add some jitter (±20%)
        let jitter = fastrand::f64() * 0.4 - 0.2; // -0.2 to +0.2
        let jittered_delay = exponential_delay * (1.0 + jitter);
        
        let final_delay = jittered_delay.min(max_delay).max(base_delay);
        Duration::from_secs_f64(final_delay)
    }
} 