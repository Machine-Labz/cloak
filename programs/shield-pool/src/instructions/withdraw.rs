use crate::constants::{
    FEE_BASIS_POINTS_DENOMINATOR, FIXED_FEE_LAMPORTS, HASH_SIZE, PUBKEY_SIZE, SP1_PROOF_SIZE, SP1_PUBLIC_INPUTS_SIZE, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use crate::state::{NullifierShard, RootsRing};
use pinocchio::program_error::ProgramError;
use pinocchio::{account_info::AccountInfo, msg, pubkey::Pubkey, ProgramResult};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};
use hex;


pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipients..., system]
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program] =
        accounts
    else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    unsafe {
        // Parse instruction data layout:
        // sp1_proof (256 bytes) + sp1_public_inputs (64 bytes) + public_root (32 bytes) +
        // public_nf (32 bytes) + public_amount (8 bytes) + public_fee_bps (2 bytes) +
        // public_outputs_hash (32 bytes) + num_outputs (1 byte) + outputs (variable)

        // Check minimum data size (we need at least the proof + public inputs + other data)
        let min_data_size =
            260 + 104 + HASH_SIZE + 1 + PUBKEY_SIZE + 8; // 260-byte proof + 104-byte public inputs + nullifier + num_outputs + recipient + amount
        if data.len() < min_data_size {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        
        // Read the SP1 proof and public inputs from the instruction data
        // Discriminator is already removed in lib.rs
        let sp1_proof_full = &data[..260]; // 260 bytes proof (with vkey hash)
        let sp1_public_inputs = &data[260..260 + 104]; // 104 bytes public inputs (our format)
        
        // Parse the public inputs in the correct order:
        // root(32) + nf(32) + outputs_hash(32) + amount(8)
        if sp1_public_inputs.len() < 104 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        
        let public_root: [u8; 32] = sp1_public_inputs[0..32].try_into().unwrap();
        let public_nf: [u8; 32] = sp1_public_inputs[32..64].try_into().unwrap();
        let _public_outputs_hash: [u8; 32] = sp1_public_inputs[64..96].try_into().unwrap();
        let public_amount = u64::from_le_bytes(sp1_public_inputs[96..104].try_into().unwrap());

        // Read the remaining instruction data
        let data_offset = 260 + 104; // 260-byte proof + 104-byte public inputs
        
        // Check if we have additional data beyond proof + public inputs
        if data.len() < data_offset + HASH_SIZE + 1 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        
        let nullifier_check: [u8; 32] = data[data_offset..data_offset + HASH_SIZE].try_into().unwrap();
        let num_outputs = data[data_offset + HASH_SIZE];

        // For simplicity, assume single output (can be extended later)
        if num_outputs != 1 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        let output_offset = data_offset + HASH_SIZE + 1; // after nullifier + num_outputs
        
        if data.len() < output_offset + PUBKEY_SIZE + 8 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        
        let recipient_pubkey: [u8; 32] = data[output_offset..output_offset + PUBKEY_SIZE].try_into().unwrap();
        let recipient_amount = u64::from_le_bytes(data[output_offset + PUBKEY_SIZE..output_offset + PUBKEY_SIZE + 8].try_into().unwrap());

        // Verify nullifier consistency
        if nullifier_check != public_nf {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        // 1. Verify SP1 proof
        // Use the full 260-byte proof directly (as in working example)
        let raw_public_inputs = sp1_public_inputs;
        verify_proof(
            sp1_proof_full,
            raw_public_inputs,
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

        // 5. Calculate fee: fixed fee + variable fee (0.005% = 5/100000)
        let variable_fee = public_amount
            .checked_mul(5) // 0.005% = 5/100000
            .ok_or(ShieldPoolError::MathOverflow)?
            .checked_div(100_000)
            .ok_or(ShieldPoolError::DivisionByZero)?;
        let total_fee = FIXED_FEE_LAMPORTS
            .checked_add(variable_fee)
            .ok_or(ShieldPoolError::MathOverflow)?;

        // 6. Verify conservation (recipient_amount + total_fee == public_amount)
        let expected_total = recipient_amount
            .checked_add(total_fee)
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
            .checked_add(total_fee)
            .ok_or(ShieldPoolError::MathOverflow)?;

        // 8. Record nullifier
        nullifier_shard.add_nullifier(&public_nf)?;
    }

    Ok(())
}

pub fn compute_outputs_hash_blake3(
    recipient: &Pubkey,
    amount: u64,
) -> Result<[u8; HASH_SIZE], ProgramError> {
    // Prepare input data for BLAKE3
    let mut input_data = Vec::new();
    input_data.extend_from_slice(recipient.as_ref());
    input_data.extend_from_slice(&amount.to_le_bytes());

    let mut hash_result = [0u8; HASH_SIZE];
    let hash = blake3::hash(&input_data);
    hash_result.copy_from_slice(hash.as_bytes());

    Ok(hash_result)
}