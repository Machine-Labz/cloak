//! Instruction builders for scramble-registry program
//!
//! Builds transactions for:
//! - mine_claim: Submit PoW solution
//! - reveal_claim: Reveal mined claim
//! - consume_claim: Consume claim (called via CPI from shield-pool)

use anyhow::{anyhow, Result};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    sysvar,
};

/// PDA derivation for ScrambleRegistry singleton
///
/// Seed: [b"registry"]
pub fn derive_registry_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"registry"], program_id)
}

/// PDA derivation for Miner account
///
/// Seed: [b"miner", miner_authority]
pub fn derive_miner_pda(program_id: &Pubkey, miner_authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"miner", miner_authority.as_ref()], program_id)
}

/// PDA derivation for Claim account
///
/// Seed: [b"claim", miner_authority, batch_hash, mined_slot_le]
pub fn derive_claim_pda(
    program_id: &Pubkey,
    miner_authority: &Pubkey,
    batch_hash: &[u8; 32],
    slot: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"claim",
            miner_authority.as_ref(),
            batch_hash,
            &slot.to_le_bytes(),
        ],
        program_id,
    )
}

/// Build mine_claim instruction
///
/// Submits a PoW solution to create a Claim.
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [WRITE] Miner PDA
/// 2. [WRITE] ScrambleRegistry PDA
/// 3. [SIGNER] Miner authority
/// 4. [] SlotHashes sysvar
/// 5. [] Clock sysvar
/// 6. [] System program
///
/// Instruction data (discriminator + args):
/// - discriminator: 2 (u8)
/// - slot: u64 LE
/// - slot_hash: [u8; 32]
/// - batch_hash: [u8; 32]
/// - nonce: u128 LE
/// - proof_hash: [u8; 32]
/// - max_consumes: u16 LE
#[allow(clippy::too_many_arguments)]
pub fn build_mine_claim_ix(
    program_id: &Pubkey,
    claim_pda: &Pubkey,
    miner_pda: &Pubkey,
    registry_pda: &Pubkey,
    miner_authority: &Pubkey,
    slot: u64,
    slot_hash: [u8; 32],
    batch_hash: [u8; 32],
    nonce: u128,
    proof_hash: [u8; 32],
    max_consumes: u16,
) -> Instruction {
    let mut data = Vec::new();

    // Discriminator
    data.push(2u8);

    // Arguments
    data.extend_from_slice(&slot.to_le_bytes());
    data.extend_from_slice(&slot_hash);
    data.extend_from_slice(&batch_hash);
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&proof_hash);
    data.extend_from_slice(&max_consumes.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*claim_pda, false),
            AccountMeta::new(*miner_pda, false),
            AccountMeta::new(*registry_pda, false),
            AccountMeta::new_readonly(*miner_authority, true),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

/// Build reveal_claim instruction
///
/// Reveals a mined claim within the reveal window.
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [] ScrambleRegistry PDA
/// 2. [SIGNER] Miner authority
/// 3. [] Clock sysvar
///
/// Instruction data:
/// - discriminator: 3 (u8)
pub fn build_reveal_claim_ix(
    program_id: &Pubkey,
    claim_pda: &Pubkey,
    registry_pda: &Pubkey,
    miner_authority: &Pubkey,
) -> Instruction {
    let mut data = Vec::new();
    data.push(3u8); // discriminator

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*claim_pda, false),
            AccountMeta::new_readonly(*registry_pda, false),
            AccountMeta::new_readonly(*miner_authority, true),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data,
    }
}

/// Build consume_claim instruction
///
/// Consumes one unit from a revealed claim.
/// NOTE: This is typically called via CPI from shield-pool withdraw.
///
/// Accounts:
/// 0. [WRITE] Claim PDA
/// 1. [WRITE] Miner PDA
/// 2. [WRITE] ScrambleRegistry PDA
/// 3. [SIGNER] Shield-pool program (CPI authority)
/// 4. [] Clock sysvar
///
/// Instruction data:
/// - discriminator: 4 (u8)
/// - expected_miner_authority: [u8; 32]
/// - expected_batch_hash: [u8; 32]
pub fn build_consume_claim_ix(
    program_id: &Pubkey,
    claim_pda: &Pubkey,
    miner_pda: &Pubkey,
    registry_pda: &Pubkey,
    shield_pool_program: &Pubkey,
    miner_authority: &Pubkey,
    batch_hash: &[u8; 32],
) -> Instruction {
    let mut data = Vec::new();

    // Discriminator
    data.push(4u8);

    // Arguments
    data.extend_from_slice(miner_authority.as_ref());
    data.extend_from_slice(batch_hash);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*claim_pda, false),
            AccountMeta::new(*miner_pda, false),
            AccountMeta::new(*registry_pda, false),
            AccountMeta::new_readonly(*shield_pool_program, true),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data,
    }
}

/// Build register_miner instruction
///
/// Registers a new miner (one-time per authority).
///
/// Accounts:
/// 0. [WRITE] Miner PDA
/// 1. [SIGNER] Miner authority
/// 2. [] System program
/// 3. [] Clock sysvar
///
/// Instruction data:
/// - discriminator: 1 (u8)
pub fn build_register_miner_ix(
    program_id: &Pubkey,
    miner_pda: &Pubkey,
    miner_authority: &Pubkey,
) -> Instruction {
    let mut data = Vec::new();
    data.push(1u8); // discriminator

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*miner_pda, false),
            AccountMeta::new_readonly(*miner_authority, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data,
    }
}

/// Helper: Build complete mine + reveal transaction flow
///
/// Returns both instructions in order:
/// 1. mine_claim
/// 2. reveal_claim
///
/// These can be submitted in a single transaction or separately.
#[allow(clippy::too_many_arguments)]
pub fn build_mine_and_reveal_instructions(
    program_id: &Pubkey,
    miner_authority: &Pubkey,
    slot: u64,
    slot_hash: [u8; 32],
    batch_hash: [u8; 32],
    nonce: u128,
    proof_hash: [u8; 32],
    max_consumes: u16,
) -> Result<(Instruction, Instruction)> {
    // Derive PDAs
    let (registry_pda, _) = derive_registry_pda(program_id);
    let (miner_pda, _) = derive_miner_pda(program_id, miner_authority);
    let (claim_pda, _) = derive_claim_pda(program_id, miner_authority, &batch_hash, slot);

    // Build mine instruction
    let mine_ix = build_mine_claim_ix(
        program_id,
        &claim_pda,
        &miner_pda,
        &registry_pda,
        miner_authority,
        slot,
        slot_hash,
        batch_hash,
        nonce,
        proof_hash,
        max_consumes,
    );

    // Build reveal instruction
    let reveal_ix = build_reveal_claim_ix(program_id, &claim_pda, &registry_pda, miner_authority);

    Ok((mine_ix, reveal_ix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_registry_pda() {
        let program_id = Pubkey::new_unique();
        let (pda, bump) = derive_registry_pda(&program_id);

        // Should be deterministic
        let (pda2, bump2) = derive_registry_pda(&program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);

        // Should be valid PDA
        assert!(bump < 255);
    }

    #[test]
    fn test_derive_miner_pda() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (pda, bump) = derive_miner_pda(&program_id, &authority);

        // Should be deterministic
        let (pda2, bump2) = derive_miner_pda(&program_id, &authority);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);

        // Different authority = different PDA
        let other_authority = Pubkey::new_unique();
        let (other_pda, _) = derive_miner_pda(&program_id, &other_authority);
        assert_ne!(pda, other_pda);
    }

    #[test]
    fn test_derive_claim_pda() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let batch_hash = [0x42; 32];
        let slot = 12345u64;

        let (pda, bump) = derive_claim_pda(&program_id, &authority, &batch_hash, slot);

        // Should be deterministic
        let (pda2, bump2) = derive_claim_pda(&program_id, &authority, &batch_hash, slot);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);

        // Different batch_hash = different PDA
        let other_batch = [0x43; 32];
        let (other_pda, _) = derive_claim_pda(&program_id, &authority, &other_batch, slot);
        assert_ne!(pda, other_pda);

        // Different slot = different PDA
        let (other_pda2, _) = derive_claim_pda(&program_id, &authority, &batch_hash, slot + 1);
        assert_ne!(pda, other_pda2);
    }

    #[test]
    fn test_build_mine_claim_ix() {
        let program_id = Pubkey::new_unique();
        let claim_pda = Pubkey::new_unique();
        let miner_pda = Pubkey::new_unique();
        let registry_pda = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let ix = build_mine_claim_ix(
            &program_id,
            &claim_pda,
            &miner_pda,
            &registry_pda,
            &authority,
            100,
            [0x42; 32],
            [0x88; 32],
            12345,
            [0xAA; 32],
            10,
        );

        // Check program ID
        assert_eq!(ix.program_id, program_id);

        // Check account count
        assert_eq!(ix.accounts.len(), 7);

        // Check data format
        assert_eq!(ix.data[0], 2); // Discriminator

        // Data should be: 1 + 8 + 32 + 32 + 16 + 32 + 2 = 123 bytes
        assert_eq!(ix.data.len(), 123);
    }

    #[test]
    fn test_build_reveal_claim_ix() {
        let program_id = Pubkey::new_unique();
        let claim_pda = Pubkey::new_unique();
        let registry_pda = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let ix = build_reveal_claim_ix(&program_id, &claim_pda, &registry_pda, &authority);

        // Check program ID
        assert_eq!(ix.program_id, program_id);

        // Check account count
        assert_eq!(ix.accounts.len(), 4);

        // Check data
        assert_eq!(ix.data.len(), 1);
        assert_eq!(ix.data[0], 3); // Discriminator
    }

    #[test]
    fn test_build_consume_claim_ix() {
        let program_id = Pubkey::new_unique();
        let claim_pda = Pubkey::new_unique();
        let miner_pda = Pubkey::new_unique();
        let registry_pda = Pubkey::new_unique();
        let shield_pool = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let batch_hash = [0x77; 32];

        let ix = build_consume_claim_ix(
            &program_id,
            &claim_pda,
            &miner_pda,
            &registry_pda,
            &shield_pool,
            &authority,
            &batch_hash,
        );

        // Check program ID
        assert_eq!(ix.program_id, program_id);

        // Check account count
        assert_eq!(ix.accounts.len(), 5);

        // Check data: 1 + 32 + 32 = 65 bytes
        assert_eq!(ix.data.len(), 65);
        assert_eq!(ix.data[0], 4); // Discriminator
    }

    #[test]
    fn test_build_register_miner_ix() {
        let program_id = Pubkey::new_unique();
        let miner_pda = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let ix = build_register_miner_ix(&program_id, &miner_pda, &authority);

        // Check program ID
        assert_eq!(ix.program_id, program_id);

        // Check account count
        assert_eq!(ix.accounts.len(), 4);

        // Check data
        assert_eq!(ix.data.len(), 1);
        assert_eq!(ix.data[0], 1); // Discriminator
    }

    #[test]
    fn test_build_mine_and_reveal() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let result = build_mine_and_reveal_instructions(
            &program_id,
            &authority,
            100,
            [0x42; 32],
            [0x88; 32],
            12345,
            [0xAA; 32],
            10,
        );

        assert!(result.is_ok());
        let (mine_ix, reveal_ix) = result.unwrap();

        // Check discriminators
        assert_eq!(mine_ix.data[0], 2);
        assert_eq!(reveal_ix.data[0], 3);

        // Both should target same program
        assert_eq!(mine_ix.program_id, program_id);
        assert_eq!(reveal_ix.program_id, program_id);
    }
}
