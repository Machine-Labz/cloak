use blake3::Hasher;
use bs58;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::db::models::JobStatus;
use crate::db::repository::{JobRepository, NullifierRepository};
use crate::error::Error;
use crate::queue::JobMessage;
use crate::AppState;

/// Process a single job from the queue
pub async fn process_job(job_message: JobMessage, state: AppState) -> Result<(), Error> {
    let job_id = job_message.job_id;

    info!("🔄 Processing job: {}", job_id);
    info!("   Request ID: {}", job_message.request_id);
    info!("   Retry count: {}", job_message.retry_count);

    // Fetch the job from database
    let job = match state.job_repo.get_job_by_id(job_id).await {
        Ok(Some(job)) => job,
        Ok(None) => {
            warn!(
                "⚠️  Job {} not found in database (stale queue entry), skipping",
                job_id
            );
            // This is likely an old job from a previous run - just skip it
            return Ok(());
        }
        Err(e) => {
            error!("❌ Failed to fetch job {}: {}", job_id, e);
            return Err(e);
        }
    };

    // Check if job is already completed or failed
    if job.status == JobStatus::Completed {
        info!("✅ Job {} already completed, skipping", job_id);
        return Ok(());
    }

    if job.status == JobStatus::Failed {
        warn!("⚠️  Job {} already marked as failed, skipping", job_id);
        return Ok(());
    }

    // Update status to processing
    if let Err(e) = state
        .job_repo
        .update_job_status(job_id, JobStatus::Processing)
        .await
    {
        error!(
            "❌ Failed to update job {} status to processing: {}",
            job_id, e
        );
        return Err(e);
    }

    info!("📝 Job {} status updated to processing", job_id);

    // Process the withdraw transaction
    // If proof is missing, requeue with backoff without counting as a failure.
    if job.proof_bytes.is_empty() {
        warn!(
            "Job {} missing proof bytes; requeueing for later processing",
            job_id
        );
        let retry_delay = Duration::from_secs(60);
        if let Err(e) = state
            .queue
            .requeue_with_delay(job_message.clone(), retry_delay)
            .await
        {
            error!("Failed to requeue job {}: {}", job_id, e);
        }
        let _ = state
            .job_repo
            .update_job_status(job_id, JobStatus::Queued)
            .await;
        return Ok(());
    }

    // Optional preflights when public_inputs are canonical 104B
    if job.public_inputs.len() == 104 {
        if let Some(arr) = job.outputs_json.as_array() {
            if arr.len() == 1 {
                if let (Some(recipient), Some(amount)) = (
                    arr[0].get("recipient").and_then(|v| v.as_str()),
                    arr[0].get("amount").and_then(|v| v.as_u64()),
                ) {
                    // outputs_hash preflight
                    if let Ok(addr_bytes) = bs58::decode(recipient).into_vec() {
                        if addr_bytes.len() == 32 {
                            let mut hasher = Hasher::new();
                            hasher.update(&addr_bytes);
                            hasher.update(&amount.to_le_bytes());
                            let calc = hasher.finalize();
                            if calc.as_bytes() != job.outputs_hash.as_slice() {
                                warn!("Job {} outputs_hash != recomputed; continuing but this will fail on-chain", job_id);
                            }
                        }
                    }

                    // conservation preflight
                    let amt = u64::from_le_bytes(job.public_inputs[96..104].try_into().unwrap());
                    let fee = crate::planner::calculate_fee(amt);
                    if amount + fee != amt {
                        warn!("Job {} conservation failed preflight; continuing but likely to fail on-chain", job_id);
                    }
                }
            }
        }
    }

    // Jitter submission slightly to reduce linkability
    let delay = crate::planner::jitter_delay(Instant::now());
    if delay > Duration::from_millis(0) {
        tokio::time::sleep(delay).await;
    }

    match process_withdraw(&job, &state).await {
        Ok(signature) => {
            info!("✅ Job {} completed successfully", job_id);
            info!("   Transaction signature: {}", signature);

            // Update job status to completed
            if let Err(e) = state
                .job_repo
                .update_job_completed(job_id, signature.clone(), signature.clone())
                .await
            {
                error!("❌ Failed to mark job {} as completed: {}", job_id, e);
                return Err(e);
            }

            // Store nullifier to prevent double-spending
            if let Err(e) = state
                .nullifier_repo
                .create_nullifier(job.nullifier.clone(), job_id)
                .await
            {
                error!("⚠️  Failed to store nullifier for job {}: {}", job_id, e);
                // Don't fail the job since the transaction succeeded
            }

            Ok(())
        }
        Err(e) => {
            error!("❌ Job {} failed: {}", job_id, e);

            // Check if we should retry
            let max_retries = 3; // TODO: Get from config
            if job_message.retry_count < max_retries {
                warn!(
                    "🔄 Retrying job {} (attempt {}/{})",
                    job_id,
                    job_message.retry_count + 1,
                    max_retries
                );

                // Requeue with exponential backoff
                let retry_delay = Duration::from_secs(30 * 2_u64.pow(job_message.retry_count));
                let mut retry_message = job_message.clone();
                retry_message.retry_count += 1;

                if let Err(requeue_err) = state
                    .queue
                    .requeue_with_delay(retry_message, retry_delay)
                    .await
                {
                    error!("❌ Failed to requeue job {}: {}", job_id, requeue_err);
                }

                // Update status to queued (for retry)
                if let Err(status_err) = state
                    .job_repo
                    .update_job_status(job_id, JobStatus::Queued)
                    .await
                {
                    error!(
                        "❌ Failed to update job {} status for retry: {}",
                        job_id, status_err
                    );
                }
            } else {
                error!("💀 Job {} exceeded max retries, marking as failed", job_id);

                // Mark job as failed
                if let Err(fail_err) = state
                    .job_repo
                    .update_job_failed(job_id, format!("Max retries exceeded: {}", e))
                    .await
                {
                    error!("❌ Failed to mark job {} as failed: {}", job_id, fail_err);
                }

                // Move to dead letter queue
                if let Err(dlq_err) = state
                    .queue
                    .dead_letter(job_message, format!("Max retries exceeded: {}", e))
                    .await
                {
                    error!(
                        "❌ Failed to move job {} to dead letter queue: {}",
                        job_id, dlq_err
                    );
                }
            }

            Err(e)
        }
    }
}

/// Process a withdraw transaction (placeholder - needs actual Solana integration)
async fn process_withdraw(job: &crate::db::models::Job, state: &AppState) -> Result<String, Error> {
    info!(
        "🔐 Building & submitting withdraw transaction for job {}",
        job.id
    );
    let sig = state.solana.submit_withdraw(job).await?;
    info!("✅ Transaction submitted: {}", sig);
    Ok(sig.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add tests for job processing
}
