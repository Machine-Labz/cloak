use mollusk_svm::result::Check;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{constants::*, state::RootsRing, tests::setup};

#[test]
fn test_complete_privacy_flow() {
    let (program_id, mollusk) = setup();

    // === Accounts ===
    let admin = Pubkey::new_from_array(five8_const::decode_32_const(
        "11111111111111111111111111111111111111111111",
    )); // ✅ CORRETO: deve coincidir com ADMIN_AUTHORITY
    let depositor = Pubkey::new_from_array([0x22u8; 32]);
    let recipient = Pubkey::new_from_array([0x33u8; 32]);

    let pool_pda = Pubkey::new_from_array([0x44u8; 32]);
    let treasury_pda = Pubkey::new_from_array([0x55u8; 32]);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let (nullifier_shard_pda, _) =
        Pubkey::find_program_address(&[b"nullifier_shard", &[0u8]], &program_id);

    // === Constants ===
    let deposit_amount = 5_000_000_000u64; // 5 SOL
    let withdraw_amount = 3_000_000_000u64; // 3 SOL
    let commitment = [0x42u8; 32]; // Commitment from deposit
    let merkle_root = [0x43u8; 32]; // Root containing our commitment
    let nullifier = [0x44u8; 32]; // Nullifier for withdraw

    println!("🔄 Testing complete privacy flow:");
    println!("  1. 💰 Deposit {} lamports", deposit_amount);
    println!("  2. 🔐 Admin pushes Merkle root");
    println!("  3. 💸 Withdraw {} lamports", withdraw_amount);

    // === STEP 1: Deposit ===
    let deposit_instruction_data = [
        vec![0], // Deposit discriminant
        deposit_amount.to_le_bytes().to_vec(),
        commitment.to_vec(),
        4u16.to_le_bytes().to_vec(),  // enc_output_len
        vec![0xAA, 0xBB, 0xCC, 0xDD], // enc_output
    ]
    .concat();

    println!("After deposit data");

    let deposit_instruction = Instruction::new_with_bytes(
        program_id,
        &deposit_instruction_data,
        vec![
            AccountMeta::new(depositor, true), // signer
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(roots_ring_pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    println!("After deposit instruction creation");

    let mut accounts: Vec<(Pubkey, Account)> = vec![
        (
            depositor,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0) + deposit_amount + 1_000_000, // Extra for fees
                data: vec![],
                owner: program_id, // ✅ CORRIGIDO: Program deve ser owner para modificar lamports
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            pool_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: program_id,
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

    println!("After accounts");

    // Execute deposit
    let deposit_result = mollusk.process_and_validate_instruction(
        &deposit_instruction,
        &accounts,
        &[Check::success()],
    );

    println!("After deposit processing and validation");

    assert!(
        deposit_result.program_result.is_ok(),
        "Deposit should succeed"
    );
    accounts = deposit_result.resulting_accounts; // Update accounts with deposit results

    println!("✅ Step 1: Deposit completed");

    // === STEP 2: Admin Push Root ===
    let admin_instruction_data = [
        vec![1], // AdminPushRoot discriminant
        merkle_root.to_vec(),
    ]
    .concat();

    let admin_instruction = Instruction::new_with_bytes(
        program_id,
        &admin_instruction_data,
        vec![
            AccountMeta::new(admin, true),           // admin signer
            AccountMeta::new(roots_ring_pda, false), // writable
        ],
    );

    // Add admin account
    accounts.push((
        admin,
        Account {
            lamports: mollusk.sysvars.rent.minimum_balance(0),
            data: vec![],
            owner: solana_sdk::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    ));

    let admin_result = mollusk.process_and_validate_instruction(
        &admin_instruction,
        &accounts,
        &[Check::success()],
    );

    assert!(
        admin_result.program_result.is_ok(),
        "Admin push root should succeed"
    );
    accounts = admin_result.resulting_accounts;

    println!("✅ Step 2: Admin pushed Merkle root");

    // === STEP 3: Withdraw (will fail at SP1 verification) ===
    let fee_bps = 60u16;
    let fee_amount = (withdraw_amount * fee_bps as u64) / FEE_BASIS_POINTS_DENOMINATOR;
    let recipient_amount = withdraw_amount - fee_amount;

    let mut outputs_hash_data = Vec::new();
    outputs_hash_data.extend_from_slice(recipient.as_ref());
    outputs_hash_data.extend_from_slice(&recipient_amount.to_le_bytes());
    let outputs_hash = blake3::hash(&outputs_hash_data);
    let outputs_hash_bytes: [u8; 32] = *outputs_hash.as_bytes();

    let sp1_proof = [0xAAu8; SP1_PROOF_SIZE];
    let sp1_public_inputs = [0xBBu8; SP1_PUBLIC_INPUTS_SIZE];

    let withdraw_instruction_data = [
        vec![2], // Withdraw discriminant
        sp1_proof.to_vec(),
        sp1_public_inputs.to_vec(),
        merkle_root.to_vec(),
        nullifier.to_vec(),
        withdraw_amount.to_le_bytes().to_vec(),
        fee_bps.to_le_bytes().to_vec(),
        outputs_hash_bytes.to_vec(),
        vec![1u8], // num_outputs
        recipient.to_bytes().to_vec(),
        recipient_amount.to_le_bytes().to_vec(),
    ]
    .concat();

    let withdraw_instruction = Instruction::new_with_bytes(
        program_id,
        &withdraw_instruction_data,
        vec![
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(treasury_pda, false),
            AccountMeta::new_readonly(roots_ring_pda, false),
            AccountMeta::new(nullifier_shard_pda, false),
            AccountMeta::new(recipient, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    // Add remaining accounts
    accounts.extend(vec![
        (
            treasury_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            nullifier_shard_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(4 + 32 * 10),
                data: vec![0u8; 4 + 32 * 10],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            recipient,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: solana_sdk::system_program::id(),
                executable: false,
                rent_epoch: 0,
            },
        ),
    ]);

    let withdraw_result =
        mollusk.process_and_validate_instruction(&withdraw_instruction, &accounts, &[]);

    assert!(
        withdraw_result.program_result.is_ok(),
        "Withdraw should succeed"
    );
    accounts = withdraw_result.resulting_accounts;

    println!("✅ Step 3: Withdraw completed");

    // === STEP 4: Verify balances ===
    let pool_balance = accounts
        .iter()
        .find(|(pk, _)| *pk == pool_pda)
        .map(|(_, acc)| acc.lamports)
        .expect("pool account not found");
    let treasury_balance = accounts
        .iter()
        .find(|(pk, _)| *pk == treasury_pda)
        .map(|(_, acc)| acc.lamports)
        .expect("treasury account not found");
    let recipient_balance = accounts
        .iter()
        .find(|(pk, _)| *pk == recipient)
        .map(|(_, acc)| acc.lamports)
        .expect("recipient account not found");

    println!("🔍 Final Balances:");
    println!("   Pool: {} lamports", pool_balance);
    println!("   Treasury: {} lamports", treasury_balance);
    println!("   Recipient: {} lamports", recipient_balance);
}
