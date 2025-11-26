use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    seeds,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::ScrambleError,
    state::{Miner, ScrambleRegistry},
};

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
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [miner_account, miner_escrow, miner_authority, _system_program, _clock_sysvar, _remaining @ ..] =
        accounts
    else {
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

    // Parse initial escrow amount (8 bytes)
    let initial_escrow = if instruction_data.len() >= 8 {
        u64::from_le_bytes(
            instruction_data[0..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        )
    } else {
        0 // Default to 0 if not provided (backwards compatible)
    };

    // Derive miner PDA
    let (miner_pda, bump) =
        find_program_address(&[b"miner", miner_authority.key().as_ref()], &crate::ID);

    let (escrow_pda, escrow_bump) = find_program_address(
        &[b"miner_escrow", miner_authority.key().as_ref()],
        &crate::ID,
    );

    // Verify provided account matches PDA
    if miner_account.key() != &miner_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    if miner_escrow.key() != &escrow_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Get utilities
    let clock = Clock::get()?;
    let current_slot = clock.slot;
    let rent = Rent::get()?;

    // Re-initialization protection: check if miner already exists
    if !miner_account.data_is_empty() {
        return Err(ScrambleError::MinerAlreadyRegistered.into());
    }

    // Create miner PDA account
    {
        let space = Miner::SIZE;
        let lamports = rent.minimum_balance(space);

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

    // ALWAYS create escrow PDA account (needed for decoy operations)
    // Even if initial_escrow is 0, create the account for future top-ups
    if miner_escrow.data_is_empty() {
        let rent_lamports = rent.minimum_balance(0);
        let total_lamports = rent_lamports + initial_escrow;
        let bump_ref = &[escrow_bump];

        let escrow_seeds = seeds!(b"miner_escrow", miner_authority.key().as_ref(), bump_ref);
        let signer = Signer::from(&escrow_seeds);

        CreateAccount {
            from: miner_authority,
            to: miner_escrow,
            lamports: total_lamports,
            space: 0,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize miner data
    let mut miner = Miner::from_account_info_unchecked(&miner_account);
    miner.initialize(miner_authority.key(), current_slot);

    Ok(())
}
