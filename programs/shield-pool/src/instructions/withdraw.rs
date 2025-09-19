use crate::compute_outputs_hash_blake3;
use crate::constants::{
    FEE_BASIS_POINTS_DENOMINATOR, HASH_SIZE, PUBKEY_SIZE, SP1_PROOF_SIZE, SP1_PUBLIC_INPUTS_SIZE,
};
use crate::error::ShieldPoolError;
use crate::state::{NullifierShard, RootsRing};
use pinocchio::{account_info::AccountInfo, msg, pubkey::Pubkey, ProgramResult};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

/// SP1 Withdraw Circuit VKey Hash
const WITHDRAW_VKEY_HASH: &str =
    "004e55c1fe353704d5c7eb1a2f4df449da8c1707127e54b4c1a5b54535fc0366";

pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!(&format!("withdraw_instruction_data:{}", hex::encode(data)));

    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipients..., system]
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program_info] =
        accounts
    else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    unsafe {
        // Parse instruction data layout:
        // sp1_proof (256 bytes) + sp1_public_inputs (64 bytes) + public_root (32 bytes) +
        // public_nf (32 bytes) + public_amount (8 bytes) + public_fee_bps (2 bytes) +
        // public_outputs_hash (32 bytes) + num_outputs (1 byte) + outputs (variable)

        // Check minimum data size
        let min_data_size = SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 3 + 8 + 2 + 1 + PUBKEY_SIZE + 8;
        if data.len() < min_data_size {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        // Read the SP1 proof and public inputs from the instruction data
        let sp1_proof = &data[0..SP1_PROOF_SIZE];
        let sp1_public_inputs = &data[SP1_PROOF_SIZE..SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE];
        
        // Read the specific values we need for validation
        let data_offset = SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE;
        let public_root = *((data.as_ptr()).add(data_offset) as *const [u8; HASH_SIZE]);
        let public_nf = *((data.as_ptr()).add(data_offset + HASH_SIZE) as *const [u8; HASH_SIZE]);
        let public_amount = *((data.as_ptr()).add(data_offset + HASH_SIZE * 2) as *const u64);
        let public_fee_bps = *((data.as_ptr()).add(data_offset + HASH_SIZE * 2 + 8) as *const u16);
        let _public_outputs_hash = *((data.as_ptr()).add(data_offset + HASH_SIZE * 2 + 8 + 2) as *const [u8; HASH_SIZE]);
        let num_outputs = *((data.as_ptr()).add(data_offset + HASH_SIZE * 3 + 8 + 2) as *const u8);

        // For simplicity, assume single output (can be extended later)
        if num_outputs != 1 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        let output_offset = data_offset + HASH_SIZE * 3 + 8 + 2 + 1;
        let recipient_pubkey = *((data.as_ptr()).add(output_offset) as *const Pubkey);
        let recipient_amount = *((data.as_ptr()).add(output_offset + PUBKEY_SIZE) as *const u64);

        // 1. Verify SP1 proof
        verify_proof(
            sp1_proof,
            sp1_public_inputs,
            WITHDRAW_VKEY_HASH,
            GROTH16_VK_5_0_0_BYTES,
        )
        .map_err(|_| ShieldPoolError::ProofInvalid)?;

        // 2. Check root exists in RootsRing
        let roots_ring = RootsRing::from_account_info(roots_ring_info)?;
        if !roots_ring.contains_root(&public_root) {
            return Err(ShieldPoolError::RootNotFound.into());
        }

        // 3. Check for double-spend
        let mut nullifier_shard = NullifierShard::from_account_info(nullifier_shard_info)?;
        if nullifier_shard.contains_nullifier(&public_nf) {
            return Err(ShieldPoolError::DoubleSpend.into());
        }

        // 4. Verify outputs hash
        let computed_outputs_hash =
            compute_outputs_hash_blake3(&recipient_pubkey, recipient_amount)?;
        if computed_outputs_hash != _public_outputs_hash {
            return Err(ShieldPoolError::OutputsMismatch.into());
        }

        // 5. Calculate fee
        let fee = public_amount
            .checked_mul(public_fee_bps as u64)
            .ok_or(ShieldPoolError::MathOverflow)?
            .checked_div(FEE_BASIS_POINTS_DENOMINATOR)
            .ok_or(ShieldPoolError::DivisionByZero)?;

        // 6. Verify conservation (recipient_amount + fee == public_amount)
        let expected_total = recipient_amount
            .checked_add(fee)
            .ok_or(ShieldPoolError::MathOverflow)?;

        if expected_total != public_amount {
            return Err(ShieldPoolError::Conservation.into());
        }

        // 7. Transfer lamports
        // Debit pool
        *pool_info.borrow_mut_lamports_unchecked() = pool_info
            .lamports()
            .checked_sub(public_amount)
            .ok_or(ShieldPoolError::InsufficientLamports)?;

        // Credit recipient
        *recipient_account.borrow_mut_lamports_unchecked() = recipient_account
            .lamports()
            .checked_add(recipient_amount)
            .ok_or(ShieldPoolError::MathOverflow)?;

        // Credit treasury with fee
        *treasury_info.borrow_mut_lamports_unchecked() = treasury_info
            .lamports()
            .checked_add(fee)
            .ok_or(ShieldPoolError::MathOverflow)?;

        // 8. Record nullifier
        nullifier_shard.add_nullifier(&public_nf)?;
    }

    Ok(())
}
