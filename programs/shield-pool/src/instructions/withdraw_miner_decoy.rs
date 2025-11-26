use five8_const::decode_32_const;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::AccountMeta,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    ProgramResult,
};

use crate::{
    error::ShieldPoolError,
    state::{NullifierShard, RootsRing},
    ID,
};

/// Scramble Registry Program ID
const SCRAMBLE_REGISTRY_ID: [u8; 32] =
    decode_32_const("EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6");

/// Maximum Merkle tree depth
const MAX_MERKLE_DEPTH: usize = 32;

/// Minimum instruction data size: amount(8) + r(32) + sk_spend(32) + leaf_index(4) + depth(1)
const MIN_INSTRUCTION_DATA_SIZE: usize = 8 + 32 + 32 + 4 + 1;

/// Miner Decoy Withdrawal - bypasses ZK proof by revealing note secrets
///
/// This instruction allows miners to withdraw their own deposited funds
/// without a ZK proof. The miner reveals (amount, r, sk_spend, leaf_index)
/// and the program verifies:
/// 1. Miner signature + valid PoW claim
/// 2. Commitment reconstruction matches Merkle tree
/// 3. Nullifier computation and recording
///
/// Accounts:
/// 0. [writable] Pool PDA
/// 1. [writable] Treasury
/// 2. [] Roots Ring
/// 3. [writable] Nullifier Shard
/// 4. [writable] Miner Escrow (recipient - must be miner's escrow PDA)
/// 5. [] Scramble Registry Program
/// 6. [writable] Claim PDA
/// 7. [writable] Miner PDA
/// 8. [writable] Registry PDA
/// 9. [] Clock Sysvar
/// 10. [signer, writable] Miner Authority
///
/// Instruction data layout:
/// [amount: 8][r: 32][sk_spend: 32][leaf_index: 4][depth: 1][merkle_siblings: depth * 32]
#[inline(always)]
pub fn process_withdraw_miner_decoy_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Validate minimum instruction data length
    if instruction_data.len() < MIN_INSTRUCTION_DATA_SIZE {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Parse accounts
    if accounts.len() < 11 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let pool_info = &accounts[0];
    let treasury_info = &accounts[1];
    let roots_ring_info = &accounts[2];
    let nullifier_shard_info = &accounts[3];
    let miner_escrow_info = &accounts[4];
    let scramble_program_info = &accounts[5];
    let claim_pda_info = &accounts[6];
    let miner_pda_info = &accounts[7];
    let registry_pda_info = &accounts[8];
    let clock_sysvar_info = &accounts[9];
    let miner_authority_info = &accounts[10];

    let amount = u64::from_le_bytes(
        instruction_data[0..8]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    let r: [u8; 32] = instruction_data[8..40]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    let sk_spend: [u8; 32] = instruction_data[40..72]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    let leaf_index = u32::from_le_bytes(
        instruction_data[72..76]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    let depth = instruction_data[76] as usize;
    if depth == 0 || depth > MAX_MERKLE_DEPTH {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Validate we have enough data for merkle siblings
    let expected_len = MIN_INSTRUCTION_DATA_SIZE + (depth * 32);
    if instruction_data.len() < expected_len {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Parse merkle siblings
    let mut merkle_siblings = [[0u8; 32]; MAX_MERKLE_DEPTH];
    for i in 0..depth {
        let start = 77 + i * 32;
        merkle_siblings[i] = instruction_data[start..start + 32]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    }

    let program_id = Pubkey::from(ID);
    let scramble_program_id = Pubkey::from(SCRAMBLE_REGISTRY_ID);

    // Verify pool ownership
    if pool_info.owner() != &program_id {
        return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
    }
    if !pool_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }
    if !treasury_info.is_writable() {
        return Err(ShieldPoolError::TreasuryNotWritable.into());
    }

    // Verify miner is signer
    if !miner_authority_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify scramble program ID
    if scramble_program_info.key() != &scramble_program_id {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Verify miner escrow is the correct PDA for this miner
    let (expected_escrow, _) = find_program_address(
        &[b"miner_escrow", miner_authority_info.key().as_ref()],
        &scramble_program_id,
    );
    if miner_escrow_info.key() != &expected_escrow {
        return Err(ShieldPoolError::InvalidRecipient.into());
    }
    if !miner_escrow_info.is_writable() {
        return Err(ShieldPoolError::RecipientNotWritable.into());
    }

    // Verify nullifier shard
    if nullifier_shard_info.owner() != &program_id {
        return Err(ShieldPoolError::NullifierShardOwnerNotProgramId.into());
    }
    if !nullifier_shard_info.is_writable() {
        return Err(ShieldPoolError::PoolNotWritable.into());
    }

    // Verify roots ring
    if roots_ring_info.owner() != &program_id {
        return Err(ShieldPoolError::RootsRingOwnerNotProgramId.into());
    }

    // 1. Compute pk_spend = BLAKE3(sk_spend)
    let pk_spend = blake3::hash(&sk_spend);

    // 2. Compute commitment = BLAKE3(amount || r || pk_spend)
    let mut commitment_hasher = blake3::Hasher::new();
    commitment_hasher.update(&amount.to_le_bytes());
    commitment_hasher.update(&r);
    commitment_hasher.update(pk_spend.as_bytes());
    let commitment = commitment_hasher.finalize();

    // 3. Verify Merkle proof
    let mut current_hash = *commitment.as_bytes();
    let mut path_index = leaf_index;

    for i in 0..depth {
        let sibling = &merkle_siblings[i];
        let mut hasher = blake3::Hasher::new();
        if path_index & 1 == 0 {
            // Current is left child
            hasher.update(&current_hash);
            hasher.update(sibling);
        } else {
            // Current is right child
            hasher.update(sibling);
            hasher.update(&current_hash);
        }
        current_hash = *hasher.finalize().as_bytes();
        path_index >>= 1;
    }

    // Check computed root is in roots ring
    {
        let roots_ring = RootsRing::from_account_info(roots_ring_info)?;
        if !roots_ring.contains_root(&current_hash) {
            return Err(ShieldPoolError::RootNotFound.into());
        }
    }

    // 4. Compute nullifier = BLAKE3(sk_spend || leaf_index)
    let mut nullifier_hasher = blake3::Hasher::new();
    nullifier_hasher.update(&sk_spend);
    nullifier_hasher.update(&leaf_index.to_le_bytes());
    let nullifier = *nullifier_hasher.finalize().as_bytes();

    // 5. Check nullifier not already used, then record it
    {
        let mut shard = NullifierShard::from_account_info(nullifier_shard_info)?;
        if shard.contains_nullifier(&nullifier) {
            return Err(ShieldPoolError::DoubleSpend.into());
        }
        shard.add_nullifier(&nullifier)?;
    }

    // === CONSUME POW CLAIM VIA CPI ===

    // Read miner authority from miner PDA to build CPI data
    let miner_authority_bytes: [u8; 32] = {
        let miner_data = miner_pda_info.try_borrow_data()?;
        if miner_data.len() < 32 {
            return Err(ShieldPoolError::InvalidMinerAccount.into());
        }
        miner_data[0..32]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidMinerAccount)?
    };

    // Verify the miner authority matches
    if miner_authority_bytes != *miner_authority_info.key().as_ref() {
        return Err(ShieldPoolError::InvalidMinerAccount.into());
    }

    // For decoy withdrawals, we use a zero batch_hash (wildcard)
    let batch_hash = [0u8; 32];

    let mut consume_ix_data = [0u8; 65];
    consume_ix_data[0] = 4; // consume_claim discriminant
    consume_ix_data[1..33].copy_from_slice(&miner_authority_bytes);
    consume_ix_data[33..65].copy_from_slice(&batch_hash);

    let account_metas = [
        AccountMeta::writable(claim_pda_info.key()),
        AccountMeta::writable(miner_pda_info.key()),
        AccountMeta::writable(registry_pda_info.key()),
        AccountMeta::readonly(&program_id), // shield_pool_program as caller
        AccountMeta::readonly(clock_sysvar_info.key()),
    ];

    let consume_ix = pinocchio::instruction::Instruction {
        program_id: scramble_program_info.key(),
        accounts: &account_metas,
        data: &consume_ix_data,
    };

    invoke_signed(
        &consume_ix,
        &[
            claim_pda_info,
            miner_pda_info,
            registry_pda_info,
            pool_info, // as shield_pool_program reference
            clock_sysvar_info,
        ],
        &[],
    )?;

    // === TRANSFER FUNDS ===

    // Calculate fee (same as regular withdrawals for consistency)
    let fee = 2_500_000u64 + (amount * 5) / 1_000; // 0.0025 SOL + 0.5%
    let recipient_amount = amount.saturating_sub(fee);

    // Verify pool has sufficient balance
    if pool_info.lamports() < amount {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    // Read fee_share_bps from registry to split fee between treasury and miner
    let fee_share_bps: u16 = {
        let registry_data = registry_pda_info.try_borrow_data()?;
        if registry_data.len() < 90 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }
        u16::from_le_bytes(
            registry_data[88..90]
                .try_into()
                .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
        )
    };

    let miner_fee_share = ((fee as u128 * fee_share_bps as u128) / 10_000) as u64;
    let protocol_fee_share = fee - miner_fee_share;

    unsafe {
        // Deduct from pool
        let pool_lamports = pool_info.lamports();
        *pool_info.borrow_mut_lamports_unchecked() = pool_lamports - amount;

        // Transfer to miner escrow
        let escrow_lamports = miner_escrow_info.lamports();
        *miner_escrow_info.borrow_mut_lamports_unchecked() = escrow_lamports + recipient_amount;

        // Transfer fees
        let treasury_lamports = treasury_info.lamports();
        *treasury_info.borrow_mut_lamports_unchecked() = treasury_lamports + protocol_fee_share;

        let authority_lamports = miner_authority_info.lamports();
        *miner_authority_info.borrow_mut_lamports_unchecked() =
            authority_lamports + miner_fee_share;
    }

    Ok(())
}
