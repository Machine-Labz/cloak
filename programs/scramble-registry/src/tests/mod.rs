use mollusk_svm::Mollusk;
use solana_sdk::pubkey::Pubkey;

#[cfg(test)]
mod initialize;

#[cfg(test)]
mod mine_claim;

#[cfg(test)]
mod reveal_claim;

#[cfg(test)]
mod consume_claim;

pub fn setup() -> (Pubkey, Mollusk) {
    let program_id = Pubkey::new_from_array(crate::ID);
    let mollusk = Mollusk::new(&program_id, "../../target/deploy/scramble_registry");

    (program_id, mollusk)
}
