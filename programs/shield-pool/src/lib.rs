use five8_const::decode_32_const;
use instructions::{
    admin_push_root::process_admin_push_root_instruction, deposit::process_deposit_instruction,
    withdraw::process_withdraw_instruction, ShieldPoolInstruction,
};
use pinocchio::{
    account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};
use solana_blake3_hasher as blake3;

use crate::constants::HASH_SIZE;

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
        ShieldPoolInstruction::Deposit => process_deposit_instruction(accounts, instruction_data),
        ShieldPoolInstruction::AdminPushRoot => {
            process_admin_push_root_instruction(accounts, instruction_data)
        }
        ShieldPoolInstruction::Withdraw => process_withdraw_instruction(accounts, instruction_data),
    }
}

pub fn compute_outputs_hash_blake3(
    recipient: &Pubkey,
    amount: u64,
) -> Result<[u8; HASH_SIZE], ProgramError> {
    // Prepare input data for BLAKE3
    let mut input_data = Vec::new();
    input_data.extend_from_slice(recipient.as_ref());
    input_data.extend_from_slice(&amount.to_le_bytes());

    // For now, return a mock hash (in real implementation, would use BLAKE3 syscall)
    let mut hash_result = [0u8; HASH_SIZE];

    // unsafe {
    //     // Prepare arguments for the syscall
    //     // r1: pointer to input data
    //     // r2: input data length (u64)
    //     // r3: pointer to output buffer
    //     // sol_blake3 syscall expects:
    //     //   r1: *const u8 (input ptr)
    //     //   r2: u64 (input len)
    //     //   r3: *mut u8 (output ptr)
    //     //   returns: 0 on success

    //     let input_ptr = input_data.as_ptr();
    //     let input_len = input_data.len() as u64;
    //     let output_ptr = hash_result.as_mut_ptr();

    //     // Use inline assembly to call the sol_blake3 syscall
    //     // The syscall number for sol_blake3 is 211
    //     // See: https://docs.solana.com/developing/runtime-facilities/syscalls#blake3
    //     // Syscall convention: r7 = syscall number

    //     let syscall_number: u64 = 211;
    //     let mut result: u64;

    //     asm!(
    //         "mov r1, {0}",
    //         "mov r2, {1}",
    //         "mov r3, {2}",
    //         "mov r7, {3}",
    //         "svc 0",
    //         "mov {4}, r0",
    //         in(reg) input_ptr,
    //         in(reg) input_len,
    //         in(reg) output_ptr,
    //         in(reg) syscall_number,
    //         lateout(reg) result,
    //         options(nostack)
    //     );

    //     if result != 0 {
    //         return Err(ProgramError::Custom(result as u32));
    //     }
    // }

    // // Simple hash simulation - in production, use proper BLAKE3 syscall
    // for (i, &byte) in input_data.iter().enumerate() {
    //     hash_result[i % HASH_SIZE] ^= byte;
    // }

    // Use blake3 to hash the input data and copy the result into hash_result
    let hash = blake3::hash(&input_data);
    hash_result.copy_from_slice(hash.as_bytes());

    msg!(&format!("hash_result: {:?}", hash_result));

    Ok(hash_result)
}
