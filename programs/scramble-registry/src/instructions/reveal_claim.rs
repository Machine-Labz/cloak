use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{msg, ProgramResult};

use crate::error::ScrambleError;
use crate::state::{Claim, ClaimStatus, ScrambleRegistry};

/// Instruction: reveal_claim
///
/// Transitions a Mined claim to Revealed status within the reveal window.
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [] ScrambleRegistry PDA (for reveal_window)
/// 2. [SIGNER] Miner authority
/// 3. [] Clock sysvar
pub fn process_reveal_claim(accounts: &[AccountInfo]) -> ProgramResult {
    // Parse accounts
    let [claim_account, registry_account, miner_authority, _clock_sysvar, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load accounts
    let claim = Claim::from_account(claim_account)?;
    let registry = ScrambleRegistry::from_account(registry_account)?;

    // Verify authority
    if claim.miner_authority != *miner_authority.key() {
        msg!("Unauthorized: miner authority mismatch");
        return Err(ScrambleError::UnauthorizedMiner.into());
    }

    // Verify status is Mined
    if claim.get_status() != ClaimStatus::Mined {
        msg!("Claim not in Mined status");
        return Err(ScrambleError::InvalidClaimStatus.into());
    }

    // Get current slot
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // Check reveal window
    let elapsed = current_slot.saturating_sub(claim.mined_at_slot);
    if elapsed > registry.reveal_window {
        msg!("Reveal window expired");
        // Mark as expired
        claim.set_status(ClaimStatus::Expired);
        return Err(ScrambleError::ClaimExpired.into());
    }

    // Transition to Revealed
    claim.reveal(current_slot, registry.claim_window);

    msg!("Claim revealed successfully");

    Ok(())
}
