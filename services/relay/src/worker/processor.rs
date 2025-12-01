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

    // Re-fetch job from database to get current status (buffer may have stale data)
    let current_job = match state.job_repo.get_job_by_id(job_id).await {
        Ok(Some(j)) => j,
        Ok(None) => {
            warn!("⚠️  Job {} not found in database, skipping", job_id);
            return Ok(());
        }
        Err(e) => {
            error!("❌ Failed to fetch job {} from database: {}", job_id, e);
            // Fall back to using the provided job object
            job
        }
    };

    info!("🔄 Processing job: {}", job_id);
    info!("   Request ID: {}", current_job.request_id);
    info!("   Status: {:?}", current_job.status);

    // Check if job is already completed or failed (using fresh DB data)
    if current_job.status == JobStatus::Completed {
        info!("✅ Job {} already completed, skipping", job_id);
        return Ok(());
    }

    if current_job.status == JobStatus::Failed {
        warn!("⚠️  Job {} already marked as failed, skipping", job_id);
        return Ok(());
    }

    // Use the fresh job data for processing
    let job = current_job;

    // CRITICAL: Double-check status one more time before updating to processing
    // This prevents race conditions where the job was just marked as completed
    let final_status_check = match state.job_repo.get_job_by_id(job_id).await {
        Ok(Some(j)) => j.status,
        Ok(None) => {
            error!("❌ Job {} not found in database", job_id);
            return Err(crate::error::Error::InternalServerError("Job not found".to_string()));
        }
        Err(e) => {
            error!("❌ Failed to fetch job {} for final status check: {}", job_id, e);
            return Err(e);
        }
    };

    if final_status_check == JobStatus::Completed {
        info!("✅ Job {} already completed (final check before processing), skipping", job_id);
        return Ok(());
    }

    if final_status_check == JobStatus::Failed {
        warn!("⚠️  Job {} already marked as failed (final check), skipping", job_id);
        return Ok(());
    }

    // Update status to processing (database protection will prevent if already completed)
    if let Err(e) = state
        .job_repo
        .update_job_status(job_id, JobStatus::Processing)
        .await
    {
        // If update failed because job is already completed, that's fine - just return
        if e.to_string().contains("already be completed") || e.to_string().contains("no effect") {
            info!("✅ Job {} was completed between checks, skipping", job_id);
            return Ok(());
        }
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
                                // Mark as completed immediately and return to prevent retry
                                if let Err(e) = state
                                    .job_repo
                                    .update_job_status(job_id, JobStatus::Completed)
                                    .await
                                {
                                    error!("❌ Failed to mark job {} as completed: {}", job_id, e);
                                }
                                // Store nullifier to prevent double-spending
                                if let Err(e) = state
                                    .nullifier_repo
                                    .create_nullifier(job.nullifier.clone(), job_id)
                                    .await
                                {
                                    tracing::debug!("Nullifier storage: {}", e);
                                }
                                return Ok(());
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
            // 0x1020 = DoubleSpend (nullifier already used = transaction already succeeded)
            let double_spend = error_str.contains("already been processed")
                || error_str.contains("0x1020")
                || error_str.contains("DoubleSpend")
                || error_str.contains("custom program error: 0x1020");

            if double_spend {
                info!(
                    "✅ Job {} - Double spend detected (0x1020), transaction was already processed",
                    job_id
                );

                // Mark as completed - transaction already succeeded on-chain
                // Try update_job_completed first, then fallback to update_job_status
                let update_result = state
                    .job_repo
                    .update_job_completed(
                        job_id,
                        "double_spend_already_processed".to_string(),
                        "double_spend_already_processed".to_string(),
                    )
                    .await;

                if let Err(e) = update_result {
                    warn!("⚠️  update_job_completed failed for job {}: {}, trying fallback", job_id, e);
                    // Fallback to simple status update
                    if let Err(e2) = state
                        .job_repo
                        .update_job_status(job_id, JobStatus::Completed)
                        .await
                    {
                        error!("❌ Failed to mark job {} as completed: {} (fallback also failed: {})", job_id, e, e2);
                        // Don't return error - continue to verification
                    } else {
                        info!("✅ Job {} marked as completed via fallback status update", job_id);
                    }
                } else {
                    info!("✅ Job {} update_job_completed succeeded", job_id);
                }

                // Store nullifier to prevent double-spending (if not already in local DB)
                if let Err(e) = state
                    .nullifier_repo
                    .create_nullifier(job.nullifier.clone(), job_id)
                    .await
                {
                    tracing::debug!("Nullifier storage: {}", e);
                }

                // Verify the status was updated (defensive check with retry)
                // Use longer delays and more attempts to account for database replication lag
                let mut verified = false;
                for attempt in 0..5 {
                    match state.job_repo.get_job_by_id(job_id).await {
                        Ok(Some(updated_job)) => {
                            if updated_job.status == JobStatus::Completed {
                                info!("✅ Job {} confirmed as completed in database (attempt {})", job_id, attempt + 1);
                                verified = true;
                                break;
                            } else {
                                if attempt < 4 {
                                    warn!(
                                        "⚠️  Job {} status update not yet visible - current status: {:?} (attempt {}/5), retrying...",
                                        job_id, updated_job.status, attempt + 1
                                    );
                                    tokio::time::sleep(Duration::from_millis(200 * (attempt + 1) as u64)).await;
                                } else {
                                    error!(
                                        "❌ Job {} status update failed - still showing as {:?} after 5 attempts. This is a critical error!",
                                        job_id, updated_job.status
                                    );
                                    // Force update one more time as last resort
                                    if let Err(e3) = state
                                        .job_repo
                                        .update_job_status(job_id, JobStatus::Completed)
                                        .await
                                    {
                                        error!("❌ Last resort status update also failed: {}", e3);
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            error!("❌ Job {} not found in database during verification (attempt {})", job_id, attempt + 1);
                            if attempt < 4 {
                                tokio::time::sleep(Duration::from_millis(200 * (attempt + 1) as u64)).await;
                            }
                        }
                        Err(e) => {
                            error!("❌ Failed to fetch job {} for verification: {}", job_id, e);
                            if attempt < 4 {
                                tokio::time::sleep(Duration::from_millis(200 * (attempt + 1) as u64)).await;
                            }
                        }
                    }
                }

                if !verified {
                    error!("❌ CRITICAL: Job {} status update verification failed - job may be reprocessed!", job_id);
                }

                return Ok(());
            }

            // Check if we should retry based on error type
            // Don't retry if job is already completed (defensive check)
            let current_status = match state.job_repo.get_job_by_id(job_id).await {
                Ok(Some(j)) => j.status,
                _ => job.status, // Fallback to original status
            };

            if current_status == JobStatus::Completed {
                info!(
                    "✅ Job {} already completed (status check), skipping retry",
                    job_id
                );
                return Ok(());
            }

            // Double-check status before retrying (defensive check against race conditions)
            let final_status_check = match state.job_repo.get_job_by_id(job_id).await {
                Ok(Some(j)) => j.status,
                _ => current_status, // Use previous check result
            };

            if final_status_check == JobStatus::Completed {
                info!(
                    "✅ Job {} already completed (final status check), skipping retry",
                    job_id
                );
                return Ok(());
            }

            // NEVER retry double-spend errors (0x1020) - transaction already succeeded
            let is_double_spend = error_str.contains("0x1020")
                || error_str.contains("DoubleSpend")
                || error_str.contains("custom program error: 0x1020");
            
            if is_double_spend {
                error!("🚫 CRITICAL: Double-spend (0x1020) reached retry logic - this should never happen! Job {} should have been marked completed earlier.", job_id);
                // Force mark as completed and return - don't retry
                if let Err(e) = state
                    .job_repo
                    .update_job_status(job_id, JobStatus::Completed)
                    .await
                {
                    error!("❌ Failed to force-complete job {}: {}", job_id, e);
                }
                return Ok(());
            }

            let should_retry = !error_str.contains("MissingAccounts") && // Don't retry account errors
                !error_str.contains("ProofInvalid"); // Don't retry proof errors

            if should_retry {
                warn!("🔄 Retrying job {} - Error: {}", job_id, e);

                // Update status to queued (for retry) - but only if not already completed
                // Database protection will prevent this if job is already completed
                if let Err(status_err) = state
                    .job_repo
                    .update_job_status(job_id, JobStatus::Queued)
                    .await
                {
                    // If update failed because job is already completed, that's fine - don't retry
                    warn!(
                        "⚠️  Cannot re-queue job {} for retry: {} (job may already be completed)",
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
