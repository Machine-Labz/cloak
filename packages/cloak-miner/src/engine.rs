//! Mining Engine - Core PoW nonce search
//!
//! Implements single-threaded brute-force search for valid nonces.
//! Future: Add multi-threaded variant for performance.

use anyhow::{anyhow, Result};
use blake3::Hasher;
use solana_sdk::pubkey::Pubkey;

/// Domain prefix for PoW preimage
const DOMAIN: &[u8] = b"CLOAK:SCRAMBLE:v1";

/// Total preimage size: 17 + 8 + 32 + 32 + 32 + 16 = 137 bytes
const PREIMAGE_SIZE: usize = 137;

/// Mining solution containing nonce and resulting hash
#[derive(Debug, Clone)]
pub struct MiningSolution {
    /// Nonce that produces valid hash
    pub nonce: u128,
    /// BLAKE3 hash of preimage (proof_hash)
    pub proof_hash: [u8; 32],
    /// Number of hash attempts made
    pub attempts: u64,
    /// Time taken to find solution
    pub mining_time: std::time::Duration,
}

/// Mining engine for finding valid PoW nonces
#[derive(Debug, Clone)]
pub struct MiningEngine {
    /// Target difficulty (256-bit LE, hash must be < this)
    pub difficulty_target: [u8; 32],
    /// Slot being referenced (from SlotHashes)
    pub slot: u64,
    /// Slot hash from SlotHashes sysvar
    pub slot_hash: [u8; 32],
    /// Miner's public key
    pub miner_pubkey: Pubkey,
    /// Batch commitment hash
    pub batch_hash: [u8; 32],
}

impl MiningEngine {
    /// Create new mining engine with parameters
    pub fn new(
        difficulty_target: [u8; 32],
        slot: u64,
        slot_hash: [u8; 32],
        miner_pubkey: Pubkey,
        batch_hash: [u8; 32],
    ) -> Self {
        Self {
            difficulty_target,
            slot,
            slot_hash,
            miner_pubkey,
            batch_hash,
        }
    }

    /// Build PoW preimage from components
    ///
    /// Layout (137 bytes):
    /// - Domain: "CLOAK:SCRAMBLE:v1" (17 bytes)
    /// - Slot: u64 LE (8 bytes)
    /// - Slot hash: [u8; 32] (32 bytes)
    /// - Miner pubkey: [u8; 32] (32 bytes)
    /// - Batch hash: [u8; 32] (32 bytes)
    /// - Nonce: u128 LE (16 bytes)
    pub fn build_preimage(&self, nonce: u128) -> [u8; PREIMAGE_SIZE] {
        let mut preimage = [0u8; PREIMAGE_SIZE];
        let mut offset = 0;

        // Domain (17 bytes)
        preimage[offset..offset + DOMAIN.len()].copy_from_slice(DOMAIN);
        offset += DOMAIN.len();

        // Slot (8 bytes LE)
        preimage[offset..offset + 8].copy_from_slice(&self.slot.to_le_bytes());
        offset += 8;

        // Slot hash (32 bytes)
        preimage[offset..offset + 32].copy_from_slice(&self.slot_hash);
        offset += 32;

        // Miner pubkey (32 bytes)
        preimage[offset..offset + 32].copy_from_slice(self.miner_pubkey.as_ref());
        offset += 32;

        // Batch hash (32 bytes)
        preimage[offset..offset + 32].copy_from_slice(&self.batch_hash);
        offset += 32;

        // Nonce (16 bytes LE)
        preimage[offset..offset + 16].copy_from_slice(&nonce.to_le_bytes());

        preimage
    }

    /// Hash preimage with BLAKE3
    pub fn hash_preimage(&self, nonce: u128) -> [u8; 32] {
        let preimage = self.build_preimage(nonce);
        let mut hasher = Hasher::new();
        hasher.update(&preimage);
        *hasher.finalize().as_bytes()
    }

    /// Check if hash meets difficulty target (256-bit LE comparison)
    ///
    /// Returns true if hash < difficulty_target
    pub fn check_difficulty(&self, hash: &[u8; 32]) -> bool {
        u256_lt(hash, &self.difficulty_target)
    }

    /// Mine for valid nonce (single-threaded brute-force)
    ///
    /// Searches from nonce=0 upward until valid hash found.
    /// Logs progress every 1M attempts.
    ///
    /// Returns Some(solution) on success, None if search exhausted (unlikely)
    pub fn mine(&self) -> Result<MiningSolution> {
        tracing::info!(
            "Mining started: slot={}, target_difficulty={:x?}...",
            self.slot,
            &self.difficulty_target[28..32]
        );

        let start_time = std::time::Instant::now();
        let mut attempts = 0u64;

        for nonce in 0u128.. {
            let hash = self.hash_preimage(nonce);

            if self.check_difficulty(&hash) {
                let elapsed = start_time.elapsed();
                let hash_rate = attempts as f64 / elapsed.as_secs_f64();

                tracing::info!(
                    "Mining SUCCESS: nonce={}, attempts={}, time={:.2}s, hash_rate={:.0} H/s",
                    nonce,
                    attempts,
                    elapsed.as_secs_f64(),
                    hash_rate
                );

                return Ok(MiningSolution {
                    nonce,
                    proof_hash: hash,
                    attempts,
                    mining_time: elapsed,
                });
            }

            attempts += 1;

            // Log progress every 1M attempts with better formatting
            if attempts % 1_000_000 == 0 {
                let elapsed = start_time.elapsed();
                let hash_rate = attempts as f64 / elapsed.as_secs_f64();
                tracing::info!(
                    "Mining progress: {}M attempts, {:.2}s, {:.0} H/s",
                    attempts / 1_000_000,
                    elapsed.as_secs_f64(),
                    hash_rate
                );
            }

            // Safety: prevent infinite loop (should never happen)
            if nonce == u128::MAX {
                return Err(anyhow!("Nonce space exhausted (this should be impossible)"));
            }
        }

        Err(anyhow!("Mining failed unexpectedly"))
    }

    /// Mine with timeout
    ///
    /// Stops mining after timeout duration, returns error if not found.
    /// Useful for preventing indefinite blocking on high difficulty.
    pub fn mine_with_timeout(&self, timeout: std::time::Duration) -> Result<MiningSolution> {
        let start_time = std::time::Instant::now();
        let mut attempts = 0u64;

        for nonce in 0u128.. {
            // Check timeout
            if start_time.elapsed() > timeout {
                let hash_rate = attempts as f64 / timeout.as_secs_f64();
                return Err(anyhow!(
                    "Mining timeout after {:.2}s ({} attempts, {:.0} H/s)",
                    timeout.as_secs_f64(),
                    attempts,
                    hash_rate
                ));
            }

            let hash = self.hash_preimage(nonce);

            if self.check_difficulty(&hash) {
                let elapsed = start_time.elapsed();
                let hash_rate = attempts as f64 / elapsed.as_secs_f64();

                tracing::info!(
                    "Mining SUCCESS: nonce={}, attempts={}, time={:.2}s, hash_rate={:.0} H/s",
                    nonce,
                    attempts,
                    elapsed.as_secs_f64(),
                    hash_rate
                );

                return Ok(MiningSolution {
                    nonce,
                    proof_hash: hash,
                    attempts,
                    mining_time: elapsed,
                });
            }

            attempts += 1;

            // Log progress every 1M attempts
            if attempts % 1_000_000 == 0 {
                let elapsed = start_time.elapsed();
                let hash_rate = attempts as f64 / elapsed.as_secs_f64();
                tracing::info!(
                    "Mining progress: {}M attempts, {:.2}s, {:.0} H/s",
                    attempts / 1_000_000,
                    elapsed.as_secs_f64(),
                    hash_rate
                );
            }
        }

        Err(anyhow!("Mining failed unexpectedly"))
    }
}

/// Compare two 32-byte arrays as 256-bit little-endian unsigned integers
///
/// Returns true if a < b
fn u256_lt(a: &[u8; 32], b: &[u8; 32]) -> bool {
    // Compare from most significant byte (index 31) to least (index 0)
    for i in (0..32).rev() {
        if a[i] < b[i] {
            return true;
        } else if a[i] > b[i] {
            return false;
        }
    }
    // Equal
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preimage_size() {
        let engine = MiningEngine::new(
            [0xFF; 32],
            12345,
            [0x42; 32],
            Pubkey::new_unique(),
            [0x88; 32],
        );

        let preimage = engine.build_preimage(0);
        assert_eq!(preimage.len(), PREIMAGE_SIZE);
        assert_eq!(preimage.len(), 137);
    }

    #[test]
    fn test_preimage_layout() {
        let slot = 0x0102030405060708u64;
        let slot_hash = [0xAA; 32];
        let miner = Pubkey::new_from_array([0xBB; 32]);
        let batch_hash = [0xCC; 32];
        let nonce = 0x0f0e0d0c0b0a0908_0706050403020100u128;

        let engine = MiningEngine::new([0xFF; 32], slot, slot_hash, miner, batch_hash);
        let preimage = engine.build_preimage(nonce);

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
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f
            ]
        );
    }

    #[test]
    fn test_u256_lt() {
        // Simple comparison
        let a = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let b = [
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        assert!(u256_lt(&a, &b));
        assert!(!u256_lt(&b, &a));

        // High bytes matter more
        let mut c = [0xFF; 32];
        c[31] = 0x00; // Most significant byte smaller
        let mut d = [0x00; 32];
        d[31] = 0x01; // Only most significant byte set

        assert!(u256_lt(&c, &d));
        assert!(!u256_lt(&d, &c));
    }

    #[test]
    fn test_mine_easy_difficulty() {
        // Set very easy difficulty (all 0xFF = accept any hash)
        let engine = MiningEngine::new(
            [0xFF; 32],
            100,
            [0x42; 32],
            Pubkey::new_unique(),
            [0x88; 32],
        );

        let solution = engine.mine().expect("Mining should succeed");
        assert_eq!(solution.nonce, 0); // First nonce should succeed

        // Verify hash matches
        let expected_hash = engine.hash_preimage(0);
        assert_eq!(solution.proof_hash, expected_hash);
    }

    #[test]
    fn test_mine_moderate_difficulty() {
        // Set moderate difficulty: hash must start with 0x00 (1/256 chance)
        let mut difficulty = [0xFF; 32];
        difficulty[0] = 0x01; // First byte must be < 0x01 (i.e., 0x00)

        let engine = MiningEngine::new(
            difficulty,
            200,
            [0x33; 32],
            Pubkey::new_unique(),
            [0x77; 32],
        );

        let solution = engine.mine().expect("Mining should succeed");

        // Verify solution is valid
        assert!(u256_lt(&solution.proof_hash, &difficulty));
    }

    #[test]
    fn test_deterministic_hash() {
        let engine = MiningEngine::new(
            [0xFF; 32],
            300,
            [0x11; 32],
            Pubkey::new_unique(),
            [0x22; 32],
        );

        let hash1 = engine.hash_preimage(42);
        let hash2 = engine.hash_preimage(42);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_changes_with_nonce() {
        let engine = MiningEngine::new(
            [0xFF; 32],
            400,
            [0x55; 32],
            Pubkey::new_unique(),
            [0x66; 32],
        );

        let hash1 = engine.hash_preimage(0);
        let hash2 = engine.hash_preimage(1);

        assert_ne!(hash1, hash2);
    }
}
