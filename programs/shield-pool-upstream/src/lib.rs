#![cfg_attr(target_arch = "bpf", no_std)]
use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

pub mod constants;
pub mod error;
pub mod groth16;
pub mod instructions;
pub mod state;

#[cfg(test)]
mod tests;

use five8_const::decode_32_const;
pub use instructions::*;
use state::Context;

program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

// Shield Pool Program ID
pub const ID: [u8; 32] = decode_32_const("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");

#[inline(always)]
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, context) = instruction_data
        .split_first()
        .map(|(discriminator, instruction_data)| {
            (
                discriminator,
                Context::from((program_id, accounts, instruction_data)),
            )
        })
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => Deposit::try_from(context)?.execute(),
        1 => AdminPushRoot::try_from(context)?.execute(),
        2 => Withdraw::try_from(context)?.execute(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
