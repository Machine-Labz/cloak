use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::Signer;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::{find_program_address, Pubkey};
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio::{msg, seeds, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{Miner, ScrambleRegistry};

/// Derive the registry PDA
pub fn derive_registry_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[b"registry"], program_id)
}

/// Instruction: initialize_registry
///
/// One-time initialization of the ScrambleRegistry singleton PDA.
///
/// Accounts:
/// 0. [WRITE] ScrambleRegistry PDA (to be created)
/// 1. [SIGNER, WRITE] Admin authority (payer)
/// 2. [] System program
///
/// Arguments:
/// - initial_difficulty: [u8; 32]
/// - min_difficulty: [u8; 32]
/// - max_difficulty: [u8; 32]
/// - target_interval_slots: u64
/// - fee_share_bps: u16
/// - reveal_window: u64
/// - claim_window: u64
/// - max_k: u16
#[allow(clippy::too_many_arguments)]
pub fn process_initialize_registry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_difficulty: [u8; 32],
    min_difficulty: [u8; 32],
    max_difficulty: [u8; 32],
    target_interval_slots: u64,
    fee_share_bps: u16,
    reveal_window: u64,
    claim_window: u64,
    max_k: u16,
) -> ProgramResult {
    // Parse accounts
    let [registry_account, admin_authority, system_program, ..] = accounts else {
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
    let (registry_pda, bump) = derive_registry_pda(program_id);

    // Verify provided account matches PDA
    if registry_account.key() != &registry_pda {
        msg!("Registry account mismatch");
        return Err(ProgramError::InvalidSeeds);
    }

    // Verify account doesn't already exist with data
    if !registry_account.data_is_empty() {
        msg!("Registry already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Calculate space and rent
    let space = std::mem::size_of::<ScrambleRegistry>();
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    msg!("Creating registry PDA");

    // Create PDA account via system program CPI
    let bump_ref = &[bump];
    let registry_seeds = seeds!(b"registry", bump_ref);
    let signer = Signer::from(&registry_seeds);

    CreateAccount {
        from: admin_authority,
        to: registry_account,
        lamports,
        space: space as u64,
        owner: program_id,
    }
    .invoke_signed(&[signer])?;

    msg!("Registry PDA created");

    // Initialize registry data directly (without checking discriminator)
    let data = unsafe {
        &mut *registry_account
            .borrow_mut_data_unchecked()
            .as_mut_ptr()
            .cast::<ScrambleRegistry>()
    };

    *data = ScrambleRegistry::new(
        *admin_authority.key(),
        initial_difficulty,
        min_difficulty,
        max_difficulty,
        target_interval_slots,
        fee_share_bps,
        reveal_window,
        claim_window,
        max_k,
    );

    msg!("ScrambleRegistry initialized");

    Ok(())
}

/// Instruction: register_miner
///
/// Initialize a Miner PDA for a given authority (anti-key-grinding).
///
/// Accounts:
/// 0. [WRITE] Miner PDA (uninitialized)
/// 1. [SIGNER] Miner authority
/// 2. [] System program
/// 3. [] Clock sysvar
pub fn process_register_miner(accounts: &[AccountInfo]) -> ProgramResult {
    // Parse accounts
    let [miner_account, miner_authority, _system_program, _clock_sysvar, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify account is uninitialized
    if !miner_account.data_is_empty() {
        msg!("Miner already registered");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Get current slot
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // In production: Create PDA via system program CPI here
    // For now, assume account is pre-allocated

    // Initialize miner
    let miner = Miner::from_account(miner_account)?;
    *miner = Miner::new(*miner_authority.key(), current_slot);

    msg!("Miner registered successfully");

    Ok(())
}
