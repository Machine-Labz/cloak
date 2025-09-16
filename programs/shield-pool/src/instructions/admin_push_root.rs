use crate::error::ShieldPoolError;
use crate::instruction_data::AdminPushRootIx;
use crate::state::RootsRing;
use five8_const::decode_32_const;
use pinocchio::{account_info::AccountInfo, msg, ProgramResult};

// Admin authority - hardcoded for MVP (can be made configurable later)
const ADMIN_AUTHORITY: [u8; 32] = decode_32_const("11111111111111111111111111111111111111111111");

pub fn process_admin_push_root_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts - expecting: [admin (signer), roots_ring (writable)]
    let [admin_info, roots_ring_info] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    // Verify admin authorization
    if !admin_info.is_signer() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if admin_info.key().as_ref() != &ADMIN_AUTHORITY {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if !roots_ring_info.is_writable() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Parse instruction data
    let admin_data = AdminPushRootIx::from_instruction_data(instruction_data);

    // Load and update RootsRing
    let mut roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    roots_ring.push_root(&admin_data.new_root())?;

    // Log the pushed root
    let root_hex = hex::encode(admin_data.new_root());
    msg!(&format!("pushed_root:{}", root_hex));

    Ok(())
}
