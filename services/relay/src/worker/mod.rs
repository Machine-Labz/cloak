pub mod processor;
pub mod window_scheduler;

use std::{sync::Arc, time::Duration};

use tracing::{debug, error, info};

use crate::{db::repository::JobRepository, AppState};

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
            // Poll database for queued jobs
            match self.state.job_repo.get_queued_jobs(1).await {
                Ok(jobs) => {
                    if let Some(job) = jobs.into_iter().next() {
                        info!("📦 Found queued job: {}", job.id);

                        // Spawn a task to process the job
                        let state = self.state.clone();
                        let semaphore = Arc::clone(&semaphore);

                        tokio::spawn(async move {
                            // Acquire semaphore permit
                            let _permit = semaphore.acquire().await.unwrap();

                            // Process the job
                            if let Err(e) = processor::process_job_direct(job, state).await {
                                error!("❌ Failed to process job: {}", e);
                            }
                        });
                    } else {
                        // No jobs available, continue polling
                        debug!("No jobs in queue, continuing to poll...");
                    }
                }
                Err(e) => {
                    error!("❌ Error polling for jobs: {}", e);
                    // Sleep a bit longer on error to avoid hammering the database
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
