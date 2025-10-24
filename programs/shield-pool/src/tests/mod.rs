use mollusk_svm::Mollusk;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::AccountState;

#[cfg(test)]
mod deposit;

#[cfg(test)]
mod admin_push_root;

#[cfg(test)]
mod withdraw;

#[cfg(test)]
mod batch_withdraw;

pub fn setup() -> (Pubkey, Mollusk) {
    let program_id = Pubkey::new_from_array(five8_const::decode_32_const(
        "11111111111111111111111111111111111111111111",
    ));
    let mut mollusk = Mollusk::new(&program_id, "../../target/deploy/shield_pool");
    mollusk_svm_programs_token::token::add_program(&mut mollusk);

    (program_id, mollusk)
}

pub fn _pack_mint(mint_authority: &Pubkey, supply: u64) -> AccountSharedData {
    let mut account = AccountSharedData::new(0, spl_token::state::Mint::LEN, &spl_token::id());
    spl_token::state::Mint {
        mint_authority: COption::Some(*mint_authority),
        supply,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    }
    .pack_into_slice(account.data_as_mut_slice());
    account
}

pub fn _pack_token_account(owner: &Pubkey, mint: &Pubkey, amount: u64) -> AccountSharedData {
    let mut account = AccountSharedData::new(0, spl_token::state::Mint::LEN, &spl_token::id());
    spl_token::state::Account {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(account.data_as_mut_slice());
    account
}
