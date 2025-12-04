use five8_const::decode_32_const;
use instructions::{
    consume_claim::process_consume_claim_instruction,
    initialize::{process_initialize_registry_instruction, process_register_miner_instruction},
    mine_claim::process_mine_claim_instruction,
    reveal_claim::process_reveal_claim_instruction,
    top_up_escrow::process_top_up_escrow_instruction,
};
use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instructions::ScrambleRegistryInstruction;

mod constants;
mod error;
pub mod instructions;
mod state;
mod utils;

pub use state::{Claim, ClaimStatus, Miner, ScrambleRegistry};

#[cfg(test)]
mod tests;

pub const ID: [u8; 32] = decode_32_const("9yoeUduVanEN5RGp144Czfa5GXNiLdGmDMAboM4vfqsm");

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
        ScrambleRegistryInstruction::TopUpEscrow => {
            process_top_up_escrow_instruction(accounts, instruction_data)
        }
    }
}
