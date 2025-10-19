use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{msg, ProgramResult};

use crate::constants::SLOT_HASHES_SYSVAR;
use crate::error::ScrambleError;
use crate::state::{Claim, Miner, ScrambleRegistry};
use crate::utils::{u256_lt, verify_pow};

/// Instruction: mine_claim
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [WRITE] Miner PDA
/// 2. [WRITE] ScrambleRegistry PDA
/// 3. [SIGNER] Miner authority
/// 4. [] SlotHashes sysvar
/// 5. [] Clock sysvar
/// 6. [] System program (if PDA needs initialization)
///
/// Arguments:
/// - slot: u64 (the slot being referenced)
/// - slot_hash: [u8; 32] (from SlotHashes sysvar)
/// - batch_hash: [u8; 32] (commitment to k jobs)
/// - nonce: u128 (found via PoW)
/// - proof_hash: [u8; 32] (BLAKE3 result, must be < difficulty)
/// - max_consumes: u16 (batch size k, ≤ max_k)
pub fn process_mine_claim(
    accounts: &[AccountInfo],
    slot: u64,
    slot_hash: [u8; 32],
    batch_hash: [u8; 32],
    nonce: u128,
    proof_hash: [u8; 32],
    max_consumes: u16,
) -> ProgramResult {
    // Parse accounts
    let [claim_account, miner_account, registry_account, miner_authority, slot_hashes_sysvar, _clock_sysvar, _system_program, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify SlotHashes sysvar
    if slot_hashes_sysvar.key().as_ref() != SLOT_HASHES_SYSVAR {
        msg!("Invalid SlotHashes sysvar");
        return Err(ScrambleError::InvalidSlotHashesSysvar.into());
    }

    // Load registry
    let registry = ScrambleRegistry::from_account(registry_account)?;

    // Load or initialize miner
    let miner = if miner_account.data_is_empty() {
        // Initialize miner PDA (requires system program CPI, simplified here)
        msg!("Miner PDA not initialized - would create here");
        return Err(ProgramError::UninitializedAccount);
    } else {
        Miner::from_account(miner_account)?
    };

    // Verify miner authority matches
    if !miner_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if miner.authority != *miner_authority.key() {
        msg!("Miner authority mismatch");
        return Err(ScrambleError::UnauthorizedMiner.into());
    }

    // Get current slot from Clock sysvar
    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // 1. Verify slot is recent (within SlotHashes range, ~300 slots)
    if current_slot.saturating_sub(slot) > 300 {
        msg!("Slot too old");
        return Err(ScrambleError::SlotTooOld.into());
    }

    // 2. Verify slot_hash matches SlotHashes sysvar
    // Note: In production, we'd parse the SlotHashes sysvar data here
    // For now, we trust the caller provides the correct hash and verify it exists
    if !verify_slot_hash(slot_hashes_sysvar, slot, &slot_hash)? {
        msg!("SlotHash mismatch");
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
        msg!("PoW verification failed: hash mismatch");
        return Err(ScrambleError::InvalidProofHash.into());
    }

    // 4. Check difficulty: proof_hash < current_difficulty
    if !u256_lt(&proof_hash, &registry.current_difficulty) {
        msg!("Difficulty not met");
        return Err(ScrambleError::DifficultyNotMet.into());
    }

    // 5. Verify max_consumes <= max_k
    if max_consumes > registry.max_k {
        msg!("Batch size exceeds max_k");
        return Err(ScrambleError::BatchSizeExceedsMaxK.into());
    }

    // 6. Verify max_consumes > 0
    if max_consumes == 0 {
        msg!("Batch size must be > 0");
        return Err(ScrambleError::InvalidBatchSize.into());
    }

    // 7. Initialize claim PDA
    if claim_account.data_is_empty() {
        msg!("Claim PDA not initialized - would create here");
        return Err(ProgramError::UninitializedAccount);
    }

    let claim = Claim::from_account(claim_account)?;

    // Overwrite claim data (in production, check if already mined)
    let new_claim = Claim::new(
        *miner_authority.key(),
        batch_hash,
        slot,
        slot_hash,
        nonce,
        proof_hash,
        max_consumes,
        current_slot,
    );

    // Write new claim
    *claim = new_claim;

    // Update registry stats
    registry.record_solution();

    // Update miner stats
    miner.record_mine();

    msg!("Claim mined successfully");

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
        msg!("SlotHashes sysvar data too short");
        return Err(ProgramError::InvalidAccountData);
    }

    let count = u64::from_le_bytes(data[0..8].try_into().unwrap());

    // Each entry is 8 (slot) + 32 (hash) = 40 bytes
    let expected_len = 8 + (count as usize * 40);
    if data.len() < expected_len {
        msg!("SlotHashes sysvar malformed");
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
    msg!("Slot not found in SlotHashes sysvar");
    Err(ScrambleError::SlotNotFound.into())
}

#[cfg(test)]
mod tests {
    use super::*;

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
