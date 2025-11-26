use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use pinocchio_system::instructions::Transfer;

use crate::utils::pda_derivation::derive_miner_escrow_pda;

/// Top Up Escrow - Add funds to miner's escrow PDA
///
/// Allows miners to add more SOL to their escrow account for decoy operations.
///
/// Accounts:
/// 0. [signer, writable] Miner authority (payer)
/// 1. [writable] Miner Escrow PDA
/// 2. [] System Program
///
/// Instruction data (after discriminant): [amount: 8]
#[inline(always)]
pub fn process_top_up_escrow_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [miner_authority, miner_escrow, _system_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify miner_authority is writable (payer)
    if !miner_authority.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify miner_escrow is writable
    if !miner_escrow.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Parse instruction data (8 bytes: amount)
    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes(
        instruction_data[0..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Validate amount > 0
    if amount == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Verify escrow PDA
    let (expected_escrow, _) = derive_miner_escrow_pda(&crate::ID, miner_authority.key());
    if miner_escrow.key() != &expected_escrow {
        return Err(ProgramError::InvalidSeeds);
    }

    // Verify escrow is owned by this program
    if miner_escrow.owner() != &crate::ID {
        return Err(ProgramError::IllegalOwner);
    }

    // Transfer SOL from authority to escrow
    Transfer {
        from: miner_authority,
        to: miner_escrow,
        lamports: amount,
    }
    .invoke()?;

    Ok(())
}
