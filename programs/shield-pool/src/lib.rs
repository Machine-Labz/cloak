use crate::instructions::{
    admin_push_root::process_admin_push_root_instruction, deposit::process_deposit_instruction,
    withdraw::process_withdraw_instruction, ShieldPoolInstruction,
};
use five8_const::decode_32_const;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    {entrypoint, ProgramResult},
};

mod constants;
mod error;
mod instructions;
mod state;

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
        ShieldPoolInstruction::Deposit => process_deposit_instruction(accounts, instruction_data)?,
        ShieldPoolInstruction::AdminPushRoot => {
            process_admin_push_root_instruction(accounts, instruction_data)?
        }
        ShieldPoolInstruction::Withdraw => {
            process_withdraw_instruction(accounts, instruction_data)?
        }
    }

    Ok(())
}
