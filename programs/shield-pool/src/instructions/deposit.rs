use crate::{error::ShieldPoolError, state::CommitmentQueue};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};
use pinocchio_system::instructions::Transfer;

#[inline(always)]
pub fn process_deposit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // SAFETY: We validate array length before accessing
    let [user, pool, _system_program, commitments_info] =
        unsafe { *(accounts.as_ptr() as *const [AccountInfo; 4]) };

    // Fast fail if data too short
    if instruction_data.len() < 40 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let amount = unsafe { *(instruction_data.as_ptr() as *const u64) };
    let commit_bytes = unsafe { *((instruction_data.as_ptr().add(8)) as *const [u8; 32]) };

    // Fast fail if user not signer or insufficient funds
    if !user.is_signer() || user.lamports() < amount {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    let program_id = Pubkey::from(crate::ID);
    if pool.owner() != &program_id {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }

    if !pool.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }

    if !commitments_info.is_writable() {
        return Err(ShieldPoolError::CommitmentsNotWritable.into());
    }

    let mut commitment_queue = CommitmentQueue::from_account_info(&commitments_info)?;

    if commitment_queue.contains(&commit_bytes) {
        return Err(ShieldPoolError::CommitmentAlreadyExists.into());
    }

    commitment_queue.append(&commit_bytes)?;

    Transfer {
        from: &user,
        to: &pool,
        lamports: amount,
    }
    .invoke()?;

    Ok(())
}
