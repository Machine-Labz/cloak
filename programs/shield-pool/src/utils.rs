use crate::constants::*;
use crate::instruction_data::WithdrawOutput;
use blake3::Hasher;
use pinocchio::program_error::ProgramError;

/// Compute outputs hash using BLAKE3
pub fn compute_outputs_hash_blake3(
    outputs: &[WithdrawOutput],
) -> Result<[u8; HASH_SIZE], ProgramError> {
    let mut hasher = Hasher::new();

    for output in outputs {
        hasher.update(output.recipient().as_ref());
        hasher.update(&output.amount().to_le_bytes());
    }

    Ok(hasher.finalize().into())
}
