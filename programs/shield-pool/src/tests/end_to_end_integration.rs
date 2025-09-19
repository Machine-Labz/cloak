use std::{
    process::{Child, Command, Stdio},
    str::FromStr,
    time::Duration,
};
use tokio::time::sleep;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;
use reqwest::Client;
use serde_json::json;

/// End-to-end integration test with real indexer and local validator
#[tokio::test]
async fn test_full_integration_with_real_indexer() {
    println!("🚀 Starting full end-to-end integration test");
    println!("   This test will:");
    println!("   1. Start a local Solana validator");
    println!("   2. Deploy the shield-pool program");
    println!("   3. Start the indexer service");
    println!("   4. Make real deposits via HTTP API");
    println!("   5. Verify Merkle tree construction");
    println!("   6. Test admin root pushing");
    println!();

    // === STEP 1: Start Local Validator ===
    println!("🔧 Starting local Solana validator...");
    let validator_process = start_local_validator().await;
    
    // Wait for validator to be ready
    sleep(Duration::from_secs(5)).await;
    
    let rpc_url = "http://localhost:8899";
    let rpc_client = RpcClient::new(rpc_url);
    
    // Wait for validator to be responsive
    let mut attempts = 0;
    while attempts < 10 {
        if rpc_client.get_health().is_ok() {
            break;
        }
        sleep(Duration::from_secs(1)).await;
        attempts += 1;
    }
    
    assert!(attempts < 10, "Validator failed to start within 10 seconds");
    println!("   ✅ Validator is running at {}", rpc_url);

    // === STEP 2: Deploy Program ===
    println!("📦 Deploying shield-pool program...");
    let program_id = deploy_program(&rpc_client).await;
    println!("   ✅ Program deployed with ID: {}", program_id);

    // === STEP 3: Start Indexer Service ===
    println!("🌐 Starting indexer service...");
    let indexer_process = start_indexer_service().await;
    
    // Wait for indexer to be ready
    sleep(Duration::from_secs(3)).await;
    
    let indexer_url = "http://localhost:3030";
    let http_client = Client::new();
    
    // Test indexer health
    let health_response = http_client
        .get(&format!("{}/merkle/root", indexer_url))
        .send()
        .await;
    
    // Indexer should respond (even if empty initially)
    assert!(health_response.is_ok(), "Indexer service not responding");
    println!("   ✅ Indexer is running at {}", indexer_url);

    // === STEP 4: Create Test Accounts ===
    println!("👥 Creating test accounts...");
    let admin_keypair = Keypair::new();
    let depositor1_keypair = Keypair::new();
    let depositor2_keypair = Keypair::new();
    
    // Airdrop SOL to accounts
    airdrop_sol(&rpc_client, &admin_keypair.pubkey(), 10_000_000_000).await; // 10 SOL
    airdrop_sol(&rpc_client, &depositor1_keypair.pubkey(), 5_000_000_000).await; // 5 SOL
    airdrop_sol(&rpc_client, &depositor2_keypair.pubkey(), 3_000_000_000).await; // 3 SOL
    
    println!("   ✅ Test accounts funded");

    // === STEP 5: Make Deposits via HTTP API ===
    println!("💰 Making deposits via indexer API...");
    
    let deposits = [
        (depositor1_keypair.pubkey(), 1_000_000_000u64, "Alice"),
        (depositor2_keypair.pubkey(), 2_000_000_000u64, "Bob"),
    ];

    for (i, (depositor, amount, name)) in deposits.iter().enumerate() {
        println!("   💰 {} depositing {} SOL", name, amount / 1_000_000_000);
        
        // Create commitment hash
        let commitment = create_commitment_hash(depositor, *amount, i as u64 + 1);
        let commitment_hex = hex::encode(commitment);
        
        // Send deposit to indexer via HTTP
        let deposit_payload = json!({
            "commitment": commitment_hex,
            "slot": 0,
            "signature": format!("test_signature_{}", i)
        });
        
        let response = http_client
            .post(&format!("{}/deposit", indexer_url))
            .json(&deposit_payload)
            .send()
            .await
            .expect("Failed to send deposit to indexer");
        
        assert!(response.status().is_success(), "Indexer deposit failed");
        
        let result: serde_json::Value = response.json().await.expect("Failed to parse response");
        println!("      📝 Indexer response: {}", result);
        
        // Also make the actual on-chain deposit
        make_onchain_deposit(&rpc_client, &program_id, &depositor1_keypair, *amount, &commitment).await;
    }

    // === STEP 6: Verify Merkle Tree ===
    println!("🌳 Verifying Merkle tree construction...");
    
    let root_response = http_client
        .get(&format!("{}/merkle/root", indexer_url))
        .send()
        .await
        .expect("Failed to get Merkle root");
    
    let root_data: serde_json::Value = root_response.json().await.expect("Failed to parse root response");
    println!("   📊 Merkle root: {}", root_data["root"]);
    println!("   📊 Tree size: {}", root_data["tree_size"]);
    
    assert_eq!(root_data["tree_size"], 2, "Should have 2 commitments in tree");

    // === STEP 7: Test Merkle Proof Generation ===
    println!("🔍 Testing Merkle proof generation...");
    
    let proof_response = http_client
        .get(&format!("{}/merkle/proof/0", indexer_url))
        .send()
        .await
        .expect("Failed to get Merkle proof");
    
    let proof_data: serde_json::Value = proof_response.json().await.expect("Failed to parse proof response");
    println!("   🔗 Proof for index 0: {}", proof_data);
    
    assert!(proof_data["proof"].is_array(), "Proof should be an array");
    assert_eq!(proof_data["index"], 0, "Should be proof for index 0");

    // === STEP 8: Test Admin Root Push ===
    println!("🔐 Testing admin root push...");
    
    let latest_root = root_data["root"].as_str().unwrap();
    let root_bytes = hex::decode(latest_root).unwrap_or_else(|_| {
        let mut root = [0u8; 32];
        let root_str_bytes = latest_root.as_bytes();
        root[..root_str_bytes.len().min(32)].copy_from_slice(&root_str_bytes[..root_str_bytes.len().min(32)]);
        root.to_vec()
    });
    
    let mut root_array = [0u8; 32];
    root_array[..root_bytes.len().min(32)].copy_from_slice(&root_bytes[..root_bytes.len().min(32)]);

    // Create admin push root instruction
    let admin_instruction_data = [
        vec![1], // AdminPushRoot discriminant
        root_array.to_vec(),
    ]
    .concat();

    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    
    let admin_instruction = Instruction::new_with_bytes(
        program_id,
        &admin_instruction_data,
        vec![
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new(roots_ring_pda, false),
        ],
    );

    // Create and send transaction
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("Failed to get recent blockhash");

    let transaction = Transaction::new_signed_with_payer(
        &[admin_instruction],
        Some(&admin_keypair.pubkey()),
        &[&admin_keypair],
        recent_blockhash,
    );

    let signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("Failed to send admin transaction");

    println!("   ✅ Admin root push transaction: {}", signature);

    // === STEP 9: Verify Integration ===
    println!("🎯 Verifying full integration...");
    
    // Check that we can query the indexer for all deposits
    let notes_response = http_client
        .get(&format!("{}/notes/range?start=0&end=10", indexer_url))
        .send()
        .await
        .expect("Failed to get notes range");
    
    let notes_data: serde_json::Value = notes_response.json().await.expect("Failed to parse notes response");
    println!("   📋 Notes in range: {}", notes_data);
    
    assert!(notes_data.is_array(), "Notes should be an array");
    assert_eq!(notes_data.as_array().unwrap().len(), 2, "Should have 2 notes");

    // === CLEANUP ===
    println!("🧹 Cleaning up...");
    
    if let Some(mut indexer) = indexer_process {
        let _ = indexer.kill();
    }
    
    if let Some(mut validator) = validator_process {
        let _ = validator.kill();
    }

    println!("✅ Full integration test completed successfully!");
    println!("🎉 Real indexer + local validator + deployed program working perfectly!");
}

/// Start a local Solana validator
async fn start_local_validator() -> Option<Child> {
    let mut cmd = Command::new("solana-test-validator");
    cmd.args(&[
        "--reset",
        "--quiet",
        "--ledger", "test-ledger",
        "--rpc-port", "8899",
        "--faucet-port", "9900",
    ]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    match cmd.spawn() {
        Ok(child) => {
            println!("   🔧 Validator process started (PID: {})", child.id());
            Some(child)
        }
        Err(e) => {
            println!("   ❌ Failed to start validator: {}", e);
            None
        }
    }
}

/// Deploy the shield-pool program to the validator
async fn deploy_program(rpc_client: &RpcClient) -> Pubkey {
    let program_id = Pubkey::from_str("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp").unwrap();
    
    // Build the program
    let build_result = Command::new("cargo")
        .args(&["build-sbf", "--manifest-path", "Cargo.toml"])
        .current_dir("/Users/marcelofeitoza/Development/solana/cloak/programs/shield-pool")
        .output()
        .expect("Failed to build program");
    
    if !build_result.status.success() {
        panic!("Program build failed: {}", String::from_utf8_lossy(&build_result.stderr));
    }
    
    // Deploy the program
    let deploy_result = Command::new("solana")
        .args(&[
            "program", "deploy",
            "target/deploy/shield_pool.so",
            "--url", "localhost",
            "--program-id", &program_id.to_string(),
        ])
        .output()
        .expect("Failed to deploy program");
    
    if !deploy_result.status.success() {
        panic!("Program deployment failed: {}", String::from_utf8_lossy(&deploy_result.stderr));
    }
    
    program_id
}

/// Start the indexer service
async fn start_indexer_service() -> Option<Child> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", "cloak-indexer"]);
    cmd.current_dir("/Users/marcelofeitoza/Development/solana/cloak");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    match cmd.spawn() {
        Ok(child) => {
            println!("   🌐 Indexer process started (PID: {})", child.id());
            Some(child)
        }
        Err(e) => {
            println!("   ❌ Failed to start indexer: {}", e);
            None
        }
    }
}

/// Airdrop SOL to an account
async fn airdrop_sol(rpc_client: &RpcClient, pubkey: &Pubkey, lamports: u64) {
    let signature = rpc_client
        .request_airdrop(pubkey, lamports)
        .await
        .expect("Failed to request airdrop");
    
    // Wait for confirmation
    loop {
        if let Ok(Some(_)) = rpc_client.get_signature_status(&signature) {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
}

/// Make an on-chain deposit
async fn make_onchain_deposit(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    depositor: &Keypair,
    amount: u64,
    commitment: &[u8; 32],
) {
    let pool_pda = Pubkey::new_from_array([0x55u8; 32]); // Use same PDA as in tests
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], program_id);
    
    let instruction_data = [
        vec![0], // Deposit discriminant
        amount.to_le_bytes().to_vec(),
        commitment.to_vec(),
        4u16.to_le_bytes().to_vec(),  // enc_output_len
        vec![0xAA, 0xBB, 0xCC, 0xDD], // enc_output
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        *program_id,
        &instruction_data,
        vec![
            AccountMeta::new(depositor.pubkey(), true),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(roots_ring_pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("Failed to get recent blockhash");

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&depositor.pubkey()),
        &[depositor],
        recent_blockhash,
    );

    let signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("Failed to send deposit transaction");
    
    println!("      🔗 On-chain deposit: {}", signature);
}

/// Create a realistic commitment hash
fn create_commitment_hash(depositor: &Pubkey, amount: u64, nonce: u64) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
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
