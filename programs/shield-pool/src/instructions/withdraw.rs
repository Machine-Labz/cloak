use crate::constants::{
    DUPLICATE_NULLIFIER_LEN, NUM_OUTPUTS_LEN, POW_BATCH_HASH_LEN, PROOF_LEN, PUB_LEN,
    RECIPIENT_ADDR_LEN, RECIPIENT_AMOUNT_LEN, SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
};
use crate::error::ShieldPoolError;
use crate::ID;
use core::convert::TryInto;
use pinocchio::cpi::invoke_signed;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Seed, Signer},
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token::instructions::Transfer as TokenTransfer;
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

const MIN_TAIL_LEN: usize = PUB_LEN + DUPLICATE_NULLIFIER_LEN + NUM_OUTPUTS_LEN;
const PER_OUTPUT_LEN: usize = RECIPIENT_ADDR_LEN + RECIPIENT_AMOUNT_LEN;
const MAX_OUTPUTS: usize = 5;

struct ParsedWithdraw<'a> {
    proof: &'a [u8],
    public_inputs: [u8; PUB_LEN],
    root: [u8; 32],
    nullifier: [u8; 32],
    outputs_hash: [u8; 32],
    public_amount: u64,
    recipients: [([u8; 32], u64); MAX_OUTPUTS],
    num_outputs: u8,
    batch_hash: Option<[u8; 32]>,
}

fn parse_withdraw_data<'a>(
    data: &'a [u8],
    expect_batch_hash: bool,
) -> Result<ParsedWithdraw<'a>, ShieldPoolError> {
    // First, read num_outputs to calculate the actual tail length
    // We need at least: MIN_TAIL_LEN bytes to read num_outputs
    if data.len() < MIN_TAIL_LEN + PROOF_LEN {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Parse public_inputs to get it out of the way, it's right after proof
    let (_public_inputs_slice, _remainder) = data.split_at(
        data.len()
            - MIN_TAIL_LEN
            - if expect_batch_hash {
                POW_BATCH_HASH_LEN
            } else {
                0
            },
    );

    // Actually, we need to be smarter. Let's parse from the end backwards.
    // Format: [proof][public_inputs=104][duplicate_nullifier=32][num_outputs=1][recipients...][batch_hash=32?]

    // Start from the end
    let mut _offset_from_end = 0;

    // Optional batch hash at the very end
    if expect_batch_hash {
        _offset_from_end += POW_BATCH_HASH_LEN;
    }

    // We need to know num_outputs to know how many recipients to parse
    // num_outputs is at: public_inputs(104) + duplicate_nullifier(32) = 136 bytes before the recipients
    // But we don't know how many recipients there are yet!

    // Let's parse forward instead
    // Skip proof first - we'll calculate its length later
    let after_proof_idx = data.len()
        - PUB_LEN
        - DUPLICATE_NULLIFIER_LEN
        - NUM_OUTPUTS_LEN
        - if expect_batch_hash {
            POW_BATCH_HASH_LEN
        } else {
            0
        };

    if data.len() < after_proof_idx {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Actually, this is getting complex. Let me parse it differently.
    // We know the structure is:
    // [proof (variable)][public_inputs (104)][dup_nullifier (32)][num_outputs (1)][recipients (num_outputs * 40)][batch_hash (32)?]

    // To find where proof ends, we need to know num_outputs first
    // Let's peek at num_outputs position
    let min_tail_with_batch = MIN_TAIL_LEN
        + if expect_batch_hash {
            POW_BATCH_HASH_LEN
        } else {
            0
        };
    if data.len() < min_tail_with_batch + PROOF_LEN {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // num_outputs is at position: data.len() - (batch_hash?) - (recipients) - 1
    // But we don't know recipients size yet. Let's read from a known position.
    // num_outputs is right after: public_inputs (104) + dup_nullifier (32) = 136 bytes from end (excluding batch and recipients)

    // Actually, let me think differently. The minimum data size is:
    // proof (at least 260) + public_inputs (104) + dup_nullifier (32) + num_outputs (1) = 397 minimum
    // Plus batch_hash if expect_batch_hash

    let _num_outputs_offset = PUB_LEN + DUPLICATE_NULLIFIER_LEN;

    // Read num_outputs by looking backwards from end
    let batch_offset = if expect_batch_hash {
        POW_BATCH_HASH_LEN
    } else {
        0
    };

    // We need to iterate to find the right position
    // Let's try all possible num_outputs values (1-10) and see which one gives us a valid proof length

    let mut found_parse: Option<(u8, usize)> = None;
    for test_num_outputs in 1..=MAX_OUTPUTS {
        let test_recipients_len = test_num_outputs * PER_OUTPUT_LEN;
        let test_tail_len = PUB_LEN
            + DUPLICATE_NULLIFIER_LEN
            + NUM_OUTPUTS_LEN
            + test_recipients_len
            + batch_offset;

        if data.len() <= test_tail_len {
            continue;
        }

        let test_proof_len = data.len() - test_tail_len;
        if test_proof_len != PROOF_LEN {
            continue;
        }

        // Check if num_outputs byte matches
        let num_outputs_idx = test_proof_len + PUB_LEN + DUPLICATE_NULLIFIER_LEN;
        if num_outputs_idx >= data.len() {
            continue;
        }

        if data[num_outputs_idx] == test_num_outputs as u8 {
            found_parse = Some((test_num_outputs as u8, test_proof_len));
            break;
        }
    }

    let (num_outputs, proof_len) = found_parse.ok_or(ShieldPoolError::InvalidInstructionData)?;

    if num_outputs == 0 || num_outputs as usize > MAX_OUTPUTS {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    let (proof, mut remainder) = data.split_at(proof_len);

    let (public_inputs_slice, rem) = remainder.split_at(PUB_LEN);
    remainder = rem;
    let mut public_inputs = [0u8; PUB_LEN];
    public_inputs.copy_from_slice(public_inputs_slice);

    let (duplicate_nullifier_slice, rem) = remainder.split_at(DUPLICATE_NULLIFIER_LEN);
    remainder = rem;
    let duplicate_nullifier: [u8; 32] = duplicate_nullifier_slice
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    let (&num_outputs_byte, rem) = remainder
        .split_first()
        .ok_or(ShieldPoolError::InvalidInstructionData)?;
    remainder = rem;

    if num_outputs_byte != num_outputs {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Parse recipients
    let mut recipients = [([0u8; 32], 0u64); MAX_OUTPUTS];
    for i in 0..num_outputs as usize {
        let (addr_slice, rem) = remainder.split_at(RECIPIENT_ADDR_LEN);
        remainder = rem;
        let address: [u8; 32] = addr_slice
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

        let (amount_slice, rem) = remainder.split_at(RECIPIENT_AMOUNT_LEN);
        remainder = rem;
        let amount = u64::from_le_bytes(
            amount_slice
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        );

        recipients[i] = (address, amount);
    }

    let batch_hash = if expect_batch_hash {
        let (hash_slice, rem) = remainder.split_at(POW_BATCH_HASH_LEN);
        remainder = rem;
        Some(
            hash_slice
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        )
    } else {
        None
    };

    if !remainder.is_empty() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

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
        recipients,
        num_outputs,
        batch_hash,
    })
}

pub fn process_withdraw_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // accounts layout:
    // Legacy native: [pool, treasury, roots_ring, nullifier_shard, recipients..., system_program]
    // Legacy SPL:    adds [token_program, pool_token, recipient_token..., treasury_token]
    // PoW modes append [scramble_program, claim_pda, miner_pda, registry_pda, clock_sysvar, miner_authority, shield_pool_program]

    if accounts.len() < 6 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let pool_info = &accounts[0];
    let treasury_info = &accounts[1];
    let roots_ring_info = &accounts[2];
    let nullifier_shard_info = &accounts[3];

    // Check if this is PoW mode by looking for the characteristic account count
    // PoW mode needs: 4 base + recipients + 1 system + 7 pow accounts
    // Minimum PoW mode: 4 + 1 recipient + 1 + 7 = 13 accounts
    let is_pow_mode = accounts.len() >= 13;

    // Parse data to determine number of recipients
    let parsed = parse_withdraw_data(data, is_pow_mode)?;
    let num_recipients = parsed.num_outputs as usize;

    // Validate account count based on mode
    if is_pow_mode {
        // Expected: 4 base + num_recipients + 1 system + 7 pow = 12 + num_recipients
        let expected_accounts = 12 + num_recipients;
        if accounts.len() < expected_accounts {
            return Err(ShieldPoolError::MissingAccounts.into());
        }
    } else {
        // Expected: 4 base + num_recipients + 1 system = 5 + num_recipients
        let expected_accounts = 5 + num_recipients;
        if accounts.len() < expected_accounts {
            return Err(ShieldPoolError::MissingAccounts.into());
        }
    }

    let recipients_start = 4;
    let recipients_end = recipients_start + num_recipients;
    let recipient_accounts = &accounts[recipients_start..recipients_end];

    // Ensure the system program position exists even if we do not use it directly
    if accounts.len() <= recipients_end {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let mut cursor = recipients_end + 1;

    let pow_context = if is_pow_mode {
        if accounts.len() < cursor + 7 {
            return Err(ShieldPoolError::MissingAccounts.into());
        }
        let ctx = PowContext {
            scramble_program_info: &accounts[cursor],
            claim_pda_info: &accounts[cursor + 1],
            miner_pda_info: &accounts[cursor + 2],
            registry_pda_info: &accounts[cursor + 3],
            clock_sysvar_info: &accounts[cursor + 4],
            miner_authority_account: &accounts[cursor + 5],
            shield_pool_program_info: &accounts[cursor + 6],
        };
        cursor += 7;
        Some(ctx)
    } else {
        None
    };

    let spl_accounts = &accounts[cursor..];

    process_withdraw_unified(
        pool_info,
        treasury_info,
        roots_ring_info,
        nullifier_shard_info,
        recipient_accounts,
        &parsed,
        pow_context,
        spl_accounts,
    )
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

struct SplContext<'a> {
    pool_token_account: &'a AccountInfo,
    recipient_token_accounts: &'a [AccountInfo],
    treasury_token_account: &'a AccountInfo,
    miner_token_account: Option<&'a AccountInfo>,
}

fn process_withdraw_unified<'a>(
    pool_info: &AccountInfo,
    treasury_info: &AccountInfo,
    roots_ring_info: &AccountInfo,
    nullifier_shard_info: &AccountInfo,
    recipient_accounts: &[AccountInfo],
    parsed: &'a ParsedWithdraw<'a>,
    pow_context: Option<PowContext<'a>>,
    spl_accounts: &'a [AccountInfo],
) -> ProgramResult {
    let program_id = Pubkey::from(ID);
    // Common validations
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

    // PoW-specific validations
    if let Some(ctx) = &pow_context {
        if ctx.shield_pool_program_info.key() != &program_id {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        if !ctx.shield_pool_program_info.executable() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        if !ctx.miner_authority_account.is_writable() {
            return Err(ShieldPoolError::InvalidMinerAccount.into());
        }
    }

    let num_recipients = parsed.num_outputs as usize;
    if recipient_accounts.len() != num_recipients {
        return Err(ShieldPoolError::MissingAccounts.into());
    }
    for recipient_account in recipient_accounts {
        if !recipient_account.is_writable() {
            return Err(ShieldPoolError::RecipientNotWritable.into());
        }
    }

    // Common logic for both modes
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

    // Validate outputs hash by hashing all recipients
    let mut hasher = blake3::Hasher::new();
    let mut total_recipient_amount = 0u64;
    for i in 0..num_recipients {
        let (address, amount) = parsed.recipients[i];
        hasher.update(&address);
        hasher.update(&amount.to_le_bytes());
        total_recipient_amount = total_recipient_amount
            .checked_add(amount)
            .ok_or(ShieldPoolError::MathOverflow)?;
    }
    let outputs_hash_local = hasher.finalize();
    if outputs_hash_local.as_bytes() != &parsed.outputs_hash {
        return Err(ShieldPoolError::InvalidOutputsHash.into());
    }

    if total_recipient_amount > parsed.public_amount {
        return Err(ShieldPoolError::InvalidAmount.into());
    }

    let expected_fee = 2_500_000u64 + (parsed.public_amount * 5) / 1_000;
    let total_fee = parsed.public_amount - total_recipient_amount;
    if total_fee != expected_fee {
        return Err(ShieldPoolError::Conservation.into());
    }

    let pool_state = crate::state::Pool::from_account_info(pool_info)?;
    let mint = pool_state.mint();
    let is_native_asset = mint == Pubkey::default();

    let spl_context = if is_native_asset {
        if !spl_accounts.is_empty() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        None
    } else {
        let mut idx = 0usize;
        let expected_len = 2 + num_recipients + 1 + if pow_context.is_some() { 1 } else { 0 };
        if spl_accounts.len() != expected_len {
            return Err(ShieldPoolError::MissingAccounts.into());
        }

        let token_program_info = &spl_accounts[idx];
        idx += 1;
        if !token_program_info.executable() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        let pool_token_account = &spl_accounts[idx];
        idx += 1;
        let recipient_token_accounts = &spl_accounts[idx..idx + num_recipients];
        idx += num_recipients;
        let treasury_token_account = &spl_accounts[idx];
        idx += 1;
        let miner_token_account = if pow_context.is_some() {
            let account = &spl_accounts[idx];
            idx += 1;
            Some(account)
        } else {
            None
        };

        if idx != spl_accounts.len() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        if !pool_token_account.is_writable() {
            return Err(ShieldPoolError::PoolNotWritable.into());
        }
        if !treasury_token_account.is_writable() {
            return Err(ShieldPoolError::TreasuryNotWritable.into());
        }
        for token_account in recipient_token_accounts.iter() {
            if !token_account.is_writable() {
                return Err(ShieldPoolError::RecipientNotWritable.into());
            }
        }
        if let Some(miner_token_account) = miner_token_account {
            if !miner_token_account.is_writable() {
                return Err(ShieldPoolError::InvalidMinerAccount.into());
            }
        }

        Some(SplContext {
            pool_token_account,
            recipient_token_accounts,
            treasury_token_account,
            miner_token_account,
        })
    };

    let mut protocol_share = total_fee;
    let mut scrambler_share: Option<u64> = None;

    if let Some(ctx) = &pow_context {
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
        let scrambler = ((total_fee as u128 * fee_share_bps as u128) / 10_000) as u64;
        scrambler_share = Some(scrambler);
        protocol_share = total_fee - scrambler;
    }

    if !is_native_asset {
        let spl_context = spl_context.expect("spl context must exist for non-native asset");

        let (pool_pda, pool_bump) =
            pinocchio::pubkey::find_program_address(&[b"pool", mint.as_ref()], &ID);
        if pool_info.key() != &pool_pda {
            return Err(ShieldPoolError::BadAccounts.into());
        }

        let pool_bump_seed = [pool_bump];
        let pool_seeds = [
            Seed::from(b"pool".as_ref()),
            Seed::from(mint.as_ref()),
            Seed::from(pool_bump_seed.as_ref()),
        ];
        let signer = [Signer::from(&pool_seeds)];

        for (i, recipient_account) in recipient_accounts.iter().enumerate() {
            let (recipient_address, amount) = parsed.recipients[i];
            if recipient_account.key().as_ref() != &recipient_address {
                return Err(ShieldPoolError::InvalidRecipient.into());
            }

            TokenTransfer {
                from: spl_context.pool_token_account,
                to: &spl_context.recipient_token_accounts[i],
                authority: pool_info,
                amount,
            }
            .invoke_signed(&signer)?;
        }

        TokenTransfer {
            from: spl_context.pool_token_account,
            to: spl_context.treasury_token_account,
            authority: pool_info,
            amount: protocol_share,
        }
        .invoke_signed(&signer)?;

        if let Some(_ctx) = &pow_context {
            let scrambler_amount =
                scrambler_share.ok_or(ShieldPoolError::InvalidInstructionData)?;
            let miner_token_account = spl_context
                .miner_token_account
                .ok_or(ShieldPoolError::MissingAccounts)?;
            if scrambler_amount > 0 {
                TokenTransfer {
                    from: spl_context.pool_token_account,
                    to: miner_token_account,
                    authority: pool_info,
                    amount: scrambler_amount,
                }
                .invoke_signed(&signer)?;
            }
        } else if scrambler_share.is_some() {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        return Ok(());
    }

    if pool_info.lamports() < parsed.public_amount {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    let pool_lamports = pool_info.lamports();
    let treasury_lamports = treasury_info.lamports();

    unsafe {
        *pool_info.borrow_mut_lamports_unchecked() = pool_lamports - parsed.public_amount;

        for (i, recipient_account) in recipient_accounts.iter().enumerate() {
            let (recipient_address, recipient_amount) = parsed.recipients[i];
            if recipient_account.key().as_ref() != &recipient_address {
                return Err(ShieldPoolError::InvalidRecipient.into());
            }

            let recipient_lamports = recipient_account.lamports();
            *recipient_account.borrow_mut_lamports_unchecked() =
                recipient_lamports + recipient_amount;
        }

        if let Some(ctx) = &pow_context {
            let scrambler_amount =
                scrambler_share.ok_or(ShieldPoolError::InvalidInstructionData)?;
            let miner_lamports = ctx.miner_authority_account.lamports();
            *treasury_info.borrow_mut_lamports_unchecked() = treasury_lamports + protocol_share;
            *ctx.miner_authority_account.borrow_mut_lamports_unchecked() =
                miner_lamports + scrambler_amount;
        } else {
            *treasury_info.borrow_mut_lamports_unchecked() = treasury_lamports + protocol_share;
        }
    }

    Ok(())
}
