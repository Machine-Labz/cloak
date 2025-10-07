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

    if instruction_data.len() < 40 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let amount = unsafe { *(instruction_data.as_ptr() as *const u64) };

    if !user.is_signer() || user.lamports() < amount {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    Transfer {
        from: &user,
        to: &pool,
        lamports: amount,
    }
    .invoke()?;

    Ok(())
}
