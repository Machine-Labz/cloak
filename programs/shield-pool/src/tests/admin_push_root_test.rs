use crate::state::RootsRing;
use mollusk_svm::Mollusk;
use solana_sdk::{
    account::Account, instruction::Instruction, pubkey::Pubkey, system_program,
};
use five8_const::decode_32_const;

#[test]
fn test_admin_push_root() {
    let program_id = Pubkey::new_from_array(decode_32_const(
        "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp",
    ));
    let mut mollusk = Mollusk::new(&program_id, "../../target/deploy/shield_pool");

    // Create test accounts
    let admin = Pubkey::new_from_array([0x11u8; 32]);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    // Create instruction data: new_root (32 bytes)
    let new_root = [0x42u8; 32];
    let instruction_data = [
        vec![2], // AdminPushRoot instruction discriminator
        new_root.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            solana_sdk::instruction::AccountMeta::new(admin, true),      // admin
            solana_sdk::instruction::AccountMeta::new(roots_ring_pda, false), // roots_ring
        ],
    );

    let mut accounts: Vec<(Pubkey, Account)> = vec![
        (
            admin,
            Account {
                lamports: 1_000_000_000,
                data: vec![],
                owner: system_program::id(),
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

    println!("Testing admin push root...");
    println!("Admin: {}", admin);
    println!("Roots Ring PDA: {}", roots_ring_pda);
    println!("New root: {}", hex::encode(new_root));

    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[],
    );

    if result.program_result.is_ok() {
        println!("✅ Admin push root succeeded!");
        
        // Verify the root was pushed
        let updated_accounts = result.resulting_accounts;
        let roots_ring_account = updated_accounts
            .iter()
            .find(|(pk, _)| *pk == roots_ring_pda)
            .expect("Roots ring account not found");
        
        let roots_ring = RootsRing::from_account_data(&roots_ring_account.1.data)
            .expect("Failed to parse RootsRing");
        
        println!("RootsRing head: {}", roots_ring.head());
        assert!(roots_ring.contains_root(&new_root), "Root should be in RootsRing");
        println!("✅ Root verification passed!");
    } else {
        println!("❌ Admin push root failed: {:?}", result.program_result);
        panic!("Admin push root should succeed");
    }
}
