use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::db::models::Job;
use crate::db::repository::JobRepository;
use crate::AppState;

/// Configuration for withdrawal window timing
#[derive(Clone, Debug)]
pub struct WindowConfig {
    /// Which slot endings trigger a window (e.g., [0, 5] means slots ending in 0 or 5)
    pub slot_patterns: Vec<u8>,
    /// Minimum jobs to accumulate before processing (optional optimization)
    pub min_batch_size: Option<usize>,
    /// Maximum jobs to hold before forcing a window (safety limit)
    pub max_batch_size: usize,
    /// How often to check the current slot (in seconds)
    pub poll_interval_secs: u64,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            slot_patterns: vec![0, 5], // Process every 5 slots (~2.5s)
            min_batch_size: None,      // No minimum
            max_batch_size: 50,        // Safety limit
            poll_interval_secs: 1,     // Check every second
        }
    }
}

impl WindowConfig {
    /// Check if a slot matches the configured pattern
    pub fn is_window_slot(&self, slot: u64) -> bool {
        let last_digit = (slot % 10) as u8;
        self.slot_patterns.contains(&last_digit)
    }
}

/// Accumulates jobs and processes them in time windows based on Solana slot numbers
pub struct WindowScheduler {
    state: AppState,
    config: WindowConfig,
    job_buffer: Arc<Mutex<VecDeque<Job>>>,
    last_processed_slot: Arc<Mutex<u64>>,
}

impl WindowScheduler {
    pub fn new(state: AppState, config: WindowConfig) -> Self {
        Self {
            state,
            config,
            job_buffer: Arc::new(Mutex::new(VecDeque::new())),
            last_processed_slot: Arc::new(Mutex::new(0)),
        }
    }

    /// Start the window scheduler loop
    pub async fn run(self: Arc<Self>) {
        info!("🚀 Window Scheduler started");
        info!("   Slot patterns: {:?}", self.config.slot_patterns);
        info!("   Min batch size: {:?}", self.config.min_batch_size);
        info!("   Max batch size: {}", self.config.max_batch_size);
        info!("   Poll interval: {}s", self.config.poll_interval_secs);

        let poll_interval = Duration::from_secs(self.config.poll_interval_secs);

        // Spawn job collection task
        let collector = Arc::clone(&self);
        tokio::spawn(async move {
            collector.collect_jobs_loop().await;
        });

        // Main window processing loop
        loop {
            if let Err(e) = self.check_and_process_window().await {
                warn!("⚠️  Window processing error: {}", e);
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Continuously collect queued jobs into buffer
    async fn collect_jobs_loop(&self) {
        loop {
            // Check current buffer size
            let buffer_size = {
                let buffer = self.job_buffer.lock().await;
                buffer.len()
            };

            // Don't overfill the buffer
            if buffer_size >= self.config.max_batch_size {
                debug!(
                    "Buffer full ({} jobs), waiting for window to process",
                    buffer_size
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            // Fetch jobs from database
            match self.state.job_repo.get_queued_jobs(10).await {
                Ok(jobs) => {
                    if !jobs.is_empty() {
                        let count = jobs.len();
                        let mut buffer = self.job_buffer.lock().await;
                        let mut added = 0;

                        for job in jobs {
                            // Check if job is already in buffer to avoid duplicates
                            let already_buffered = buffer.iter().any(|j| j.id == job.id);
                            if !already_buffered {
                                buffer.push_back(job);
                                added += 1;
                            } else {
                                debug!("Job {} already in buffer, skipping", job.id);
                            }
                        }

                        if added > 0 {
                            debug!(
                                "📦 Collected {} new jobs into buffer (now {} total)",
                                added,
                                buffer.len()
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to fetch queued jobs: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Check if current slot matches window pattern and process if ready
    async fn check_and_process_window(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get current Solana slot
        let current_slot = self.state.solana.get_slot().await?;

        // Check if this is a window slot
        if !self.config.is_window_slot(current_slot) {
            debug!("Slot {} - not a window slot, waiting...", current_slot);
            return Ok(());
        }

        // Check if we already processed this slot
        let mut last_slot = self.last_processed_slot.lock().await;
        if *last_slot == current_slot {
            debug!("Slot {} - already processed this window", current_slot);
            return Ok(());
        }

        // Get buffered jobs
        let jobs_to_process = {
            let mut buffer = self.job_buffer.lock().await;
            let count = buffer.len();

            // Check minimum batch size requirement
            if let Some(min_size) = self.config.min_batch_size {
                if count < min_size {
                    debug!(
                        "Slot {} - only {} jobs buffered, waiting for {} (min batch)",
                        current_slot, count, min_size
                    );
                    return Ok(());
                }
            }

            // No jobs to process
            if count == 0 {
                debug!("Slot {} - no jobs in buffer", current_slot);
                return Ok(());
            }

            // Take all jobs from buffer
            let jobs: Vec<Job> = buffer.drain(..).collect();
            jobs
        };

        // Update last processed slot
        *last_slot = current_slot;
        drop(last_slot); // Release lock

        // Process the batch
        info!(
            "🪟 Window opened at slot {} - processing {} jobs",
            current_slot,
            jobs_to_process.len()
        );

        self.process_batch(jobs_to_process).await;

        Ok(())
    }

    /// Process a batch of jobs
    async fn process_batch(&self, jobs: Vec<Job>) {
        let batch_size = jobs.len();
        let start_time = std::time::Instant::now();

        // Process jobs concurrently with semaphore to limit parallelism
        let semaphore = Arc::new(tokio::sync::Semaphore::new(10)); // Max 10 concurrent
        let mut handles = Vec::new();

        for job in jobs {
            let state = self.state.clone();
            let semaphore = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                if let Err(e) = super::processor::process_job_direct(job.clone(), state).await {
                    warn!("❌ Failed to process job {}: {}", job.id, e);
                }
            });

            handles.push(handle);
        }

        // Wait for all jobs to complete
        for handle in handles {
            let _ = handle.await;
        }

        let duration = start_time.elapsed();
        info!(
            "✅ Batch complete: {} jobs processed in {:?}",
            batch_size, duration
        );
    }
}
