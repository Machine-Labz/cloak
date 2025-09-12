use crate::constants::*;
use crate::error::ShieldPoolError;
use crate::state::RootsRing;
use pinocchio::{account_info::AccountInfo, ProgramResult};

pub fn process_admin_push_root_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [roots_ring_info] = accounts else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    if instruction_data.len() != HASH_SIZE {
        return Err(ShieldPoolError::BadIxLength.into());
    }

    let mut roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    let new_root = instruction_data
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    roots_ring.push_root(&new_root)?;

    Ok(())
}
