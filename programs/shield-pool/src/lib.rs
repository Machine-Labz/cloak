use crate::instructions::ShieldPoolInstruction;
use instructions::{
    admin_push_root::process_admin_push_root_instruction,
    batch_withdraw::process_batch_withdraw_instruction, deposit::process_deposit_instruction,
    withdraw::process_withdraw_instruction,
};
use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::constants::GLOBAL_PROGRAM_ID;
pub mod constants;
mod error;
pub mod instructions;
mod state;

#[cfg(test)]
mod tests;

// Shield Pool Program ID
const ID: [u8; 32] = GLOBAL_PROGRAM_ID;

program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (instruction_discriminant, instruction_data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match ShieldPoolInstruction::try_from(instruction_discriminant)? {
        ShieldPoolInstruction::Deposit => process_deposit_instruction(accounts, instruction_data),
        ShieldPoolInstruction::AdminPushRoot => {
            process_admin_push_root_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Withdraw => process_withdraw_instruction(accounts, instruction_data),
        ShieldPoolInstruction::BatchWithdraw => {
            process_batch_withdraw_instruction(accounts, instruction_data)
        }
    }
}
