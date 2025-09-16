use std::fmt::format;

use five8_const::decode_32_const;
use pinocchio::{
    account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

mod constants;
mod error;
mod instruction_data;
mod instructions;
mod state;
mod utils;

use instructions::{
    admin_push_root::process_admin_push_root_instruction, deposit::process_deposit_instruction,
    withdraw::process_withdraw_instruction, ShieldPoolInstruction,
};

// Re-export commonly used types
pub use instruction_data::{AdminPushRootIx, DepositIx, WithdrawIx, WithdrawOutput};

#[cfg(test)]
mod tests;

// Shield Pool Program ID - placeholder for now
const ID: [u8; 32] = decode_32_const("99999999999999999999999999999999999999999999");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (instruction_discriminant, instruction_data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match ShieldPoolInstruction::try_from(instruction_discriminant)? {
        ShieldPoolInstruction::Deposit => process_deposit_instruction(accounts, instruction_data),
        ShieldPoolInstruction::AdminPushRoot => {
            process_admin_push_root_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Withdraw => process_withdraw_instruction(accounts, instruction_data),
    }
}
