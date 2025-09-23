/// SP1 Withdraw Circuit VKey Hash
pub const WITHDRAW_VKEY_HASH: &str =
    "0x00bd8641c64a42569ccb77a27f17ef25ebd948047ab0791466eaca34a8da76ad";

// Constants for proof and public input offsets
pub const PROOF_LEN: usize = 260;
pub const PUB_LEN: usize = 104;

pub const PROOF_OFF: usize = 0;
pub const PUB_OFF: usize = PROOF_OFF + PROOF_LEN;

pub const PUB_ROOT_OFF: usize = PUB_OFF + 0;
pub const PUB_NF_OFF: usize = PUB_OFF + 32;
pub const PUB_OUT_HASH_OFF: usize = PUB_OFF + 64;
pub const PUB_AMOUNT_OFF: usize = PUB_OFF + 96;

// Recipient data offsets
pub const NULLIFIER_OFF: usize = PUB_OFF + PUB_LEN; // 364
pub const NULLIFIER_LEN: usize = 32;
pub const NUM_OUTPUTS_OFF: usize = NULLIFIER_OFF + NULLIFIER_LEN; // 396
pub const RECIP_OFF: usize = NUM_OUTPUTS_OFF + 1; // 397
pub const RECIP_ADDR_LEN: usize = 32;
pub const RECIP_AMT_OFF: usize = RECIP_OFF + RECIP_ADDR_LEN; // 429
