use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{msg, ProgramResult};

use crate::error::ScrambleError;
use crate::state::{Claim, ClaimStatus, Miner, ScrambleRegistry};

/// Instruction: consume_claim
///
/// Consumes one unit from a revealed claim (called during CPI from shield-pool withdraw).
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [WRITE] Miner PDA (to record consume)
/// 2. [WRITE] ScrambleRegistry PDA (to decrement active claims when fully consumed)
/// 3. [SIGNER] Shield-pool program (CPI authority)
/// 4. [] Clock sysvar
///
/// Arguments:
/// - expected_miner_authority: Pubkey (anti-replay: must match claim.miner_authority)
/// - expected_batch_hash: [u8; 32] (anti-replay: must match claim.batch_hash)
pub fn process_consume_claim(
    accounts: &[AccountInfo],
    expected_miner_authority: [u8; 32],
    expected_batch_hash: [u8; 32],
) -> ProgramResult {
    // Parse accounts
    let [claim_account, miner_account, registry_account, shield_pool_program, _clock_sysvar, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify CPI caller is shield-pool program
    // Note: In production, hardcode shield-pool program ID or pass as registry field
    if !shield_pool_program.is_signer() {
        msg!("consume_claim must be called via CPI from shield-pool");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load accounts
    let claim = Claim::from_account(claim_account)?;
    let miner = Miner::from_account(miner_account)?;
    let registry = ScrambleRegistry::from_account(registry_account)?;

    // Get current slot
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // Anti-replay: verify miner_authority matches
    if claim.miner_authority.as_ref() != expected_miner_authority {
        msg!("Miner authority mismatch");
        return Err(ScrambleError::UnauthorizedMiner.into());
    }

    // Anti-replay: verify batch_hash matches
    if claim.batch_hash != expected_batch_hash {
        msg!("Batch hash mismatch");
        return Err(ScrambleError::BatchHashMismatch.into());
    }

    // Verify claim is consumable
    if !claim.is_consumable(current_slot) {
        msg!("Claim not consumable");

        // Check if expired and mark accordingly
        if claim.is_expired(current_slot) {
            claim.set_status(ClaimStatus::Expired);
            return Err(ScrambleError::ClaimExpired.into());
        }

        return Err(ScrambleError::InvalidClaimStatus.into());
    }

    // Consume one unit
    let was_fully_consumed = claim.consumed_count == claim.max_consumes;

    claim.consume()?;

    let is_now_fully_consumed = claim.consumed_count == claim.max_consumes;

    // Update miner stats
    miner.record_consume();

    // Update registry active claims counter
    if is_now_fully_consumed && !was_fully_consumed {
        registry.decrement_active();
    }

    msg!("Claim consumed successfully");

    Ok(())
}
