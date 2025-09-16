use mollusk_svm::result::Check;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
};

use crate::{state::RootsRing, tests::setup};

#[test]
fn deposit_test() {
    let (program_id, mollusk) = setup();

    let user = Pubkey::new_from_array(five8_const::decode_32_const(
        "11111111111111111111111111111111111111111111",
    ));
    let signer = user;

    let (pool_info_pda, _) =
        Pubkey::find_program_address(&[b"pool_info", &user.to_bytes()], &program_id);
    let (roots_ring_info_pda, _) =
        Pubkey::find_program_address(&[b"roots_ring_info", &user.to_bytes()], &program_id);

    // Create instruction data according to DepositIx format
    let amount = 100_000u64;
    let leaf_commit = [0x42u8; 32];
    let enc_output = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let enc_output_len = enc_output.len() as u16;

    let instruction_data = [
        vec![0],
        amount.to_le_bytes().to_vec(),
        leaf_commit.to_vec(),
        enc_output_len.to_le_bytes().to_vec(),
        enc_output.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(pool_info_pda, false),
            AccountMeta::new(roots_ring_info_pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    let mut accounts: Vec<(Pubkey, Account)> = vec![
        (
            signer,
            Account {
                lamports: mollusk
                    .sysvars
                    .rent
                    .minimum_balance(spl_token::state::Account::LEN),
                data: vec![0u8; spl_token::state::Account::LEN],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            pool_info_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![0u8; 0],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring_info_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(RootsRing::SIZE),
                data: vec![0u8; RootsRing::SIZE],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            solana_sdk::system_program::id(),
            Account {
                lamports: 0,
                data: vec![],
                owner: solana_sdk::system_program::id(),
                executable: true,
                rent_epoch: 0,
            },
        ),
    ];

    // Get user's balance before deposit
    let user_balance_before = accounts
        .iter()
        .find(|(pk, _)| *pk == signer)
        .map(|(_, acc)| acc.lamports)
        .expect("user account not found");

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    assert!(
        !result.program_result.is_err(),
        "Deposit test failed: {:?}",
        result.program_result
    );

    // Get user's balance after deposit using resulting_accounts
    let user_balance_after = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == signer)
        .map(|(_, acc)| acc.lamports)
        .expect("user account not found after");

    assert!(
        user_balance_after < user_balance_before,
        "User's balance did not decrease after deposit: before={}, after={}",
        user_balance_before,
        user_balance_after
    );
}
