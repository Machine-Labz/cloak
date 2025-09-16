use crate::constants::FEE_BASIS_POINTS_DENOMINATOR;
use crate::error::ShieldPoolError;
use crate::instruction_data::WithdrawIx;
use crate::state::{NullifierShard, RootsRing};
use crate::utils::compute_outputs_hash_blake3;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

/// SP1 Withdraw Circuit VKey Hash
const WITHDRAW_VKEY_HASH: &str = env!("VKEY_HASH");

pub fn process_withdraw_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipients..., system]
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program_info] =
        accounts
    else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    let withdraw_data: WithdrawIx = WithdrawIx::from_instruction_data(instruction_data);

    // let roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    // let mut nullifier_shard = NullifierShard::from_account_info(nullifier_shard_info)?;
    let computed_outputs_hash = compute_outputs_hash_blake3(&withdraw_data.outputs())?;
    // let total_output: u64 = withdraw_data.outputs().iter().map(|o| o.amount()).sum();
    // let fee = withdraw_data
    //     .public_amount()
    //     .checked_mul(withdraw_data.public_fee_bps() as u64)
    //     .ok_or(ShieldPoolError::MathOverflow)?
    //     .checked_div(FEE_BASIS_POINTS_DENOMINATOR)
    //     .ok_or(ShieldPoolError::DivisionByZero)?;
    // let expected_total = total_output
    //     .checked_add(fee)
    //     .ok_or(ShieldPoolError::MathOverflow)?;

    // unsafe {
    //     *pool_info.borrow_mut_lamports_unchecked() -= withdraw_data.public_amount();
    //     *recipient_account.borrow_mut_lamports_unchecked() += total_output;
    //     *treasury_info.borrow_mut_lamports_unchecked() += fee;
    // }

    // nullifier_shard.add_nullifier(&withdraw_data.public_nf())?;

    Ok(())
}
