use core::convert::TryInto;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, sysvars::Sysvar,
    ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

/// WithdrawSwap instruction - Transaction 1 of 2 for swap withdrawals
///
/// This instruction:
/// 1. Verifies the ZK proof (swap mode)
/// 2. Checks nullifier hasn't been used
/// 3. Creates SwapState PDA to store swap parameters
/// 4. Withdraws SOL from pool → SwapState PDA (native SOL, held by PDA)
///
/// Instruction data layout:
/// [proof (260)][public_inputs (104)][duplicate_nullifier (32)]
/// [output_mint (32)][recipient_ata (32)][min_output_amount (8)]
/// Total: 468 bytes (no batch_hash for swaps)
///
/// Account layout:
/// 0. pool_pda (writable)
/// 1. treasury (writable)
/// 2. roots_ring_pda (readonly)
/// 3. nullifier_shard_pda (writable)
/// 4. swap_state_pda (writable, to be created)
/// 5. system_program (readonly)
/// 6. payer (signer, writable) - pays for PDA creation
use crate::constants::{
    DUPLICATE_NULLIFIER_LEN, PROOF_LEN, PUB_LEN, SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
};
use crate::{
    error::ShieldPoolError,
    state::{NullifierShard, Pool, RootsRing, SwapState},
    ID,
};

const OUTPUT_MINT_LEN: usize = 32;
const RECIPIENT_ATA_LEN: usize = 32;
const MIN_OUTPUT_AMOUNT_LEN: usize = 8;

const SWAP_DATA_LEN: usize = PROOF_LEN
    + PUB_LEN
    + DUPLICATE_NULLIFIER_LEN
    + OUTPUT_MINT_LEN
    + RECIPIENT_ATA_LEN
    + MIN_OUTPUT_AMOUNT_LEN;

struct ParsedWithdrawSwap<'a> {
    proof: &'a [u8],
    public_inputs: [u8; PUB_LEN],
    root: [u8; 32],
    nullifier: [u8; 32],
    outputs_hash: [u8; 32],
    public_amount: u64,
    output_mint: Pubkey,
    recipient_ata: Pubkey,
    min_output_amount: u64,
}

fn parse_withdraw_swap_data(data: &[u8]) -> Result<ParsedWithdrawSwap, ShieldPoolError> {
    if data.len() != SWAP_DATA_LEN {
        return Err(ShieldPoolError::InvalidInstructionData);
    }

    let mut offset = 0;

    // Parse proof (260 bytes)
    let proof = &data[offset..offset + PROOF_LEN];
    offset += PROOF_LEN;

    // Parse public inputs (104 bytes)
    let public_inputs_slice = &data[offset..offset + PUB_LEN];
    let mut public_inputs = [0u8; PUB_LEN];
    public_inputs.copy_from_slice(public_inputs_slice);
    offset += PUB_LEN;

    // Parse duplicate nullifier (32 bytes)
    let duplicate_nullifier: [u8; 32] = data[offset..offset + DUPLICATE_NULLIFIER_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    offset += DUPLICATE_NULLIFIER_LEN;

    // Parse output mint (32 bytes)
    let output_mint_bytes: [u8; 32] = data[offset..offset + OUTPUT_MINT_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let output_mint = Pubkey::from(output_mint_bytes);
    offset += OUTPUT_MINT_LEN;

    // Parse recipient ATA (32 bytes)
    let recipient_ata_bytes: [u8; 32] = data[offset..offset + RECIPIENT_ATA_LEN]
        .try_into()
        .map_err(|_| ShieldPoolError::InvalidInstructionData)?;
    let recipient_ata = Pubkey::from(recipient_ata_bytes);
    offset += RECIPIENT_ATA_LEN;

    // Parse min output amount (8 bytes)
    let min_output_amount = u64::from_le_bytes(
        data[offset..offset + MIN_OUTPUT_AMOUNT_LEN]
            .try_into()
            .map_err(|_| ShieldPoolError::InvalidInstructionData)?,
    );

    // Extract fields from public inputs
    // Layout: [root (32)][nullifier (32)][outputs_hash (32)][amount (8)]
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

    // Verify duplicate nullifier matches
    if nullifier != duplicate_nullifier {
        return Err(ShieldPoolError::NullifierMismatch);
    }

    Ok(ParsedWithdrawSwap {
        proof,
        public_inputs,
        root,
        nullifier,
        outputs_hash,
        public_amount,
        output_mint,
        recipient_ata,
        min_output_amount,
    })
}

pub fn process_withdraw_swap_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse accounts
    let [pool_info, treasury_info, roots_ring_info, nullifier_shard_info, swap_state_info, _system_program_info, payer_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify program ownership
    if pool_info.owner() != &ID {
        return Err(ShieldPoolError::InvalidAccountOwner.into());
    }

    // Parse instruction data
    let parsed = parse_withdraw_swap_data(data)?;

    // Load pool state
    let pool = Pool::from_account_info(pool_info)?;

    // Verify pool is for native SOL (swaps only supported for native SOL for now)
    if !pool.is_native() {
        return Err(ShieldPoolError::InvalidInstructionData.into());
    }

    // Verify root is in the roots ring
    let roots_ring = RootsRing::from_account_info(roots_ring_info)?;
    if !roots_ring.contains_root(&parsed.root) {
        return Err(ShieldPoolError::InvalidRoot.into());
    }

    // Verify nullifier hasn't been used
    let mut nullifier_shard = NullifierShard::from_account_info(nullifier_shard_info)?;
    if nullifier_shard.contains_nullifier(&parsed.nullifier) {
        return Err(ShieldPoolError::NullifierAlreadyUsed.into());
    }

    // Compute expected outputs_hash for swap mode
    // outputs_hash = H(output_mint || recipient_ata || min_output_amount || public_amount)
    let mut hasher = blake3::Hasher::new();
    hasher.update(parsed.output_mint.as_ref());
    hasher.update(parsed.recipient_ata.as_ref());
    hasher.update(&parsed.min_output_amount.to_le_bytes());
    hasher.update(&parsed.public_amount.to_le_bytes());
    let expected_outputs_hash = hasher.finalize();

    if &parsed.outputs_hash != expected_outputs_hash.as_bytes() {
        return Err(ShieldPoolError::InvalidOutputsHash.into());
    }

    // Verify ZK proof
    verify_proof(
        parsed.proof,
        &parsed.public_inputs[..SP1_PUB_LEN],
        WITHDRAW_VKEY_HASH,
        GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| ShieldPoolError::ProofInvalid)?;

    // Mark nullifier as used
    nullifier_shard.add_nullifier(&parsed.nullifier)?;

    // Derive SwapState PDA
    let (swap_state_pubkey, bump) =
        pinocchio::pubkey::find_program_address(&[SwapState::SEED_PREFIX, &parsed.nullifier], &ID);

    if swap_state_info.key() != &swap_state_pubkey {
        return Err(ShieldPoolError::InvalidAccountAddress.into());
    }

    // Create SwapState PDA
    let rent = pinocchio::sysvars::rent::Rent::get()?;
    let required_lamports = rent.minimum_balance(SwapState::SIZE);

    let bump_seed = [bump];
    let seeds = [
        pinocchio::instruction::Seed::from(SwapState::SEED_PREFIX),
        pinocchio::instruction::Seed::from(parsed.nullifier.as_ref()),
        pinocchio::instruction::Seed::from(bump_seed.as_ref()),
    ];
    let signer = pinocchio::instruction::Signer::from(&seeds);

    pinocchio_system::instructions::CreateAccount {
        from: payer_info,
        to: swap_state_info,
        lamports: required_lamports,
        space: SwapState::SIZE as u64,
        owner: &Pubkey::from(ID),
    }
    .invoke_signed(&[signer])?;

    // Initialize SwapState
    let mut swap_state = SwapState::from_account_info_unchecked(swap_state_info);
    let current_slot = pinocchio::sysvars::clock::Clock::get()?.slot;

    // Set timeout to 200 slots (~100 seconds) from now
    let timeout_slot = current_slot + 200;

    swap_state.initialize(
        &parsed.nullifier,
        parsed.public_amount,
        &parsed.output_mint,
        &parsed.recipient_ata,
        parsed.min_output_amount,
        current_slot,
        timeout_slot,
        bump,
    );

    // Transfer lamports: Pool → Treasury → SwapState
    // 1. Pool → Treasury: full public_amount (100M)
    // 2. Treasury → SwapState: public_amount - fee (99.5M)
    // 3. Treasury keeps: fee (0.5M)
    let variable_fee = (parsed.public_amount * 5) / 1_000;
    let amount_to_transfer = parsed.public_amount - variable_fee;

    let pool_lamports = pool_info.lamports();
    let treasury_lamports = treasury_info.lamports();
    let swap_state_lamports = swap_state_info.lamports();

    unsafe {
        // Pool → Treasury (full amount)
        *pool_info.borrow_mut_lamports_unchecked() = pool_lamports - parsed.public_amount;

        // Treasury → SwapState (amount minus fee)
        // Net effect on treasury: +public_amount -amount_to_transfer = +fee
        *treasury_info.borrow_mut_lamports_unchecked() =
            treasury_lamports + parsed.public_amount - amount_to_transfer;

        // SwapState receives amount minus fee
        *swap_state_info.borrow_mut_lamports_unchecked() = swap_state_lamports + amount_to_transfer;
    }

    Ok(())
}
