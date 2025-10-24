use five8_const::decode_32_const;

/// SP1 Withdraw Circuit VKey Hash (updated for batch support)
pub const WITHDRAW_VKEY_HASH: &str =
    "0x007fa4568b7cec225191ed264b459df3455dfb26ea1aea361650f3080cec3f65";
pub const GLOBAL_PROGRAM_ID: [u8; 32] =
    decode_32_const("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");

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
