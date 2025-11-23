/// RecoverSwapFunds instruction - Allows user to reclaim SOL if relay fails
///
/// This instruction can only be called AFTER the timeout_slot has passed.
/// It allows the user who initiated the swap to recover their SOL if the relay
/// failed to complete the swap within the timeout window.
///
/// Flow:
/// 1. Verify SwapState PDA exists and matches nullifier
/// 2. Verify current slot > timeout_slot
/// 3. Close SwapState PDA and return all lamports to user
///
/// Instruction data layout:
/// [nullifier (32)] - Used to derive SwapState PDA
/// Total: 32 bytes
///
/// Account layout:
/// 0. swap_state_pda (writable) - Will be closed
/// 1. user (writable) - Receives refund (must be signer)
/// 2. clock_sysvar (readonly) - For current slot

use crate::error::ShieldPoolError;
use crate::state::SwapState;
use crate::ID;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

const NULLIFIER_LEN: usize = 32;

pub fn process_recover_swap_funds(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts
    let [swap_state_info, user_info, _clock_sysvar] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify user is signer (only user can recover their funds)
    if !user_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse instruction data
    if data.len() != NULLIFIER_LEN {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let nullifier: [u8; 32] = data[..NULLIFIER_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    // Load SwapState
    let swap_state = SwapState::from_account_info(swap_state_info)?;

    // Verify nullifier matches
    if swap_state.nullifier() != nullifier {
        return Err(ShieldPoolError::NullifierMismatch.into());
    }

    // Verify SwapState PDA derivation
    let (expected_swap_state_pubkey, _bump) =
        pinocchio::pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier], &ID);

    if swap_state_info.key() != &expected_swap_state_pubkey {
        return Err(ShieldPoolError::InvalidAccountAddress.into());
    }

    // Verify timeout has passed
    let clock = Clock::get()?;
    let current_slot = clock.slot;
    let timeout_slot = swap_state.timeout_slot();

    if current_slot <= timeout_slot {
        return Err(ShieldPoolError::SwapTimeoutNotExpired.into());
    }

    // Close SwapState PDA - transfer all lamports to user
    let swap_state_lamports = swap_state_info.lamports();
    let user_lamports = user_info.lamports();

    unsafe {
        *swap_state_info.borrow_mut_lamports_unchecked() = 0;
        *user_info.borrow_mut_lamports_unchecked() = user_lamports + swap_state_lamports;
    }

    // Zero out the SwapState data (mark as closed)
    let data = unsafe { swap_state_info.borrow_mut_data_unchecked() };
    data.fill(0);

    Ok(())
}
