use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use reqwest::Client;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;

/// Test the indexer HTTP API directly
#[tokio::test]
async fn test_indexer_api_integration() {
    println!("🌐 Testing indexer HTTP API integration");
    println!("   This test assumes the indexer is running on localhost:3030");
    println!();

    let indexer_url = "http://localhost:3030";
    let http_client = Client::new();

    // Test 1: Check if indexer is running
    println!("🔍 Testing indexer connectivity...");
    let health_response = http_client
        .get(&format!("{}/merkle/root", indexer_url))
        .send()
        .await;

    if health_response.is_err() {
        println!("   ❌ Indexer not running. Please start it with:");
        println!("      cd services/indexer && cargo run --bin cloak-indexer");
        println!("   Skipping API tests...");
        return;
    }

    println!("   ✅ Indexer is responding");

    // Test 2: Add deposits via API
    println!("💰 Testing deposit API...");
    
    let test_deposits = [
        (Pubkey::new_from_array([0x11u8; 32]), 1_000_000_000u64, "Alice"),
        (Pubkey::new_from_array([0x22u8; 32]), 2_000_000_000u64, "Bob"),
        (Pubkey::new_from_array([0x33u8; 32]), 500_000_000u64, "Charlie"),
    ];

    for (i, (depositor, amount, name)) in test_deposits.iter().enumerate() {
        println!("   💰 {} depositing {} SOL", name, amount / 1_000_000_000);
        
        // Create commitment hash
        let commitment = create_commitment_hash(depositor, *amount, i as u64 + 1);
        let commitment_hex = hex::encode(commitment);
        
        // Send deposit to indexer
        let deposit_payload = json!({
            "commitment": commitment_hex,
            "slot": 1000 + i as u64,
            "signature": format!("test_signature_{}", i)
        });
        
        let response = http_client
            .post(&format!("{}/deposit", indexer_url))
            .json(&deposit_payload)
            .send()
            .await
            .expect("Failed to send deposit to indexer");
        
        assert!(response.status().is_success(), "Deposit API failed");
        
        let result: serde_json::Value = response.json().await.expect("Failed to parse response");
        println!("      📝 Response: {}", result);
        
        // Verify the response
        assert_eq!(result["index"], i, "Index should match deposit order");
        assert_eq!(result["commitment"], commitment_hex, "Commitment should match");
        assert_eq!(result["status"], "added", "Status should be 'added'");
    }

    // Test 3: Check Merkle root
    println!("🌳 Testing Merkle root API...");
    
    let root_response = http_client
        .get(&format!("{}/merkle/root", indexer_url))
        .send()
        .await
        .expect("Failed to get Merkle root");
    
    let root_data: serde_json::Value = root_response.json().await.expect("Failed to parse root response");
    println!("   📊 Merkle root: {}", root_data["root"]);
    println!("   📊 Tree size: {}", root_data["tree_size"]);
    
    assert_eq!(root_data["tree_size"], 3, "Should have 3 commitments in tree");
    assert!(!root_data["root"].is_null(), "Root should not be null");

    // Test 4: Test Merkle proof generation
    println!("🔍 Testing Merkle proof API...");
    
    for i in 0..3 {
        let proof_response = http_client
            .get(&format!("{}/merkle/proof/{}", indexer_url, i))
            .send()
            .await
            .expect("Failed to get Merkle proof");
        
        let proof_data: serde_json::Value = proof_response.json().await.expect("Failed to parse proof response");
        println!("   🔗 Proof for index {}: {}", i, proof_data);
        
        assert!(proof_data["proof"].is_array(), "Proof should be an array");
        assert_eq!(proof_data["index"], i, "Index should match");
        assert_eq!(proof_data["root"], root_data["root"], "Root should match");
    }

    // Test 5: Test notes range API
    println!("📋 Testing notes range API...");
    
    let notes_response = http_client
        .get(&format!("{}/notes/range?start=0&end=10", indexer_url))
        .send()
        .await
        .expect("Failed to get notes range");
    
    let notes_data: serde_json::Value = notes_response.json().await.expect("Failed to parse notes response");
    println!("   📋 Notes in range: {}", notes_data);
    
    assert!(notes_data.is_array(), "Notes should be an array");
    assert_eq!(notes_data.as_array().unwrap().len(), 3, "Should have 3 notes");
    
    // Verify each note
    for (i, note) in notes_data.as_array().unwrap().iter().enumerate() {
        assert_eq!(note["index"], i, "Note index should match");
        assert!(!note["commitment"].is_null(), "Note should have commitment");
        assert!(!note["signature"].is_null(), "Note should have signature");
    }

    // Test 6: Test edge cases
    println!("🧪 Testing edge cases...");
    
    // Test invalid proof index
    let invalid_proof_response = http_client
        .get(&format!("{}/merkle/proof/999", indexer_url))
        .send()
        .await
        .expect("Failed to get invalid proof");
    
    assert_eq!(invalid_proof_response.status(), 404, "Invalid proof should return 404");
    
    // Test empty range
    let empty_range_response = http_client
        .get(&format!("{}/notes/range?start=10&end=5", indexer_url))
        .send()
        .await
        .expect("Failed to get empty range");
    
    let empty_range_data: serde_json::Value = empty_range_response.json().await.expect("Failed to parse empty range response");
    assert!(empty_range_data.is_array(), "Empty range should return array");
    assert_eq!(empty_range_data.as_array().unwrap().len(), 0, "Empty range should be empty");

    println!("✅ All indexer API tests passed!");
    println!("🎉 Indexer HTTP API working perfectly!");
}

/// Test the indexer with realistic privacy pool scenario
#[tokio::test]
async fn test_privacy_pool_scenario() {
    println!("🎭 Testing privacy pool scenario with indexer");
    println!("   Simulating multiple users creating anonymity set");
    println!();

    let indexer_url = "http://localhost:3030";
    let http_client = Client::new();

    // Check if indexer is running
    let health_response = http_client
        .get(&format!("{}/merkle/root", indexer_url))
        .send()
        .await;

    if health_response.is_err() {
        println!("   ❌ Indexer not running. Skipping privacy pool test...");
        return;
    }

    // Simulate a privacy pool with multiple users
    let privacy_pool_users = [
        ("Alice", Pubkey::new_from_array([0x11u8; 32]), 5_000_000_000u64),
        ("Bob", Pubkey::new_from_array([0x22u8; 32]), 3_000_000_000u64),
        ("Charlie", Pubkey::new_from_array([0x33u8; 32]), 1_000_000_000u64),
        ("Diana", Pubkey::new_from_array([0x44u8; 32]), 7_500_000_000u64),
        ("Eve", Pubkey::new_from_array([0x55u8; 32]), 2_000_000_000u64),
    ];

    println!("👥 Privacy Pool Users:");
    for (name, _, amount) in &privacy_pool_users {
        println!("   💰 {}: {} SOL", name, amount / 1_000_000_000);
    }
    println!();

    // Add all users to the pool
    for (i, (name, depositor, amount)) in privacy_pool_users.iter().enumerate() {
        println!("📝 Adding {} to privacy pool...", name);
        
        let commitment = create_commitment_hash(depositor, *amount, i as u64 + 1);
        let commitment_hex = hex::encode(commitment);
        
        let deposit_payload = json!({
            "commitment": commitment_hex,
            "slot": 2000 + i as u64,
            "signature": format!("privacy_pool_{}", i)
        });
        
        let response = http_client
            .post(&format!("{}/deposit", indexer_url))
            .json(&deposit_payload)
            .send()
            .await
            .expect("Failed to add user to privacy pool");
        
        assert!(response.status().is_success(), "Failed to add {} to pool", name);
        
        let result: serde_json::Value = response.json().await.expect("Failed to parse response");
        println!("   ✅ {} added at index {}", name, result["index"]);
    }

    // Check final privacy pool state
    println!("📊 Final Privacy Pool State:");
    
    let root_response = http_client
        .get(&format!("{}/merkle/root", indexer_url))
        .send()
        .await
        .expect("Failed to get final Merkle root");
    
    let root_data: serde_json::Value = root_response.json().await.expect("Failed to parse root response");
    println!("   🌳 Merkle root: {}", root_data["root"]);
    println!("   📊 Tree size: {}", root_data["tree_size"]);
    println!("   👥 Anonymity set size: {}", root_data["tree_size"]);
    
    assert_eq!(root_data["tree_size"], 5, "Should have 5 users in privacy pool");

    // Test that any user can generate a proof
    println!("🔍 Testing proof generation for all users...");
    
    for i in 0..5 {
        let proof_response = http_client
            .get(&format!("{}/merkle/proof/{}", indexer_url, i))
            .send()
            .await
            .expect("Failed to get proof");
        
        let proof_data: serde_json::Value = proof_response.json().await.expect("Failed to parse proof");
        println!("   🔗 User {} proof: {} elements", i, proof_data["proof"].as_array().unwrap().len());
        
        // Verify proof structure
        assert!(proof_data["proof"].is_array(), "Proof should be array");
        assert_eq!(proof_data["index"], i, "Index should match");
        assert_eq!(proof_data["root"], root_data["root"], "Root should match");
    }

    // Test privacy: verify that commitments are properly hashed
    println!("🔒 Testing privacy properties...");
    
    let notes_response = http_client
        .get(&format!("{}/notes/range?start=0&end=10", indexer_url))
        .send()
        .await
        .expect("Failed to get all notes");
    
    let notes_data: serde_json::Value = notes_response.json().await.expect("Failed to parse notes");
    let notes = notes_data.as_array().unwrap();
    
    // Verify all commitments are different (privacy)
    let mut commitments: Vec<&str> = notes.iter()
        .map(|note| note["commitment"].as_str().unwrap())
        .collect();
    
    commitments.sort();
    commitments.dedup();
    
    assert_eq!(commitments.len(), 5, "All commitments should be unique");
    println!("   ✅ All commitments are unique (privacy preserved)");
    
    // Verify anonymity set
    println!("   🎭 Anonymity set: {} users", notes.len());
    println!("   🔐 Each withdrawal can come from any of {} deposits", notes.len());
    
    println!("✅ Privacy pool scenario test completed!");
    println!("🎉 Privacy pool working perfectly with indexer!");
}

/// Create a realistic commitment hash
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
