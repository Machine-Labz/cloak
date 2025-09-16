use crate::constants::HASH_SIZE;
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Compute BLAKE3 hash for outputs verification
/// For now, this is a simplified version that just hashes recipient + amount
