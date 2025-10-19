pub mod consume_claim;
pub mod initialize;
pub mod mine_claim;
pub mod reveal_claim;

pub use consume_claim::process_consume_claim;
pub use initialize::{process_initialize_registry, process_register_miner};
pub use mine_claim::process_mine_claim;
pub use reveal_claim::process_reveal_claim;
