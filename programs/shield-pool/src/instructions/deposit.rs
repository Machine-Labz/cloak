use crate::{error::ShieldPoolError, state::{CommitmentQueue, Pool}};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use pinocchio_system::instructions::Transfer as SystemTransfer;
use pinocchio_token::instructions::Transfer as TokenTransfer;

#[inline(always)]
pub fn process_deposit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Fast fail if data too short
    if instruction_data.len() < 40 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let amount = unsafe { *(instruction_data.as_ptr() as *const u64) };
    let commit_bytes = unsafe { *((instruction_data.as_ptr().add(8)) as *const [u8; 32]) };

    let program_id = Pubkey::from(crate::ID);

    // Check if this is SOL or SPL token deposit based on account count
    // SOL: [user, pool, system_program, commitments] = 4 accounts
    // SPL: [user, user_token_account, pool, pool_token_account, token_program, system_program, commitments] = 7 accounts
    if accounts.len() == 4 {
        // Native SOL deposit
        let [user, pool, _system_program, commitments_info] =
            unsafe { *(accounts.as_ptr() as *const [AccountInfo; 4]) };

        process_native_deposit(&user, &pool, &commitments_info, amount, &commit_bytes, &program_id)
    } else if accounts.len() >= 7 {
        // SPL token deposit
        let [user, user_token_account, pool, pool_token_account, _token_program, _system_program, commitments_info] =
            unsafe { *(accounts.as_ptr() as *const [AccountInfo; 7]) };

        process_token_deposit(
            &user,
            &user_token_account,
            &pool,
            &pool_token_account,
            &commitments_info,
            amount,
            &commit_bytes,
            &program_id,
        )
    } else {
        Err(ShieldPoolError::MissingAccounts.into())
    }
}

#[inline(always)]
fn process_native_deposit(
    user: &AccountInfo,
    pool: &AccountInfo,
    commitments_info: &AccountInfo,
    amount: u64,
    commit_bytes: &[u8; 32],
    program_id: &Pubkey,
) -> ProgramResult {
    // Verify user is signer and has sufficient funds
    if !user.is_signer() || user.lamports() < amount {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if pool.owner() != program_id {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }

    if !pool.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }

    if !commitments_info.is_writable() {
        return Err(ShieldPoolError::CommitmentsNotWritable.into());
    }

    // Verify pool is configured for native SOL
    let pool_state = Pool::from_account_info(pool)?;
    if !pool_state.is_native() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let mut commitment_queue = CommitmentQueue::from_account_info(commitments_info)?;

    if commitment_queue.contains(commit_bytes) {
        return Err(ShieldPoolError::CommitmentAlreadyExists.into());
    }

    commitment_queue.append(commit_bytes)?;

    SystemTransfer {
        from: user,
        to: pool,
        lamports: amount,
    }
    .invoke()?;

    Ok(())
}

#[inline(always)]
fn process_token_deposit(
    user: &AccountInfo,
    user_token_account: &AccountInfo,
    pool: &AccountInfo,
    pool_token_account: &AccountInfo,
    commitments_info: &AccountInfo,
    amount: u64,
    commit_bytes: &[u8; 32],
    program_id: &Pubkey,
) -> ProgramResult {
    // Verify user is signer
    if !user.is_signer() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if pool.owner() != program_id {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }

    if !pool.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }

    if !commitments_info.is_writable() {
        return Err(ShieldPoolError::CommitmentsNotWritable.into());
    }

    // Verify pool is configured for SPL tokens (not native)
    let pool_state = Pool::from_account_info(pool)?;
    if pool_state.is_native() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let mut commitment_queue = CommitmentQueue::from_account_info(commitments_info)?;

    if commitment_queue.contains(commit_bytes) {
        return Err(ShieldPoolError::CommitmentAlreadyExists.into());
    }

    commitment_queue.append(commit_bytes)?;

    // Transfer SPL tokens from user to pool
    TokenTransfer {
        from: user_token_account,
        to: pool_token_account,
        authority: user,
        amount,
    }
    .invoke()?;

    Ok(())
}
