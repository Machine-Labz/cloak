use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

pub mod nullifier_shard;
pub mod roots_ring;

pub use nullifier_shard::NullifierShard;
pub use roots_ring::RootsRing;

/// Context: Holds program_id, accounts and instruction data for instruction processing
pub struct Context<'info> {
    pub program_id: &'info Pubkey,
    pub accounts: &'info [AccountInfo],
    pub instruction_data: &'info [u8],
}

impl<'info> From<(&'info Pubkey, &'info [AccountInfo], &'info [u8])> for Context<'info> {
    #[inline(always)]
    fn from(value: (&'info Pubkey, &'info [AccountInfo], &'info [u8])) -> Self {
        Context {
            program_id: value.0,
            accounts: value.1,
            instruction_data: value.2,
        }
    }
}
