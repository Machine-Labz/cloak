use crate::state::RootsRing;
use crate::{constants::ADMIN_AUTHORITY, error::ShieldPoolError};
use pinocchio::{account_info::AccountInfo, ProgramResult};

pub fn process_admin_push_root_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts - expecting: [admin (signer), roots_ring (writable)]
    let [admin_info, roots_ring_info] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    // Verify admin authorization
    if !admin_info.is_signer()
        || admin_info.key() != &ADMIN_AUTHORITY
        || !roots_ring_info.is_writable()
    {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Parse instruction data
    let admin_data = unsafe { *((instruction_data.as_ptr()).add(0) as *const [u8; 32]) };

    // Load and update RootsRing
    let mut roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    roots_ring.push_root(&admin_data)?;

    Ok(())
}
