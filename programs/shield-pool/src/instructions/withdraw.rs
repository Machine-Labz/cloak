use crate::constants::{
    DUPLICATE_NULLIFIER_LEN, NUM_OUTPUTS_LEN, POW_BATCH_HASH_LEN, PROOF_LEN, PUB_LEN,
    RECIPIENT_ADDR_LEN, RECIPIENT_AMOUNT_LEN, SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use crate::ID;
use core::convert::TryInto;
use pinocchio::cpi::invoke_signed;
use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

const BASE_TAIL_LEN: usize =
    PUB_LEN + DUPLICATE_NULLIFIER_LEN + NUM_OUTPUTS_LEN + RECIPIENT_ADDR_LEN + RECIPIENT_AMOUNT_LEN;

struct ParsedWithdraw<'a> {
    proof: &'a [u8],
    public_inputs: [u8; PUB_LEN],
    root: [u8; 32],
    nullifier: [u8; 32],
    outputs_hash: [u8; 32],
    public_amount: u64,
    recipient_address: [u8; 32],
    recipient_amount: u64,
    batch_hash: Option<[u8; 32]>,
}

fn parse_withdraw_data<'a>(
    data: &'a [u8],
    expect_batch_hash: bool,
) -> Result<ParsedWithdraw<'a>, ShieldPoolError> {
    let tail_len = BASE_TAIL_LEN
        + if expect_batch_hash {
            POW_BATCH_HASH_LEN
        } else {
            0
        };
    if data.len() <= tail_len {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let proof_len = data.len() - tail_len;
    let (proof, remainder) = data.split_at(proof_len);
    if proof.is_empty() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let (public_inputs_slice, remainder) = remainder.split_at(PUB_LEN);
    let mut public_inputs = [0u8; PUB_LEN];
    public_inputs.copy_from_slice(public_inputs_slice);

    if proof.len() != PROOF_LEN {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let (duplicate_nullifier_slice, remainder) = remainder.split_at(DUPLICATE_NULLIFIER_LEN);
    let duplicate_nullifier: [u8; 32] = duplicate_nullifier_slice
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    let (&num_outputs, remainder) = remainder
        .split_first()
        .ok_or(ShieldPoolError::InvalidInstructionData)?;
    if num_outputs != 1 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let (recipient_address_slice, remainder) = remainder.split_at(RECIPIENT_ADDR_LEN);
    let recipient_address: [u8; 32] = recipient_address_slice
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    let (recipient_amount_slice, remainder) = remainder.split_at(RECIPIENT_AMOUNT_LEN);
    let recipient_amount = u64::from_le_bytes(
        recipient_amount_slice
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    let batch_hash = if expect_batch_hash {
        let (hash_slice, remainder) = remainder.split_at(POW_BATCH_HASH_LEN);
        if !remainder.is_empty() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        Some(
            hash_slice
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        )
    } else {
        if !remainder.is_empty() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        None
    };

    let root: [u8; 32] = public_inputs[0..32]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let nullifier: [u8; 32] = public_inputs[32..64]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let outputs_hash: [u8; 32] = public_inputs[64..96]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let public_amount = u64::from_le_bytes(
        public_inputs[96..104]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    if duplicate_nullifier != nullifier {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    Ok(ParsedWithdraw {
        proof,
        public_inputs,
        root,
        nullifier,
        outputs_hash,
        public_amount,
        recipient_address,
        recipient_amount,
        batch_hash,
    })
}

pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let is_pow_mode = accounts.len() >= 13;

    if is_pow_mode {
        let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program, scramble_program_info, claim_pda_info, miner_pda_info, registry_pda_info, clock_sysvar_info, miner_authority_account, shield_pool_program_info, ..] =
            accounts
        else {
            return Err(ShieldPoolError::MissingAccounts.into());
        };

        process_withdraw_pow_mode(
            pool_info,
            treasury_info,
            roots_ring_info,
            nullifier_shard_info,
            recipient_account,
            scramble_program_info,
            claim_pda_info,
            miner_pda_info,
            registry_pda_info,
            clock_sysvar_info,
            miner_authority_account,
            shield_pool_program_info,
            data,
        )
    } else {
        let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, recipient_account, _system_program, ..] =
            accounts
        else {
            return Err(ShieldPoolError::MissingAccounts.into());
        };

        process_withdraw_legacy_mode(
            pool_info,
            treasury_info,
            roots_ring_info,
            nullifier_shard_info,
            recipient_account,
            data,
        )
    }
}

fn process_withdraw_pow_mode(
    pool_info: &AccountInfo,
    treasury_info: &AccountInfo,
    roots_ring_info: &AccountInfo,
    nullifier_shard_info: &AccountInfo,
    recipient_account: &AccountInfo,
    scramble_program_info: &AccountInfo,
    claim_pda_info: &AccountInfo,
    miner_pda_info: &AccountInfo,
    registry_pda_info: &AccountInfo,
    clock_sysvar_info: &AccountInfo,
    miner_authority_account: &AccountInfo,
    shield_pool_program_info: &AccountInfo,
    data: &[u8],
) -> ProgramResult {
    // Get shield-pool program ID (this program)
    let program_id = Pubkey::from(ID);

    // Verify shield_pool_program_info matches expected program ID and is executable
    if shield_pool_program_info.key() != &program_id {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }
    if !shield_pool_program_info.executable() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }
    if pool_info.owner() != &program_id {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }
    if !pool_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }
    if !treasury_info.is_writable() {
        return Err(ShieldPoolError::TreasuryNotWritable.into());
    }
    if roots_ring_info.owner() != &program_id {
        return Err(ShieldPoolError::RootsRingOwnerNotProgramId.into());
    }
    if nullifier_shard_info.owner() != &program_id {
        return Err(ShieldPoolError::NullifierShardOwnerNotProgramId.into());
    }
    if !nullifier_shard_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }
    if !recipient_account.is_writable() {
        return Err(ShieldPoolError::RecipientNotWritable.into());
    }
    if !miner_authority_account.is_writable() {
        return Err(ShieldPoolError::InvalidMinerAccount.into());
    }

    let parsed = parse_withdraw_data(data, true)?;

    verify_proof(
        parsed.proof,
        &parsed.public_inputs[..SP1_PUB_LEN],
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    {
        let roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;
        if !roots_ring.contains_root(&parsed.root) {
            return Err(ShieldPoolError::RootNotFound.into());
        }
    }

    {
        let mut shard = crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;
        if shard.contains_nullifier(&parsed.nullifier) {
            return Err(ShieldPoolError::DoubleSpend.into());
        }
        shard.add_nullifier(&parsed.nullifier)?;
    }

    let mut hash_input = [0u8; RECIPIENT_ADDR_LEN + RECIPIENT_AMOUNT_LEN];
    hash_input[..RECIPIENT_ADDR_LEN].copy_from_slice(&parsed.recipient_address);
    hash_input[RECIPIENT_ADDR_LEN..].copy_from_slice(&parsed.recipient_amount.to_le_bytes());
    let outputs_hash_local = blake3::hash(&hash_input);
    if outputs_hash_local.as_bytes() != &parsed.outputs_hash {
        return Err(ShieldPoolError::InvalidOutputsHash.into());
    }

    if parsed.recipient_amount > parsed.public_amount {
        return Err(ShieldPoolError::InvalidAmount.into());
    }

    let expected_fee = 2_500_000u64 + (parsed.public_amount * 5) / 1_000;
    let total_fee = parsed.public_amount - parsed.recipient_amount;
    if total_fee != expected_fee {
        return Err(ShieldPoolError::Conservation.into());
    }

    if pool_info.lamports() < parsed.public_amount {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    let batch_hash = parsed
        .batch_hash
        .ok_or(ShieldPoolError::InvalidInstructionData)?;

    // Read miner authority and drop borrow before CPI
    let miner_authority: [u8; 32] = {
        let miner_data = miner_pda_info.try_borrow_data()?;
        if miner_data.len() < 32 {
            return Err(ShieldPoolError::InvalidMinerAccount.into());
        }
        // Miner account layout: [authority: 32][total_mined: 8][total_consumed: 8][registered_at_slot: 8]
        // No discriminator - authority is at offset 0
        miner_data[0..32]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidMinerAccount)?
    }; // miner_data is dropped here

    let mut consume_ix_data = [0u8; 65];
    consume_ix_data[0] = 4; // consume_claim discriminant
    consume_ix_data[1..33].copy_from_slice(&miner_authority);
    consume_ix_data[33..65].copy_from_slice(&batch_hash);

    // For CPI: Solana runtime automatically grants signer privilege to the calling program
    // The 4th account is shield-pool program - runtime will make it signer in CPI context
    let account_metas = [
        AccountMeta::writable(claim_pda_info.key()),
        AccountMeta::writable(miner_pda_info.key()),
        AccountMeta::writable(registry_pda_info.key()),
        AccountMeta::readonly(shield_pool_program_info.key()), // Shield-pool program - will be signer in CPI
        AccountMeta::readonly(clock_sysvar_info.key()),
    ];

    let consume_ix = pinocchio::instruction::Instruction {
        program_id: scramble_program_info.key(),
        accounts: &account_metas,
        data: &consume_ix_data,
    };

    // Use invoke_signed to grant signer privilege to the calling program (shield-pool)
    invoke_signed(
        &consume_ix,
        &[
            claim_pda_info,
            miner_pda_info,
            registry_pda_info,
            shield_pool_program_info,
            clock_sysvar_info,
        ],
        &[],
    )?;

    let registry_data = registry_pda_info.try_borrow_data()?;
    if registry_data.len() < 90 {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }
    let fee_share_bps = u16::from_le_bytes(
        registry_data[88..90]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    let scrambler_share = ((total_fee as u128 * fee_share_bps as u128) / 10_000) as u64;
    let protocol_share = total_fee - scrambler_share;

    let pool_lamports = pool_info.lamports();
    let recipient_lamports = recipient_account.lamports();
    let treasury_lamports = treasury_info.lamports();
    let miner_lamports = miner_authority_account.lamports();

    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() =
            pool_lamports - parsed.public_amount;
        *recipient_account.borrow_mut_lamports_unchecked() =
            recipient_lamports + parsed.recipient_amount;
        *treasury_info.borrow_mut_lamports_unchecked() =
            treasury_lamports + protocol_share;
        *miner_authority_account.borrow_mut_lamports_unchecked() =
            miner_lamports + scrambler_share;
    }

    Ok(())
}

fn process_withdraw_legacy_mode(
    pool_info: &AccountInfo,
    treasury_info: &AccountInfo,
    roots_ring_info: &AccountInfo,
    nullifier_shard_info: &AccountInfo,
    recipient_account: &AccountInfo,
    data: &[u8],
) -> ProgramResult {
    let program_id = Pubkey::from(ID);
    if pool_info.owner() != &program_id {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }
    if !pool_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }
    if !treasury_info.is_writable() {
        return Err(ShieldPoolError::TreasuryNotWritable.into());
    }
    if roots_ring_info.owner() != &program_id {
        return Err(ShieldPoolError::RootsRingOwnerNotProgramId.into());
    }
    if nullifier_shard_info.owner() != &program_id {
        return Err(ShieldPoolError::NullifierShardOwnerNotProgramId.into());
    }
    if !nullifier_shard_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }
    if !recipient_account.is_writable() {
        return Err(ShieldPoolError::RecipientNotWritable.into());
    }

    let parsed = parse_withdraw_data(data, false)?;

    verify_proof(
        parsed.proof,
        &parsed.public_inputs[..SP1_PUB_LEN],
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    {
        let roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;
        if !roots_ring.contains_root(&parsed.root) {
            return Err(ShieldPoolError::RootNotFound.into());
        }
    }

    {
        let mut shard = crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;
        if shard.contains_nullifier(&parsed.nullifier) {
            return Err(ShieldPoolError::DoubleSpend.into());
        }
        shard.add_nullifier(&parsed.nullifier)?;
    }

    let mut hash_input = [0u8; RECIPIENT_ADDR_LEN + RECIPIENT_AMOUNT_LEN];
    hash_input[..RECIPIENT_ADDR_LEN].copy_from_slice(&parsed.recipient_address);
    hash_input[RECIPIENT_ADDR_LEN..].copy_from_slice(&parsed.recipient_amount.to_le_bytes());
    let outputs_hash_local = blake3::hash(&hash_input);
    if outputs_hash_local.as_bytes() != &parsed.outputs_hash {
        return Err(ShieldPoolError::InvalidOutputsHash.into());
    }

    if parsed.recipient_amount > parsed.public_amount {
        return Err(ShieldPoolError::InvalidAmount.into());
    }

    let expected_fee = 2_500_000u64 + (parsed.public_amount * 5) / 1_000;
    let total_fee = parsed.public_amount - parsed.recipient_amount;
    if total_fee != expected_fee {
        return Err(ShieldPoolError::Conservation.into());
    }

    if pool_info.lamports() < parsed.public_amount {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    let protocol_share = total_fee;

    let pool_lamports = pool_info.lamports();
    let recipient_lamports = recipient_account.lamports();
    let treasury_lamports = treasury_info.lamports();

    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() =
            pool_lamports - parsed.public_amount;
        *recipient_account.borrow_mut_lamports_unchecked() =
            recipient_lamports + parsed.recipient_amount;
        *treasury_info.borrow_mut_lamports_unchecked() =
            treasury_lamports + protocol_share;
    }

    Ok(())
}
