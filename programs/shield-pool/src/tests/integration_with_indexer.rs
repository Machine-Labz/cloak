use mollusk_svm::result::Check;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{constants::*, state::RootsRing, tests::setup};

/// Create a realistic commitment hash from depositor and amount
fn create_commitment_hash(depositor: &Pubkey, amount: u64, nonce: u64) -> [u8; 32] {
    let mut hasher = DefaultHasher::new();
    depositor.hash(&mut hasher);
    amount.hash(&mut hasher);
    nonce.hash(&mut hasher);
    let hash = hasher.finish();
    
    let mut commitment = [0u8; 32];
    commitment[0..8].copy_from_slice(&hash.to_le_bytes());
    commitment[8..16].copy_from_slice(&(hash << 1).to_le_bytes());
    commitment[16..24].copy_from_slice(&(hash << 2).to_le_bytes());
    commitment[24..32].copy_from_slice(&(hash << 3).to_le_bytes());
    commitment
}

/// Simulate what an indexer would do with deposit events
struct SimpleIndexer {
    commitments: Vec<String>,
    merkle_roots: Vec<String>,
}

impl SimpleIndexer {
    fn new() -> Self {
        Self {
            commitments: Vec::new(),
            merkle_roots: Vec::new(),
        }
    }

    fn add_commitment(&mut self, commitment: &str) {
        println!("📝 Indexer: Adding commitment {}", commitment);
        self.commitments.push(commitment.to_string());
        
        // Simulate building Merkle tree and computing root
        let root = self.compute_merkle_root();
        self.merkle_roots.push(root.clone());
        println!("🌳 Indexer: New Merkle root computed: {}", root);
    }

    fn compute_merkle_root(&self) -> String {
        // Simple simulation of Merkle root computation
        let mut hasher = DefaultHasher::new();
        for commitment in &self.commitments {
            commitment.hash(&mut hasher);
        }
        let root_hash = hasher.finish();
        hex::encode(root_hash.to_le_bytes())
    }

    fn get_latest_root(&self) -> Option<String> {
        self.merkle_roots.last().cloned()
    }

    fn get_commitment_count(&self) -> usize {
        self.commitments.len()
    }
}

#[test]
fn test_integration_with_indexer() {
    let (program_id, mollusk) = setup();

    // Initialize indexer
    let mut indexer = SimpleIndexer::new();

    // === Test Setup ===
    let admin = Pubkey::new_from_array(five8_const::decode_32_const(
        "11111111111111111111111111111111111111111111",
    ));
    let depositor1 = Pubkey::new_from_array([0x22u8; 32]);
    let depositor2 = Pubkey::new_from_array([0x33u8; 32]);
    let depositor3 = Pubkey::new_from_array([0x44u8; 32]);

    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], &program_id);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);

    println!("🚀 Starting integration test with indexer simulation");
    println!("   Program ID: {}", program_id);
    println!("   Pool PDA: {}", pool_pda);
    println!("   Roots Ring PDA: {}", roots_ring_pda);
    println!();

    // === STEP 1: Multiple Deposits ===
    let deposits = [
        (depositor1, 1_000_000_000u64, 1u64), // 1 SOL
        (depositor2, 2_000_000_000u64, 2u64), // 2 SOL  
        (depositor3, 500_000_000u64, 3u64),   // 0.5 SOL
    ];

    let mut accounts: Vec<(Pubkey, Account)> = vec![
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

    for (i, (depositor, amount, nonce)) in deposits.iter().enumerate() {
        println!("💰 Processing deposit {} from {}", i + 1, depositor);
        
        // Add depositor account if not already present (must be first in accounts list)
        if !accounts.iter().any(|(pk, _)| pk == depositor) {
            accounts.insert(0, (
                *depositor,
                Account {
                    lamports: mollusk.sysvars.rent.minimum_balance(0) + amount + 1_000_000,
                    data: vec![],
                    owner: solana_sdk::system_program::id(), // Depositor is a regular account
                    executable: false,
                    rent_epoch: 0,
                },
            ));
        }
        
        // Create realistic commitment
        let commitment = create_commitment_hash(depositor, *amount, *nonce);
        let commitment_hex = hex::encode(commitment);
        
        // Create deposit instruction
        let deposit_instruction_data = [
            vec![1], // Deposit discriminant (1, not 0)
            amount.to_le_bytes().to_vec(),
            commitment.to_vec(),
            4u16.to_le_bytes().to_vec(),  // enc_output_len
            vec![0xAA, 0xBB, 0xCC, 0xDD], // enc_output
        ]
        .concat();

        // Debug: Print instruction data
        println!("   🔍 Debug: Instruction data length: {}", deposit_instruction_data.len());
        println!("   🔍 Debug: First byte (discriminant): {}", deposit_instruction_data[0]);

        let deposit_instruction = Instruction::new_with_bytes(
            program_id,
            &deposit_instruction_data,
            vec![
                AccountMeta::new(*depositor, true),
                AccountMeta::new(pool_pda, false),
                AccountMeta::new(roots_ring_pda, false),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
        );

        // Debug: Print account information
        println!("   🔍 Debug: Instruction accounts:");
        for (i, account_meta) in deposit_instruction.accounts.iter().enumerate() {
            println!("      {}: {} (is_signer: {}, is_writable: {})", 
                i, account_meta.pubkey, account_meta.is_signer, account_meta.is_writable);
        }
        println!("   🔍 Debug: Available accounts:");
        for (i, (pubkey, account)) in accounts.iter().enumerate() {
            println!("      {}: {} (lamports: {})", i, pubkey, account.lamports);
        }

        // Execute deposit
        let deposit_result = mollusk.process_and_validate_instruction(
            &deposit_instruction,
            &accounts,
            &[Check::success()],
        );

        assert!(
            deposit_result.program_result.is_ok(),
            "Deposit {} should succeed",
            i + 1
        );
        accounts = deposit_result.resulting_accounts;

        // Simulate indexer processing the deposit event
        indexer.add_commitment(&commitment_hex);
        
        println!("   ✅ Deposit {} completed", i + 1);
        println!("   📊 Indexer now has {} commitments", indexer.get_commitment_count());
        println!();
    }

    // === STEP 2: Admin Push Root ===
    if let Some(latest_root) = indexer.get_latest_root() {
        println!("🔐 Admin pushing latest Merkle root: {}", latest_root);
        
        // Convert hex root back to bytes
        let root_bytes = hex::decode(&latest_root).unwrap_or_else(|_| {
            // If hex decode fails, create a proper 32-byte array
            let mut root = [0u8; 32];
            let root_str_bytes = latest_root.as_bytes();
            root[..root_str_bytes.len().min(32)].copy_from_slice(&root_str_bytes[..root_str_bytes.len().min(32)]);
            root.to_vec()
        });
        
        let mut root_array = [0u8; 32];
        root_array[..root_bytes.len().min(32)].copy_from_slice(&root_bytes[..root_bytes.len().min(32)]);

        let admin_instruction_data = [
            vec![2], // AdminPushRoot discriminant
            root_array.to_vec(),
        ]
        .concat();

        let admin_instruction = Instruction::new_with_bytes(
            program_id,
            &admin_instruction_data,
            vec![
                AccountMeta::new(admin, true),
                AccountMeta::new(roots_ring_pda, false),
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

        println!("   ✅ Root pushed to on-chain RootsRing");
        println!();
    }

    // === STEP 3: Verify Integration ===
    println!("🎯 Integration Test Results:");
    println!("   📊 Total deposits processed: {}", deposits.len());
    println!("   🌳 Total Merkle roots computed: {}", indexer.merkle_roots.len());
    println!("   🔗 Latest root: {}", indexer.get_latest_root().unwrap_or("None".to_string()));
    println!();
    
    // Verify that we have the expected number of commitments
    assert_eq!(indexer.get_commitment_count(), 3);
    assert_eq!(indexer.merkle_roots.len(), 3);
    
    println!("✅ Integration test completed successfully!");
    println!("🎉 Shield Pool program + Indexer integration working!");
}

#[test]
fn test_realistic_privacy_flow_simulation() {
    println!("🔒 Simulating complete privacy flow with indexer...");
    
    let (program_id, _mollusk) = setup();
    let mut indexer = SimpleIndexer::new();
    
    // Simulate multiple users making deposits over time
    let users = [
        ("Alice", Pubkey::new_from_array([0x11u8; 32]), 5_000_000_000u64),
        ("Bob", Pubkey::new_from_array([0x22u8; 32]), 3_000_000_000u64),
        ("Charlie", Pubkey::new_from_array([0x33u8; 32]), 1_000_000_000u64),
        ("Diana", Pubkey::new_from_array([0x44u8; 32]), 7_500_000_000u64),
    ];

    println!("👥 Users making deposits:");
    for (i, (name, pubkey, amount)) in users.iter().enumerate() {
        let commitment = create_commitment_hash(pubkey, *amount, i as u64 + 1);
        let commitment_hex = hex::encode(commitment);
        
        println!("   💰 {} deposits {} SOL", name, amount / 1_000_000_000);
        println!("      Commitment: {}", commitment_hex);
        
        indexer.add_commitment(&commitment_hex);
        
        if i == 1 {
            println!("      🌳 After {} deposits, root: {}", i + 2, indexer.get_latest_root().unwrap());
        }
    }

    println!();
    println!("📈 Final Privacy Pool State:");
    println!("   Total deposits: {}", indexer.get_commitment_count());
    println!("   Anonymity set size: {}", indexer.get_commitment_count());
    println!("   Latest Merkle root: {}", indexer.get_latest_root().unwrap());
    println!("   Program ID: {}", program_id);
    
    // At this point, users could withdraw from the pool using zero-knowledge proofs
    // without revealing which deposit they're withdrawing from
    println!();
    println!("🎭 Privacy achieved!");
    println!("   Users can now withdraw anonymously using ZK proofs");
    println!("   Each withdrawal can come from any of the {} deposits", indexer.get_commitment_count());
    
    assert_eq!(indexer.get_commitment_count(), 4);
    assert!(indexer.get_latest_root().is_some());
}
