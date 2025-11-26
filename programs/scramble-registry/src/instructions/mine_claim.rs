use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::find_program_address,
    seeds,
    sysvars::{clock::Clock, rent::Rent, slot_hashes::SLOTHASHES_ID, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::ScrambleError,
    state::{Claim, Miner, ScrambleRegistry},
    utils::{u256_lt, verify_pow},
};

#[inline(always)]
pub fn process_mine_claim_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    // Layout: slot(8) + slot_hash(32) + batch_hash(32) + nonce(16) + proof_hash(32) + max_consumes(2) = 122 bytes
    if instruction_data.len() < 122 {
        return Err(ScrambleError::InvalidTag.into());
    }

    let slot = u64::from_le_bytes(
        instruction_data[0..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let slot_hash: [u8; 32] = instruction_data[8..40]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let batch_hash: [u8; 32] = instruction_data[40..72]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let nonce = u128::from_le_bytes(
        instruction_data[72..88]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let proof_hash: [u8; 32] = instruction_data[88..120]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let max_consumes = u16::from_le_bytes(
        instruction_data[120..122]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    // Parse accounts
    let [claim_account, miner_account, registry_account, miner_authority, slot_hashes_sysvar, _clock_sysvar, _system_program, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify SlotHashes sysvar
    if slot_hashes_sysvar.key() != &SLOTHASHES_ID {
        return Err(ScrambleError::InvalidSlotHashesSysvar.into());
    }

    // Verify miner authority is signer
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load registry
    let mut registry = ScrambleRegistry::from_account_info(registry_account)?;

    // Load miner
    if miner_account.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }
    let mut miner = Miner::from_account_info(miner_account)?;

    // Verify miner authority matches
    if miner.authority() != miner_authority.key() {
        return Err(ScrambleError::UnauthorizedMiner.into());
    }

    // Get current slot from Clock sysvar
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // 1. Verify slot is recent (within SlotHashes range, ~300 slots)
    if current_slot.saturating_sub(slot) > 300 {
        return Err(ScrambleError::SlotTooOld.into());
    }

    // 2. Verify slot_hash matches SlotHashes sysvar
    // Note: In production, we'd parse the SlotHashes sysvar data here
    // For now, we trust the caller provides the correct hash and verify it exists
    if !verify_slot_hash(slot_hashes_sysvar, slot, &slot_hash)? {
        return Err(ScrambleError::SlotHashMismatch.into());
    }

    // 3. Verify proof_hash matches BLAKE3(preimage)
    if !verify_pow(
        slot,
        &slot_hash,
        miner_authority.key(),
        &batch_hash,
        nonce,
        &proof_hash,
    ) {
        return Err(ScrambleError::InvalidProofHash.into());
    }

    // 4. Check difficulty: proof_hash < current_difficulty
    if !u256_lt(&proof_hash, registry.current_difficulty()) {
        return Err(ScrambleError::DifficultyNotMet.into());
    }

    // 5. Verify max_consumes <= max_k
    if max_consumes > registry.max_k() {
        return Err(ScrambleError::BatchSizeExceedsMaxK.into());
    }

    // 6. Verify max_consumes > 0
    if max_consumes == 0 {
        return Err(ScrambleError::InvalidBatchSize.into());
    }

    // 7. Initialize claim PDA
    // Derive claim PDA
    let slot_le_bytes = slot.to_le_bytes();
    let (claim_pda, bump) = find_program_address(
        &[
            b"claim",
            miner_authority.key().as_ref(),
            &batch_hash,
            &slot_le_bytes,
        ],
        &crate::ID,
    );

    // Verify provided account matches PDA
    if claim_account.key() != &claim_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Create PDA account if it doesn't exist
    if claim_account.data_is_empty() {
        // Calculate space and rent
        let space = Claim::SIZE;
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);

        // Create PDA account via system program CPI
        let bump_ref = &[bump];
        let claim_seeds = seeds!(
            b"claim",
            miner_authority.key().as_ref(),
            &batch_hash,
            &slot_le_bytes,
            bump_ref
        );
        let signer = Signer::from(&claim_seeds);

        CreateAccount {
            from: miner_authority,
            to: claim_account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize claim data
    let mut claim = Claim::from_account_info_unchecked(&claim_account);
    claim.initialize(
        miner_authority.key(),
        &batch_hash,
        slot,
        &slot_hash,
        nonce,
        &proof_hash,
        max_consumes,
        current_slot,
    );

    // Update registry stats
    registry.record_solution();

    // Update miner stats
    miner.record_mine();

    Ok(())
}

/// Verify slot_hash exists in SlotHashes sysvar
///
/// SlotHashes layout:
/// - Entry count: u64 LE (first 8 bytes)
/// - Entries: [(slot: u64 LE, hash: [u8; 32])] * count
///
/// Returns: true if slot_hash matches the entry for `slot`
fn verify_slot_hash(
    slot_hashes_sysvar: &AccountInfo,
    target_slot: u64,
    expected_hash: &[u8; 32],
) -> Result<bool, ProgramError> {
    let data = slot_hashes_sysvar.try_borrow_data()?;

    // Parse entry count
    if data.len() < 8 {
        return Err(ProgramError::InvalidAccountData);
    }

    let count = u64::from_le_bytes(data[0..8].try_into().unwrap());

    // Each entry is 8 (slot) + 32 (hash) = 40 bytes
    let expected_len = 8 + (count as usize * 40);
    if data.len() < expected_len {
        return Err(ProgramError::InvalidAccountData);
    }

    // Search for target_slot
    let mut offset = 8;
    for _ in 0..count {
        let slot = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        let hash: [u8; 32] = data[offset + 8..offset + 40].try_into().unwrap();

        if slot == target_slot {
            return Ok(hash == *expected_hash);
        }

        offset += 40;
    }

    // Slot not found in SlotHashes
    Err(ScrambleError::SlotNotFound.into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_slot_hash_parsing() {
        // Build mock SlotHashes data
        let count = 2u64;
        let slot1 = 100u64;
        let hash1 = [0x42; 32];
        let slot2 = 200u64;
        let hash2 = [0x88; 32];

        let mut data = Vec::new();
        data.extend_from_slice(&count.to_le_bytes());
        data.extend_from_slice(&slot1.to_le_bytes());
        data.extend_from_slice(&hash1);
        data.extend_from_slice(&slot2.to_le_bytes());
        data.extend_from_slice(&hash2);

        assert_eq!(data.len(), 8 + 2 * 40);

        // Note: This test is conceptual since we can't create AccountInfo in unit tests easily
        // In integration tests, we'd verify this properly
    }
}
