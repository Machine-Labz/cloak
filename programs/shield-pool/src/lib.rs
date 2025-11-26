use five8_const::decode_32_const;
use instructions::*;
use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instructions::ShieldPoolInstruction;

mod constants;
mod error;
pub mod instructions;
mod state;

pub use state::CommitmentQueue;

#[cfg(test)]
mod tests;

pub const ID: [u8; 32] = decode_32_const("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");

program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let expected_program_id = Pubkey::from(ID);
    if program_id != &expected_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (instruction_discriminant, instruction_data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match ShieldPoolInstruction::try_from(instruction_discriminant)? {
        ShieldPoolInstruction::Deposit => {
            deposit::process_deposit_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::AdminPushRoot => {
            admin_push_root::process_admin_push_root_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Withdraw => {
            withdraw::process_withdraw_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Initialize => {
            initialize::process_initialize_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::WithdrawMinerDecoy => {
            withdraw_miner_decoy::process_withdraw_miner_decoy_instruction(
                accounts,
                instruction_data,
            )
        }
    }
}
