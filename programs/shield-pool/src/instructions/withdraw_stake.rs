use crate::constants::{
    DUPLICATE_NULLIFIER_LEN, POW_BATCH_HASH_LEN, PROOF_LEN, PUB_LEN,
    SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use core::convert::TryInto;
use pinocchio::cpi::invoke_signed;
use pinocchio::{
    account_info::AccountInfo,
    instruction::AccountMeta,
    pubkey::Pubkey,
    ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

struct ParsedWithdrawStake<'a> {
    proof: &'a [u8],
    public_inputs: [u8; PUB_LEN],
    root: [u8; 32],
    nullifier: [u8; 32],
    public_amount: u64,
    stake_account: [u8; 32],
    batch_hash: Option<[u8; 32]>,
}

fn parse_withdraw_stake_data<'a>(
    data: &'a [u8],
    expect_batch_hash: bool,
) -> Result<ParsedWithdrawStake<'a>, ShieldPoolError> {
    // Format: [proof (variable)][public_inputs (104)][dup_nullifier (32)][stake_account (32)][batch_hash (32)?]
    let min_len = PROOF_LEN + PUB_LEN + DUPLICATE_NULLIFIER_LEN + 32
        + if expect_batch_hash { POW_BATCH_HASH_LEN } else { 0 };
    
    if data.len() < min_len {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Parse from the end backwards
    let mut offset = data.len();
    
    // Optional batch hash
    if expect_batch_hash {
        offset -= POW_BATCH_HASH_LEN;
    }
    
    // Stake account (32 bytes)
    offset -= 32;
    let stake_account: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    
    // Duplicate nullifier (32 bytes)
    offset -= DUPLICATE_NULLIFIER_LEN;
    let duplicate_nullifier: [u8; 32] = data[offset..offset + DUPLICATE_NULLIFIER_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    
    // Public inputs (104 bytes)
    offset -= PUB_LEN;
    let public_inputs: [u8; PUB_LEN] = data[offset..offset + PUB_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    
    // Proof (everything before public inputs)
    let proof = &data[..offset];
    if proof.len() < PROOF_LEN {
        return Err(ShieldPoolError::InvalidProofSize.into());
    }

    // Parse public inputs
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

    // Verify outputs_hash matches stake_account
    // For staking, outputs_hash = BLAKE3(stake_account || amount)
    let mut hasher = blake3::Hasher::new();
    hasher.update(&stake_account);
    hasher.update(&public_amount.to_le_bytes());
    let computed_hash = hasher.finalize();
    if computed_hash.as_bytes() != &outputs_hash {
        return Err(ShieldPoolError::InvalidOutputsHash.into());
    }

    let batch_hash = if expect_batch_hash {
        Some(
            data[data.len() - POW_BATCH_HASH_LEN..]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        )
    } else {
        None
    };

    Ok(ParsedWithdrawStake {
        proof,
        public_inputs,
        root,
        nullifier,
        public_amount,
        stake_account,
        batch_hash,
    })
}

pub fn process_withdraw_stake_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Accounts layout:
    // [pool, treasury, roots_ring, nullifier_shard, stake_account, system_program]
    // PoW mode adds: [scramble_program, claim_pda, miner_pda, registry_pda, clock_sysvar, miner_authority, shield_pool_program]
    
    if accounts.len() < 6 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let pool_info = &accounts[0];
    let treasury_info = &accounts[1];
    let roots_ring_info = &accounts[2];
    let nullifier_shard_info = &accounts[3];
    let stake_account_info = &accounts[4];
    let _system_program_info = &accounts[5];

    // Check if PoW mode
    let is_pow_mode = accounts.len() >= 13;
    let expected_accounts = if is_pow_mode { 13 } else { 6 };
    if accounts.len() < expected_accounts {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    // Parse instruction data
    let parsed = parse_withdraw_stake_data(data, is_pow_mode)?;

    // Verify stake account matches
    if stake_account_info.key().as_ref() != &parsed.stake_account {
        return Err(ShieldPoolError::InvalidRecipient.into());
    }

    if !stake_account_info.is_writable() {
        return Err(ShieldPoolError::RecipientNotWritable.into());
    }

    // Verify proof
    verify_proof(
        parsed.proof,
        &parsed.public_inputs[..SP1_PUB_LEN],
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    // Verify root in ring buffer
    {
        let roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;
        if !roots_ring.contains_root(&parsed.root) {
            return Err(ShieldPoolError::RootNotFound.into());
        }
    }

    // Check and mark nullifier as spent
    {
        let mut shard = crate::state::NullifierShard::from_account_info(nullifier_shard_info)?;
        if shard.contains_nullifier(&parsed.nullifier) {
            return Err(ShieldPoolError::DoubleSpend.into());
        }
        shard.add_nullifier(&parsed.nullifier)?;
    }

    // Verify pool is native SOL (staking only works with SOL)
    let pool_state = crate::state::Pool::from_account_info(pool_info)?;
    let mint = pool_state.mint();
    if mint != Pubkey::default() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Calculate fee (same as withdraw: 0.0025 SOL + 0.5%)
    let expected_fee = 2_500_000u64 + (parsed.public_amount * 5) / 1_000;
    let stake_amount = parsed.public_amount - expected_fee;

    // Verify amount conservation
    if stake_amount + expected_fee != parsed.public_amount {
        return Err(ShieldPoolError::Conservation.into());
    }

    // Handle PoW claim consumption if enabled
    let mut protocol_share = expected_fee;
    let mut scrambler_share: Option<u64> = None;

    if is_pow_mode {
        let pow_accounts = &accounts[6..13];
        let ctx = PowContext {
            scramble_program_info: &pow_accounts[0],
            claim_pda_info: &pow_accounts[1],
            miner_pda_info: &pow_accounts[2],
            registry_pda_info: &pow_accounts[3],
            clock_sysvar_info: &pow_accounts[4],
            miner_authority_account: &pow_accounts[5],
            shield_pool_program_info: &pow_accounts[6],
        };

        let batch_hash = parsed
            .batch_hash
            .ok_or(ShieldPoolError::InvalidInstructionData)?;

        let miner_authority: [u8; 32] = {
            let miner_data = ctx.miner_pda_info.try_borrow_data()?;
            if miner_data.len() < 32 {
                return Err(ShieldPoolError::InvalidMinerAccount.into());
            }
            miner_data[0..32]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidMinerAccount)?
        };

        let mut consume_ix_data = [0u8; 65];
        consume_ix_data[0] = 4;
        consume_ix_data[1..33].copy_from_slice(&miner_authority);
        consume_ix_data[33..65].copy_from_slice(&batch_hash);

        let account_metas = [
            AccountMeta::writable(ctx.claim_pda_info.key()),
            AccountMeta::writable(ctx.miner_pda_info.key()),
            AccountMeta::writable(ctx.registry_pda_info.key()),
            AccountMeta::readonly(ctx.shield_pool_program_info.key()),
            AccountMeta::readonly(ctx.clock_sysvar_info.key()),
        ];

        let consume_ix = pinocchio::instruction::Instruction {
            program_id: ctx.scramble_program_info.key(),
            accounts: &account_metas,
            data: &consume_ix_data,
        };

        invoke_signed(
            &consume_ix,
            &[
                ctx.claim_pda_info,
                ctx.miner_pda_info,
                ctx.registry_pda_info,
                ctx.shield_pool_program_info,
                ctx.clock_sysvar_info,
            ],
            &[],
        )?;

        let registry_data = ctx.registry_pda_info.try_borrow_data()?;
        if registry_data.len() < 90 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        let fee_share_bps = u16::from_le_bytes(
            registry_data[88..90]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        );
        let scrambler = ((expected_fee as u128 * fee_share_bps as u128) / 10_000) as u64;
        scrambler_share = Some(scrambler);
        protocol_share = expected_fee - scrambler;
    }

    // Check pool has sufficient lamports
    if pool_info.lamports() < parsed.public_amount {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    // Transfer lamports
    let pool_lamports = pool_info.lamports();
    let stake_lamports = stake_account_info.lamports();
    let treasury_lamports = treasury_info.lamports();

    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() = pool_lamports - parsed.public_amount;
        *stake_account_info.borrow_mut_lamports_unchecked() = stake_lamports + stake_amount;
        *treasury_info.borrow_mut_lamports_unchecked() = treasury_lamports + protocol_share;
    }

    // Transfer scrambler share if PoW mode
    if is_pow_mode {
        let ctx = PowContext {
            scramble_program_info: &accounts[6],
            claim_pda_info: &accounts[7],
            miner_pda_info: &accounts[8],
            registry_pda_info: &accounts[9],
            clock_sysvar_info: &accounts[10],
            miner_authority_account: &accounts[11],
            shield_pool_program_info: &accounts[12],
        };
        if let Some(scrambler_amount) = scrambler_share {
            let miner_lamports = ctx.miner_authority_account.lamports();
            unsafe {
                *ctx.miner_authority_account.borrow_mut_lamports_unchecked() =
                    miner_lamports + scrambler_amount;
            }
        }
    }

    Ok(())
}

struct PowContext<'a> {
    scramble_program_info: &'a AccountInfo,
    claim_pda_info: &'a AccountInfo,
    miner_pda_info: &'a AccountInfo,
    registry_pda_info: &'a AccountInfo,
    clock_sysvar_info: &'a AccountInfo,
    miner_authority_account: &'a AccountInfo,
    shield_pool_program_info: &'a AccountInfo,
}

