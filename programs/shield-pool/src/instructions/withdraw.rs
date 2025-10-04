use crate::constants::{
    PROOF_LEN, PROOF_OFF, PUB_AMOUNT_OFF, PUB_LEN, PUB_NF_OFF, PUB_OFF, PUB_OUT_HASH_OFF,
    PUB_ROOT_OFF, RECIP_ADDR_LEN, RECIP_AMT_OFF, RECIP_OFF, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use pinocchio::{account_info::AccountInfo, ProgramResult};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipient, system]
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program] =
        accounts
    else {
        return Err(ShieldPoolError::MissingAccounts.into());
    };

    // Pool should be a system program account (like a wallet), not program-owned
    // No ownership check needed for pool account

    // Writable checks
    if !pool_info.is_writable() || !treasury_info.is_writable() || !recipient_account.is_writable()
    {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Instruction data consists of:
    // 0 to 259: proof (260 bytes)
    // 260 to 363: public inputs (104 bytes)
    // 364 to 395: nullifier (32 bytes)
    // 396: number of outputs (1 byte)
    // 397 to 428: recipient address (32 bytes)
    // 429 to 436: recipient amount (8 bytes)
    // Total length: 437 (as in working version)

    if data.len() < 437 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Extract proof and public inputs using constants
    let sp1_proof: &[u8] = &data[PROOF_OFF..(PROOF_OFF + PROOF_LEN)];
    let raw_public_inputs: &[u8] = &data[PUB_OFF..(PUB_OFF + PUB_LEN)];

    // Verify SP1 proof (essential for security)
    // Compress public inputs to 64 bytes for SP1 Solana verifier
    let compressed_public_inputs = &raw_public_inputs[..64];

    // Use SP1 verification with our circuit's verification key hash
    verify_proof(
        sp1_proof,
        compressed_public_inputs,
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    // Extract public inputs using unaligned reads
    let public_amount = unsafe {
        core::ptr::read_unaligned(
            raw_public_inputs.as_ptr().add(PUB_AMOUNT_OFF - PUB_OFF) as *const u64
        )
    };

    // Extract root from public inputs
    let mut root = [0u8; 32];
    unsafe {
        core::ptr::copy_nonoverlapping(
            raw_public_inputs.as_ptr().add(PUB_ROOT_OFF - PUB_OFF),
            root.as_mut_ptr(),
            32,
        );
    }

    // Extract nullifier from public inputs
    let mut nf = [0u8; 32];
    unsafe {
        core::ptr::copy_nonoverlapping(
            raw_public_inputs.as_ptr().add(PUB_NF_OFF - PUB_OFF),
            nf.as_mut_ptr(),
            32,
        );
    }

    // Extract outputs_hash from public inputs
    let mut outputs_hash_public = [0u8; 32];
    unsafe {
        core::ptr::copy_nonoverlapping(
            raw_public_inputs.as_ptr().add(PUB_OUT_HASH_OFF - PUB_OFF),
            outputs_hash_public.as_mut_ptr(),
            32,
        );
    }

    // Extract recipient data
    let mut recipient_addr = [0u8; 32];
    unsafe {
        core::ptr::copy_nonoverlapping(
            data.as_ptr().add(RECIP_OFF),
            recipient_addr.as_mut_ptr(),
            RECIP_ADDR_LEN,
        ); // Bytes 397-428
    }

    let recipient_amount =
        unsafe { core::ptr::read_unaligned(data.as_ptr().add(RECIP_AMT_OFF) as *const u64) };

    // Verify root exists in RootsRing
    let roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;
    if !roots_ring.contains_root(&root) {
        return Err(ShieldPoolError::RootNotFound.into());
    }

    // Check for double-spend
    let mut shard = crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;
    if shard.contains_nullifier(&nf) {
        return Err(ShieldPoolError::DoubleSpend.into());
    }

    // Bind outputs_hash to actual recipient and amount
    let mut buf = [0u8; 32 + 8];
    unsafe {
        core::ptr::copy_nonoverlapping(recipient_addr.as_ptr(), buf.as_mut_ptr(), 32);
    }
    buf[32..40].copy_from_slice(&recipient_amount.to_le_bytes());
    let outputs_hash_local = *blake3::hash(&buf).as_bytes();

    if outputs_hash_local != outputs_hash_public {
        return Err(ShieldPoolError::InvalidOutputsHash.into());
    }

    // Validate amounts and calculate fee
    if recipient_amount > public_amount {
        return Err(ShieldPoolError::InvalidAmount.into());
    }

    let expected_fee = {
        const FIXED: u64 = 2_500_000; // 0.0025 SOL
        const VAR_NUM: u64 = 5; // 0.5%
        const VAR_DEN: u64 = 1_000; // 0.5% = 5/1000
        FIXED + ((public_amount.saturating_mul(VAR_NUM)) / VAR_DEN)
    };
    let total_fee = public_amount - recipient_amount;
    if total_fee != expected_fee {
        return Err(ShieldPoolError::Conservation.into());
    }

    // Check pool has sufficient balance
    if pool_info.lamports() < public_amount {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    // Record nullifier before moving funds (fail-closed)
    shard.add_nullifier(&nf)?;

    // Perform lamport transfers
    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() -= public_amount; // Move funds from pool to recipient
        *recipient_account.borrow_mut_lamports_unchecked() += recipient_amount; // Move funds from pool to recipient
        *treasury_info.borrow_mut_lamports_unchecked() += total_fee; // Move funds from pool to treasury
    }

    Ok(())
}
