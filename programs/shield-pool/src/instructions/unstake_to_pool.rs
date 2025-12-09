use crate::constants::{PROOF_LEN, PUB_LEN};
use crate::error::ShieldPoolError;
use core::convert::TryInto;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

/// UnstakeToPool: Withdraw from a deactivated stake account directly into the shield pool
/// 
/// This enables private unstaking:
/// 1. User deactivates stake account (public, but doesn't reveal destination)
/// 2. After cooldown, user calls UnstakeToPool with ZK proof
/// 3. Funds go to shield pool with new commitment (private)
/// 
/// The ZK proof proves:
/// - User owns the stake account (knows the stake authority secret key)
/// - The commitment is correctly formed: C = H(amount || r || pk_spend)
/// - The stake account address matches the one in the proof
/// 
/// Accounts layout:
/// [0] pool - Shield pool PDA (writable)
/// [1] roots_ring - Roots ring PDA (writable, to push new root)
/// [2] stake_account - Stake account to unstake from (writable)
/// [3] stake_authority - Stake authority (signer)
/// [4] clock_sysvar - Clock sysvar
/// [5] stake_history - Stake history sysvar
/// [6] stake_program - Stake program

/// Stake program ID
const STAKE_PROGRAM_ID: [u8; 32] = [
    0x06, 0xa1, 0xd8, 0x17, 0x91, 0x37, 0x54, 0x2a,
    0x98, 0x34, 0x37, 0xbd, 0xfe, 0x2a, 0x7a, 0xb2,
    0x55, 0x7f, 0x53, 0x5c, 0x8a, 0x78, 0x72, 0x2b,
    0x68, 0xa4, 0x9d, 0xc0, 0x00, 0x00, 0x00, 0x00,
];

/// Vkey hash for UnstakeToPool proofs (will be set after circuit is compiled)
/// For now, we use a placeholder - this MUST be updated with the real vkey hash
pub const UNSTAKE_TO_POOL_VKEY_HASH: [u8; 32] = [0u8; 32]; // TODO: Update after circuit compilation

/// Parsed instruction data for UnstakeToPool
struct ParsedUnstakeToPool<'a> {
    /// ZK proof (Groth16, 260 bytes)
    proof: &'a [u8],
    /// Public inputs (104 bytes): commitment(32) || stake_account_hash(32) || outputs_hash(32) || amount(8)
    public_inputs: [u8; PUB_LEN],
    /// The commitment to be added to the merkle tree
    commitment: [u8; 32],
    /// Amount being unstaked (in lamports)
    amount: u64,
    /// Stake account address (must match the one being unstaked from)
    stake_account: [u8; 32],
}

fn parse_unstake_to_pool_data(data: &[u8]) -> Result<ParsedUnstakeToPool, ShieldPoolError> {
    // Format: [proof (260)][public_inputs (104)][stake_account (32)]
    let min_len = PROOF_LEN + PUB_LEN + 32;
    
    if data.len() < min_len {
        return Err(ShieldPoolError::InvalidInstructionData);
    }

    let mut offset = 0;

    // Proof (260 bytes)
    let proof = &data[offset..offset + PROOF_LEN];
    offset += PROOF_LEN;

    // Public inputs (104 bytes)
    let public_inputs: [u8; PUB_LEN] = data[offset..offset + PUB_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    offset += PUB_LEN;

    // Stake account (32 bytes)
    let stake_account: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;

    // Parse public inputs
    // Format: commitment(32) || stake_account_hash(32) || outputs_hash(32) || amount(8)
    // Note: For unstake, we repurpose the fields:
    // - "root" position holds the commitment (what we're adding to tree)
    // - "nullifier" position holds stake_account_hash (proves ownership)
    // - "outputs_hash" is unused (set to zeros)
    // - amount is the unstake amount
    let commitment: [u8; 32] = public_inputs[0..32]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let stake_account_hash: [u8; 32] = public_inputs[32..64]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let amount = u64::from_le_bytes(
        public_inputs[96..104]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    // Verify stake_account_hash matches the provided stake_account
    let mut hasher = blake3::Hasher::new();
    hasher.update(&stake_account);
    let computed_hash = hasher.finalize();
    if computed_hash.as_bytes() != &stake_account_hash {
        return Err(ShieldPoolError::InvalidStakeAccount);
    }

    Ok(ParsedUnstakeToPool {
        proof,
        public_inputs,
        commitment,
        amount,
        stake_account,
    })
}

/// Build the Stake program Withdraw instruction data
/// See: https://docs.rs/solana-program/latest/solana_program/stake/instruction/enum.StakeInstruction.html
fn build_stake_withdraw_data(lamports: u64) -> [u8; 12] {
    let mut data = [0u8; 12];
    // StakeInstruction::Withdraw = 4
    data[0..4].copy_from_slice(&4u32.to_le_bytes());
    // lamports (u64)
    data[4..12].copy_from_slice(&lamports.to_le_bytes());
    data
}

pub fn process_unstake_to_pool_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Validate account count
    if accounts.len() < 7 {
        return Err(ShieldPoolError::MissingAccounts.into());
    }

    let pool_info = &accounts[0];
    let roots_ring_info = &accounts[1];
    let stake_account_info = &accounts[2];
    let stake_authority_info = &accounts[3];
    let clock_info = &accounts[4];
    let stake_history_info = &accounts[5];
    let stake_program_info = &accounts[6];

    // Parse instruction data
    let parsed = parse_unstake_to_pool_data(data)?;

    // Verify stake account matches
    if stake_account_info.key().as_ref() != &parsed.stake_account {
        return Err(ShieldPoolError::InvalidStakeAccount.into());
    }

    // Verify stake authority is signer
    if !stake_authority_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify stake account is writable
    if !stake_account_info.is_writable() {
        return Err(ShieldPoolError::RecipientNotWritable.into());
    }

    // Verify pool is writable
    if !pool_info.is_writable() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Verify stake program ID
    if stake_program_info.key().as_ref() != &STAKE_PROGRAM_ID {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Verify this is native SOL pool
    let pool_state = crate::state::Pool::from_account_info(pool_info)?;
    if pool_state.mint() != Pubkey::default() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // TODO: Verify ZK proof when vkey is set
    // For now, we skip proof verification until the unstake circuit is implemented
    // This is UNSAFE and must be enabled before production!
    // verify_proof(
    //     parsed.proof,
    //     &parsed.public_inputs[..SP1_PUB_LEN],
    //     &UNSTAKE_TO_POOL_VKEY_HASH,
    //     GROTH16_VK_5_0_0_BYTES,
    // )
    // .map_err(|_| ShieldPoolError::ProofInvalid)?;

    // Get stake account balance
    let stake_balance = stake_account_info.lamports();
    
    // Verify amount matches (or is close to, accounting for rent)
    if parsed.amount > stake_balance {
        return Err(ShieldPoolError::InsufficientLamports.into());
    }

    // Build Stake program Withdraw instruction
    // Withdraw instruction accounts:
    // [0] stake_account (writable)
    // [1] recipient (writable) - pool in our case
    // [2] clock_sysvar
    // [3] stake_history_sysvar
    // [4] withdrawer (signer)
    let withdraw_data = build_stake_withdraw_data(parsed.amount);

    let stake_program_key = Pubkey::from(STAKE_PROGRAM_ID);
    
    let withdraw_accounts = [
        AccountMeta::writable(stake_account_info.key()),
        AccountMeta::writable(pool_info.key()),
        AccountMeta::readonly(clock_info.key()),
        AccountMeta::readonly(stake_history_info.key()),
        AccountMeta::readonly_signer(stake_authority_info.key()),
    ];

    let withdraw_ix = Instruction {
        program_id: &stake_program_key,
        accounts: &withdraw_accounts,
        data: &withdraw_data,
    };

    // Invoke the stake withdraw CPI
    invoke(
        &withdraw_ix,
        &[
            stake_account_info,
            pool_info,
            clock_info,
            stake_history_info,
            stake_authority_info,
        ],
    )?;

    // Now the lamports are in the pool - add commitment to merkle tree
    // Push the new commitment as a root (the indexer will handle merkle tree updates)
    let mut roots_ring = crate::state::RootsRing::from_account_info(roots_ring_info)?;
    roots_ring.push_root(&parsed.commitment)?;

    // Note: Events are tracked by the indexer via instruction data parsing
    // The commitment and amount are embedded in the instruction data

    Ok(())
}
