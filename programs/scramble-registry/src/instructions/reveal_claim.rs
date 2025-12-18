use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    error::ScrambleError,
    state::{Claim, ClaimStatus, ScrambleRegistry},
};

#[inline(always)]
pub fn process_reveal_claim_instruction(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [claim_account, registry_account, miner_authority, _clock_sysvar, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load accounts
    let mut claim = Claim::from_account_info(claim_account)?;
    let registry = ScrambleRegistry::from_account_info(registry_account)?;

    // Verify authority
    if claim.miner_authority() != miner_authority.key() {
        return Err(ScrambleError::UnauthorizedMiner.into());
    }

    // Verify status is Mined
    if claim.status() != ClaimStatus::Mined {
        return Err(ScrambleError::InvalidClaimStatus.into());
    }

    // Get current slot
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // Check reveal window
    let elapsed = current_slot.saturating_sub(claim.mined_at_slot());
    if elapsed > registry.reveal_window() {
        return Err(ScrambleError::ClaimExpired.into());
    }

    // Transition to Revealed
    claim.reveal(current_slot, registry.claim_window());

    Ok(())
}
