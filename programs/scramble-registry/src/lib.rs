pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

pub use constants::*;
pub use error::ScrambleError;
pub use instructions::{
    process_consume_claim, process_initialize_registry, process_mine_claim,
    process_register_miner, process_reveal_claim,
};
pub use state::{Claim, ClaimStatus, Miner, ScrambleRegistry};
pub use utils::{hash_pow_preimage, u256_lt, verify_pow};

use pinocchio::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

entrypoint!(process_instruction);

/// Program entrypoint
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let (discriminator, args) = instruction_data.split_at(1);

    match discriminator[0] {
        // 0: initialize_registry
        0 => {
            if args.len() < 32 + 32 + 32 + 8 + 2 + 8 + 8 + 2 {
                return Err(ProgramError::InvalidInstructionData);
            }

            let initial_difficulty: [u8; 32] = args[0..32].try_into().unwrap();
            let min_difficulty: [u8; 32] = args[32..64].try_into().unwrap();
            let max_difficulty: [u8; 32] = args[64..96].try_into().unwrap();
            let target_interval_slots = u64::from_le_bytes(args[96..104].try_into().unwrap());
            let fee_share_bps = u16::from_le_bytes(args[104..106].try_into().unwrap());
            let reveal_window = u64::from_le_bytes(args[106..114].try_into().unwrap());
            let claim_window = u64::from_le_bytes(args[114..122].try_into().unwrap());
            let max_k = u16::from_le_bytes(args[122..124].try_into().unwrap());

            process_initialize_registry(
                program_id,
                accounts,
                initial_difficulty,
                min_difficulty,
                max_difficulty,
                target_interval_slots,
                fee_share_bps,
                reveal_window,
                claim_window,
                max_k,
            )
        }

        // 1: register_miner
        1 => process_register_miner(accounts),

        // 2: mine_claim
        2 => {
            if args.len() < 8 + 32 + 32 + 16 + 32 + 2 {
                return Err(ProgramError::InvalidInstructionData);
            }

            let slot = u64::from_le_bytes(args[0..8].try_into().unwrap());
            let slot_hash: [u8; 32] = args[8..40].try_into().unwrap();
            let batch_hash: [u8; 32] = args[40..72].try_into().unwrap();
            let nonce = u128::from_le_bytes(args[72..88].try_into().unwrap());
            let proof_hash: [u8; 32] = args[88..120].try_into().unwrap();
            let max_consumes = u16::from_le_bytes(args[120..122].try_into().unwrap());

            process_mine_claim(
                accounts,
                slot,
                slot_hash,
                batch_hash,
                nonce,
                proof_hash,
                max_consumes,
            )
        }

        // 3: reveal_claim
        3 => process_reveal_claim(accounts),

        // 4: consume_claim
        4 => {
            if args.len() < 32 + 32 {
                return Err(ProgramError::InvalidInstructionData);
            }

            let expected_miner_authority: [u8; 32] = args[0..32].try_into().unwrap();
            let expected_batch_hash: [u8; 32] = args[32..64].try_into().unwrap();

            process_consume_claim(accounts, expected_miner_authority, expected_batch_hash)
        }

        _ => {
            msg!("Unknown instruction");
            Err(ProgramError::InvalidInstructionData)
        }
    }
}
