use crate::constants::{
    PROOF_LEN, PROOF_OFF, PUB_AMOUNT_OFF, PUB_LEN, PUB_NF_OFF, PUB_OFF, PUB_OUT_HASH_OFF,
    PUB_ROOT_OFF, RECIP_AMT_OFF, RECIP_OFF, SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use crate::ID;
use pinocchio::{account_info::AccountInfo, ProgramResult};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipient, system]
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program] =
        accounts
    else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    // Pool should be program-owned for the program to modify its lamports
    if pool_info.owner() != &ID {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }
    if !pool_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }
    if !treasury_info.is_writable() {
        return Err(ShieldPoolError::TreasuryNotWritable.into());
    }
    if !recipient_account.is_writable() {
        return Err(ShieldPoolError::RecipientNotWritable.into());
    }

    // Instruction data consists of:
    // 0 to 259: proof (260 bytes)
    // 260 to 363: public inputs (104 bytes)
    // 364 to 395: nullifier (32 bytes)
    // 396: number of outputs (1 byte)
    // 397 to 428: recipient address (32 bytes)
    // 429 to 436: recipient amount (8 bytes)
    // Total length: 437
    if data.len() < 437 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let sp1_proof = &data[PROOF_OFF..(PROOF_OFF + PROOF_LEN)];
    let raw_public_inputs = &data[PUB_OFF..(PUB_OFF + PUB_LEN)];

    verify_proof(
        sp1_proof,
        &raw_public_inputs[..SP1_PUB_LEN],
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    let (public_amount, recipient_amount, total_fee) = unsafe {
        let public_amount =
            *((raw_public_inputs.as_ptr().add(PUB_AMOUNT_OFF - PUB_OFF)) as *const u64);

        let root = *((raw_public_inputs.as_ptr().add(PUB_ROOT_OFF - PUB_OFF)) as *const [u8; 32]);

        let nf = *((raw_public_inputs.as_ptr().add(PUB_NF_OFF - PUB_OFF)) as *const [u8; 32]);

        let outputs_hash_public =
            *((raw_public_inputs.as_ptr().add(PUB_OUT_HASH_OFF - PUB_OFF)) as *const [u8; 32]);

        let recipient_addr = *((data.as_ptr().add(RECIP_OFF)) as *const [u8; 32]);

        let recipient_amount = *((data.as_ptr().add(RECIP_AMT_OFF)) as *const u64);

        let roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;

        if !roots_ring.contains_root(&root) {
            return Err(ShieldPoolError::RootNotFound.into());
        }

        let mut shard = crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;
        if shard.contains_nullifier(&nf) {
            return Err(ShieldPoolError::DoubleSpend.into());
        }

        let mut buf = [0u8; 32 + 8];
        buf[..32].copy_from_slice(&recipient_addr);
        let outputs_hash_local = *blake3::hash(&buf).as_bytes();
        if outputs_hash_local != outputs_hash_public {
            return Err(ShieldPoolError::InvalidOutputsHash.into());
        }

        if recipient_amount > public_amount {
            return Err(ShieldPoolError::InvalidAmount.into());
        }

        let expected_fee = {
            let fixed_fee = 2_500_000; // 0.0025 SOL
            let variable_fee = (public_amount * 5) / 1_000; // 0.5% = 5/1000
            fixed_fee + variable_fee
        };
        let total_fee = public_amount - recipient_amount;
        if total_fee != expected_fee {
            return Err(ShieldPoolError::Conservation.into());
        }

        if pool_info.lamports() < public_amount {
            return Err(ShieldPoolError::InsufficientLamports.into());
        }

        shard.add_nullifier(&nf)?;

        (public_amount, recipient_amount, total_fee)
    };

    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() -= public_amount;
        *recipient_account.borrow_mut_lamports_unchecked() += recipient_amount;
        *treasury_info.borrow_mut_lamports_unchecked() += total_fee;
    }

    Ok(())
}
