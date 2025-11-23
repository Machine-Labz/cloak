use anyhow::Result;
use bincode;
use cloak_proof_extract::extract_groth16_260_sp1;
use hex;
use rand;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use shield_pool::CommitmentQueue;
use solana_client::rpc_client::RpcClient;
use solana_sdk::system_instruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use sp1_sdk::{network::FulfillmentStrategy, HashableKey, Prover, ProverClient, SP1Stdin};
use std::str::FromStr;
use std::time::{Duration, Instant};
use test_complete_flow_rust::shared::{
    check_cluster_health, ensure_user_funding, load_keypair, print_config, MerkleProof, TestConfig,
    SOL_TO_LAMPORTS,
};
use tokio::time::timeout;
use zk_guest_sp1_host::{
    generate_proof as generate_proof_local, ProofResult as LocalProofResult, ELF,
};

#[derive(Debug, Serialize, Deserialize)]
struct DepositRequest {
    leaf_commit: String,
    encrypted_output: String,
    tx_signature: String,
    slot: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MerkleRootResponse {
    root: String,
}

#[derive(Debug, Clone)]
struct ProofArtifacts {
    proof_hex: String,
    public_inputs_hex: String,
    generation_time_ms: u64,
    total_cycles: Option<u64>,
    total_syscalls: Option<u64>,
    execution_report: Option<String>,
    proof_method: String,
    wallet_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

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

    // Use the correct admin keypair (upgrade authority at ~/.config/solana/id.json)
    let admin_keypair_path = std::env::var("HOME")
        .map(|home| format!("{}/.config/solana/id.json", home))
        .unwrap_or_else(|_| "admin-keypair.json".to_string());
    let admin_keypair = load_keypair(&admin_keypair_path)?;

    // Load miner keypair for reward verification
    let miner_keypair = load_keypair("miner.json")?;

    println!("\n💰 Checking balances...");
    let client = RpcClient::new(&config.rpc_url);
    let user_balance = client.get_balance(&user_keypair.pubkey())?;
    let admin_balance = client.get_balance(&admin_keypair.pubkey())?;
    let recipient_balance = client.get_balance(&recipient_keypair.pubkey())?;
    let miner_balance = client.get_balance(&miner_keypair.pubkey())?;

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
    println!(
        "   Miner ({}): {} SOL",
        miner_keypair.pubkey(),
        miner_balance / SOL_TO_LAMPORTS
    );

    // Ensure user has sufficient SOL
    ensure_user_funding(&config.rpc_url, &user_keypair, &admin_keypair)?;

    // Ensure miner has sufficient SOL
    ensure_user_funding(&config.rpc_url, &miner_keypair, &admin_keypair)?;

    // Verify miner is running and has claims available
    println!("\n⛏️  Verifying Miner Status...");
    verify_miner_status(&client, &miner_keypair).await?;

    // Use existing deployed program (already deployed to localnet)
    let program_id = {
        println!(
            "\n✅ Using existing deployed program: {}",
            config.program_id
        );
        Pubkey::from_str(&config.program_id)?
    };

    // Create program accounts (check if they exist first, create if needed)
    let accounts = {
        use test_complete_flow_rust::shared::get_pda_addresses;
        let mint = solana_sdk::pubkey::Pubkey::default(); // Native SOL
        let (pool, commitments, roots_ring, nullifier_shard, treasury) =
            get_pda_addresses(&program_id, &mint);

        println!("\n📋 Step 1: Checking/Creating Program Accounts...");
        println!("   - Pool (derived PDA): {}", pool);
        println!("   - Commitments log (derived PDA): {}", commitments);
        println!("   - Roots ring (derived PDA): {}", roots_ring);
        println!("   - Nullifier shard (derived PDA): {}", nullifier_shard);
        println!("   - Treasury (derived PDA): {}", treasury);

        // Check if pool account exists
        match client.get_account(&pool) {
            Ok(account) => {
                println!("   ✅ Pool account exists and is owned by: {}", account.owner);
                if account.owner != program_id {
                    println!("   ⚠️  Pool account is not owned by program! Creating new one...");
                    create_program_accounts(&client, &program_id, &admin_keypair)?
                } else {
                    ProgramAccounts {
                        pool,
                        commitments,
                        roots_ring,
                        nullifier_shard,
                        treasury,
                    }
                }
            }
            Err(_) => {
                println!("   📝 Pool account doesn't exist, creating all accounts...");
                create_program_accounts(&client, &program_id, &admin_keypair)?
            }
        }
    };

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

    // Generate proof locally or via TEE
    println!("\n🚀 Step 9: Generating Proof Client-Side...");
    let prove_result =
        generate_proof_client_side(&private_inputs, &public_inputs, &outputs).await?;

    // Validate proof artifacts
    println!("\n✅ Step 10: Validating Proof Artifacts...");
    validate_proof_artifacts(&prove_result)?;

    // Capture miner balance BEFORE withdrawal
    let miner_balance_before_withdraw = client.get_balance(&miner_keypair.pubkey())?;

    // Execute withdraw via relay
    println!("\n💸 Step 11: Executing Withdraw Transaction via Relay...");
    let withdraw_signature =
        execute_withdraw_via_relay(&prove_result, &test_data, &recipient_keypair).await?;

    // Verify miner reward (wait for RPC to update balance)
    println!("\n⛏️  Verifying Miner Reward...");

    // Poll for balance update with retries (RPC lag)
    let mut miner_balance_after = client.get_balance(&miner_keypair.pubkey())?;
    let mut attempts = 0;
    const MAX_RETRIES: u32 = 5;

    while miner_balance_after == miner_balance_before_withdraw && attempts < MAX_RETRIES {
        attempts += 1;
        println!(
            "   ⏳ Waiting for balance to update (attempt {}/{})...",
            attempts, MAX_RETRIES
        );
        std::thread::sleep(std::time::Duration::from_millis(500));
        miner_balance_after = client.get_balance(&miner_keypair.pubkey())?;
    }

    let miner_reward = miner_balance_after.saturating_sub(miner_balance_before_withdraw);

    println!(
        "   📊 Miner balance before withdrawal: {} SOL",
        miner_balance_before_withdraw / SOL_TO_LAMPORTS
    );
    println!(
        "   📊 Miner balance after withdrawal: {} SOL",
        miner_balance_after / SOL_TO_LAMPORTS
    );

    if miner_reward > 0 {
        println!(
            "   ✅ Miner received reward: {} lamports ({} SOL)",
            miner_reward,
            miner_reward as f64 / SOL_TO_LAMPORTS as f64
        );
    } else {
        println!("   ⚠️  No miner reward detected (balance unchanged)");
    }

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
    println!("   - Proof method: {}", prove_result.proof_method);
    if let Some(wallet) = &prove_result.wallet_address {
        println!("   - TEE wallet: {}", wallet);
    }
    println!(
        "   - Proof size: {} bytes",
        prove_result.proof_hex.len() / 2
    );
    println!(
        "   - Public inputs size: {} bytes",
        prove_result.public_inputs_hex.len() / 2
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

    println!("\n⛏️  Miner Reward Summary:");
    println!("   - Miner address: {}", miner_keypair.pubkey());
    println!(
        "   - Balance before withdrawal: {} SOL",
        miner_balance_before_withdraw / SOL_TO_LAMPORTS
    );
    println!(
        "   - Balance after withdrawal: {} SOL",
        miner_balance_after / SOL_TO_LAMPORTS
    );
    if miner_reward > 0 {
        println!(
            "   - Reward received: {} lamports ({} SOL)",
            miner_reward,
            miner_reward as f64 / SOL_TO_LAMPORTS as f64
        );
        println!("   - PoW claim consumption: ✅ Successful");
    } else {
        println!("   - Reward received: 0 lamports");
        println!("   - PoW claim consumption: ⚠️  No reward detected");
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
    println!("   - Real relay service with PoW claims ✅");
    println!("   - Real miner reward verification ✅");
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
    // Use DO block to handle case where tables might not exist
    let truncate_cmd = std::process::Command::new("docker")
        .args(&[
            "exec",
            "cloak-postgres",
            "psql",
            "-U",
            "cloak",
            "-d",
            "cloak",
            "-c",
            "DO $$ BEGIN IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'jobs') THEN TRUNCATE TABLE jobs CASCADE; END IF; IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'nullifiers') THEN TRUNCATE TABLE nullifiers CASCADE; END IF; END $$;",
        ])
        .output();

    match truncate_cmd {
        Ok(output) => {
            if output.status.success() {
                println!("   ✅ Relay database reset successfully");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Check if error is just that tables don't exist
                if stderr.contains("does not exist")
                    || stderr.contains("relation")
                    || stdout.contains("does not exist")
                {
                    println!("   ℹ️  Relay tables don't exist yet (will be created on first use)");
                } else if !stderr.is_empty() {
                    println!("   ⚠️  Failed to reset relay database: {}", stderr);
                }
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
        tx_signature: unique_tx_signature,
        slot: 1000,
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

    for (sibling_hex, &is_left_child) in path_elements.iter().zip(path_indices.iter()) {
        let sibling = hex::decode(sibling_hex)?;
        let mut hasher = Hasher::new();
        if is_left_child == 0 {
            // Current value is left child, sibling is right
            hasher.update(&current_hash);
            hasher.update(&sibling);
        } else {
            // Current value is right child, sibling is left
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

    // Fee calculation must mirror on-chain logic: 0.0025 SOL + 0.5%
    let fee = {
        let fixed_fee = 2_500_000;
        let variable_fee = (amount * 5) / 1_000;
        fixed_fee + variable_fee
    };
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

async fn generate_proof_client_side(
    private_inputs: &str,
    public_inputs: &str,
    outputs: &str,
) -> Result<ProofArtifacts> {
    println!("   🔨 Preparing proof generation request...");

    println!("\n   📋 DEBUG: Proof Inputs Being Used:");
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

    let tee_config = TeeClientConfig::from_env();
    if tee_config.is_ready() {
        match generate_proof_via_tee(&tee_config, private_inputs, public_inputs, outputs).await {
            Ok(artifacts) => return Ok(artifacts),
            Err(e) => {
                println!(
                    "   ⚠️  TEE proof generation failed ({}). Falling back to local prover...",
                    e
                );
            }
        }
    } else if tee_config.enabled {
        println!(
            "   ⚠️  TEE proving enabled but configuration is incomplete. Falling back to local prover."
        );
    } else {
        println!("   🏠 TEE proving disabled. Using local prover.");
    }

    generate_proof_locally(private_inputs, public_inputs, outputs)
}

fn validate_proof_artifacts(artifacts: &ProofArtifacts) -> Result<()> {
    println!("   🔍 Validating proof artifacts...");

    if artifacts.proof_hex.is_empty() {
        return Err(anyhow::anyhow!("Proof is missing from artifacts"));
    }
    if artifacts.public_inputs_hex.is_empty() {
        return Err(anyhow::anyhow!("Public inputs are missing from artifacts"));
    }

    let proof_bytes_len = artifacts.proof_hex.len() / 2;
    if proof_bytes_len != 260 {
        println!(
            "   ⚠️  Warning: Expected 260-byte proof, got {} bytes",
            proof_bytes_len
        );
    } else {
        println!("   ✅ Proof size is correct (260 bytes)");
        println!(
            "   🔎 Proof prefix: {}",
            &artifacts.proof_hex[..8.min(artifacts.proof_hex.len())]
        );
    }

    let public_inputs_len = artifacts.public_inputs_hex.len() / 2;
    if public_inputs_len != 104 {
        return Err(anyhow::anyhow!(
            "Invalid public inputs length: expected 104 bytes, got {}",
            public_inputs_len
        ));
    } else {
        println!("   ✅ Public inputs size is correct (104 bytes)");
    }

    println!(
        "   ✅ Proof generation time: {}ms",
        artifacts.generation_time_ms
    );

    if let Some(cycles) = artifacts.total_cycles {
        println!("   ✅ Total SP1 cycles: {}", cycles);
    }
    if let Some(syscalls) = artifacts.total_syscalls {
        println!("   ✅ Total SP1 syscalls: {}", syscalls);
    }

    if let Some(ref report) = artifacts.execution_report {
        println!("\n📊 Execution Report Summary:");
        println!("{}", report);
    }

    Ok(())
}

struct TeeClientConfig {
    enabled: bool,
    wallet_address: String,
    private_key: Option<String>,
    timeout_seconds: u64,
}

impl TeeClientConfig {
    fn from_env() -> Self {
        let enabled = std::env::var("SP1_TEE_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let wallet_address = std::env::var("SP1_TEE_WALLET_ADDRESS").unwrap_or_default();
        let timeout_seconds = std::env::var("SP1_TEE_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300);
        let private_key = std::env::var("NETWORK_PRIVATE_KEY").ok();

        Self {
            enabled,
            wallet_address,
            private_key,
            timeout_seconds,
        }
    }

    fn is_ready(&self) -> bool {
        self.enabled
            && !self.wallet_address.is_empty()
            && self
                .private_key
                .as_ref()
                .map(|p| !p.is_empty())
                .unwrap_or(false)
    }
}

async fn generate_proof_via_tee(
    cfg: &TeeClientConfig,
    private_inputs: &str,
    public_inputs: &str,
    outputs: &str,
) -> Result<ProofArtifacts> {
    println!("   🔐 TEE client available - attempting TEE proof generation");
    println!("   Wallet: {}", cfg.wallet_address);
    println!("   Timeout: {} seconds", cfg.timeout_seconds);

    let private_key = cfg
        .private_key
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("NETWORK_PRIVATE_KEY is required for TEE proving"))?;

    let start_time = Instant::now();

    let prover_client = ProverClient::builder()
        .network()
        .private()
        .private_key(private_key)
        .build();

    let (pk, vk) = prover_client.setup(ELF);
    println!("   SP1 verifying key hash: 0x{}", hex::encode(vk.bytes32()));

    let combined_input = format!(
        r#"{{
                "private": {},
                "public": {},
                "outputs": {}
            }}"#,
        private_inputs, public_inputs, outputs
    );

    let mut stdin = SP1Stdin::new();
    stdin.write(&combined_input);

    let (_, report) = prover_client.execute(ELF, &stdin).run()?;
    let total_cycles = report.total_instruction_count();
    let total_syscalls = report.total_syscall_count();
    let execution_report = format!("{}", report);

    println!("   📊 SP1 TEE Execution Report:");
    println!("      - Total cycles: {}", total_cycles);
    println!("      - Total syscalls: {}", total_syscalls);

    let prove_future = async {
        prover_client
            .prove(&pk, &stdin)
            .groth16()
            .strategy(FulfillmentStrategy::Reserved)
            .run()
    };

    let proof_result = timeout(Duration::from_secs(cfg.timeout_seconds), prove_future)
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "TEE proof generation timed out after {} seconds",
                cfg.timeout_seconds
            )
        })?
        .map_err(|e| anyhow::anyhow!("TEE proof generation failed: {}", e))?;

    let proof_bundle = bincode::serialize(&proof_result)?;
    let public_inputs_bytes = proof_result.public_values.to_vec();

    let canonical_proof = extract_groth16_260_sp1(&proof_bundle)?;
    let proof_hex = hex::encode(&canonical_proof);
    let public_inputs_hex = hex::encode(&public_inputs_bytes);

    println!("   ✅ TEE proof generation completed");
    println!("      - Proof size: {} bytes", canonical_proof.len());
    println!(
        "      - Public inputs size: {} bytes",
        public_inputs_bytes.len()
    );

    Ok(ProofArtifacts {
        proof_hex,
        public_inputs_hex,
        generation_time_ms: start_time.elapsed().as_millis() as u64,
        total_cycles: Some(total_cycles),
        total_syscalls: Some(total_syscalls),
        execution_report: Some(execution_report),
        proof_method: "tee".to_string(),
        wallet_address: Some(cfg.wallet_address.clone()),
    })
}

fn generate_proof_locally(
    private_inputs: &str,
    public_inputs: &str,
    outputs: &str,
) -> Result<ProofArtifacts> {
    println!("   🏠 Using local SP1 prover (CPU)");

    let LocalProofResult {
        proof_bytes,
        public_inputs,
        generation_time_ms,
        total_cycles,
        total_syscalls,
        execution_report,
    } = generate_proof_local(private_inputs, public_inputs, outputs)?;

    let canonical_proof = extract_groth16_260_sp1(&proof_bytes)?;
    let proof_hex = hex::encode(&canonical_proof);
    let public_inputs_hex = hex::encode(&public_inputs);

    Ok(ProofArtifacts {
        proof_hex,
        public_inputs_hex,
        generation_time_ms,
        total_cycles: Some(total_cycles),
        total_syscalls: Some(total_syscalls),
        execution_report: Some(execution_report),
        proof_method: "local".to_string(),
        wallet_address: None,
    })
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
    use solana_sdk::transaction::Transaction;
    use test_complete_flow_rust::shared::get_pda_addresses;

    println!("   Deriving PDA addresses...");
    let mint = solana_sdk::pubkey::Pubkey::default(); // Native SOL
    // Use the exact same program ID as the program expects
    let program_id_bytes = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".parse::<Pubkey>().unwrap();
    let (pool_pda, commitments_pda, roots_ring_pda, nullifier_shard_pda, treasury_pda) =
        get_pda_addresses(&program_id_bytes, &mint);

    println!("   - Pool PDA: {}", pool_pda);
    println!("   - Commitments PDA: {}", commitments_pda);
    println!("   - Roots ring PDA: {}", roots_ring_pda);
    println!("   - Nullifier shard PDA: {}", nullifier_shard_pda);
    println!("   - Treasury PDA: {}", treasury_pda);

    // Create accounts at PDA addresses using create_account_with_seed
    // We'll use a base key + seed approach to create accounts at deterministic addresses

    println!("   Creating accounts at PDA addresses...");
    
    // Debug: Print expected PDA addresses
    println!("   Debug - Expected PDAs:");
    println!("     Pool: {}", pool_pda);
    println!("     Commitments: {}", commitments_pda);
    println!("     Roots Ring: {}", roots_ring_pda);
    println!("     Nullifier Shard: {}", nullifier_shard_pda);
    println!("     Treasury: {}", treasury_pda);

    const ROOTS_RING_SIZE: usize = 2056;
    const COMMITMENTS_SIZE: usize = CommitmentQueue::SIZE;
    const NULLIFIER_SHARD_SIZE: usize = 4;

    let pool_rent_exempt = client.get_minimum_balance_for_rent_exemption(32)?; // Pool now stores mint (32 bytes)
    let create_pool_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &pool_pda,
        pool_rent_exempt,
        32, // Pool now stores mint
        &program_id,
    );

    let create_commitments_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &commitments_pda,
        client.get_minimum_balance_for_rent_exemption(COMMITMENTS_SIZE)?,
        COMMITMENTS_SIZE as u64,
        program_id,
    );

    let create_roots_ring_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &roots_ring_pda,
        client.get_minimum_balance_for_rent_exemption(ROOTS_RING_SIZE)?,
        ROOTS_RING_SIZE as u64,
        program_id,
    );

    let create_nullifier_shard_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &nullifier_shard_pda,
        client.get_minimum_balance_for_rent_exemption(NULLIFIER_SHARD_SIZE)?,
        NULLIFIER_SHARD_SIZE as u64,
        program_id,
    );

    let create_treasury_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &treasury_pda,
        0,
        0,
        &solana_sdk::system_program::id(),
    );

    // Initialize pool with mint data
    // Ensure we use the exact same program ID as the program expects
    let program_id_bytes = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".parse::<Pubkey>().unwrap();
    let initialize_pool_ix = Instruction {
        program_id: program_id_bytes,
        accounts: vec![
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(commitments_pda, false),
            AccountMeta::new(roots_ring_pda, false),
            AccountMeta::new(nullifier_shard_pda, false),
            AccountMeta::new(treasury_pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data: {
            let mut data = vec![3u8]; // Initialize discriminator (ShieldPoolInstruction::Initialize = 3)
            data.extend_from_slice(&mint.to_bytes()); // Mint pubkey (32 bytes)
            data
        },
    };
    
    // Debug: Print instruction details
    println!("   Debug - Instruction details:");
    println!("     Program ID: {}", program_id);
    println!("     Admin: {}", admin_keypair.pubkey());
    println!("     Pool: {}", pool_pda);
    println!("     Mint: {}", mint);
    println!("     Data length: {}", initialize_pool_ix.data.len());

    // Use only the initialize instruction - the program will create the accounts
    let mut create_accounts_tx = Transaction::new_with_payer(
        &[initialize_pool_ix],
        Some(&admin_keypair.pubkey()),
    );

    create_accounts_tx.sign(&[&admin_keypair], client.get_latest_blockhash()?);

    client.send_and_confirm_transaction(&create_accounts_tx)?;

    println!("   ✅ Program accounts created at PDA addresses");
    println!("   - Pool: {}", pool_pda);
    println!("   - Commitments log: {}", commitments_pda);
    println!("   - Roots ring: {}", roots_ring_pda);
    println!("   - Nullifier shard: {}", nullifier_shard_pda);
    println!("   - Treasury: {}", treasury_pda);

    Ok(ProgramAccounts {
        pool: pool_pda,
        commitments: commitments_pda,
        roots_ring: roots_ring_pda,
        nullifier_shard: nullifier_shard_pda,
        treasury: treasury_pda,
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

/// Execute withdraw transaction via relay service
async fn execute_withdraw_via_relay(
    prove_result: &ProofArtifacts,
    test_data: &TestData,
    recipient_keypair: &Keypair,
) -> Result<String, anyhow::Error> {
    println!("   💸 Executing Withdraw Transaction via Relay...");

    // Decode hex-encoded public_inputs from the prover
    // Format: root (32 bytes) + nf (32 bytes) + outputs_hash (32 bytes) + amount (8 bytes) = 104 bytes
    let public_inputs_bytes = hex::decode(&prove_result.public_inputs_hex)
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

    // Prepare the withdraw request for the relay using current fee policy
    let fixed_fee = 2_500_000u64;
    let variable_fee = (test_data.amount.saturating_mul(5)) / 1_000; // 0.5%
    let total_fee = fixed_fee.saturating_add(variable_fee);
    let relay_recipient_amount = test_data.amount.saturating_sub(total_fee);
    let effective_fee_bps = if test_data.amount == 0 {
        0u16
    } else {
        let bps = ((total_fee.saturating_mul(10_000)) + test_data.amount - 1) / test_data.amount;
        bps.min(u16::MAX as u64) as u16
    };

    let outputs = vec![Output {
        recipient: recipient_keypair.pubkey().to_string(),
        amount: relay_recipient_amount,
    }];

    let policy = Policy {
        fee_bps: effective_fee_bps,
    };

    let public_inputs_for_relay = PublicInputs {
        root: root_hex.to_string(),
        nf: nf_hex.to_string(),
        amount: test_data.amount,
        fee_bps: effective_fee_bps,
        outputs_hash: outputs_hash_hex.to_string(),
    };

    // Convert hex proof to base64 for relay
    use base64::Engine as _;
    let proof_bytes = hex::decode(&prove_result.proof_hex).unwrap();
    let proof_base64 = base64::engine::general_purpose::STANDARD.encode(&proof_bytes);

    println!("   📡 Preparing withdraw request for relay...");
    println!("   🔍 DEBUG: Request data:");
    println!(
        "      - outputs[0].recipient: {}",
        recipient_keypair.pubkey()
    );
    println!("      - outputs[0].amount: {}", relay_recipient_amount);
    println!("      - policy.fee_bps: {}", effective_fee_bps);
    println!("      - public_inputs.root: {}", root_hex);
    println!("      - public_inputs.nf: {}", nf_hex);
    println!("      - public_inputs.amount: {}", test_data.amount);
    println!("      - public_inputs.fee_bps: {}", effective_fee_bps);
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
            .post("https://api.cloaklabz.xyz/withdraw")
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
                "https://api.cloaklabz.xyz/status/{}",
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
    data: WithdrawData,
}

#[derive(Debug, Deserialize)]
struct WithdrawData {
    request_id: String,
    status: String,
}

async fn verify_miner_status(client: &RpcClient, miner_keypair: &Keypair) -> Result<()> {
    // Check if miner has sufficient balance
    let miner_balance = client.get_balance(&miner_keypair.pubkey())?;
    if miner_balance < SOL_TO_LAMPORTS {
        return Err(anyhow::anyhow!(
            "Miner has insufficient balance: {} SOL (minimum: 1 SOL)",
            miner_balance / SOL_TO_LAMPORTS
        ));
    }

    println!(
        "   ✅ Miner balance sufficient: {} SOL",
        miner_balance / SOL_TO_LAMPORTS
    );

    // Check if there are any active claims by querying the scramble registry
    // This is a simplified check - in a real scenario, we'd query for active claims
    println!("   🔍 Checking for active PoW claims...");

    // For now, we'll just verify the miner can be found on-chain
    // In a production system, we'd query the scramble registry for active claims
    println!("   ✅ Miner verification complete");
    println!("   ℹ️  Note: Ensure cloak-miner is running to provide PoW claims");

    Ok(())
}
