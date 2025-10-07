use crate::constants::{PUB_LEN, WITHDRAW_VKEY_HASH};
use crate::error::ShieldPoolError;
use crate::ID;
use pinocchio::{account_info::AccountInfo, ProgramResult};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

/// Batch withdraw: verify single proof for N withdrawals
/// Data: proof(260) + public_values(N×104) + num_w(1) + [num_outputs(1) + recipient(32) + amount(8)]×N
pub fn process_batch_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if accounts.len() < 6 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, _rest @ ..] = accounts
    else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    if pool_info.owner() != &ID {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }
    if !pool_info.is_writable() || !treasury_info.is_writable() {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    if data.len() < 261 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let sp1_proof = &data[0..260];
    let mut num_withdrawals = 0usize;
    let mut public_values_size = 0usize;

    for n in 1..=20 {
        let pub_size = n * PUB_LEN;
        let num_w_offset = 260 + pub_size;

        if data.len() > num_w_offset {
            let candidate_num = data[num_w_offset] as usize;
            let expected_total = 260 + pub_size + 1 + (candidate_num * 41);

            if data.len() >= expected_total && candidate_num == n {
                num_withdrawals = candidate_num;
                public_values_size = pub_size;
                break;
            }
        }
    }

    if num_withdrawals == 0 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let sp1_public_values = &data[260..260 + public_values_size];

    verify_proof(
        sp1_proof,
        sp1_public_values,
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    // Load state accounts once
    let roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;
    let mut nullifier_shard =
        crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;

    // Offset where withdrawal data starts: proof(260) + public_values(N×104) + num_w(1)
    let withdrawals_start = 260 + public_values_size + 1;

    // Process each withdrawal
    for i in 0..num_withdrawals {
        unsafe {
            // Extract public inputs from sp1_public_values (already verified in proof!)
            let pub_offset = i * PUB_LEN;
            let public_amount = *((sp1_public_values.as_ptr().add(pub_offset + 96)) as *const u64);
            let root = *((sp1_public_values.as_ptr().add(pub_offset)) as *const [u8; 32]);
            let nf = *((sp1_public_values.as_ptr().add(pub_offset + 32)) as *const [u8; 32]);
            let outputs_hash_public =
                *((sp1_public_values.as_ptr().add(pub_offset + 64)) as *const [u8; 32]);

            // Extract withdrawal-specific data (41 bytes: num_outputs + recipient + amount)
            let wd_offset = withdrawals_start + (i * 41);
            let recipient_addr = *((data.as_ptr().add(wd_offset + 1)) as *const [u8; 32]);
            let recipient_amount = *((data.as_ptr().add(wd_offset + 33)) as *const u64);

            // Fast validation - early returns instead of assertions
            if !roots_ring.contains_root(&root) {
                return Err(ShieldPoolError::RootNotFound.into());
            }
            if nullifier_shard.contains_nullifier(&nf) {
                return Err(ShieldPoolError::DoubleSpend.into());
            }

            // Bind outputs_hash to actual recipient and amount
            let mut buf = [0u8; 32 + 8];
            buf[..32].copy_from_slice(&recipient_addr);
            buf[32..40].copy_from_slice(&recipient_amount.to_le_bytes());
            let outputs_hash_local = *blake3::hash(&buf).as_bytes();

            if outputs_hash_local != outputs_hash_public {
                return Err(ShieldPoolError::InvalidOutputsHash.into());
            }

            // Validate amounts and calculate fee
            if recipient_amount > public_amount {
                return Err(ShieldPoolError::InvalidAmount.into());
            }

            let total_fee = public_amount - recipient_amount;
            let expected_fee = 2_500_000 + ((public_amount * 5) / 1_000);
            if total_fee != expected_fee {
                return Err(ShieldPoolError::Conservation.into());
            }

            if pool_info.lamports() < public_amount {
                return Err(ShieldPoolError::InsufficientLamports.into());
            }

            // Record nullifier
            nullifier_shard.add_nullifier(&nf)?;

            // Get recipient - direct array access
            let recipient_account = accounts
                .get(5 + i)
                .ok_or(ShieldPoolError::MissingAccounts)?;
            if !recipient_account.is_writable() {
                return Err(ShieldPoolError::RecipientNotWritable.into());
            }

            // Perform lamport transfers
            *pool_info.borrow_mut_lamports_unchecked() -= public_amount;
            *recipient_account.borrow_mut_lamports_unchecked() += recipient_amount;
            *treasury_info.borrow_mut_lamports_unchecked() += total_fee;
        }
    }

    Ok(())
}
