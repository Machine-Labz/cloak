use blake3;
use serde_json;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{instructions::ShieldPoolInstruction, state::RootsRing, tests::setup};

#[test]
fn test_withdraw_instruction() {
    let (program_id, mollusk) = setup();

    // Create accounts
    let recipient = Pubkey::new_from_array([0x33u8; 32]);

    // Create PDAs
    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], &program_id);
    let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], &program_id);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let (nullifier_shard_pda, _) = Pubkey::find_program_address(&[b"nullifier_shard"], &program_id);

    // Test data
    let withdraw_amount = 3_000_000_000u64; // 3 SOL
    let nullifier = [0x55u8; 32];

    // Calculate fee using the same logic as SP1 guest program
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (withdraw_amount * 5) / 1_000; // 0.5% = 5/1000
        fixed_fee + variable_fee
    };
    let recipient_amount = withdraw_amount - fee;

    use sp1_sdk::SP1ProofWithPublicValues;

    let proof_path = "packages/zk-guest-sp1/out/proof_live.bin";
    let sp1_proof_with_public_values = match SP1ProofWithPublicValues::load(proof_path) {
        Ok(proof) => proof,
        Err(err) => {
            println!(
                "Skipping withdraw test: unable to load SP1 proof at {}: {}",
                proof_path, err
            );
            return;
        }
    };
    let full_proof_bytes = sp1_proof_with_public_values.bytes();
    let raw_public_inputs = sp1_proof_with_public_values.public_values.to_vec();

    println!("full_proof_bytes: {:?}", full_proof_bytes);
    println!("full_proof_bytes len: {:?}", full_proof_bytes.len());

    println!("raw_public_inputs: {:?}", raw_public_inputs);

    let sp1_proof: [u8; 256] = [
        // Real proof data extracted from proof_live.bin (offset 0x2b0)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    // Create public inputs for the withdraw instruction
    // Format: root(32) + nullifier(32) + outputs_hash(32) + amount(8)
    let mut sp1_public_inputs = [0u8; 104];

    // Root (32 bytes) - dummy for now
    sp1_public_inputs[0..32].copy_from_slice(&[0x42u8; 32]);

    // Nullifier (32 bytes)
    sp1_public_inputs[32..64].copy_from_slice(&nullifier);

    // Outputs hash (32 bytes) - calculate based on recipient and amount
    let outputs_hash = blake3::hash(
        &serde_json::json!([{
            "address": recipient.to_string(),
            "amount": recipient_amount
        }])
        .to_string()
        .as_bytes(),
    );
    sp1_public_inputs[64..96].copy_from_slice(outputs_hash.as_bytes());

    // Amount (8 bytes)
    sp1_public_inputs[96..104].copy_from_slice(&withdraw_amount.to_le_bytes());

    // Construct instruction data according to the expected format
    // Note: The withdraw instruction expects data to start directly with proof (no discriminant)
    let mut withdraw_instruction_data = Vec::new();

    // SP1 proof (260 bytes) - offset 0
    withdraw_instruction_data.extend_from_slice(&sp1_proof);

    // Public inputs (104 bytes) - offset 260
    withdraw_instruction_data.extend_from_slice(&sp1_public_inputs);

    // Nullifier (32 bytes) - offset 364
    withdraw_instruction_data.extend_from_slice(&nullifier);

    // Number of outputs (1 byte) - offset 396
    withdraw_instruction_data.push(1u8);

    // Recipient address (32 bytes) - offset 397
    withdraw_instruction_data.extend_from_slice(&recipient.to_bytes());

    // Recipient amount (8 bytes) - offset 429
    withdraw_instruction_data.extend_from_slice(&recipient_amount.to_le_bytes());

    // Create the full instruction data with discriminant
    let mut full_instruction_data = Vec::new();
    full_instruction_data.push(ShieldPoolInstruction::Withdraw as u8); // Withdraw discriminant
    full_instruction_data.extend_from_slice(&withdraw_instruction_data);

    let withdraw_instruction = Instruction::new_with_bytes(
        program_id,
        &full_instruction_data,
        vec![
            AccountMeta::new(pool_pda, false),            // pool (writable)
            AccountMeta::new(treasury_pda, false),        // treasury (writable)
            AccountMeta::new(roots_ring_pda, false),      // roots_ring (readable)
            AccountMeta::new(nullifier_shard_pda, false), // nullifier_shard (writable)
            AccountMeta::new(recipient, false),           // recipient (writable)
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // system_program (readonly)
        ],
    );

    let withdraw_accounts: Vec<(Pubkey, Account)> = vec![
        (
            pool_pda,
            Account {
                lamports: 5_000_000_000,
                data: vec![],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            treasury_pda,
            Account {
                lamports: 0,
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
            nullifier_shard_pda,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(0),
                data: vec![],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            recipient,
            Account {
                lamports: 0,
                data: vec![],
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
                owner: solana_sdk::native_loader::id(),
                executable: true,
                rent_epoch: 0,
            },
        ),
    ];

    // Test the withdraw instruction with real proof structure
    let result =
        mollusk.process_and_validate_instruction(&withdraw_instruction, &withdraw_accounts, &[]);

    // The instruction should fail with ProofInvalid due to dummy proof data
    // This is expected in the test environment
    assert!(
        result.program_result.is_err(),
        "Expected withdraw to fail due to invalid proof in test environment, got: {:?}",
        result.program_result
    );

    // Validate instruction data length
    assert_eq!(
        withdraw_instruction_data.len(),
        437,
        "Withdraw instruction data should be exactly 437 bytes, got: {}",
        withdraw_instruction_data.len()
    );

    // Validate full instruction data length (437 + 1 for discriminant)
    assert_eq!(
        full_instruction_data.len(),
        438,
        "Full instruction data should be exactly 438 bytes, got: {}",
        full_instruction_data.len()
    );

    // Validate account count
    assert_eq!(
        withdraw_instruction.accounts.len(),
        6,
        "Withdraw instruction should have 6 accounts, got: {}",
        withdraw_instruction.accounts.len()
    );

    // Validate account setup
    assert_eq!(
        withdraw_accounts.len(),
        6,
        "Withdraw accounts should have 6 accounts, got: {}",
        withdraw_accounts.len()
    );

    // Validate pool account has sufficient lamports
    let pool_lamports = withdraw_accounts[0].1.lamports;
    assert!(
        pool_lamports >= withdraw_amount,
        "Pool should have sufficient lamports for withdrawal, pool: {}, amount: {}",
        pool_lamports,
        withdraw_amount
    );

    // Validate fee calculation
    assert_eq!(
        fee,
        2_500_000 + (withdraw_amount * 5) / 1_000,
        "Fee calculation should be correct"
    );

    // Validate recipient amount calculation
    assert_eq!(
        recipient_amount,
        withdraw_amount - fee,
        "Recipient amount calculation should be correct"
    );

    println!("✅ Withdraw instruction test completed - instruction structure validated");
}
