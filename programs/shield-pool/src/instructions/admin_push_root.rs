use crate::error::ShieldPoolError;
use crate::state::RootsRing;
use five8_const::decode_32_const;
use pinocchio::{account_info::AccountInfo, msg, ProgramResult};

// Admin authority - hardcoded for MVP (can be made configurable later)
const ADMIN_AUTHORITY: [u8; 32] = decode_32_const("mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa");

pub fn process_admin_push_root_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!(&format!(
        "admin_push_root_instruction_data:{}",
        hex::encode(instruction_data)
    ));

    // Parse accounts - expecting: [admin (signer), roots_ring (writable)]
    let [admin_info, roots_ring_info] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    // Verify admin authorization
    if !admin_info.is_signer() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // For testing, allow any signer to be admin
    // TODO: Restore proper admin authority check in production
    if admin_info.key().as_ref() != &ADMIN_AUTHORITY {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if !roots_ring_info.is_writable() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Parse instruction data
    let admin_data = unsafe { *((instruction_data.as_ptr()).add(0) as *const [u8; 32]) };

    // Load and update RootsRing
    let mut roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    roots_ring.push_root(&admin_data)?;

    // Log the pushed root
    let root_hex = hex::encode(admin_data);
    msg!(&format!("pushed_root:{}", root_hex));

    Ok(())
}
