use crate::instructions::ScrambleRegistryInstruction;
use five8_const::decode_32_const;
use instructions::{
    consume_claim::process_consume_claim_instruction,
    initialize::{process_initialize_registry_instruction, process_register_miner_instruction},
    mine_claim::process_mine_claim_instruction,
    reveal_claim::process_reveal_claim_instruction,
};
use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

mod constants;
mod error;
pub mod instructions;
mod state;
mod utils;

pub use state::{Claim, ClaimStatus, Miner, ScrambleRegistry};

#[cfg(test)]
mod tests;

// Scramble Registry Program ID
const ID: [u8; 32] = decode_32_const("EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6");

program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

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

    match ScrambleRegistryInstruction::try_from(instruction_discriminant)? {
        ScrambleRegistryInstruction::InitializeRegistry => {
            process_initialize_registry_instruction(accounts, instruction_data)
        }
        ScrambleRegistryInstruction::RegisterMiner => {
            process_register_miner_instruction(accounts, instruction_data)
        }
        ScrambleRegistryInstruction::MineClaim => {
            process_mine_claim_instruction(accounts, instruction_data)
        }
        ScrambleRegistryInstruction::RevealClaim => {
            process_reveal_claim_instruction(accounts, instruction_data)
        }
        ScrambleRegistryInstruction::ConsumeClaim => {
            process_consume_claim_instruction(accounts, instruction_data)
        }
    }
}
