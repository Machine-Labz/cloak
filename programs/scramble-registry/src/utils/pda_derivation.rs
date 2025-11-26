pub use pinocchio::pubkey::{find_program_address, Pubkey};

/// Derive the miner escrow PDA
/// Seed: [b"miner_escrow", miner_authority]
#[inline(always)]
pub fn derive_miner_escrow_pda(program_id: &Pubkey, miner_authority: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[b"miner_escrow", miner_authority.as_ref()], program_id)
}

pub fn find_miner_escrow_pda(program_id: &Pubkey, miner_authority: &Pubkey) -> Pubkey {
    derive_miner_escrow_pda(program_id, miner_authority).0
}

/// Derive the registry PDA
/// Seed: [b"registry"]
#[inline(always)]
pub fn derive_registry_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[b"registry"], program_id)
}
