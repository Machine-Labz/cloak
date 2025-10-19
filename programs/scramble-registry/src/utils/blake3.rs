use blake3::Hasher;
use pinocchio::pubkey::Pubkey;

use crate::constants::{DOMAIN, PREIMAGE_SIZE};

/// Build PoW preimage from components
///
/// Preimage layout (137 bytes):
/// - Domain: "CLOAK:SCRAMBLE:v1" (17 bytes)
/// - Slot: u64 LE (8 bytes)
/// - Slot hash: [u8; 32] (32 bytes)
/// - Miner pubkey: [u8; 32] (32 bytes)
/// - Batch hash: [u8; 32] (32 bytes)
/// - Nonce: u128 LE (16 bytes)
fn build_preimage(
    slot: u64,
    slot_hash: &[u8; 32],
    miner_pubkey: &Pubkey,
    batch_hash: &[u8; 32],
    nonce: u128,
) -> [u8; PREIMAGE_SIZE] {
    let mut preimage = [0u8; PREIMAGE_SIZE];
    let mut offset = 0;

    // Domain (17 bytes)
    preimage[offset..offset + DOMAIN.len()].copy_from_slice(DOMAIN);
    offset += DOMAIN.len();

    // Slot (8 bytes LE)
    preimage[offset..offset + 8].copy_from_slice(&slot.to_le_bytes());
    offset += 8;

    // Slot hash (32 bytes)
    preimage[offset..offset + 32].copy_from_slice(slot_hash);
    offset += 32;

    // Miner pubkey (32 bytes)
    preimage[offset..offset + 32].copy_from_slice(miner_pubkey.as_ref());
    offset += 32;

    // Batch hash (32 bytes)
    preimage[offset..offset + 32].copy_from_slice(batch_hash);
    offset += 32;

    // Nonce (16 bytes LE)
    preimage[offset..offset + 16].copy_from_slice(&nonce.to_le_bytes());

    preimage
}

/// Compute BLAKE3 hash of PoW preimage
///
/// Returns the 32-byte BLAKE3 hash that should be compared against
/// the difficulty target.
pub fn hash_pow_preimage(
    slot: u64,
    slot_hash: &[u8; 32],
    miner_pubkey: &Pubkey,
    batch_hash: &[u8; 32],
    nonce: u128,
) -> [u8; 32] {
    let preimage = build_preimage(slot, slot_hash, miner_pubkey, batch_hash, nonce);
    let mut hasher = Hasher::new();
    hasher.update(&preimage);
    *hasher.finalize().as_bytes()
}

/// Verify proof-of-work: recompute hash and compare against expected
///
/// This function:
/// 1. Rebuilds the preimage from components
/// 2. Computes BLAKE3(preimage)
/// 3. Verifies it matches the expected hash
///
/// Returns: true if hash matches
pub fn verify_pow(
    slot: u64,
    slot_hash: &[u8; 32],
    miner_pubkey: &Pubkey,
    batch_hash: &[u8; 32],
    nonce: u128,
    expected_hash: &[u8; 32],
) -> bool {
    let computed = hash_pow_preimage(slot, slot_hash, miner_pubkey, batch_hash, nonce);
    computed == *expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preimage_size() {
        let slot = 12345u64;
        let slot_hash = [0x42; 32];
        let miner_pubkey = Pubkey::from([0x11; 32]);
        let batch_hash = [0x22; 32];
        let nonce = 0x1234567890abcdef_fedcba0987654321u128;

        let preimage = build_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, nonce);
        assert_eq!(preimage.len(), PREIMAGE_SIZE);
        assert_eq!(preimage.len(), 137);
    }

    #[test]
    fn test_preimage_layout() {
        let slot = 0x0102030405060708u64;
        let slot_hash = [0xAA; 32];
        let miner_pubkey = Pubkey::from([0xBB; 32]);
        let batch_hash = [0xCC; 32];
        let nonce = 0x0f0e0d0c0b0a0908_0706050403020100u128;

        let preimage = build_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, nonce);

        // Check domain
        assert_eq!(&preimage[0..17], b"CLOAK:SCRAMBLE:v1");

        // Check slot (LE)
        assert_eq!(
            &preimage[17..25],
            &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );

        // Check slot_hash
        assert_eq!(&preimage[25..57], &[0xAA; 32]);

        // Check miner_pubkey
        assert_eq!(&preimage[57..89], &[0xBB; 32]);

        // Check batch_hash
        assert_eq!(&preimage[89..121], &[0xCC; 32]);

        // Check nonce (LE)
        assert_eq!(
            &preimage[121..137],
            &[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                0x0d, 0x0e, 0x0f
            ]
        );
    }

    #[test]
    fn test_hash_deterministic() {
        let slot = 100u64;
        let slot_hash = [0x33; 32];
        let miner_pubkey = Pubkey::from([0x44; 32]);
        let batch_hash = [0x55; 32];
        let nonce = 42u128;

        let hash1 = hash_pow_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, nonce);
        let hash2 = hash_pow_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, nonce);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_changes_with_nonce() {
        let slot = 100u64;
        let slot_hash = [0x33; 32];
        let miner_pubkey = Pubkey::from([0x44; 32]);
        let batch_hash = [0x55; 32];

        let hash1 = hash_pow_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, 42);
        let hash2 = hash_pow_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, 43);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_pow_success() {
        let slot = 100u64;
        let slot_hash = [0x33; 32];
        let miner_pubkey = Pubkey::from([0x44; 32]);
        let batch_hash = [0x55; 32];
        let nonce = 42u128;

        let expected = hash_pow_preimage(slot, &slot_hash, &miner_pubkey, &batch_hash, nonce);

        assert!(verify_pow(
            slot,
            &slot_hash,
            &miner_pubkey,
            &batch_hash,
            nonce,
            &expected
        ));
    }

    #[test]
    fn test_verify_pow_failure() {
        let slot = 100u64;
        let slot_hash = [0x33; 32];
        let miner_pubkey = Pubkey::from([0x44; 32]);
        let batch_hash = [0x55; 32];
        let nonce = 42u128;

        let wrong_hash = [0xFF; 32];

        assert!(!verify_pow(
            slot,
            &slot_hash,
            &miner_pubkey,
            &batch_hash,
            nonce,
            &wrong_hash
        ));
    }
}
