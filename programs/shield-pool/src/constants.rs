use five8_const::decode_32_const;

/// SP1 Withdraw Circuit VKey Hash
pub const WITHDRAW_VKEY_HASH: &str =
    "0x00ddd4143efe13f6bad7ab5b316ba3b4cdec48f781b2edadc9a026320ad940a6";

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
