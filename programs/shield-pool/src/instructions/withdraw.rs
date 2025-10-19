use crate::constants::{
    PROOF_LEN, PROOF_OFF, PUB_AMOUNT_OFF, PUB_LEN, PUB_NF_OFF, PUB_OFF, PUB_OUT_HASH_OFF,
    PUB_ROOT_OFF, RECIP_AMT_OFF, RECIP_OFF, SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use crate::ID;
use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, program::invoke, ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipient, system,
    //                              scramble_program, claim_pda, miner_pda, registry_pda, clock]
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program, scramble_program_info, claim_pda_info, miner_pda_info, registry_pda_info, clock_sysvar_info, ..] =
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
    // 437 to 468: batch_hash (32 bytes) - NEW for PoW
    // Total length: 469
    if data.len() < 469 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Extract proof and public inputs using constants
    let sp1_proof: &[u8] = &data[PROOF_OFF..(PROOF_OFF + PROOF_LEN)];
    let raw_public_inputs: &[u8] = &data[PUB_OFF..(PUB_OFF + PUB_LEN)];

    // Verify SP1 proof
    // Use full 104-byte public inputs
    // The SP1 guest program commits: root(32) + nf(32) + outputs_hash(32) + amount(8) = 104 bytes
    let full_public_inputs = &raw_public_inputs[..SP1_PUB_LEN];

    // Use SP1 verification with our circuit's verification key hash
    verify_proof(
        sp1_proof,
        full_public_inputs,
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
        assert!(
            roots_ring.contains_root(&root),
            "Root not found in RootsRing"
        );

        let mut shard = crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;
        assert!(!shard.contains_nullifier(&nf), "Nullifier already used");

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

    // PoW: Call consume_claim CPI before transfers
    unsafe {
        // Extract batch_hash from instruction data (bytes 437-468)
        let batch_hash: &[u8; 32] = &*(data.as_ptr().add(437) as *const [u8; 32]);

        // Extract miner authority from miner PDA account data
        // Miner PDA layout: discriminator(8) + authority(32) + ...
        let miner_data = miner_pda_info.try_borrow_data()?;
        if miner_data.len() < 40 {
            return Err(ShieldPoolError::InvalidMinerAccount.into());
        }
        let miner_authority: &[u8; 32] = &*(miner_data.as_ptr().add(8) as *const [u8; 32]);

        // Build consume_claim instruction data
        // Layout: discriminator(1) + miner_authority(32) + batch_hash(32) = 65 bytes
        let mut consume_ix_data = [0u8; 65];
        consume_ix_data[0] = 4; // consume_claim discriminator
        consume_ix_data[1..33].copy_from_slice(miner_authority);
        consume_ix_data[33..65].copy_from_slice(batch_hash);

        // Build consume_claim instruction
        // Accounts: [claim_pda(W), miner_pda(W), registry_pda(W), shield_pool(S), clock]
        let account_metas = [
            AccountMeta::writable(claim_pda_info.key()),
            AccountMeta::writable(miner_pda_info.key()),
            AccountMeta::writable(registry_pda_info.key()),
            AccountMeta::readonly_signer(pool_info.key()),
            AccountMeta::readonly(clock_sysvar_info.key()),
        ];

        let consume_ix = pinocchio::instruction::Instruction {
            program_id: scramble_program_info.key(),
            accounts: &account_metas,
            data: &consume_ix_data,
        };

        // CPI to scramble-registry::consume_claim
        // This will:
        // 1. Verify claim is revealed and not expired
        // 2. Verify miner_authority and batch_hash match (anti-replay)
        // 3. Increment consumed_count
        // 4. Mark as Consumed if fully used
        invoke(
            &consume_ix,
            &[
                claim_pda_info,
                miner_pda_info,
                registry_pda_info,
                pool_info,
                clock_sysvar_info,
            ],
        )?;
    }

    // Perform lamport transfers
    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() -= public_amount; // Move funds from pool to recipient
        *recipient_account.borrow_mut_lamports_unchecked() += recipient_amount; // Move funds from pool to recipient
        *treasury_info.borrow_mut_lamports_unchecked() += total_fee; // Move funds from pool to treasury
    }

    Ok(())
}
