use five8_const::decode_32_const;

/// SP1 Withdraw Circuit VKey Hash
pub const WITHDRAW_VKEY_HASH: &str =
    "0x005a30dd3f168f7ec0e019c122ab3661454cff0be2462406c3844edaf5ef721e";

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
