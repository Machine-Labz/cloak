use blake3::Hasher;
use mollusk_svm::result::Check;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{instructions::ShieldPoolInstruction, state::RootsRing, tests::setup};

#[test]
fn test_deposit_instruction() {
    let (program_id, mut mollusk) = setup();

    // Create test accounts
    let user = Pubkey::new_from_array([0x11u8; 32]);
    let pool = Pubkey::new_from_array([0x22u8; 32]);
    let (roots_ring, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    // Generate test data like in localnet_test.rs
    let amount = 1_000_000_000u64; // 1 SOL
    let sk_spend = [0x42u8; 32];
    let r = [0x43u8; 32];

    // Compute commitment = H(amount || r || pk_spend) exactly like SP1 guest program
    let pk_spend = blake3::hash(&sk_spend);
    let mut hasher = Hasher::new();
    hasher.update(&amount.to_le_bytes());
    hasher.update(&r);
    hasher.update(pk_spend.as_bytes());
    let commitment = hasher.finalize();
    let leaf_commit = commitment.as_bytes();

    // Create instruction data
    let instruction_data = [
        vec![ShieldPoolInstruction::Deposit as u8],
        amount.to_le_bytes().to_vec(),
        leaf_commit.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(user, true),        // user (signer)
            AccountMeta::new(pool, false),       // pool (writable)
            AccountMeta::new(roots_ring, false), // roots_ring (writable)
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    // Setup accounts with proper balances
    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            user,
            Account {
                lamports: 2_000_000_000, // 2 SOL for user
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
                owner: program_id, // Pool owned by program
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring,
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

    // Execute the deposit instruction
    let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);

    // Verify the instruction succeeded
    assert!(
        !result.program_result.is_err(),
        "Deposit instruction should succeed, got: {:?}",
        result.program_result
    );

    // Verify user's balance decreased by the deposit amount
    let updated_user_account = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == user)
        .map(|(_, acc)| acc)
        .expect("User account not found after deposit");

    assert_eq!(
        updated_user_account.lamports,
        1_000_000_000, // 2 SOL - 1 SOL = 1 SOL
        "User balance should decrease by deposit amount"
    );

    // Verify pool's balance increased by the deposit amount
    let updated_pool_account = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == pool)
        .map(|(_, acc)| acc)
        .expect("Pool account not found after deposit");

    assert_eq!(
        updated_pool_account.lamports, amount,
        "Pool balance should increase by deposit amount"
    );

    println!("✅ Deposit test completed successfully");
    println!(
        "   - User balance: {} SOL",
        updated_user_account.lamports / 1_000_000_000
    );
    println!(
        "   - Pool balance: {} SOL",
        updated_pool_account.lamports / 1_000_000_000
    );
    println!("   - Commitment: {}", hex::encode(leaf_commit));
}

#[test]
fn test_deposit_insufficient_funds() {
    let (program_id, mut mollusk) = setup();

    let user = Pubkey::new_from_array([0x11u8; 32]);
    let pool = Pubkey::new_from_array([0x22u8; 32]);
    let (roots_ring, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    let amount = 2_000_000_000u64; // 2 SOL
    let leaf_commit = [0x42u8; 32];

    let instruction_data = [
        vec![ShieldPoolInstruction::Deposit as u8],
        amount.to_le_bytes().to_vec(),
        leaf_commit.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(pool, false),
            AccountMeta::new(roots_ring, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    // User has insufficient funds (only 1 SOL but trying to deposit 2 SOL)
    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            user,
            Account {
                lamports: 1_000_000_000, // Only 1 SOL
                data: vec![],
                owner: solana_sdk::system_program::id(),
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            pool,
            Account {
                lamports: 0,
                data: vec![],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring,
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

    let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);

    // Should fail due to insufficient funds
    assert!(
        result.program_result.is_err(),
        "Deposit should fail with insufficient funds"
    );

    println!("✅ Insufficient funds test completed successfully");
}
