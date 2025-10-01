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
            priority: 100, // Default priority
            retry_count: 0,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }
}

#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub max_retries: u32,
    pub base_retry_delay: Duration,
    pub max_retry_delay: Duration,
    pub processing_timeout: Duration,
    pub cleanup_interval: Duration,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_retry_delay: Duration::from_secs(30),
            max_retry_delay: Duration::from_secs(300), // 5 minutes
            processing_timeout: Duration::from_secs(600), // 10 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

impl QueueConfig {
    pub fn calculate_retry_delay(&self, retry_count: u32) -> Duration {
        let delay = self.base_retry_delay.as_secs() * 2_u64.pow(retry_count);
        Duration::from_secs(delay.min(self.max_retry_delay.as_secs()))
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
