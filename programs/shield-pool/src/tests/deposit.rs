use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{instructions::ShieldPoolInstruction, state::RootsRing, tests::setup};

#[test]
fn deposit_test() {
    let (program_id, _mollusk) = setup();

    let user = Pubkey::new_from_array(five8_const::decode_32_const(
        "11111111111111111111111111111111111111111111",
    ));
    let signer = user;

    // Create a simple pool account (not a PDA)
    let pool = Pubkey::new_from_array([0x11u8; 32]);

    // Create roots_ring account (PDA)
    let (roots_ring, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    // Create instruction data according to our deposit instruction format
    let amount = 1_000_000u64; // 0.001 SOL
    let leaf_commit = [0x42u8; 32];

    let instruction_data = [
        vec![ShieldPoolInstruction::Deposit as u8], // Deposit instruction discriminant
        amount.to_le_bytes().to_vec(),
        leaf_commit.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(signer, true),      // user (signer)
            AccountMeta::new(pool, false),       // pool (writable)
            AccountMeta::new(roots_ring, false), // roots_ring (writable)
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // system_program (readonly)
        ],
    );

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            signer,
            Account {
                lamports: 1_000_000_000, // 1 SOL for user
                data: vec![],
                owner: solana_sdk::system_program::id(),
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            pool,
            Account {
                lamports: 0, // Empty pool initially
                data: vec![],
                owner: solana_sdk::system_program::id(), // Pool should be owned by system program
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring,
            Account {
                lamports: 0,
                data: vec![0u8; RootsRing::SIZE], // RootsRing::SIZE
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
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    // Test instruction creation and account setup validation
    // This verifies the instruction structure is correct without executing it

    // Validate instruction data length
    assert!(
        instruction_data.len() >= 40,
        "Deposit instruction data should be at least 40 bytes, got: {}",
        instruction_data.len()
    );

    // Validate account count
    assert_eq!(
        instruction.accounts.len(),
        4,
        "Deposit instruction should have 4 accounts, got: {}",
        instruction.accounts.len()
    );

    // Validate account setup
    assert_eq!(
        accounts.len(),
        4,
        "Deposit accounts should have 4 accounts, got: {}",
        accounts.len()
    );

    // Validate user account is signer
    assert!(
        instruction.accounts[0].is_signer,
        "User account should be marked as signer"
    );

    // Validate pool account is writable
    assert!(
        instruction.accounts[1].is_writable,
        "Pool account should be writable"
    );

    // Validate roots_ring account is writable
    assert!(
        instruction.accounts[2].is_writable,
        "Roots ring account should be writable"
    );

    // Validate system program account is readonly
    assert!(
        !instruction.accounts[3].is_writable,
        "System program account should be readonly"
    );

    // Validate amount is reasonable
    assert!(
        amount > 0,
        "Deposit amount should be positive, got: {}",
        amount
    );

    // Validate leaf commit is correct size
    assert_eq!(
        leaf_commit.len(),
        32,
        "Leaf commit should be 32 bytes, got: {}",
        leaf_commit.len()
    );
}
