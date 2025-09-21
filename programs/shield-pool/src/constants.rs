/// SP1 Withdraw Circuit VKey Hash
pub const WITHDRAW_VKEY_HASH: &str =
    "0x00111a0ebe7f27f7ab2e9cce10e77fcae52d24bb2425498700f437232917ad83";

/// Program constants
pub const ROOTS_RING_SIZE: usize = 64;
pub const ROOTS_RING_LEN: usize = 8 + (ROOTS_RING_SIZE * 32); // head + roots
pub const NULLIFIER_SHARD_HEADER_SIZE: usize = 4; // count field

/// Instruction discriminators
pub const TAG_DEPOSIT: u8 = 0x01;
pub const TAG_ADMIN_PUSH_ROOT: u8 = 0x02;
pub const TAG_WITHDRAW: u8 = 0x03;

/// SP1 proof constants
pub const SP1_PROOF_SIZE: usize = 260;
pub const SP1_PUBLIC_INPUTS_SIZE: usize = 104; // root(32) + nf(32) + outputs_hash(32) + amount(8)

/// Hash sizes
pub const HASH_SIZE: usize = 32;
pub const PUBKEY_SIZE: usize = 32;

/// Fee calculation
pub const FEE_BASIS_POINTS_DENOMINATOR: u64 = 10_000;
pub const FIXED_FEE_LAMPORTS: u64 = 50_000_000; // 0.05 SOL
