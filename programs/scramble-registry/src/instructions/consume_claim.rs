use crate::error::ScrambleError;
use crate::state::{Claim, Miner, ScrambleRegistry};
use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{msg, ProgramResult};

#[inline(always)]
pub fn process_consume_claim_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    // Layout: expected_miner_authority(32) + expected_batch_hash(32) = 64 bytes
    if instruction_data.len() < 64 {
        return Err(ScrambleError::InvalidTag.into());
    }

    let expected_miner_authority: [u8; 32] = instruction_data[0..32]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let expected_batch_hash: [u8; 32] = instruction_data[32..64]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
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
    let mut claim = Claim::from_account_info(claim_account)?;
    let mut miner = Miner::from_account_info(miner_account)?;
    let mut registry = ScrambleRegistry::from_account_info(registry_account)?;

    // Get current slot
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // Anti-replay: verify miner_authority matches
    if claim.miner_authority().as_ref() != expected_miner_authority {
        msg!("Miner authority mismatch");
        return Err(ScrambleError::UnauthorizedMiner.into());
    }

    // Anti-replay: verify batch_hash matches (unless wildcard)
    if !claim.is_wildcard() && claim.batch_hash() != &expected_batch_hash {
        msg!("Batch hash mismatch (claim is not wildcard)");
        return Err(ScrambleError::BatchHashMismatch.into());
    }

    if claim.is_wildcard() {
        msg!("Using wildcard claim (batch_hash check skipped)");
    }

    // Verify claim is consumable
    if !claim.is_consumable(current_slot) {
        msg!("Claim not consumable");

        // Check if expired
        if claim.is_expired(current_slot) {
            return Err(ScrambleError::ClaimExpired.into());
        }

        return Err(ScrambleError::InvalidClaimStatus.into());
    }

    // Consume one unit
    let was_fully_consumed = claim.consumed_count() == claim.max_consumes();

    claim.consume()?;

    let is_now_fully_consumed = claim.consumed_count() == claim.max_consumes();

    // Update miner stats
    miner.record_consume();

    // Update registry active claims counter
    if is_now_fully_consumed && !was_fully_consumed {
        registry.decrement_active();
    }

    msg!("Claim consumed successfully");

    Ok(())
}
