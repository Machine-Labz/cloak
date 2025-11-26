//! Instruction builders for decoy operations
//!
//! Builds transactions for:
//! - Deposit to shield-pool (standard deposit, creates commitment)
//! - WithdrawMinerDecoy from shield-pool (reveals note secrets, no ZK proof)
//! - TopUpEscrow on scramble-registry

use std::str::FromStr;

use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use solana_system_interface::program as system_program;

/// Shield Pool program ID
pub const SHIELD_POOL_PROGRAM_ID: &str = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp";

/// Derive miner escrow PDA from scramble-registry program
pub fn derive_miner_escrow_pda(
    scramble_program_id: &Pubkey,
    miner_authority: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"miner_escrow", miner_authority.as_ref()],
        scramble_program_id,
    )
}

/// Build shield-pool deposit instruction
///
/// Standard deposit - creates a commitment in the shield-pool.
/// This is the same instruction regular users use.
///
/// Accounts:
/// 0. [SIGNER, WRITE] User (payer)
/// 1. [WRITE] Pool PDA
/// 2. [] System program
/// 3. [WRITE] Commitments queue
///
/// Instruction data:
/// - discriminator: 0 (u8)
/// - amount: u64 LE
/// - commitment: [u8; 32]
pub fn build_deposit_ix(
    user: &Pubkey,
    pool_pda: &Pubkey,
    commitments_pda: &Pubkey,
    amount: u64,
    commitment: [u8; 32],
) -> Result<Instruction> {
    let shield_pool_program_id = Pubkey::from_str(SHIELD_POOL_PROGRAM_ID)?;

    let mut data = Vec::with_capacity(41);
    data.push(0u8); // Discriminator for Deposit
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&commitment);

    Ok(Instruction {
        program_id: shield_pool_program_id,
        accounts: vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*pool_pda, false),
            AccountMeta::new_readonly(Pubkey::from(system_program::id().to_bytes()), false),
            AccountMeta::new(*commitments_pda, false),
        ],
        data,
    })
}

/// Build shield-pool WithdrawMinerDecoy instruction
///
/// Miner reveals note secrets to withdraw without ZK proof.
/// Consumes a PoW claim.
///
/// Accounts:
/// 0. [WRITE] Pool PDA
/// 1. [WRITE] Treasury
/// 2. [] Roots Ring
/// 3. [WRITE] Nullifier Shard
/// 4. [WRITE] Miner Escrow (recipient)
/// 5. [] Scramble Registry Program
/// 6. [WRITE] Claim PDA
/// 7. [WRITE] Miner PDA
/// 8. [WRITE] Registry PDA
/// 9. [] Clock Sysvar
/// 10. [SIGNER, WRITE] Miner Authority
///
/// Instruction data:
/// - discriminator: 4 (u8)
/// - amount: u64 LE
/// - r: [u8; 32]
/// - sk_spend: [u8; 32]
/// - leaf_index: u32 LE
/// - depth: u8
/// - merkle_siblings: [depth * 32 bytes]
#[allow(clippy::too_many_arguments)]
pub fn build_withdraw_miner_decoy_ix(
    pool_pda: &Pubkey,
    treasury_pda: &Pubkey,
    roots_ring_pda: &Pubkey,
    nullifier_shard_pda: &Pubkey,
    miner_escrow_pda: &Pubkey,
    scramble_program_id: &Pubkey,
    claim_pda: &Pubkey,
    miner_pda: &Pubkey,
    registry_pda: &Pubkey,
    miner_authority: &Pubkey,
    amount: u64,
    r: [u8; 32],
    sk_spend: [u8; 32],
    leaf_index: u32,
    merkle_siblings: &[[u8; 32]],
) -> Result<Instruction> {
    let shield_pool_program_id = Pubkey::from_str(SHIELD_POOL_PROGRAM_ID)?;

    let depth = merkle_siblings.len() as u8;

    // Data: discriminator(1) + amount(8) + r(32) + sk_spend(32) + leaf_index(4) + depth(1) + siblings(depth*32)
    let data_len = 1 + 8 + 32 + 32 + 4 + 1 + (depth as usize * 32);
    let mut data = Vec::with_capacity(data_len);

    data.push(4u8); // Discriminator for WithdrawMinerDecoy
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&r);
    data.extend_from_slice(&sk_spend);
    data.extend_from_slice(&leaf_index.to_le_bytes());
    data.push(depth);
    for sibling in merkle_siblings {
        data.extend_from_slice(sibling);
    }

    Ok(Instruction {
        program_id: shield_pool_program_id,
        accounts: vec![
            AccountMeta::new(*pool_pda, false),
            AccountMeta::new(*treasury_pda, false),
            AccountMeta::new_readonly(*roots_ring_pda, false),
            AccountMeta::new(*nullifier_shard_pda, false),
            AccountMeta::new(*miner_escrow_pda, false),
            AccountMeta::new_readonly(*scramble_program_id, false),
            AccountMeta::new(*claim_pda, false),
            AccountMeta::new(*miner_pda, false),
            AccountMeta::new(*registry_pda, false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new(*miner_authority, true),
        ],
        data,
    })
}

/// Build scramble-registry TopUpEscrow instruction
///
/// Adds SOL to miner's escrow for future operations.
///
/// Accounts:
/// 0. [SIGNER, WRITE] Miner authority (payer)
/// 1. [WRITE] Miner Escrow PDA
/// 2. [] System program
///
/// Instruction data:
/// - discriminator: 5 (u8)
/// - amount: u64 LE
pub fn build_top_up_escrow_ix(
    scramble_program_id: &Pubkey,
    miner_authority: &Pubkey,
    miner_escrow_pda: &Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = Vec::with_capacity(9);
    data.push(5u8); // Discriminator for TopUpEscrow
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: *scramble_program_id,
        accounts: vec![
            AccountMeta::new(*miner_authority, true),
            AccountMeta::new(*miner_escrow_pda, false),
            AccountMeta::new_readonly(Pubkey::from(system_program::id().to_bytes()), false),
        ],
        data,
    }
}

/// Build register miner instruction with initial escrow
///
/// Updated to include miner_escrow account and initial escrow amount.
///
/// Accounts:
/// 0. [WRITE] Miner PDA
/// 1. [WRITE] Miner Escrow PDA
/// 2. [SIGNER, WRITE] Miner authority
/// 3. [] System program
/// 4. [] Clock sysvar
///
/// Instruction data:
/// - discriminator: 1 (u8)
/// - initial_escrow: u64 LE (optional, defaults to 0)
pub fn build_register_miner_with_escrow_ix(
    scramble_program_id: &Pubkey,
    miner_pda: &Pubkey,
    miner_escrow_pda: &Pubkey,
    miner_authority: &Pubkey,
    initial_escrow: u64,
) -> Instruction {
    let mut data = Vec::with_capacity(9);
    data.push(1u8); // Discriminator for RegisterMiner
    data.extend_from_slice(&initial_escrow.to_le_bytes());

    Instruction {
        program_id: *scramble_program_id,
        accounts: vec![
            AccountMeta::new(*miner_pda, false),
            AccountMeta::new(*miner_escrow_pda, false),
            AccountMeta::new(*miner_authority, true),
            AccountMeta::new_readonly(Pubkey::from(system_program::id().to_bytes()), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_deposit_ix() {
        let user = Pubkey::new_unique();
        let pool = Pubkey::new_unique();
        let commitments = Pubkey::new_unique();
        let amount = 1_000_000_000u64;
        let commitment = [0x42u8; 32];

        let ix = build_deposit_ix(&user, &pool, &commitments, amount, commitment).unwrap();

        assert_eq!(ix.accounts.len(), 4);
        assert_eq!(ix.data[0], 0); // Discriminator
        assert_eq!(ix.data.len(), 1 + 8 + 32); // discriminator + amount + commitment
    }

    #[test]
    fn test_build_withdraw_miner_decoy_ix() {
        let pool = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let roots_ring = Pubkey::new_unique();
        let nullifier_shard = Pubkey::new_unique();
        let escrow = Pubkey::new_unique();
        let scramble_program = Pubkey::new_unique();
        let claim = Pubkey::new_unique();
        let miner_pda = Pubkey::new_unique();
        let registry = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let amount = 1_000_000_000u64;
        let r = [0x11u8; 32];
        let sk_spend = [0x22u8; 32];
        let leaf_index = 100u32;
        let merkle_siblings = vec![[0x33u8; 32]; 20]; // 20-depth tree

        let ix = build_withdraw_miner_decoy_ix(
            &pool,
            &treasury,
            &roots_ring,
            &nullifier_shard,
            &escrow,
            &scramble_program,
            &claim,
            &miner_pda,
            &registry,
            &authority,
            amount,
            r,
            sk_spend,
            leaf_index,
            &merkle_siblings,
        )
        .unwrap();

        assert_eq!(ix.accounts.len(), 11);
        assert_eq!(ix.data[0], 4); // Discriminator

        // Data: 1 + 8 + 32 + 32 + 4 + 1 + (20 * 32) = 718 bytes
        let expected_len = 1 + 8 + 32 + 32 + 4 + 1 + (20 * 32);
        assert_eq!(ix.data.len(), expected_len);
    }

    #[test]
    fn test_build_top_up_escrow_ix() {
        let program = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let escrow = Pubkey::new_unique();
        let amount = 500_000_000u64;

        let ix = build_top_up_escrow_ix(&program, &authority, &escrow, amount);

        assert_eq!(ix.accounts.len(), 3);
        assert_eq!(ix.data[0], 5); // Discriminator
        assert_eq!(ix.data.len(), 9); // 1 + 8
    }

    #[test]
    fn test_build_register_miner_with_escrow_ix() {
        let program = Pubkey::new_unique();
        let miner_pda = Pubkey::new_unique();
        let escrow = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let initial = 1_000_000_000u64;

        let ix =
            build_register_miner_with_escrow_ix(&program, &miner_pda, &escrow, &authority, initial);

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.data[0], 1); // Discriminator
        assert_eq!(ix.data.len(), 9); // 1 + 8
    }
}
