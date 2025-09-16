use crate::error::ShieldPoolError;
use pinocchio::{account_info::AccountInfo, msg, ProgramResult};

pub fn process_deposit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [user, pool_info, _roots_ring_info, _system_program] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    // Parse instruction data;
    unsafe {
        let amount = *((instruction_data.as_ptr()).add(0) as *const u64);
        *user.borrow_mut_lamports_unchecked() -= amount;
        *pool_info.borrow_mut_lamports_unchecked() += amount;
    }

    // Log the deposit commitment for indexer
    unsafe {
        // Read 32 bytes from instruction_data starting at offset 8
        let commit_ptr = instruction_data.as_ptr().add(8) as *const [u8; 32];
        let commit_bytes: [u8; 32] = *commit_ptr;
        let commit_hex = hex::encode(commit_bytes);
        msg!(&format!("deposit_commit:{}", commit_hex));
    }

    Ok(())
}
