use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{JobMessage, JobQueue, QueueConfig};
use crate::db::models::{Job, JobStatus};
use crate::db::repository::{JobRepository, NullifierRepository};
use crate::error::Error;
use crate::solana::SolanaService;
use crate::validation::ValidationService;

pub struct JobProcessor {
    queue: Arc<dyn JobQueue>,
    job_repo: Arc<dyn JobRepository>,
    nullifier_repo: Arc<dyn NullifierRepository>,
    solana_service: Arc<SolanaService>,
    validation_service: Arc<ValidationService>,
    config: QueueConfig,
    worker_id: String,
}

impl JobProcessor {
    pub fn new(
        queue: Arc<dyn JobQueue>,
        job_repo: Arc<dyn JobRepository>,
        nullifier_repo: Arc<dyn NullifierRepository>,
        solana_service: Arc<SolanaService>,
        validation_service: Arc<ValidationService>,
        config: QueueConfig,
    ) -> Self {
        let worker_id = format!("worker-{}", Uuid::new_v4());
        Self {
            queue,
            job_repo,
            nullifier_repo,
            solana_service,
            validation_service,
            config,
            worker_id,
        }
    }

    /// Start the job processor worker loop
    pub async fn start(&self) -> Result<(), Error> {
        info!("Starting job processor worker: {}", self.worker_id);

        loop {
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No job was processed, sleep briefly to avoid busy loop
                        sleep(Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    error!("Error in job processor: {}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }

            // Occasionally clean up processing queue and retry queue
            if fastrand::f32() < 0.01 { // 1% chance per iteration
                if let Err(e) = self.maintenance().await {
                    warn!("Maintenance error: {}", e);
                }
            }
        }
    }

    /// Process the next available job from the queue
    async fn process_next_job(&self) -> Result<bool, Error> {
        let message = match self.queue.dequeue(Duration::from_secs(10)).await? {
            Some(msg) => msg,
            None => return Ok(false), // No job available
        };

        info!("Processing job: {} (attempt {})", message.request_id, message.retry_count + 1);

        match self.process_job(message.clone()).await {
            Ok(()) => {
                info!("Job completed successfully: {}", message.request_id);
                Ok(true)
            }
            Err(e) => {
                error!("Job failed: {} - {}", message.request_id, e);
                self.handle_job_failure(message, e).await?;
                Ok(true)
            }
        }
    }

    /// Process a single job
    async fn process_job(&self, message: JobMessage) -> Result<(), Error> {
        // 1. Get job details from database
        let job = self.job_repo.get_job_by_id(message.job_id).await?
            .ok_or_else(|| Error::NotFound)?;

        // 2. Update job status to processing
        self.job_repo.update_job_processing(job.id, None).await?;

        // 3. Check for double spending
        if self.nullifier_repo.exists_nullifier(&job.nullifier).await? {
            let error_msg = "Nullifier already exists (double spend attempt)";
            self.job_repo.update_job_failed(job.id, error_msg.to_string()).await?;
            return Err(Error::ValidationError(error_msg.to_string()));
        }

        // 4. Validate the withdraw request
        self.validation_service.validate_withdraw_proof(&job).await
            .map_err(|e| {
                let _ = self.job_repo.update_job_failed(job.id, e.to_string());
                e
            })?;

        // 5. Submit transaction to Solana
        let tx_result = self.solana_service.submit_withdraw(&job).await
            .map_err(|e| {
                let _ = self.job_repo.update_job_failed(job.id, e.to_string());
                e
            })?;

        // 6. Record nullifier to prevent double spend
        self.nullifier_repo.create_nullifier(job.nullifier.clone(), job.id).await?;

        // 7. Update job as completed
        self.job_repo.update_job_completed(
            job.id, 
            tx_result.transaction_id.clone(), 
            tx_result.signature.clone()
        ).await?;

        // 8. Mark job as completed in queue
        // For Redis queue, we can mark the job as completed
        // This is a Redis-specific optimization
        debug!("Job processing completed: {} -> {}", message.request_id, tx_result.signature);
        Ok(())
    }

    /// Handle job failure with retry logic
    async fn handle_job_failure(&self, message: JobMessage, error: Error) -> Result<(), Error> {
        // Increment retry count in database
        self.job_repo.increment_retry_count(message.job_id).await?;

        if message.retry_count >= self.config.max_retries {
            // Max retries exceeded, move to dead letter queue
            self.queue.dead_letter(message.clone(), error.to_string()).await?;
            self.job_repo.update_job_failed(message.job_id, format!("Max retries exceeded: {}", error)).await?;
        } else {
            // Schedule retry with exponential backoff
            let delay = self.config.calculate_retry_delay(message.retry_count);
            self.queue.requeue_with_delay(message.clone(), delay).await?;
            
            // Update job status back to queued
            self.job_repo.update_job_status(message.job_id, JobStatus::Queued).await?;
        }

        Ok(())
    }

    /// Perform maintenance tasks
    async fn maintenance(&self) -> Result<(), Error> {
        // Generic maintenance tasks can be added here
        // Redis-specific maintenance would be handled by Redis queue itself
        debug!("Performing maintenance tasks");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub signature: String,
    pub block_height: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{CreateJob, JobStatus};
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock implementations for testing
    struct MockJobQueue;
    
    #[async_trait::async_trait]
    impl JobQueue for MockJobQueue {
        async fn enqueue(&self, _message: JobMessage) -> Result<(), Error> { Ok(()) }
        async fn dequeue(&self, _timeout: Duration) -> Result<Option<JobMessage>, Error> { Ok(None) }
        async fn requeue_with_delay(&self, _message: JobMessage, _delay: Duration) -> Result<(), Error> { Ok(()) }
        async fn dead_letter(&self, _message: JobMessage, _reason: String) -> Result<(), Error> { Ok(()) }
        async fn queue_size(&self) -> Result<u64, Error> { Ok(0) }
        async fn health_check(&self) -> Result<(), Error> { Ok(()) }
    }

    #[test]
    fn test_queue_config_retry_delay() {
        let config = QueueConfig::default();
        
        let delay1 = config.calculate_retry_delay(0);
        let delay2 = config.calculate_retry_delay(1);
        let delay3 = config.calculate_retry_delay(2);
        
        // Each retry should have increasing delay
        assert!(delay1 <= delay2);
        assert!(delay2 <= delay3);
        
        // Should not exceed max delay
        let delay_max = config.calculate_retry_delay(10);
        assert!(delay_max <= config.retry_delay_max);
    }
} 