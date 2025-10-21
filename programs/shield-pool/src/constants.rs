use five8_const::decode_32_const;

/// SP1 Withdraw Circuit VKey Hash
pub const WITHDRAW_VKEY_HASH: &str =
    "0x000b018a2ab0d934353a926ae6783d4f0555c18419bef6745fffea81a796a057";

pub const ADMIN_AUTHORITY: [u8; 32] =
    decode_32_const("mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa");

// Layout constants for withdraw instruction payloads
pub const PROOF_LEN: usize = 260;
pub const PUB_LEN: usize = 104; // root (32) || nullifier (32) || outputs_hash (32) || amount (8)
pub const SP1_PUB_LEN: usize = 104; // SP1 verifier expects the full 104-byte public inputs slice
pub const DUPLICATE_NULLIFIER_LEN: usize = 32;
pub const NUM_OUTPUTS_LEN: usize = 1;
pub const RECIPIENT_ADDR_LEN: usize = 32;
pub const RECIPIENT_AMOUNT_LEN: usize = 8;
pub const POW_BATCH_HASH_LEN: usize = 32;
