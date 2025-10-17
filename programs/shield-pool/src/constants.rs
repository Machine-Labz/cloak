use five8_const::decode_32_const;

/// SP1 Withdraw Circuit VKey Hash
pub const WITHDRAW_VKEY_HASH: &str =
    "0x00891071711b7773d95f6a941e9e42ad39b14964cac0c90804c64ec7bfd69d3e";

pub const ADMIN_AUTHORITY: [u8; 32] =
    decode_32_const("mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa");

// Constants for proof and public input offsets
pub const PROOF_LEN: usize = 260; // Groth16 proof length (with vkey hash, as in working version)
pub const PUB_LEN: usize = 104; // Full public inputs length (as in working version)
pub const SP1_PUB_LEN: usize = 104; // SP1 Solana verifier expects 104-byte public inputs (root + nf + outputs_hash + amount)

pub const PROOF_OFF: usize = 0; // No discriminator offset (as in working version)
pub const PUB_OFF: usize = PROOF_OFF + PROOF_LEN;

pub const PUB_ROOT_OFF: usize = PUB_OFF + 0;
pub const PUB_NF_OFF: usize = PUB_OFF + 32;
pub const PUB_OUT_HASH_OFF: usize = PUB_OFF + 64;
pub const PUB_AMOUNT_OFF: usize = PUB_OFF + 96;

// Recipient data offsets
pub const NULLIFIER_OFF: usize = PUB_OFF + PUB_LEN; // 364 (260 + 104)
pub const NULLIFIER_LEN: usize = 32;
pub const NUM_OUTPUTS_OFF: usize = NULLIFIER_OFF + NULLIFIER_LEN; // 396
pub const RECIP_OFF: usize = NUM_OUTPUTS_OFF + 1; // 397
pub const RECIP_ADDR_LEN: usize = 32;
pub const RECIP_AMT_OFF: usize = RECIP_OFF + RECIP_ADDR_LEN; // 429
