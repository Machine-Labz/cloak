/// PoW domain tag for BLAKE3 hashing
pub const DOMAIN: &[u8] = b"CLOAK:SCRAMBLE:v1";

/// Job commitment domain tag
pub const JOB_DOMAIN: &[u8] = b"CLOAK:JOB:v1";

/// Batch commitment domain tag
pub const BATCH_DOMAIN: &[u8] = b"CLOAK:BATCH:v1";

/// Preimage size: DOMAIN(17) + slot(8) + slot_hash(32) + miner(32) + batch_hash(32) + nonce(16)
pub const PREIMAGE_SIZE: usize = 17 + 8 + 32 + 32 + 32 + 16; // 137 bytes

/// Maximum fee share basis points (5000 = 50%)
pub const MAX_FEE_SHARE_BPS: u16 = 5000;

/// Maximum batch size to prevent DoS
pub const MAX_BATCH_SIZE: u16 = 20;

/// SlotHashes sysvar pubkey
pub const SLOT_HASHES_SYSVAR: [u8; 32] = [
    0x06, 0xa7, 0xd5, 0x17, 0x19, 0x2c, 0x56, 0x8e, 0xe0, 0x8a, 0x84, 0x5f, 0x73, 0xd2, 0x97, 0x88,
    0xcf, 0x03, 0x5c, 0x30, 0x48, 0xb1, 0x5b, 0x36, 0x06, 0x36, 0xf6, 0xbb, 0x56, 0xda, 0x41, 0xbb,
];

/// Default difficulty retarget interval (slots)
pub const DEFAULT_RETARGET_INTERVAL: u64 = 1000;

/// Default target: 1 solution per this many slots
pub const DEFAULT_TARGET_INTERVAL: u64 = 100;

/// Difficulty adjustment clamp (±20% per epoch)
pub const DIFFICULTY_CLAMP_MIN: f64 = 0.8;
pub const DIFFICULTY_CLAMP_MAX: f64 = 1.2;
