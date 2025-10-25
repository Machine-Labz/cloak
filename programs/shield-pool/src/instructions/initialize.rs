use crate::{
    constants::ADMIN_AUTHORITY, error::ShieldPoolError, state::{CommitmentQueue, NullifierShard, RootsRing}, ID
};
use pinocchio::sysvars::rent::Rent;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::{find_program_address, Pubkey},
    sysvars::Sysvar,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

#[inline(always)]
pub fn process_initialize_instruction(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    if accounts.len() < 7 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    // SAFETY: We validate array length before accessing
    let [admin, pool, commitments, roots_ring, nullifier_shard, treasury, _system_program] =
        unsafe { *(accounts.as_ptr() as *const [AccountInfo; 7]) };

    // Check admin authority
    if admin.key() != &ADMIN_AUTHORITY {
        return Err(ShieldPoolError::BadAccounts.into());
    }
    if !admin.is_signer() {
        return Err(ShieldPoolError::InvalidAdminAuthority.into());
    }

    let program_id = Pubkey::from(ID);
    let rent = Rent::get()?;

    create_pda_account(&admin, &pool, &program_id, b"pool", 0, &rent)?;
    create_pda_account(
        &admin,
        &commitments,
        &program_id,
        b"commitments",
        CommitmentQueue::SIZE,
        &rent,
    )?;
    create_pda_account(
        &admin,
        &roots_ring,
        &program_id,
        b"roots_ring",
        RootsRing::SIZE,
        &rent,
    )?;
    const NULLIFIER_SHARD_SPACE: usize =
        NullifierShard::MIN_SIZE + NullifierShard::MAX_NULLIFIERS * 32;
    create_pda_account(
        &admin,
        &nullifier_shard,
        &program_id,
        b"nullifier_shard",
        NULLIFIER_SHARD_SPACE,
        &rent,
    )?;
    create_pda_account(&admin, &treasury, &program_id, b"treasury", 0, &rent)?;
    Ok(())
}

#[inline(always)]
fn create_pda_account(
    admin: &AccountInfo,
    target: &AccountInfo,
    program_id: &Pubkey,
    seed: &'static [u8],
    space: usize,
    rent: &Rent,
) -> ProgramResult {
    let (expected_address, bump) = find_program_address(&[seed], &ID);
    if target.key() != &expected_address {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if target.lamports() > 0 {
        if target.owner() != program_id {
            return Err(ShieldPoolError::BadAccounts.into());
        }

        if target.data_len() != space {
            return Err(ShieldPoolError::BadAccounts.into());
        }

        return Ok(());
    }

    let lamports = rent.minimum_balance(space);
    let bump_seed = [bump];
    let seeds = [Seed::from(seed), Seed::from(bump_seed.as_ref())];
    let signer = Signer::from(&seeds);

    CreateAccount {
        from: admin,
        to: target,
        lamports,
        space: space as u64,
        owner: program_id,
    }
    .invoke_signed(&[signer])
}