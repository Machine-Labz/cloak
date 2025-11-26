//! Cloak Miner - Standalone PoW mining CLI for Cloak protocol
//! 1. Mines PoW claims continuously
//! 2. Submits mine_claim and reveal_claim transactions
//! 3. Manages claim lifecycle (expiry, consumption tracking)
//! 4. Creates decoy deposits for privacy
//!
//! Miners run this independently and earn fees when their claims are consumed.

pub mod batch;
pub mod constants;
pub mod decoy;
pub mod decoy_instructions;
pub mod decoy_manager;
pub mod engine;
pub mod instructions;
pub mod manager;
pub mod rpc;

pub use batch::{compute_batch_hash, compute_single_job_hash};
pub use decoy::{DecoyNote, NoteStorage};
pub use decoy_instructions::{
    build_deposit_ix, build_register_miner_with_escrow_ix, build_top_up_escrow_ix,
    derive_miner_escrow_pda,
};
pub use decoy_manager::{DecoyManager, DecoyResult, DecoyStats};
pub use engine::{MiningEngine, MiningSolution};
pub use instructions::{
    build_consume_claim_ix, build_mine_and_reveal_instructions, build_mine_claim_ix,
    build_register_miner_ix, build_reveal_claim_ix, derive_claim_pda, derive_miner_pda,
    derive_registry_pda,
};
pub use manager::{ClaimManager, ClaimState};
pub use rpc::{fetch_recent_slot_hash, fetch_registry, get_current_slot, RegistryState};
