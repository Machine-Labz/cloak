use crate::constants::*;
use crate::error::ShieldPoolError;
use crate::state::{NullifierShard, RootsRing};
use blake3::Hasher;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

/// SP1 Withdraw Circuit VKey Hash - loaded from vkey_hash.txt file at build time
/// Falls back to hardcoded value if file not found
const WITHDRAW_VKEY_HASH: &str = env!("VKEY_HASH");

pub fn process_withdraw_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse accounts - expecting: [pool, treasury, roots_ring, nullifier_shard, recipients..., system]
    if accounts.len() < 5 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let pool_info = &accounts[0];
    let treasury_info = &accounts[1];
    let roots_ring_info = &accounts[2];
    let nullifier_shard_info = &accounts[3];
    let system_program_info = accounts.last().unwrap();
    let recipient_accounts = &accounts[4..accounts.len() - 1];

    // Parse instruction data
    let withdraw_data = WithdrawInstructionData::parse(instruction_data)?;

    // 1. Verify SP1 proof
    verify_proof(
        &withdraw_data.sp1_proof,
        &withdraw_data.sp1_public_inputs,
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    // 2. Check root exists in RootsRing
    let roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    if !roots_ring.contains_root(&withdraw_data.public_root) {
        return Err(ShieldPoolError::RootNotFound.into());
    }

    // 3. Check for double-spend
    let mut nullifier_shard = NullifierShard::from_account_info(nullifier_shard_info)?;
    if nullifier_shard.contains_nullifier(&withdraw_data.public_nf) {
        return Err(ShieldPoolError::DoubleSpend.into());
    }

    // 4. Recompute outputs hash and verify
    let computed_outputs_hash = compute_outputs_hash_blake3(&withdraw_data.outputs)?;
    if computed_outputs_hash != withdraw_data.public_outputs_hash {
        return Err(ShieldPoolError::OutputsMismatch.into());
    }

    // 5. Verify conservation (sum of outputs + fee == amount)
    let total_output: u64 = withdraw_data.outputs.iter().map(|o| o.amount).sum();
    let fee = withdraw_data
        .public_amount
        .checked_mul(withdraw_data.public_fee_bps as u64)
        .ok_or(ShieldPoolError::MathOverflow)?
        .checked_div(FEE_BASIS_POINTS_DENOMINATOR)
        .ok_or(ShieldPoolError::DivisionByZero)?;

    let expected_total = total_output
        .checked_add(fee)
        .ok_or(ShieldPoolError::MathOverflow)?;

    if expected_total != withdraw_data.public_amount {
        return Err(ShieldPoolError::Conservation.into());
    }

    // 6. Verify recipient accounts match outputs
    if recipient_accounts.len() != withdraw_data.outputs.len() {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    for (recipient_info, output) in recipient_accounts.iter().zip(withdraw_data.outputs.iter()) {
        if *recipient_info.key() != output.recipient {
            return Err(ShieldPoolError::InvalidRecipient.into());
        }
    }

    // 7. Transfer lamports
    // Debit pool
    *pool_info.try_borrow_mut_lamports()? = pool_info
        .lamports()
        .checked_sub(withdraw_data.public_amount)
        .ok_or(ShieldPoolError::InsufficientLamports)?;

    // Credit recipients
    for (recipient_info, output) in recipient_accounts.iter().zip(withdraw_data.outputs.iter()) {
        *recipient_info.try_borrow_mut_lamports()? = recipient_info
            .lamports()
            .checked_add(output.amount)
            .ok_or(ShieldPoolError::MathOverflow)?;
    }

    // Credit treasury with fee
    *treasury_info.try_borrow_mut_lamports()? = treasury_info
        .lamports()
        .checked_add(fee)
        .ok_or(ShieldPoolError::MathOverflow)?;

    // 8. Record nullifier
    nullifier_shard.add_nullifier(&withdraw_data.public_nf)?;

    Ok(())
}

struct WithdrawInstructionData {
    sp1_proof: [u8; SP1_PROOF_SIZE],
    sp1_public_inputs: [u8; SP1_PUBLIC_INPUTS_SIZE],
    public_root: [u8; HASH_SIZE],
    public_nf: [u8; HASH_SIZE],
    public_amount: u64,
    public_fee_bps: u16,
    public_outputs_hash: [u8; HASH_SIZE],
    outputs: Vec<Output>,
}

#[derive(Clone)]
struct Output {
    recipient: Pubkey,
    amount: u64,
}

impl WithdrawInstructionData {
    fn parse(data: &[u8]) -> Result<Self, ProgramError> {
        let mut offset = 0;

        if data.len() < SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 3 + 8 + 2 + 1 {
            return Err(ShieldPoolError::BadIxLength.into());
        }

        // Parse SP1 proof (256 bytes)
        let sp1_proof = data[offset..offset + SP1_PROOF_SIZE]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidProofSize)?;
        offset += SP1_PROOF_SIZE;

        // Parse SP1 public inputs (64 bytes)
        let sp1_public_inputs = data[offset..offset + SP1_PUBLIC_INPUTS_SIZE]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidPublicInputs)?;
        offset += SP1_PUBLIC_INPUTS_SIZE;

        // Parse public root (32 bytes)
        let public_root = data[offset..offset + HASH_SIZE]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
        offset += HASH_SIZE;

        // Parse public nullifier (32 bytes)
        let public_nf = data[offset..offset + HASH_SIZE]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
        offset += HASH_SIZE;

        // Parse amount (8 bytes LE)
        let public_amount = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        );
        offset += 8;

        // Parse fee_bps (2 bytes LE)
        let public_fee_bps = u16::from_le_bytes(
            data[offset..offset + 2]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        );
        offset += 2;

        // Parse outputs hash (32 bytes)
        let public_outputs_hash = data[offset..offset + HASH_SIZE]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
        offset += HASH_SIZE;

        // Parse number of outputs (1 byte)
        if offset >= data.len() {
            return Err(ShieldPoolError::BadIxLength.into());
        }
        let num_outputs = data[offset];
        offset += 1;

        // Parse outputs
        let mut outputs = Vec::with_capacity(num_outputs as usize);
        for _ in 0..num_outputs {
            if offset + PUBKEY_SIZE + 8 > data.len() {
                return Err(ShieldPoolError::BadIxLength.into());
            }

            let recipient = data[offset..offset + PUBKEY_SIZE]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
            offset += PUBKEY_SIZE;

            let amount = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
            );
            offset += 8;

            outputs.push(Output { recipient, amount });
        }

        Ok(WithdrawInstructionData {
            sp1_proof,
            sp1_public_inputs,
            public_root,
            public_nf,
            public_amount,
            public_fee_bps,
            public_outputs_hash,
            outputs,
        })
    }
}

fn compute_outputs_hash_blake3(outputs: &[Output]) -> Result<[u8; HASH_SIZE], ProgramError> {
    let mut hasher = Hasher::new();

    for output in outputs {
        hasher.update(output.recipient.as_ref());
        hasher.update(&output.amount.to_le_bytes());
    }

    Ok(hasher.finalize().into())
}
