use crate::error::ShieldPoolError;
use crate::state::RootsRing;
use five8_const::decode_32_const;
use pinocchio::{account_info::AccountInfo, ProgramResult};

const ADMIN_AUTHORITY: [u8; 32] = decode_32_const("aboDQRWHMesuZReBn9EZcRQMW9i9KJ7Pmw7CncP8SuB");

pub fn process_admin_push_root_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [admin_info, roots_ring_info] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    if !admin_info.is_signer()
        || admin_info.key() != &ADMIN_AUTHORITY
        || !roots_ring_info.is_writable()
    {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    let root = unsafe { *((instruction_data.as_ptr()) as *const [u8; 32]) };
    let mut roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    roots_ring.push_root(&root)?;

    Ok(())
}
