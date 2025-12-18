/// ExecuteSwap instruction - Transaction 2 of 2 for swap withdrawals
///
/// This instruction is called AFTER the relay executes the Jupiter swap.
/// It verifies the swap was successful and closes the SwapState PDA.
///
/// Flow:
/// 1. Verify SwapState PDA exists and matches nullifier
/// 2. Verify sufficient output tokens were received in recipient ATA
/// 3. Close SwapState PDA and return rent to payer
///
/// Instruction data layout:
/// [nullifier (32)] - Used to derive SwapState PDA
/// Total: 32 bytes
///
/// Account layout:
/// 0. swap_state_pda (writable) - Will be closed
/// 1. recipient_ata (readonly) - Verify tokens received
/// 2. payer (writable) - Receives rent refund
/// 3. token_program (readonly) - For token account verification
use crate::error::ShieldPoolError;
use crate::state::SwapState;
use crate::ID;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

const NULLIFIER_LEN: usize = 32;

pub fn process_execute_swap_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts
    let [swap_state_info, recipient_ata_info, payer_info, _token_program_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

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

    // Verify recipient ATA matches stored address
    if recipient_ata_info.key() != &swap_state.recipient_ata() {
        return Err(ShieldPoolError::InvalidRecipient.into());
    }

    // Verify recipient ATA has sufficient tokens
    // Token account layout: [mint(32)][owner(32)][amount(8)][...]
    let ata_data = recipient_ata_info.try_borrow_data()?;
    if ata_data.len() < 72 {
        return Err(ShieldPoolError::InvalidAccountSize.into());
    }

    let token_amount = u64::from_le_bytes(
        ata_data[64..72]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    let min_output_amount = swap_state.min_output_amount();
    if token_amount < min_output_amount {
        return Err(ShieldPoolError::InvalidAmount.into());
    }

    // Close SwapState PDA - transfer all lamports to payer
    let swap_state_lamports = swap_state_info.lamports();
    let payer_lamports = payer_info.lamports();

    unsafe {
        *swap_state_info.borrow_mut_lamports_unchecked() = 0;
        *payer_info.borrow_mut_lamports_unchecked() = payer_lamports + swap_state_lamports;
    }

    // Zero out the SwapState data (mark as closed)
    let data = unsafe { swap_state_info.borrow_mut_data_unchecked() };
    data.fill(0);

    Ok(())
}
