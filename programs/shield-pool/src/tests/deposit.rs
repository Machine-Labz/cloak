use blake3::Hasher;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{instructions::ShieldPoolInstruction, state::CommitmentQueue, tests::setup};

#[test]
fn test_deposit_instruction() {
    let (program_id, mollusk) = setup();

    let user = Pubkey::new_from_array([0x11u8; 32]);
    let mint = Pubkey::default(); // Native SOL
    let (pool, _) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], &program_id);
    let (commitments_log, _) =
        Pubkey::find_program_address(&[b"commitments", mint.as_ref()], &program_id);

    let amount = 0u64;
    let sk_spend = [0x42u8; 32];
    let r = [0x43u8; 32];

    let pk_spend = blake3::hash(&sk_spend);
    let mut hasher = Hasher::new();
    hasher.update(&amount.to_le_bytes());
    hasher.update(&r);
    hasher.update(pk_spend.as_bytes());
    let commitment = hasher.finalize();
    let leaf_commit = commitment.as_bytes();

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
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new(commitments_log, false),
        ],
    );

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            user,
            Account {
                lamports: 2_000_000_000,
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
                data: vec![0u8; 32], // Pool::SIZE - all zeros = native SOL (Pubkey::default())
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
        (
            commitments_log,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(CommitmentQueue::SIZE),
                data: vec![0u8; CommitmentQueue::SIZE],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);
    assert!(
        !result.program_result.is_err(),
        "Deposit instruction should succeed, got: {:?}",
        result.program_result
    );

    println!("✅ Deposit test completed successfully");
    println!("   - Commitment: {}", hex::encode(leaf_commit));

    let commitments_account = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == commitments_log)
        .map(|(_, acc)| acc)
        .expect("Commitments account not found after deposit");

    let total_commits = u64::from_le_bytes(
        commitments_account.data[0..8]
            .try_into()
            .expect("commitment log header"),
    );
    assert_eq!(total_commits, 1, "Commitment count should increment");

    let mut stored_commitment = [0u8; 32];
    stored_commitment.copy_from_slice(
        &commitments_account.data[CommitmentQueue::HEADER_SIZE..CommitmentQueue::HEADER_SIZE + 32],
    );
    assert_eq!(
        stored_commitment, *leaf_commit,
        "Stored commitment mismatch"
    );
}

#[test]
fn test_deposit_insufficient_funds() {
    let (program_id, mollusk) = setup();

    let user = Pubkey::new_from_array([0x11u8; 32]);
    let mint = Pubkey::default(); // Native SOL
    let (pool, _) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], &program_id);
    let (commitments_log, _) =
        Pubkey::find_program_address(&[b"commitments", mint.as_ref()], &program_id);

    let amount = 2_000_000_000u64;
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
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new(commitments_log, false),
        ],
    );

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            user,
            Account {
                lamports: 1_000_000_000,
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
                data: vec![0u8; 32], // Pool::SIZE - all zeros = native SOL (Pubkey::default())
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
        (
            commitments_log,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(CommitmentQueue::SIZE),
                data: vec![0u8; CommitmentQueue::SIZE],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);
    assert!(
        result.program_result.is_err(),
        "Deposit should fail due to insufficient funds",
    );
}

#[test]
fn test_deposit_duplicate_commitment() {
    let (program_id, mollusk) = setup();

    let user = Pubkey::new_from_array([0x91u8; 32]);
    let mint = Pubkey::default(); // Native SOL
    let (pool, _) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], &program_id);
    let (commitments_log, _) =
        Pubkey::find_program_address(&[b"commitments", mint.as_ref()], &program_id);

    let amount = 0u64;
    let commitment = [0xAAu8; 32];

    let instruction_data = [
        vec![ShieldPoolInstruction::Deposit as u8],
        amount.to_le_bytes().to_vec(),
        commitment.to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(pool, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new(commitments_log, false),
        ],
    );

    let accounts: Vec<(Pubkey, Account)> = vec![
        (
            user,
            Account {
                lamports: 1_000_000_000,
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
                data: vec![0u8; 32], // Pool::SIZE - all zeros = native SOL (Pubkey::default())
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
        (
            commitments_log,
            Account {
                lamports: mollusk.sysvars.rent.minimum_balance(CommitmentQueue::SIZE),
                data: vec![0u8; CommitmentQueue::SIZE],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ];

    let first = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);
    assert!(
        !first.program_result.is_err(),
        "Initial deposit should succeed",
    );

    let system_program_id = solana_sdk::system_program::id();
    let system_account_template = accounts
        .iter()
        .find(|(pk, _)| *pk == system_program_id)
        .cloned()
        .expect("System program account missing");
    let commitments_pubkey = commitments_log;

    let mut second_accounts = first.resulting_accounts.clone();

    if second_accounts
        .iter()
        .all(|(pk, _)| *pk != system_program_id)
    {
        second_accounts.insert(2, system_account_template);
    }

    second_accounts.sort_by_key(|(pk, _)| {
        if pk == &user {
            0
        } else if pk == &pool {
            1
        } else if pk == &system_program_id {
            2
        } else if pk == &commitments_pubkey {
            3
        } else {
            4
        }
    });

    let second = mollusk.process_and_validate_instruction(&instruction, &second_accounts, &[]);
    assert!(
        second.program_result.is_err(),
        "Duplicate commitment should be rejected",
    );
}
