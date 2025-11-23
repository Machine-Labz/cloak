use blake3::Hasher;
use bs58;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::db::models::{Job, JobStatus};
use crate::db::repository::{JobRepository, NullifierRepository};
use crate::error::Error;
use crate::AppState;

/// Process a single job directly from database
pub async fn process_job_direct(job: Job, state: AppState) -> Result<(), Error> {
    let job_id = job.id;

    info!("🔄 Processing job: {}", job_id);
    info!("   Request ID: {}", job.request_id);
    info!("   Status: {:?}", job.status);

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
                    let fee = crate::planner::calculate_fee_legacy(amt);
                    if amount + fee != amt {
                        warn!("Job {} conservation failed preflight; continuing but likely to fail on-chain", job_id);
                    }
                }
            }
        }
    }

    // Check if nullifier already exists on-chain before attempting withdraw
    tracing::info!("🔍 Checking if nullifier already exists on-chain");
    match state.solana.check_nullifier_exists(&job.nullifier).await {
        Ok(true) => {
            tracing::info!(
                "✅ Nullifier already exists on-chain - transaction was already processed"
            );
            // Mark job as completed since the transaction was already successfully processed
            if let Err(e) = state
                .job_repo
                .update_job_status(job_id, JobStatus::Completed)
                .await
            {
                error!("❌ Failed to mark job {} as completed: {}", job_id, e);
                return Err(e);
            }

            // Store nullifier to prevent double-spending (if not already in local DB)
            if let Err(e) = state
                .nullifier_repo
                .create_nullifier(job.nullifier.clone(), job_id)
                .await
            {
                // Ignore duplicate key errors - nullifier might already be in our DB
                tracing::debug!("Nullifier storage: {}", e);
            }

            return Ok(());
        }
        Ok(false) => {
            tracing::info!("✓ Nullifier not found on-chain, proceeding with withdraw");
        }
        Err(e) => {
            tracing::warn!(
                "⚠️ Failed to check nullifier on-chain: {}, proceeding anyway",
                e
            );
            // Continue with transaction - the on-chain program will reject if duplicate
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

            let error_str = e.to_string();

            // Check if the error indicates nullifier already used (0x1023)
            let nullifier_already_used = error_str.contains("0x1023")
                || error_str.contains("NullifierAlreadyUsed")
                || error_str.contains("custom program error: 0x1023");

            // Check if this is a swap job
            let is_swap_job = job.outputs_json.get("swap").is_some();

            if nullifier_already_used {
                // For swap jobs, error 0x1023 means TX1 (WithdrawSwap) succeeded
                // but we need to check if the full swap flow is complete
                if is_swap_job {
                    warn!(
                        "⚠️  Job {} - Nullifier already used (0x1023) on swap job",
                        job_id
                    );
                    warn!("   This means WithdrawSwap (TX1) succeeded but ExecuteSwap (TX2) may be pending");
                    warn!("   Checking if SwapState PDA exists...");

                    // Check if SwapState PDA exists
                    // Parse nullifier from public_inputs
                    if job.public_inputs.len() >= 64 {
                        let mut nullifier = [0u8; 32];
                        nullifier.copy_from_slice(&job.public_inputs[32..64]);

                        // Check if PDA exists
                        match state.solana.check_swap_state_exists(&nullifier).await {
                            Ok(true) => {
                                info!(
                                    "✓ Job {} - SwapState PDA exists! TX1 (WithdrawSwap) succeeded",
                                    job_id
                                );
                                info!("   TX2 (Jupiter swap + ExecuteSwap) still needs to be executed");
                                info!("   Requeueing job to resume from TX2...");

                                // Requeue the job - submit_withdraw_with_swap will now skip TX1 and do TX2 only
                                if let Err(e) = state
                                    .job_repo
                                    .update_job_status(job_id, JobStatus::Queued)
                                    .await
                                {
                                    error!("❌ Failed to requeue job {} for TX2: {}", job_id, e);
                                }

                                return Ok(());
                            }
                            Ok(false) => {
                                info!(
                                    "✅ Job {} - SwapState PDA doesn't exist, swap fully completed",
                                    job_id
                                );
                                // PDA doesn't exist = swap completed successfully
                                // Fall through to mark as completed
                            }
                            Err(e) => {
                                warn!("⚠️  Job {} - Failed to check SwapState PDA: {}", job_id, e);
                                // Can't determine status, mark as failed to avoid incorrect completion
                                if let Err(e) = state
                                    .job_repo
                                    .update_job_status(job_id, JobStatus::Failed)
                                    .await
                                {
                                    error!("❌ Failed to mark job {} as failed: {}", job_id, e);
                                }
                                return Ok(());
                            }
                        }
                    }
                }

                // For non-swap jobs or completed swaps, mark as completed
                info!(
                    "✅ Job {} - Transaction already processed, marking as completed",
                    job_id
                );

                // Mark job as completed since transaction was successful (just duplicate attempt)
                if let Err(e) = state
                    .job_repo
                    .update_job_status(job_id, JobStatus::Completed)
                    .await
                {
                    error!("❌ Failed to mark job {} as completed: {}", job_id, e);
                }

                // Store nullifier to prevent double-spending (if not already in local DB)
                if let Err(e) = state
                    .nullifier_repo
                    .create_nullifier(job.nullifier.clone(), job_id)
                    .await
                {
                    // Ignore duplicate key errors
                    tracing::debug!("Nullifier storage: {}", e);
                }

                return Ok(());
            }

            // Check for double-spend errors (0x1020) - these are always completed
            let double_spend = error_str.contains("already been processed")
                || error_str.contains("0x1020")
                || error_str.contains("DoubleSpend")
                || error_str.contains("custom program error: 0x1020");

            if double_spend {
                info!(
                    "✅ Job {} - Double spend detected, transaction was already processed",
                    job_id
                );

                if let Err(e) = state
                    .job_repo
                    .update_job_status(job_id, JobStatus::Completed)
                    .await
                {
                    error!("❌ Failed to mark job {} as completed: {}", job_id, e);
                }

                if let Err(e) = state
                    .nullifier_repo
                    .create_nullifier(job.nullifier.clone(), job_id)
                    .await
                {
                    tracing::debug!("Nullifier storage: {}", e);
                }

                return Ok(());
            }

            // Check if we should retry based on error type
            let should_retry = !error_str.contains("MissingAccounts") && // Don't retry account errors
                !error_str.contains("ProofInvalid"); // Don't retry proof errors

            if should_retry {
                warn!("🔄 Retrying job {} - Error: {}", job_id, e);

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
                error!("❌ Job {} failed permanently: {}", job_id, e);

                // Update job status to failed
                if let Err(e) = state
                    .job_repo
                    .update_job_status(job_id, JobStatus::Failed)
                    .await
                {
                    error!("❌ Failed to update job {} status to failed: {}", job_id, e);
                }
            }

            Ok(())
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
