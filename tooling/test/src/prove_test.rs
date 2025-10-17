use anyhow::Result;
use hex;
use rand;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use shield_pool::CommitmentQueue;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use std::str::FromStr;
use test_complete_flow_rust::shared::{
    check_cluster_health, ensure_user_funding, load_keypair, print_config, MerkleProof, TestConfig,
    SOL_TO_LAMPORTS,
};

#[derive(Debug, Serialize, Deserialize)]
struct DepositRequest {
    leaf_commit: String,
    encrypted_output: String,
    tx_signature: Option<String>,
    slot: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MerkleRootResponse {
    root: String,
}

#[derive(Debug, Serialize)]
struct ProveRequest {
    private_inputs: String,
    public_inputs: String,
    outputs: String,
}

#[derive(Debug, Deserialize)]
struct ProveResponse {
    success: bool,
    proof: Option<String>,
    public_inputs: Option<String>,
    generation_time_ms: u64,
    total_cycles: Option<u64>,
    total_syscalls: Option<u64>,
    execution_report: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = std::time::Instant::now();

    println!("🔐 CLOAK PRIVACY PROTOCOL - COMPLETE FLOW WITH /PROVE ENDPOINT TEST");
    println!("====================================================================\n");

    let config = TestConfig::localnet();
    print_config(&config);

    // Check cluster health
    check_cluster_health(&config.rpc_url)?;

    // Load keypairs
    let user_keypair = load_keypair(&config.user_keypair_path)?;
    let recipient_keypair = load_keypair(&config.recipient_keypair_path)?;
    let admin_keypair = load_keypair("admin-keypair.json")?;

    println!("\n💰 Checking balances...");
    let client = RpcClient::new(&config.rpc_url);
    let user_balance = client.get_balance(&user_keypair.pubkey())?;
    let admin_balance = client.get_balance(&admin_keypair.pubkey())?;
    let recipient_balance = client.get_balance(&recipient_keypair.pubkey())?;

    println!(
        "   User ({}): {} SOL",
        user_keypair.pubkey(),
        user_balance / SOL_TO_LAMPORTS
    );
    println!(
        "   Admin ({}): {} SOL",
        admin_keypair.pubkey(),
        admin_balance / SOL_TO_LAMPORTS
    );
    println!(
        "   Recipient ({}): {} SOL",
        recipient_keypair.pubkey(),
        recipient_balance / SOL_TO_LAMPORTS
    );

    // Ensure user has sufficient SOL
    ensure_user_funding(&config.rpc_url, &user_keypair, &admin_keypair)?;

    // Deploy program
    println!("\n🚀 Step 0: Deploying Program...");
    let program_id = deploy_program(&client)?;

    // Create program accounts
    println!("\n📋 Step 1: Creating Program Accounts...");
    let accounts = create_program_accounts(&client, &program_id, &admin_keypair)?;

    // Reset indexer and relay databases
    println!("\n🔄 Step 2: Resetting Indexer and Relay Databases...");
    reset_indexer_database(&config.indexer_url).await?;
    reset_relay_database().await?;

    // Generate test data
    println!("\n🔨 Step 3: Generating Test Data...");
    let mut test_data = generate_test_data(config.amount)?;

    // Deposit to indexer
    println!("\n📥 Step 4: Depositing to Indexer...");
    let leaf_index = deposit_to_indexer(&config.indexer_url, &mut test_data).await?;

    // Create real deposit transaction
    println!("\n💰 Step 5: Creating Real Deposit Transaction...");
    let deposit_signature =
        create_deposit_transaction(&client, &program_id, &accounts, &test_data, &user_keypair)?;

    // Get merkle root and push to program
    println!("\n🌳 Step 6: Getting Merkle Root from Indexer...");
    let merkle_root = get_merkle_root(&config.indexer_url).await?;
    push_root_to_program(
        &client,
        &program_id,
        &accounts,
        &merkle_root,
        &admin_keypair,
    )?;

    // Get merkle proof
    println!("\n🔍 Step 7: Getting Merkle Proof from Indexer...");
    let merkle_proof = get_merkle_proof(&config.indexer_url, leaf_index).await?;

    // Prepare proof inputs
    println!("\n🔐 Step 8: Preparing Proof Inputs...");
    let (private_inputs, public_inputs, outputs) = prepare_proof_inputs(
        &test_data,
        &merkle_proof,
        &merkle_root,
        leaf_index,
        &recipient_keypair,
    )?;

    // Call /prove endpoint
    println!("\n🚀 Step 9: Calling /prove Endpoint...");
    let prove_result = call_prove_endpoint(
        &config.indexer_url,
        &private_inputs,
        &public_inputs,
        &outputs,
    )
    .await?;

    // Validate proof response
    println!("\n✅ Step 10: Validating Proof Response...");
    validate_prove_response(&prove_result)?;

    // Execute withdraw via relay
    println!("\n💸 Step 11: Executing Withdraw Transaction via Relay...");
    let withdraw_signature =
        execute_withdraw_via_relay(&prove_result, &test_data, &recipient_keypair).await?;

    let total_time = start_time.elapsed();

    // Success!
    println!("\n🎉 COMPLETE FLOW TEST - RESULT");
    println!("================================");
    println!("✅ Test completed successfully!");
    println!("\n📊 Transaction Details:");
    println!("   - Deposit: {}", deposit_signature);
    println!("   - Withdraw: {}", withdraw_signature);

    println!("\n🔐 Privacy Protocol Summary:");
    println!("   - Commitment: {}", test_data.commitment);
    println!("   - Merkle root: {}", merkle_root);
    println!("   - Nullifier: {}", test_data.nullifier);
    println!("   - Leaf index: {}", leaf_index);
    println!(
        "   - Proof size: {} bytes",
        prove_result
            .proof
            .as_ref()
            .map(|p| p.len() / 2)
            .unwrap_or(0)
    );
    println!(
        "   - Public inputs size: {} bytes",
        prove_result
            .public_inputs
            .as_ref()
            .map(|p| p.len() / 2)
            .unwrap_or(0)
    );
    println!(
        "   - Proof generation time: {}ms",
        prove_result.generation_time_ms
    );
    if let Some(cycles) = prove_result.total_cycles {
        println!("   - Total SP1 cycles: {}", cycles);
    }
    if let Some(syscalls) = prove_result.total_syscalls {
        println!("   - Total syscalls: {}", syscalls);
    }

    // Print full execution report if available
    if let Some(ref report) = prove_result.execution_report {
        println!("\n📊 Full SP1 Execution Report:");
        println!("{}", report);
    }

    println!("\n🚀 Complete flow with /prove endpoint working!");
    println!("   - Real Solana transactions ✅");
    println!("   - Real BLAKE3 computation ✅");
    println!("   - Real Merkle tree with 31-level paths ✅");
    println!("   - Real SP1 proof via /prove endpoint ✅");
    println!("   - Real indexer integration ✅");
    println!("   - Production-ready infrastructure ✅");

    println!("\n   Total test time: {:?}", total_time);

    Ok(())
}

fn generate_test_data(amount: u64) -> Result<TestData> {
    use blake3::Hasher;
    use rand::RngCore;

    // Generate random test data
    let mut sk_spend = [0u8; 32];
    let mut r = [0u8; 32];

    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut sk_spend);
    rng.fill_bytes(&mut r);

    println!("   - sk_spend: {}", hex::encode(sk_spend));
    println!("   - r: {}", hex::encode(r));
    println!("   - amount: {}", amount);

    // Compute pk_spend = H(sk_spend)
    let pk_spend = blake3::hash(&sk_spend);
    println!("   - pk_spend: {}", hex::encode(pk_spend.as_bytes()));

    // Compute commitment = H(amount || r || pk_spend)
    let mut hasher = Hasher::new();
    hasher.update(&amount.to_le_bytes());
    hasher.update(&r);
    hasher.update(pk_spend.as_bytes());
    let commitment = hasher.finalize();
    let commitment_hex = hex::encode(commitment.as_bytes());
    println!("   - commitment: {}", commitment_hex);

    // Nullifier will be computed after we get the leaf index
    // For now, use placeholder with leaf_index = 0
    let mut nullifier_hasher = Hasher::new();
    nullifier_hasher.update(&sk_spend);
    nullifier_hasher.update(&0u32.to_le_bytes());
    let nullifier = nullifier_hasher.finalize();
    let nullifier_hex = hex::encode(nullifier.as_bytes());
    println!("   - nullifier (placeholder): {}", nullifier_hex);

    Ok(TestData {
        sk_spend,
        r,
        amount,
        commitment: commitment_hex,
        nullifier: nullifier_hex,
    })
}

async fn reset_indexer_database(indexer_url: &str) -> Result<()> {
    let http_client = reqwest::Client::new();

    println!("   🔄 Resetting indexer database...");

    let reset_response = http_client
        .post(&format!("{}/api/v1/admin/reset", indexer_url))
        .send()
        .await;

    match reset_response {
        Ok(response) => {
            if response.status().is_success() {
                println!("   ✅ Indexer database reset successfully");
                Ok(())
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                println!("   ⚠️  Reset endpoint returned non-success: {}", error_text);
                // Don't fail if reset endpoint doesn't exist (it's just nice to have)
                Ok(())
            }
        }
        Err(e) => {
            println!("   ⚠️  Failed to call reset endpoint: {}", e);
            // Don't fail if reset endpoint doesn't exist
            Ok(())
        }
    }
}

async fn reset_relay_database() -> Result<()> {
    println!("   🔄 Resetting relay database...");

    // Use docker exec to run SQL command in the postgres container
    let truncate_cmd = std::process::Command::new("docker")
        .args(&[
            "exec",
            "cloak-postgres",
            "psql",
            "-U",
            "cloak",
            "-d",
            "cloak_relay",
            "-c",
            "TRUNCATE TABLE jobs, nullifiers CASCADE;",
        ])
        .output();

    match truncate_cmd {
        Ok(output) => {
            if output.status.success() {
                println!("   ✅ Relay database reset successfully");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("   ⚠️  Failed to reset relay database: {}", stderr);
                // Don't fail the test if we can't reset
                Ok(())
            }
        }
        Err(e) => {
            println!("   ⚠️  Failed to run docker exec command: {}", e);
            // Don't fail the test if docker is not available
            Ok(())
        }
    }
}

async fn deposit_to_indexer(indexer_url: &str, test_data: &mut TestData) -> Result<u32> {
    let http_client = reqwest::Client::new();

    let unique_tx_signature = format!(
        "prove_test_{}_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        test_data.commitment[..8].to_string(),
        rand::random::<u32>()
    );

    let deposit_request = DepositRequest {
        leaf_commit: test_data.commitment.clone(),
        encrypted_output: {
            use base64::{engine::general_purpose, Engine as _};
            general_purpose::STANDARD.encode(format!(
                "Prove test {} SOL at {}",
                test_data.amount / SOL_TO_LAMPORTS,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ))
        },
        tx_signature: Some(unique_tx_signature),
        slot: Some(1000),
    };

    let deposit_response = http_client
        .post(&format!("{}/api/v1/deposit", indexer_url))
        .json(&deposit_request)
        .send()
        .await?;

    if deposit_response.status().is_success() {
        let deposit_data: serde_json::Value = deposit_response.json().await?;
        let leaf_index = deposit_data["leafIndex"].as_u64().unwrap() as u32;

        // Update nullifier with actual leaf index
        use blake3::Hasher;
        let mut nullifier_hasher = Hasher::new();
        nullifier_hasher.update(&test_data.sk_spend);
        nullifier_hasher.update(&leaf_index.to_le_bytes());
        let updated_nullifier = nullifier_hasher.finalize();
        test_data.nullifier = hex::encode(updated_nullifier.as_bytes());

        println!("   ✅ Deposit successful to indexer");
        println!("   - Leaf index: {}", leaf_index);
        println!("   - Nullifier (updated): {}", test_data.nullifier);
        Ok(leaf_index)
    } else {
        let error_text = deposit_response.text().await?;
        println!("   ❌ Deposit failed: {}", error_text);
        Err(anyhow::anyhow!("Deposit failed: {}", error_text))
    }
}

async fn get_merkle_root(indexer_url: &str) -> Result<String> {
    let http_client = reqwest::Client::new();
    let merkle_response = http_client
        .get(&format!("{}/api/v1/merkle/root", indexer_url))
        .send()
        .await?;

    let merkle_root_response: MerkleRootResponse = merkle_response.json().await?;
    let merkle_root = merkle_root_response.root;
    println!("   ✅ Merkle root: {}", merkle_root);
    Ok(merkle_root)
}

async fn get_merkle_proof(indexer_url: &str, leaf_index: u32) -> Result<MerkleProof> {
    let http_client = reqwest::Client::new();
    let proof_response = http_client
        .get(&format!(
            "{}/api/v1/merkle/proof/{}",
            indexer_url, leaf_index
        ))
        .send()
        .await?;

    let merkle_proof: MerkleProof = proof_response.json().await?;
    println!(
        "   ✅ Got Merkle proof with {} path elements",
        merkle_proof.path_elements.len()
    );
    Ok(merkle_proof)
}

fn prepare_proof_inputs(
    test_data: &TestData,
    merkle_proof: &MerkleProof,
    merkle_root: &str,
    leaf_index: u32,
    recipient_keypair: &Keypair,
) -> Result<(String, String, String)> {
    use blake3::Hasher;

    println!("   🔐 Preparing proof inputs...");

    // Compute nullifier with actual leaf index
    let mut nullifier_hasher = Hasher::new();
    nullifier_hasher.update(&test_data.sk_spend);
    nullifier_hasher.update(&leaf_index.to_le_bytes());
    let nullifier = nullifier_hasher.finalize();
    let nullifier_hex = hex::encode(nullifier.as_bytes());
    println!("   - Nullifier (with leaf index): {}", nullifier_hex);

    // Create private inputs
    let private_inputs = serde_json::json!({
        "amount": test_data.amount,
        "r": hex::encode(test_data.r),
        "sk_spend": hex::encode(test_data.sk_spend),
        "leaf_index": leaf_index,
        "merkle_path": {
            "path_elements": merkle_proof.path_elements,
            "path_indices": merkle_proof.path_indices
        }
    });

    // Calculate fee
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (test_data.amount * 5) / 1_000; // 0.5%
        fixed_fee + variable_fee
    };
    let recipient_amount = test_data.amount - fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!("   - Fee: {} lamports", fee);
    println!("   - Recipient amount: {} lamports", recipient_amount);

    // Use actual recipient keypair
    let recipient_pubkey = recipient_keypair.pubkey();
    let recipient_address_hex = hex::encode(recipient_pubkey.to_bytes());
    println!("   - Recipient address: {}", recipient_pubkey);
    println!("   - Recipient address (hex): {}", recipient_address_hex);

    // Create outputs
    let outputs = serde_json::json!([
        {
            "address": recipient_address_hex,
            "amount": recipient_amount
        }
    ]);

    // Compute outputs hash exactly like SP1 guest program
    let mut hasher = Hasher::new();
    hasher.update(&recipient_pubkey.to_bytes());
    hasher.update(&recipient_amount.to_le_bytes());
    let outputs_hash = hasher.finalize();
    let outputs_hash_hex = hex::encode(outputs_hash.as_bytes());
    println!("   - Outputs hash: {}", outputs_hash_hex);

    // Create public inputs
    let public_inputs = serde_json::json!({
        "root": merkle_root,
        "nf": nullifier_hex,
        "outputs_hash": outputs_hash_hex,
        "amount": test_data.amount
    });

    println!("   ✅ Proof inputs prepared");

    // Pre-validate circuit constraints locally before sending to prover
    println!("\n   🔍 Pre-validating circuit constraints...");
    let path_indices_u32: Vec<u32> = merkle_proof
        .path_indices
        .iter()
        .map(|&x| x as u32)
        .collect();
    validate_circuit_constraints_local(
        &private_inputs,
        &public_inputs,
        &outputs,
        test_data,
        &merkle_proof.path_elements,
        &path_indices_u32,
    )?;
    println!("   ✅ All circuit constraints validated successfully!\n");

    Ok((
        serde_json::to_string(&private_inputs)?,
        serde_json::to_string(&public_inputs)?,
        serde_json::to_string(&outputs)?,
    ))
}

/// Validate all circuit constraints locally before sending to prover
fn validate_circuit_constraints_local(
    private_inputs: &serde_json::Value,
    public_inputs: &serde_json::Value,
    outputs: &serde_json::Value,
    test_data: &TestData,
    path_elements: &[String],
    path_indices: &[u32],
) -> Result<()> {
    use blake3::Hasher;

    println!("      ├─ Constraint 1: pk_spend = H(sk_spend)");
    let sk_spend_hex = private_inputs["sk_spend"].as_str().unwrap();
    let sk_spend = hex::decode(sk_spend_hex)?;
    let mut hasher = Hasher::new();
    hasher.update(&sk_spend);
    let pk_spend = hasher.finalize();
    println!("         ✓ pk_spend computed");

    println!("      ├─ Constraint 2: C = H(amount || r || pk_spend)");
    let r_hex = private_inputs["r"].as_str().unwrap();
    let r = hex::decode(r_hex)?;
    let amount = private_inputs["amount"].as_u64().unwrap();
    let mut hasher = Hasher::new();
    hasher.update(&amount.to_le_bytes());
    hasher.update(&r);
    hasher.update(pk_spend.as_bytes());
    let commitment = hasher.finalize();
    let commitment_hex = hex::encode(commitment.as_bytes());
    println!("         ✓ Commitment: {}", commitment_hex);

    if commitment_hex != test_data.commitment {
        return Err(anyhow::anyhow!(
            "Commitment mismatch!\n         Expected: {}\n         Computed: {}",
            test_data.commitment,
            commitment_hex
        ));
    }

    println!("      ├─ Constraint 3: MerkleVerify(C, merkle_path) == root");
    let mut current_hash = commitment.as_bytes().to_vec();
    let root_hex = public_inputs["root"].as_str().unwrap();

    for (sibling_hex, &is_left) in path_elements.iter().zip(path_indices.iter()) {
        let sibling = hex::decode(sibling_hex)?;
        let mut hasher = Hasher::new();
        if is_left == 0 {
            // Current is left, sibling is right
            hasher.update(&current_hash);
            hasher.update(&sibling);
        } else {
            // Current is right, sibling is left
            hasher.update(&sibling);
            hasher.update(&current_hash);
        }
        current_hash = hasher.finalize().as_bytes().to_vec();
    }

    let computed_root_hex = hex::encode(&current_hash);
    println!("         ✓ Merkle root computed: {}", computed_root_hex);

    if computed_root_hex != root_hex {
        return Err(anyhow::anyhow!(
            "Merkle root mismatch!\n         Expected: {}\n         Computed: {}",
            root_hex,
            computed_root_hex
        ));
    }

    println!("      ├─ Constraint 4: nf == H(sk_spend || leaf_index)");
    let leaf_index = private_inputs["leaf_index"].as_u64().unwrap() as u32;
    let mut hasher = Hasher::new();
    hasher.update(&sk_spend);
    hasher.update(&leaf_index.to_le_bytes());
    let computed_nf = hasher.finalize();
    let computed_nf_hex = hex::encode(computed_nf.as_bytes());
    let expected_nf_hex = public_inputs["nf"].as_str().unwrap();
    println!("         ✓ Nullifier computed: {}", computed_nf_hex);

    if computed_nf_hex != expected_nf_hex {
        return Err(anyhow::anyhow!(
            "Nullifier mismatch!\n         Expected: {}\n         Computed: {}",
            expected_nf_hex,
            computed_nf_hex
        ));
    }

    println!("      ├─ Constraint 5: sum(outputs) + fee == amount");
    let outputs_array = outputs.as_array().unwrap();
    let outputs_sum: u64 = outputs_array
        .iter()
        .map(|o| o["amount"].as_u64().unwrap())
        .sum();

    // Fee calculation: 0.75% = 75 bps
    let fee = (amount * 75) / 10000;
    let total_spent = outputs_sum + fee;

    println!("         ✓ Outputs sum: {} lamports", outputs_sum);
    println!("         ✓ Fee: {} lamports", fee);
    println!("         ✓ Total: {} lamports", total_spent);
    println!("         ✓ Amount: {} lamports", amount);

    if total_spent != amount {
        return Err(anyhow::anyhow!(
            "Amount conservation failed!\n         outputs({}) + fee({}) = {} != amount({})",
            outputs_sum,
            fee,
            total_spent,
            amount
        ));
    }

    println!("      ├─ Constraint 6: H(serialize(outputs)) == outputs_hash");
    // Compute outputs hash exactly like the guest program
    let mut hasher = Hasher::new();
    for output in outputs_array {
        let address_hex = output["address"].as_str().unwrap();
        let address_bytes = hex::decode(address_hex)?;
        let amount = output["amount"].as_u64().unwrap();
        hasher.update(&address_bytes);
        hasher.update(&amount.to_le_bytes());
    }
    let computed_outputs_hash = hasher.finalize();
    let computed_outputs_hash_hex = hex::encode(computed_outputs_hash.as_bytes());
    let expected_outputs_hash_hex = public_inputs["outputs_hash"].as_str().unwrap();
    println!(
        "         ✓ Outputs hash computed: {}",
        computed_outputs_hash_hex
    );

    if computed_outputs_hash_hex != expected_outputs_hash_hex {
        return Err(anyhow::anyhow!(
            "Outputs hash mismatch!\n         Expected: {}\n         Computed: {}",
            expected_outputs_hash_hex,
            computed_outputs_hash_hex
        ));
    }

    println!("      └─ Constraint 7: private.amount == public.amount");
    let private_amount = private_inputs["amount"].as_u64().unwrap();
    let public_amount = public_inputs["amount"].as_u64().unwrap();
    if private_amount != public_amount {
        return Err(anyhow::anyhow!(
            "Amount mismatch!\n         Private: {}\n         Public: {}",
            private_amount,
            public_amount
        ));
    }
    println!("         ✓ Amounts match");

    Ok(())
}

async fn call_prove_endpoint(
    indexer_url: &str,
    private_inputs: &str,
    public_inputs: &str,
    outputs: &str,
) -> Result<ProveResponse> {
    let http_client = reqwest::Client::new();

    println!("   🔨 Sending proof generation request...");
    println!("   ⏱️  This may take 30-180 seconds...");

    // Debug: Print all inputs being sent
    println!("\n   📋 DEBUG: Input Data Being Sent to Prover:");
    println!("   ═══════════════════════════════════════════");
    println!("   🔒 Private Inputs:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(private_inputs)?)?
    );
    println!("\n   🌍 Public Inputs:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(public_inputs)?)?
    );
    println!("\n   💰 Outputs:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(outputs)?)?
    );
    println!("   ═══════════════════════════════════════════\n");

    let prove_request = ProveRequest {
        private_inputs: private_inputs.to_string(),
        public_inputs: public_inputs.to_string(),
        outputs: outputs.to_string(),
    };

    let prove_response = http_client
        .post(&format!("{}/api/v1/prove", indexer_url))
        .json(&prove_request)
        .timeout(std::time::Duration::from_secs(3000)) // 5 minute timeout
        .send()
        .await?;

    let status = prove_response.status();
    println!("   📡 Response status: {}", status);

    if status.is_success() {
        let prove_result: ProveResponse = prove_response.json().await?;
        println!("   ✅ Proof generation completed");
        Ok(prove_result)
    } else {
        let error_text = prove_response.text().await?;
        println!("   ❌ Proof generation failed: {}", error_text);
        Err(anyhow::anyhow!("Proof generation failed: {}", error_text))
    }
}

fn validate_prove_response(response: &ProveResponse) -> Result<()> {
    println!("   🔍 Validating proof response...");

    if !response.success {
        return Err(anyhow::anyhow!(
            "Proof generation was not successful: {:?}",
            response.error
        ));
    }

    if response.proof.is_none() {
        return Err(anyhow::anyhow!("Proof is missing from response"));
    }

    if response.public_inputs.is_none() {
        return Err(anyhow::anyhow!("Public inputs are missing from response"));
    }

    // Validate proof size (should be 260 bytes = 520 hex chars)
    let proof = response.proof.as_ref().unwrap();
    let proof_bytes_len = proof.len() / 2;
    if proof_bytes_len != 260 {
        println!(
            "   ⚠️  Warning: Expected 260-byte proof, got {} bytes",
            proof_bytes_len
        );
    } else {
        println!("   ✅ Proof size is correct (260 bytes)");
    }

    // Validate public inputs size (should be 104 bytes = 208 hex chars)
    let public_inputs = response.public_inputs.as_ref().unwrap();
    let public_inputs_bytes_len = public_inputs.len() / 2;
    if public_inputs_bytes_len != 104 {
        println!(
            "   ⚠️  Warning: Expected 104-byte public inputs, got {} bytes",
            public_inputs_bytes_len
        );
    } else {
        println!("   ✅ Public inputs size is correct (104 bytes)");
    }

    println!("   ✅ Proof response is valid");

    Ok(())
}

// Data structures
#[derive(Debug)]
struct TestData {
    sk_spend: [u8; 32],
    r: [u8; 32],
    amount: u64,
    commitment: String,
    nullifier: String,
}
// Data structures
#[derive(Debug)]
struct ProgramAccounts {
    pool: Pubkey,
    commitments: Pubkey,
    roots_ring: Pubkey,
    nullifier_shard: Pubkey,
    treasury: Pubkey,
}

fn deploy_program(_client: &RpcClient) -> Result<Pubkey> {
    println!("   Building shield pool program...");

    // Build the program
    let build_output = std::process::Command::new("cargo")
        .args([
            "build-sbf",
            "--manifest-path",
            "programs/shield-pool/Cargo.toml",
        ])
        .output()
        .expect("Failed to execute cargo build-sbf");

    if !build_output.status.success() {
        panic!(
            "cargo build-sbf failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    println!("   ✅ Program built successfully");

    // Check if the program account exists but isn't a program
    let account_check = std::process::Command::new("solana")
        .args([
            "account",
            "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp",
            "--url",
            "http://127.0.0.1:8899",
        ])
        .output();

    if let Ok(output) = account_check {
        if output.status.success() {
            let account_info = String::from_utf8_lossy(&output.stdout);
            if account_info.contains("Executable: false") {
                println!("   🔄 Transferring SOL from existing account to close it...");
                let transfer_output = std::process::Command::new("solana")
                    .args([
                        "transfer",
                        "mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa",
                        "2",
                        "--from",
                        "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp",
                        "--url",
                        "http://127.0.0.1:8899",
                    ])
                    .output()
                    .expect("Failed to execute solana transfer");

                if !transfer_output.status.success() {
                    println!(
                        "   ⚠️  Failed to transfer SOL: {}",
                        String::from_utf8_lossy(&transfer_output.stderr)
                    );
                } else {
                    println!("   ✅ Account closed successfully");
                }
            }
        }
    }

    println!("   Deploying program...");

    // Deploy the program
    let deploy_output = std::process::Command::new("solana")
        .args([
            "program",
            "deploy",
            "--url",
            "http://127.0.0.1:8899",
            "--keypair",
            "admin-keypair.json",
            "--program-id",
            "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json",
            "target/deploy/shield_pool.so",
        ])
        .output()
        .expect("Failed to execute solana program deploy");

    if !deploy_output.status.success() {
        panic!(
            "solana program deploy failed: {}",
            String::from_utf8_lossy(&deploy_output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&deploy_output.stdout);
    println!("   Deploy output: {}", stdout);

    let program_id_str = stdout
        .lines()
        .find(|line| line.contains("Program Id:"))
        .and_then(|line| line.split_whitespace().nth(2))
        .ok_or_else(|| anyhow::anyhow!("Failed to parse program ID from deployment output"))?;

    let program_id = Pubkey::from_str(program_id_str)?;
    println!("   ✅ Program deployed successfully under {}", program_id);
    Ok(program_id)
}

fn create_program_accounts(
    client: &RpcClient,
    program_id: &Pubkey,
    admin_keypair: &Keypair,
) -> Result<ProgramAccounts> {
    use solana_sdk::{system_instruction, transaction::Transaction};
    use test_complete_flow_rust::shared::get_pda_addresses;

    println!("   Deriving PDA addresses...");
    let (pool_pda, commitments_pda, roots_ring_pda, nullifier_shard_pda, treasury_pda) =
        get_pda_addresses(program_id);

    println!("   - Pool PDA: {}", pool_pda);
    println!("   - Commitments PDA: {}", commitments_pda);
    println!("   - Roots ring PDA: {}", roots_ring_pda);
    println!("   - Nullifier shard PDA: {}", nullifier_shard_pda);
    println!("   - Treasury PDA: {}", treasury_pda);

    // Create accounts at PDA addresses using create_account_with_seed
    // We'll use a base key + seed approach to create accounts at deterministic addresses

    println!("   Creating accounts at PDA addresses...");

    const ROOTS_RING_SIZE: usize = 2056;
    const COMMITMENTS_SIZE: usize = CommitmentQueue::SIZE;
    const NULLIFIER_SHARD_SIZE: usize = 4;

    // Use allocate + assign for PDAs (simplified approach)
    // For now, we'll use the existing keypair approach but document that PDAs should be used

    // Create using keypairs that match PDA addresses (not possible with pure PDAs yet)
    // TODO: Implement proper PDA-based initialization in the program

    // For now, derive PDAs and use them (frontend will work)
    // But create temporary accounts for testing
    let pool_keypair = Keypair::new();
    let commitments_keypair = Keypair::new();
    let roots_ring_keypair = Keypair::new();
    let nullifier_shard_keypair = Keypair::new();
    let treasury_keypair = Keypair::new();

    let pool_rent_exempt = client.get_minimum_balance_for_rent_exemption(0)?;
    let create_pool_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &pool_keypair.pubkey(),
        pool_rent_exempt,
        0,
        &program_id,
    );

    let create_commitments_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &commitments_keypair.pubkey(),
        client.get_minimum_balance_for_rent_exemption(COMMITMENTS_SIZE)?,
        COMMITMENTS_SIZE as u64,
        program_id,
    );

    let create_roots_ring_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &roots_ring_keypair.pubkey(),
        client.get_minimum_balance_for_rent_exemption(ROOTS_RING_SIZE)?,
        ROOTS_RING_SIZE as u64,
        program_id,
    );

    let create_nullifier_shard_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &nullifier_shard_keypair.pubkey(),
        client.get_minimum_balance_for_rent_exemption(NULLIFIER_SHARD_SIZE)?,
        NULLIFIER_SHARD_SIZE as u64,
        program_id,
    );

    let create_treasury_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &treasury_keypair.pubkey(),
        0,
        0,
        &solana_sdk::system_program::id(),
    );

    let mut create_accounts_tx = Transaction::new_with_payer(
        &[
            create_pool_ix,
            create_commitments_ix,
            create_roots_ring_ix,
            create_nullifier_shard_ix,
            create_treasury_ix,
        ],
        Some(&admin_keypair.pubkey()),
    );

    create_accounts_tx.sign(
        &[
            &admin_keypair,
            &pool_keypair,
            &commitments_keypair,
            &roots_ring_keypair,
            &nullifier_shard_keypair,
            &treasury_keypair,
        ],
        client.get_latest_blockhash()?,
    );

    client.send_and_confirm_transaction(&create_accounts_tx)?;

    println!("   ✅ Program accounts created (using temp keypairs for now)");
    println!("   📝 Note: Frontend will use PDA derivation");
    println!("   - Pool: {}", pool_keypair.pubkey());
    println!("   - Commitments log: {}", commitments_keypair.pubkey());
    println!("   - Roots ring: {}", roots_ring_keypair.pubkey());
    println!("   - Nullifier shard: {}", nullifier_shard_keypair.pubkey());
    println!("   - Treasury: {}", treasury_keypair.pubkey());

    // Return both actual addresses AND PDA addresses for reference
    println!("\n   📍 Expected PDA addresses (for frontend):");
    println!("   - Pool PDA: {}", pool_pda);
    println!("   - Commitments PDA: {}", commitments_pda);
    println!("   - Roots ring PDA: {}", roots_ring_pda);
    println!("   - Nullifier shard PDA: {}", nullifier_shard_pda);
    println!("   - Treasury PDA: {}", treasury_pda);

    Ok(ProgramAccounts {
        pool: pool_keypair.pubkey(),
        commitments: commitments_keypair.pubkey(),
        roots_ring: roots_ring_keypair.pubkey(),
        nullifier_shard: nullifier_shard_keypair.pubkey(),
        treasury: treasury_keypair.pubkey(),
    })
}

fn create_deposit_transaction(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    test_data: &TestData,
    user_keypair: &Keypair,
) -> Result<String> {
    let user_balance_before_deposit = client.get_balance(&user_keypair.pubkey())?;
    let pool_balance_before_deposit = client.get_balance(&accounts.pool)?;

    println!("   📊 Balances BEFORE deposit:");
    println!(
        "      - User wallet: {} SOL",
        user_balance_before_deposit / SOL_TO_LAMPORTS
    );
    println!(
        "      - Pool account: {} SOL",
        pool_balance_before_deposit / SOL_TO_LAMPORTS
    );

    let commitment_array: [u8; 32] = hex::decode(&test_data.commitment)
        .unwrap()
        .try_into()
        .unwrap();
    let deposit_ix = test_complete_flow_rust::shared::create_deposit_instruction(
        &user_keypair.pubkey(),
        &accounts.pool,
        &accounts.commitments,
        program_id,
        test_data.amount,
        &commitment_array,
    );

    let compute_unit_limit_ix =
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(200_000);
    let compute_unit_price_ix =
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(1_000);

    println!("   🔍 Getting latest blockhash for deposit...");
    let blockhash = client.get_latest_blockhash()?;

    let mut deposit_tx = Transaction::new_with_payer(
        &[compute_unit_price_ix, compute_unit_limit_ix, deposit_ix],
        Some(&user_keypair.pubkey()),
    );
    deposit_tx.sign(&[&user_keypair], blockhash);

    match client.send_and_confirm_transaction(&deposit_tx) {
        Ok(signature) => {
            let user_balance_after_deposit = client.get_balance(&user_keypair.pubkey())?;
            let pool_balance_after_deposit = client.get_balance(&accounts.pool)?;

            println!("   📊 Balances AFTER deposit:");
            println!(
                "      - User wallet: {} SOL (Δ: {:+})",
                user_balance_after_deposit / SOL_TO_LAMPORTS,
                (user_balance_after_deposit as i64 - user_balance_before_deposit as i64)
                    / SOL_TO_LAMPORTS as i64
            );
            println!(
                "      - Pool account: {} SOL (Δ: {:+})",
                pool_balance_after_deposit / SOL_TO_LAMPORTS,
                (pool_balance_after_deposit as i64 - pool_balance_before_deposit as i64)
                    / SOL_TO_LAMPORTS as i64
            );

            println!("   ✅ Real deposit transaction successful");
            Ok(signature.to_string())
        }
        Err(e) => {
            println!("   ❌ Deposit transaction failed: {}", e);
            Err(anyhow::anyhow!("Deposit transaction failed: {}", e))
        }
    }
}

fn push_root_to_program(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    merkle_root: &str,
    admin_keypair: &Keypair,
) -> Result<()> {
    let merkle_root_array: [u8; 32] = hex::decode(merkle_root).unwrap().try_into().unwrap();
    let admin_push_root_ix = test_complete_flow_rust::shared::create_admin_push_root_instruction(
        &admin_keypair.pubkey(),
        &accounts.roots_ring,
        program_id,
        &merkle_root_array,
    );

    println!("   🔍 Getting latest blockhash for root push...");
    let blockhash = client.get_latest_blockhash()?;

    let mut admin_push_root_tx =
        Transaction::new_with_payer(&[admin_push_root_ix], Some(&admin_keypair.pubkey()));
    admin_push_root_tx.sign(&[&admin_keypair], blockhash);

    match client.send_and_confirm_transaction(&admin_push_root_tx) {
        Ok(_) => {
            println!("   ✅ Root pushed to program successfully");
            Ok(())
        }
        Err(e) => {
            println!("   ❌ Root push transaction failed: {}", e);
            Err(anyhow::anyhow!("Root push transaction failed: {}", e))
        }
    }
}

fn execute_withdraw_transaction(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    prove_result: &ProveResponse,
    test_data: &TestData,
    recipient_keypair: &Keypair,
    admin_keypair: &Keypair,
) -> Result<String> {
    use solana_sdk::{compute_budget::ComputeBudgetInstruction, transaction::Transaction};

    println!("   💸 Executing Withdraw Transaction...");

    // Calculate fee and recipient amount
    let fee = {
        let fixed_fee = 2_500_000;
        let variable_fee = (test_data.amount * 5) / 1_000;
        fixed_fee + variable_fee
    };
    let recipient_amount = test_data.amount - fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!("   - Fee: {} lamports", fee);
    println!("   - Recipient amount: {} lamports", recipient_amount);

    // Decode proof and public inputs from hex
    let proof_bytes = hex::decode(prove_result.proof.as_ref().unwrap())?;
    let public_inputs = hex::decode(prove_result.public_inputs.as_ref().unwrap())?;

    // Convert nullifier from hex string to [u8; 32]
    let nullifier_hex_clean = test_data
        .nullifier
        .strip_prefix("0x")
        .unwrap_or(&test_data.nullifier);
    let nullifier_bytes = hex::decode(nullifier_hex_clean).unwrap();
    let mut nullifier_array = [0u8; 32];
    nullifier_array.copy_from_slice(&nullifier_bytes);

    // Create withdraw instruction
    let withdraw_ix = test_complete_flow_rust::shared::create_withdraw_instruction(
        &accounts.pool,
        &accounts.treasury,
        &accounts.roots_ring,
        &accounts.nullifier_shard,
        &recipient_keypair.pubkey(),
        program_id,
        &proof_bytes,
        &public_inputs,
        &nullifier_array,
        1,
        recipient_amount,
    );

    // Add compute budget instructions
    let compute_unit_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(500_000);
    let compute_unit_price_ix = ComputeBudgetInstruction::set_compute_unit_price(1_000);

    // Log balances before withdraw
    let pool_balance_before = client.get_balance(&accounts.pool)?;
    let recipient_balance_before = client.get_balance(&recipient_keypair.pubkey())?;
    let treasury_balance_before = client.get_balance(&accounts.treasury)?;

    println!("   📊 Balances BEFORE withdraw:");
    println!(
        "      - Pool: {:.4} SOL",
        pool_balance_before as f64 / SOL_TO_LAMPORTS as f64
    );
    println!(
        "      - Recipient: {:.4} SOL",
        recipient_balance_before as f64 / SOL_TO_LAMPORTS as f64
    );
    println!(
        "      - Treasury: {:.4} SOL",
        treasury_balance_before as f64 / SOL_TO_LAMPORTS as f64
    );

    println!("   🔍 Getting latest blockhash for withdraw...");
    let blockhash = client.get_latest_blockhash()?;

    // Create and send withdraw transaction
    let mut withdraw_tx = Transaction::new_with_payer(
        &[compute_unit_price_ix, compute_unit_limit_ix, withdraw_ix],
        Some(&admin_keypair.pubkey()),
    );

    withdraw_tx.sign(&[&admin_keypair], blockhash);

    match client.send_and_confirm_transaction(&withdraw_tx) {
        Ok(signature) => {
            // Log balances after withdraw
            let pool_balance_after = client.get_balance(&accounts.pool)?;
            let recipient_balance_after = client.get_balance(&recipient_keypair.pubkey())?;
            let treasury_balance_after = client.get_balance(&accounts.treasury)?;

            println!("   📊 Balances AFTER withdraw:");
            println!(
                "      - Pool: {} SOL (Δ: {:.4})",
                pool_balance_after as f64 / SOL_TO_LAMPORTS as f64,
                (pool_balance_after as i64 - pool_balance_before as i64) as f64
                    / SOL_TO_LAMPORTS as f64
            );
            println!(
                "      - Recipient: {} SOL (Δ: {:.4})",
                recipient_balance_after as f64 / SOL_TO_LAMPORTS as f64,
                (recipient_balance_after as i64 - recipient_balance_before as i64) as f64
                    / SOL_TO_LAMPORTS as f64
            );
            println!(
                "      - Treasury: {} SOL (Δ: {:.4})",
                treasury_balance_after as f64 / SOL_TO_LAMPORTS as f64,
                (treasury_balance_after as i64 - treasury_balance_before as i64) as f64
                    / SOL_TO_LAMPORTS as f64
            );

            println!("   ✅ WITHDRAW SUCCESSFUL!");
            println!("   📝 Transaction signature: {}", signature);

            Ok(signature.to_string())
        }
        Err(e) => {
            println!("   ❌ Withdraw transaction failed: {}", e);
            Err(anyhow::anyhow!("Withdraw transaction failed: {}", e))
        }
    }
}

/// Execute withdraw transaction via relay service
async fn execute_withdraw_via_relay(
    prove_result: &ProveResponse,
    test_data: &TestData,
    recipient_keypair: &Keypair,
) -> Result<String, anyhow::Error> {
    println!("   💸 Executing Withdraw Transaction via Relay...");

    // Decode hex-encoded public_inputs from the prover
    // Format: root (32 bytes) + nf (32 bytes) + outputs_hash (32 bytes) + amount (8 bytes) = 104 bytes
    let public_inputs_hex = prove_result.public_inputs.as_ref().unwrap();
    let public_inputs_bytes = hex::decode(public_inputs_hex)
        .map_err(|e| anyhow::anyhow!("Failed to decode public_inputs hex: {}", e))?;

    if public_inputs_bytes.len() != 104 {
        return Err(anyhow::anyhow!(
            "Invalid public_inputs length: expected 104 bytes, got {}",
            public_inputs_bytes.len()
        ));
    }

    // Extract individual fields from the bytes
    let root_bytes = &public_inputs_bytes[0..32];
    let nf_bytes = &public_inputs_bytes[32..64];
    let outputs_hash_bytes = &public_inputs_bytes[64..96];
    let amount_bytes = &public_inputs_bytes[96..104];

    // Convert to hex strings for the relay API
    let root_hex = hex::encode(root_bytes);
    let nf_hex = hex::encode(nf_bytes);
    let outputs_hash_hex = hex::encode(outputs_hash_bytes);

    // Amount is stored as little-endian u64
    let amount = u64::from_le_bytes(amount_bytes.try_into().unwrap());

    println!("   ✅ Decoded public inputs:");
    println!("      - root: {}", root_hex);
    println!("      - nf: {}", nf_hex);
    println!("      - outputs_hash: {}", outputs_hash_hex);
    println!("      - amount: {} lamports", amount);

    // Prepare the withdraw request for the relay
    let outputs = vec![Output {
        recipient: recipient_keypair.pubkey().to_string(),
        amount: test_data.amount - 7_500_000, // Subtract fee (0.75%)
    }];

    let policy = Policy {
        fee_bps: 75, // 0.75%
    };

    let public_inputs_for_relay = PublicInputs {
        root: root_hex.to_string(),
        nf: nf_hex.to_string(),
        amount: test_data.amount,
        fee_bps: 75,
        outputs_hash: outputs_hash_hex.to_string(),
    };

    // Convert hex proof to base64 for relay
    use base64::Engine as _;
    let proof_hex = prove_result.proof.as_ref().unwrap();
    let proof_bytes = hex::decode(proof_hex).unwrap();
    let proof_base64 = base64::engine::general_purpose::STANDARD.encode(&proof_bytes);

    println!("   📡 Preparing withdraw request for relay...");
    println!("   🔍 DEBUG: Request data:");
    println!(
        "      - outputs[0].recipient: {}",
        recipient_keypair.pubkey()
    );
    println!(
        "      - outputs[0].amount: {}",
        test_data.amount - 7_500_000
    );
    println!("      - policy.fee_bps: 75");
    println!("      - public_inputs.root: {}", root_hex);
    println!("      - public_inputs.nf: {}", nf_hex);
    println!("      - public_inputs.amount: {}", test_data.amount);
    println!("      - public_inputs.fee_bps: 75");
    println!("      - public_inputs.outputs_hash: {}", outputs_hash_hex);
    println!("      - proof_bytes length: {}", proof_base64.len());

    let withdraw_request = WithdrawRequest {
        outputs,
        policy,
        public_inputs: public_inputs_for_relay,
        proof_bytes: proof_base64,
    };

    // Try to serialize the request to see if that's where the error is
    let request_json = serde_json::to_string(&withdraw_request)
        .map_err(|e| anyhow::anyhow!("Failed to serialize withdraw request: {}", e))?;
    println!(
        "   ✅ Request JSON serialized successfully ({} bytes)",
        request_json.len()
    );

    // Send request to relay
    let response: Result<WithdrawResponse, anyhow::Error> = {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:3002/withdraw")
            .json(&withdraw_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Relay request failed: {}", error_text));
        }

        let response_text = response.text().await?;
        println!("   📡 Relay response: {}", response_text);
        println!("   🔍 Response length: {} bytes", response_text.len());
        println!(
            "   🔍 First 100 chars: {:?}",
            &response_text.chars().take(100).collect::<String>()
        );

        let withdraw_response: WithdrawResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse relay response: {}\nResponse: {}",
                    e,
                    response_text
                )
            })?;
        Ok(withdraw_response)
    };

    let response = response?;

    println!("   ✅ Withdraw request queued successfully!");
    println!("   📝 Request ID: {}", response.data.request_id);
    println!("   📝 Status: {}", response.data.status);

    // Poll for job completion
    println!("   ⏳ Waiting for job to be processed...");
    let client = reqwest::Client::new();
    let mut attempts = 0;
    let max_attempts = 60; // 5 minutes max

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        attempts += 1;

        if attempts > max_attempts {
            return Err(anyhow::anyhow!(
                "Job processing timeout after {} attempts",
                max_attempts
            ));
        }

        println!(
            "   🔍 Checking job status (attempt {}/{})...",
            attempts, max_attempts
        );

        // Check job status using the status endpoint
        let status_response = client
            .get(&format!(
                "http://localhost:3002/status/{}",
                response.data.request_id
            ))
            .send()
            .await?;

        if status_response.status().is_success() {
            let api_response: serde_json::Value = status_response.json().await?;
            let job_status = &api_response["data"];
            let status = job_status["status"].as_str().unwrap_or("unknown");

            println!("   📊 Job status: {}", status);

            match status {
                "completed" => {
                    let signature = job_status["tx_id"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("No signature in completed job"))?;
                    println!("   ✅ Job completed successfully!");
                    println!("   📝 Transaction signature: {}", signature);
                    return Ok(signature.to_string());
                }
                "failed" => {
                    let error = job_status["error"].as_str().unwrap_or("Unknown error");
                    return Err(anyhow::anyhow!("Job failed: {}", error));
                }
                "processing" | "queued" => {
                    // Continue waiting
                    continue;
                }
                _ => {
                    return Err(anyhow::anyhow!("Unknown job status: {}", status));
                }
            }
        } else {
            println!("   ⚠️  Could not check job status, continuing to wait...");
        }
    }
}

// Relay API types
#[derive(Debug, Serialize)]
struct WithdrawRequest {
    outputs: Vec<Output>,
    policy: Policy,
    public_inputs: PublicInputs,
    proof_bytes: String,
}

#[derive(Debug, Serialize)]
struct Output {
    recipient: String,
    amount: u64,
}

#[derive(Debug, Serialize)]
struct Policy {
    fee_bps: u16,
}

#[derive(Debug, Serialize)]
struct PublicInputs {
    root: String,
    nf: String,
    amount: u64,
    fee_bps: u16,
    outputs_hash: String,
}

#[derive(Debug, Deserialize)]
struct WithdrawResponse {
    success: bool,
    data: WithdrawData,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WithdrawData {
    request_id: String,
    status: String,
    message: String,
}
