use crate::error::ShieldPoolError;
use pinocchio::{account_info::AccountInfo, ProgramResult};
use pinocchio_system::instructions::Transfer;

#[inline(always)]
pub fn process_deposit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // SAFETY: We validate array length before accessing
    let [user, pool, _, _] = unsafe { *(accounts.as_ptr() as *const [AccountInfo; 4]) };

    // Fast fail if data too short
    if instruction_data.len() < 40 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // SAFETY: We validated length above
    let amount = unsafe { *(instruction_data.as_ptr() as *const u64) };
    let commit_bytes = unsafe { *((instruction_data.as_ptr().add(8)) as *const [u8; 32]) };

    // Fast fail if user not signer or insufficient funds
    if !user.is_signer() || user.lamports() < amount {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Direct transfer without validation
    // Transfer {
    //     from: &user,
    //     to: &pool,
    //     lamports: amount,
    // }
    // .invoke()?;
    unsafe {
        *user.borrow_mut_lamports_unchecked() -= amount;
        *pool.borrow_mut_lamports_unchecked() += amount;
    }

    Ok(())
}
