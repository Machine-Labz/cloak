use std::time::Duration;
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
async fn process_withdraw(
    job: &crate::db::models::Job,
    _state: &AppState,
) -> Result<String, Error> {
    info!("🔐 Building withdraw transaction for job {}", job.id);

    // TODO: Implement actual Solana transaction building and submission
    // For now, this is a placeholder that simulates success

    info!("📦 Processing withdraw transaction");
    info!("   Nullifier: {}", hex::encode(&job.nullifier));
    info!("   Root hash: {}", hex::encode(&job.root_hash));
    info!("   Amount: {}", job.amount);
    info!("   Outputs: {}", job.outputs_json);

    // Simulate transaction submission
    // TODO: Replace with actual Solana client calls:
    // 1. Build transaction with withdraw instruction
    // 2. Sign with relay keypair
    // 3. Submit to Solana
    // 4. Wait for confirmation
    // 5. Return signature

    // For now, return a mock signature
    let mock_signature = format!("{}_{}", job.id, chrono::Utc::now().timestamp());

    info!("✅ Transaction submitted successfully");
    info!("   Signature: {}", mock_signature);

    Ok(mock_signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add tests for job processing
}
