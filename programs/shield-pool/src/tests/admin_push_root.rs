use mollusk_svm::result::Check;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{instructions::ShieldPoolInstruction, state::RootsRing, tests::setup};

#[test]
fn test_admin_push_root_instruction() {
    let (program_id, mut mollusk) = setup();

    // Create admin account (must match ADMIN_AUTHORITY in the program)
    let admin_pubkey = Pubkey::new_from_array(five8_const::decode_32_const(
        "mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa",
    ));

    // Create a roots ring PDA
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    // Create test root to push
    let new_root = [0x42u8; 32]; // Test root

    // Create instruction data according to AdminPushRootIx format
    let instruction_data = [
        vec![ShieldPoolInstruction::AdminPushRoot as u8], // Instruction discriminant for AdminPushRoot
        new_root.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(admin_pubkey, true),    // admin is signer
            AccountMeta::new(roots_ring_pda, false), // roots_ring is writable (but not signer)
        ],
    );

    // Initialize RootsRing account with proper data
    let roots_ring_data = vec![0u8; RootsRing::SIZE];
    // Initialize the ring buffer (head = 0, all roots = 0)
    // This matches what RootsRing::init() does

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            admin_pubkey,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: solana_sdk::system_program::id(), // Admin is a regular account
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(RootsRing::SIZE),
                data: roots_ring_data,
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);

    assert!(
        !result.program_result.is_err(),
        "AdminPushRoot test failed: {:?}",
        result.program_result
    );

    // Verify that the root was actually added to the ring
    let updated_account = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == roots_ring_pda)
        .map(|(_, acc)| acc)
        .expect("roots_ring account not found after");

    // Check that the head was updated (should be 1 after first push)
    let head = updated_account.data[0];
    assert_eq!(head, 1, "Head should be 1 after first root push");

    // Check that the root was stored at the correct position
    // Root should be at offset 8 + (head * 32) = 8 + (1 * 32) = 40
    let stored_root = &updated_account.data[40..72];
    assert_eq!(
        stored_root, new_root,
        "Root was not stored correctly in the ring buffer"
    );

    println!("✅ AdminPushRoot test completed successfully");
    println!("   - Head: {}", head);
    println!("   - Root: {}", hex::encode(new_root));
}

#[test]
fn test_admin_push_root_unauthorized() {
    let (program_id, mut mollusk) = setup();

    // Create unauthorized admin account (not the correct ADMIN_AUTHORITY)
    let unauthorized_admin = Pubkey::new_from_array([0x99u8; 32]);

    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let new_root = [0x42u8; 32];

    let instruction_data = [
        vec![ShieldPoolInstruction::AdminPushRoot as u8],
        new_root.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(unauthorized_admin, true), // unauthorized admin
            AccountMeta::new(roots_ring_pda, false),
        ],
    );

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            unauthorized_admin,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: solana_sdk::system_program::id(),
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(RootsRing::SIZE),
                data: vec![0u8; RootsRing::SIZE],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);

    // Should fail due to unauthorized admin
    assert!(
        result.program_result.is_err(),
        "AdminPushRoot should fail with unauthorized admin"
    );

    println!("✅ Unauthorized admin test completed successfully");
}

#[test]
fn test_admin_push_root_multiple_roots() {
    let (program_id, mut mollusk) = setup();

    let admin_pubkey = Pubkey::new_from_array(five8_const::decode_32_const(
        "mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa",
    ));

    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    // Push first root
    let root1 = [0x42u8; 32];
    let instruction_data1 = [
        vec![ShieldPoolInstruction::AdminPushRoot as u8],
        root1.to_vec(),
    ]
    .concat();

    let instruction1 = Instruction::new_with_bytes(
        program_id,
        &instruction_data1,
        vec![
            AccountMeta::new(admin_pubkey, true),
            AccountMeta::new(roots_ring_pda, false),
        ],
    );

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            admin_pubkey,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: solana_sdk::system_program::id(),
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            roots_ring_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(RootsRing::SIZE),
                data: vec![0u8; RootsRing::SIZE],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    let result1 = mollusk.process_and_validate_instruction(&instruction1, &accounts, &[]);
    if result1.program_result.is_err() {
        println!("First root push should succeed");
    }
   

    // Get updated accounts for second push
    let updated_accounts = result1.resulting_accounts;

    // Push second root
    let root2 = [0x43u8; 32];
    let instruction_data2 = [
        vec![ShieldPoolInstruction::AdminPushRoot as u8],
        root2.to_vec(),
    ]
    .concat();

    let instruction2 = Instruction::new_with_bytes(
        program_id,
        &instruction_data2,
        vec![
            AccountMeta::new(admin_pubkey, true),
            AccountMeta::new(roots_ring_pda, false),
        ],
    );

    let result2 = mollusk.process_and_validate_instruction(&instruction2, &updated_accounts, &[]);
    assert!(
        !result2.program_result.is_err(),
        "Second root push should succeed"
    );

    // Verify both roots are stored correctly
    let final_account = result2
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == roots_ring_pda)
        .map(|(_, acc)| acc)
        .expect("roots_ring account not found after second push");

    let head = final_account.data[0];
    assert_eq!(head, 2, "Head should be 2 after second root push");

    // Check first root is still there
    let stored_root1 = &final_account.data[40..72]; // offset 8 + (1 * 32)
    assert_eq!(stored_root1, root1, "First root should still be stored");

    // Check second root is stored
    let stored_root2 = &final_account.data[72..104]; // offset 8 + (2 * 32)
    assert_eq!(stored_root2, root2, "Second root should be stored");

    println!("✅ Multiple roots test completed successfully");
    println!("   - Head: {}", head);
    println!("   - Root 1: {}", hex::encode(root1));
    println!("   - Root 2: {}", hex::encode(root2));
}
