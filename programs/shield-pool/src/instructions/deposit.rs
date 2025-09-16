use crate::error::ShieldPoolError;
use pinocchio::{account_info::AccountInfo, msg, ProgramResult};

pub fn process_deposit_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user, pool_info, _roots_ring_info, _system_program] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    unsafe {
        // Parse instruction data: amount (8 bytes) + leaf_commit (32 bytes) + enc_output_len (2 bytes) + enc_output (variable)
        let amount = *((data.as_ptr()).add(0) as *const u64);
        let leaf_commit = *((data.as_ptr()).add(8) as *const [u8; 32]);

        // Transfer lamports from user to pool
        *user.borrow_mut_lamports_unchecked() -= amount;
        *pool_info.borrow_mut_lamports_unchecked() += amount;

        // Log the deposit commitment for indexer
        let commit_hex = hex::encode(leaf_commit);
        msg!(&format!("deposit_commit:{}", commit_hex));
    }

    Ok(())
}
