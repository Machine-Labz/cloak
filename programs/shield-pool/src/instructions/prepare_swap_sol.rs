/// PrepareSwapSol instruction
///
/// Transfers SOL from SwapState PDA to wSOL ATA (leaving rent-exempt minimum).
/// The relay must call SyncNative separately to wrap SOL → wSOL.
/// This must be called before ExecuteSwapViaOrca.
///
/// Account layout:
/// 0. swap_state_pda (writable) - PDA holding SOL for swap
/// 1. swap_wsol_ata (writable) - wSOL token account for swap_pda
///
/// Data layout: none (reads amount from SwapState)
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, sysvars::Sysvar,
    ProgramResult,
};

use crate::{error::ShieldPoolError, state::SwapState, ID};

pub fn process_prepare_swap_sol(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // Parse accounts (only 2 accounts needed - no CPIs)
    let [swap_state_info, swap_wsol_ata_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify swap_state_pda is writable
    if !swap_state_info.is_writable() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Verify swap_wsol_ata is writable
    if !swap_wsol_ata_info.is_writable() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Read SwapState
    let swap_state = SwapState::from_account_info(swap_state_info)?;
    let nullifier = swap_state.nullifier();

    // Derive PDA to verify it matches
    let (expected_swap_state_pda, _bump) =
        pinocchio::pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier], &ID);

    if swap_state_info.key() != &expected_swap_state_pda {
        return Err(ShieldPoolError::InvalidAccountAddress.into());
    }

    // Transfer SOL from SwapState PDA to wSOL ATA, leaving rent-exempt minimum
    // (exactly like ReleaseSwapFunds pattern)
    // We can't call CPIs after lamport manipulation or we get UnbalancedInstruction error
    let current_lamports = swap_state_info.lamports();
    let wsol_ata_lamports = swap_wsol_ata_info.lamports();

    // Calculate rent-exempt minimum for SwapState (we need to leave it open for signing)
    let rent = pinocchio::sysvars::rent::Rent::get()?;
    let rent_exempt_minimum = rent.minimum_balance(SwapState::SIZE);

    // Validate we have enough lamports to keep rent-exempt minimum
    if current_lamports <= rent_exempt_minimum {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    // Transfer all available lamports minus rent-exempt minimum
    // (SwapState has rent + (public_amount - fee), so we transfer public_amount - fee)
    let amount_to_transfer = current_lamports - rent_exempt_minimum;

    unsafe {
        *swap_state_info.borrow_mut_lamports_unchecked() = rent_exempt_minimum;
        *swap_wsol_ata_info.borrow_mut_lamports_unchecked() =
            wsol_ata_lamports + amount_to_transfer;
    }

    Ok(())
}
