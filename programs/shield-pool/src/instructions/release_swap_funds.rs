use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::{error::ShieldPoolError, state::SwapState, ID};

/// ReleaseSwapFunds releases the SOL from SwapState PDA to the relay
/// so the relay can perform the swap off-chain (via Jupiter or Orca)
pub fn process_release_swap_funds(accounts: &[AccountInfo]) -> ProgramResult {
    let [swap_state_info, relay_info, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify relay is the signer
    if !relay_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load and verify SwapState
    let swap_state = SwapState::from_account_info(swap_state_info)?;

    // Verify SwapState is owned by our program
    if swap_state_info.owner() != &ID {
        return Err(ProgramError::IllegalOwner);
    }

    // Verify SwapState is initialized by checking if nullifier is not empty
    if swap_state.nullifier() == [0u8; 32] {
        return Err(ShieldPoolError::InvalidNullifier.into());
    }

    // Calculate how much SOL to transfer (all lamports minus rent-exempt minimum)
    // We need to leave the account with enough lamports to remain rent-exempt
    // SwapState is 121 bytes, so we calculate the proper rent-exempt amount
    let rent = Rent::get()?;
    let rent_exempt_minimum = rent.minimum_balance(SwapState::SIZE);
    let current_lamports = swap_state_info.lamports();

    if current_lamports <= rent_exempt_minimum {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    let amount_to_transfer = current_lamports - rent_exempt_minimum;

    // Transfer lamports from SwapState PDA to relay
    unsafe {
        *swap_state_info.borrow_mut_lamports_unchecked() = rent_exempt_minimum;
        *relay_info.borrow_mut_lamports_unchecked() = relay_info.lamports() + amount_to_transfer;
    }

    Ok(())
}
