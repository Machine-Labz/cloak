use crate::error::ScrambleError;
use crate::state::{Miner, ScrambleRegistry};
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::Signer;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::{find_program_address, Pubkey};
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio::{seeds, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

/// Derive the registry PDA
fn derive_registry_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[b"registry"], program_id)
}

#[inline(always)]
pub fn process_initialize_registry_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    // Layout: initial_difficulty(32) + min_difficulty(32) + max_difficulty(32) +
    //         target_interval_slots(8) + fee_share_bps(2) + reveal_window(8) +
    //         claim_window(8) + max_k(2) = 124 bytes
    if instruction_data.len() < 124 {
        return Err(ScrambleError::InvalidTag.into());
    }

    let initial_difficulty: [u8; 32] = instruction_data[0..32]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let min_difficulty: [u8; 32] = instruction_data[32..64]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let max_difficulty: [u8; 32] = instruction_data[64..96]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let target_interval_slots = u64::from_le_bytes(
        instruction_data[96..104]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let fee_share_bps = u16::from_le_bytes(
        instruction_data[104..106]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let reveal_window = u64::from_le_bytes(
        instruction_data[106..114]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let claim_window = u64::from_le_bytes(
        instruction_data[114..122]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let max_k = u16::from_le_bytes(
        instruction_data[122..124]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    // Parse accounts
    let [registry_account, admin_authority, _system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !admin_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify admin is writable (needs to pay rent)
    if !admin_authority.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Derive PDA
    let (registry_pda, bump) = derive_registry_pda(&crate::ID);

    // Verify provided account matches PDA
    if registry_account.key() != &registry_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Calculate space and rent
    let space = ScrambleRegistry::SIZE;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    // Create PDA account if it doesn't exist
    if registry_account.data_is_empty() {
        // Create PDA account via system program CPI
        let bump_ref = &[bump];
        let registry_seeds = seeds!(b"registry", bump_ref);
        let signer = Signer::from(&registry_seeds);

        CreateAccount {
            from: admin_authority,
            to: registry_account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize registry data
    let mut registry = ScrambleRegistry::from_account_info_unchecked(&registry_account);
    registry.initialize(
        admin_authority.key(),
        &initial_difficulty,
        &min_difficulty,
        &max_difficulty,
        target_interval_slots,
        fee_share_bps,
        reveal_window,
        claim_window,
        max_k,
    );


    Ok(())
}

#[inline(always)]
pub fn process_register_miner_instruction(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [miner_account, miner_authority, _system_program, _clock_sysvar, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify authority is writable (payer)
    if !miner_authority.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Derive miner PDA
    let (miner_pda, bump) =
        find_program_address(&[b"miner", miner_authority.key().as_ref()], &crate::ID);

    // Verify provided account matches PDA
    if miner_account.key() != &miner_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Get current slot
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // Calculate space and rent
    let space = Miner::SIZE;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    // Create PDA account if it doesn't exist
    if miner_account.data_is_empty() {

        // Create PDA account via system program CPI
        let bump_ref = &[bump];
        let miner_seeds = seeds!(b"miner", miner_authority.key().as_ref(), bump_ref);
        let signer = Signer::from(&miner_seeds);

        CreateAccount {
            from: miner_authority,
            to: miner_account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize miner data
    let mut miner = Miner::from_account_info_unchecked(&miner_account);
    miner.initialize(miner_authority.key(), current_slot);

    Ok(())
}
