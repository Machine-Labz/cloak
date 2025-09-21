use crate::{constants::HASH_SIZE, instructions::ShieldPoolInstruction};
use five8_const::decode_32_const;
use instructions::{
    admin_push_root::process_admin_push_root_instruction, deposit::process_deposit_instruction,
    withdraw::process_withdraw_instruction,
};
use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use solana_blake3_hasher as blake3;

mod constants;
mod error;
mod instructions;
mod state;

#[cfg(test)]
mod tests;

// Shield Pool Program ID
const ID: [u8; 32] = decode_32_const("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");

program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // if program_id != &ID {
    //     return Err(ProgramError::IncorrectProgramId);
    // }

    let (instruction_discriminant, instruction_data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match ShieldPoolInstruction::try_from(instruction_discriminant)? {
        ShieldPoolInstruction::Initialize => {
            instructions::initialize::process_initialize_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Deposit => process_deposit_instruction(accounts, instruction_data),
        ShieldPoolInstruction::AdminPushRoot => {
            process_admin_push_root_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Withdraw => process_withdraw_instruction(accounts, instruction_data),
    }
}

