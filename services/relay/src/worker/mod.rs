pub mod processor;

use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::AppState;

pub struct Worker {
    state: AppState,
    poll_interval: Duration,
    max_concurrent_jobs: usize,
}

impl Worker {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            poll_interval: Duration::from_secs(1),
            max_concurrent_jobs: 10,
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn with_max_concurrent_jobs(mut self, max: usize) -> Self {
        self.max_concurrent_jobs = max;
        self
    }

    /// Start the worker loop
    pub async fn run(self) {
        info!("🚀 Worker started");
        info!("   Poll interval: {:?}", self.poll_interval);
        info!("   Max concurrent jobs: {}", self.max_concurrent_jobs);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent_jobs));

        loop {
            // Try to dequeue a job
            match self.state.queue.dequeue(self.poll_interval).await {
                Ok(Some(job_message)) => {
                    info!("📦 Dequeued job: {}", job_message.job_id);

                    // Spawn a task to process the job
                    let state = self.state.clone();
                    let semaphore = Arc::clone(&semaphore);

                    tokio::spawn(async move {
                        // Acquire semaphore permit
                        let _permit = semaphore.acquire().await.unwrap();

                        // Process the job
                        if let Err(e) = processor::process_job(job_message, state).await {
                            error!("❌ Failed to process job: {}", e);
                        }
                    });
                }
                Ok(None) => {
                    // No jobs available, continue polling
                    debug!("No jobs in queue, continuing to poll...");
                }
                Err(e) => {
                    error!("❌ Error dequeueing job: {}", e);
                    // Sleep a bit longer on error to avoid hammering the queue
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}
