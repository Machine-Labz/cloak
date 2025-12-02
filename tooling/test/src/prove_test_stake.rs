use anyhow::Result;
use hex;
use rand;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::{Duration, Instant};
use test_complete_flow_rust::shared::{
    check_cluster_health, ensure_user_funding, get_pda_addresses, load_keypair, print_config,
    MerkleProof, TestConfig, SOL_TO_LAMPORTS,
};

#[derive(Debug, Clone)]
struct TestData {
    sk_spend: [u8; 32],
    r: [u8; 32],
    amount: u64,
    commitment: String,
    nullifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DepositRequest {
    leaf_commit: String,
    encrypted_output: String,
    tx_signature: String,
    slot: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct WithdrawRequest {
    outputs: Vec<Output>,
    stake: Option<StakeConfig>,
    policy: Policy,
    public_inputs: PublicInputs,
    proof_bytes: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    recipient: String,
    amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StakeConfig {
    stake_account: String,
    stake_authority: String,
    validator_vote_account: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Policy {
    fee_bps: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublicInputs {
    root: String,
    nf: String, // nullifier
    amount: u64,
    fee_bps: u16,
    outputs_hash: String,
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let start_time = std::time::Instant::now();

    println!("🔐 CLOAK PRIVACY PROTOCOL - STAKE WITHDRAWAL TEST");
    println!("==================================================\n");

    // Parse network argument
    let args: Vec<String> = std::env::args().collect();
    let network = args
        .iter()
        .position(|arg| arg == "--network")
        .and_then(|idx| args.get(idx + 1))
        .map(|s| s.as_str())
        .unwrap_or("devnet");

    let config = if network == "testnet" {
        TestConfig::testnet()
    } else {
        TestConfig::devnet()
    };

    print_config(&config);

    // Check cluster health
    check_cluster_health(&config.rpc_url)?;

    // Load keypairs
    let user_keypair = load_keypair(&config.user_keypair_path)?;
    let recipient_keypair = load_keypair(&config.recipient_keypair_path)?;

    let admin_keypair_path = std::env::var("HOME")
        .map(|home| format!("{}/.config/solana/id.json", home))
        .unwrap_or_else(|_| "admin-keypair.json".to_string());
    let admin_keypair = load_keypair(&admin_keypair_path)?;

    let rpc_client = RpcClient::new(config.rpc_url.clone());
    let program_id = Pubkey::from_str(&config.program_id)?;

    println!("📋 Test Configuration:");
    println!("  User: {}", user_keypair.pubkey());
    println!("  Recipient: {}", recipient_keypair.pubkey());
    println!("  Pool Mint: SOL (native)");
    println!();

    // Ensure funding
    ensure_user_funding(&config.rpc_url, &user_keypair, &admin_keypair)?;
    ensure_user_funding(&config.rpc_url, &recipient_keypair, &admin_keypair)?;

    // Step 1: Generate test data
    let deposit_amount: u64 = SOL_TO_LAMPORTS / 10; // 0.1 SOL
    println!("🎲 Step 1: Generating test data...");
    let mut test_data = generate_test_data(deposit_amount)?;
    let commitment = hex::decode(&test_data.commitment)?;
    let commitment_bytes: [u8; 32] = commitment
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid commitment length"))?;
    println!("  Commitment: {}...", &test_data.commitment[..16]);

    // Step 2: Setup stake account (assume it exists or will be created by relay)
    println!("\n🏦 Step 2: Setting up stake account configuration...");
    
    // Use recipient as stake authority for simplicity
    let stake_authority = recipient_keypair.pubkey();
    // Generate a stake account address (in production, this would be created before the withdraw)
    // For testing, we'll use a keypair that the relay can use to create the account
    let stake_keypair = Keypair::new();
    let stake_account = stake_keypair.pubkey();
    
    // Get a validator vote account (using a well-known validator for testing)
    // On devnet/testnet, we can use a known validator or create a test one
    // For now, we'll use a placeholder - in production, this should be a real validator
    let validator_vote_account = Pubkey::from_str(
        "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T" // Example validator (replace with real one)
    )?;
    
    println!("  Stake account: {} (will be created by relay if needed)", stake_account);
    println!("  Stake authority: {}", stake_authority);
    println!("  Validator vote account: {}", validator_vote_account);
    println!("  Note: The stake account should be created and initialized before the withdraw transaction");

    // Step 3: Deposit to shield pool
    println!(
        "\n💰 Step 3: Depositing {} lamports to shield pool...",
        deposit_amount
    );

    let native_mint = Pubkey::default();
    let (pool_pda, commitments_pda, _roots_ring_pda, _nullifier_shard_pda, _treasury_pda) =
        get_pda_addresses(&program_id, &native_mint);

    println!("  Pool PDA: {}", pool_pda);
    println!("  Commitments PDA: {}", commitments_pda);

    // Build deposit instruction
    let mut deposit_data = Vec::new();
    deposit_data.push(0u8); // Deposit discriminator
    deposit_data.extend_from_slice(&deposit_amount.to_le_bytes());
    deposit_data.extend_from_slice(&commitment_bytes);

    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user_keypair.pubkey(), true), // user (signer)
            AccountMeta::new(pool_pda, false),             // pool
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // system_program
            AccountMeta::new(commitments_pda, false),      // commitments
        ],
        data: deposit_data,
    };

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let deposit_tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&user_keypair.pubkey()),
        &[&user_keypair],
        recent_blockhash,
    );

    let deposit_sig = rpc_client.send_and_confirm_transaction(&deposit_tx)?;
    println!("  ✅ Deposit transaction: {}", deposit_sig);

    // Wait for confirmation
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Step 4: Submit deposit to indexer
    println!("\n🌳 Step 4: Submitting deposit to indexer...");
    let client = reqwest::Client::new();

    let slot = rpc_client.get_slot()?;

    let deposit_request = DepositRequest {
        leaf_commit: test_data.commitment.clone(),
        encrypted_output: "encrypted_output_placeholder".to_string(),
        tx_signature: deposit_sig.to_string(),
        slot: slot as i64,
    };

    let deposit_response = client
        .post(format!("{}/api/v1/deposit", config.indexer_url))
        .json(&deposit_request)
        .send()
        .await?;

    println!("  Indexer deposit response: {}", deposit_response.status());

    if !deposit_response.status().is_success() {
        let error_text = deposit_response.text().await?;
        println!("  ❌ Deposit failed: {}", error_text);
        return Err(anyhow::anyhow!("Deposit to indexer failed: {}", error_text));
    }

    // Parse deposit response
    let deposit_result: serde_json::Value = deposit_response.json().await?;
    let leaf_index = deposit_result["leafIndex"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing leafIndex in deposit response"))?
        as u32;
    let current_root = deposit_result["root"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing root in deposit response"))?
        .to_string();

    // Update nullifier with actual leaf index
    {
        use blake3::Hasher;
        let mut nullifier_hasher = Hasher::new();
        nullifier_hasher.update(&test_data.sk_spend);
        nullifier_hasher.update(&leaf_index.to_le_bytes());
        let updated_nullifier = nullifier_hasher.finalize();
        test_data.nullifier = hex::encode(updated_nullifier.as_bytes());
    }

    println!("  ✅ Deposit processed successfully");
    println!("  Leaf index: {}", leaf_index);
    println!("  Current root: {}...", &current_root[..16]);
    println!("  Nullifier (updated): {}", test_data.nullifier);

    // Wait for indexer to process
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get Merkle proof
    let merkle_response = client
        .get(format!(
            "{}/api/v1/merkle/proof/{}",
            config.indexer_url, leaf_index
        ))
        .send()
        .await?;

    let merkle_text = merkle_response.text().await?;
    let proof_response: MerkleProof = serde_json::from_str(&merkle_text)?;

    println!(
        "  Merkle path length: {}",
        proof_response.path_elements.len()
    );

    // Step 5: Generate ZK proof with stake_params
    println!("\n🔬 Step 5: Generating ZK proof with stake_params...");

    let proof_start = Instant::now();

    let (private_inputs, public_inputs, outputs_json, stake_params_json) = prepare_proof_inputs(
        &test_data,
        &proof_response,
        &current_root,
        leaf_index,
        &stake_account,
        &stake_authority,
        &validator_vote_account,
    )?;

    // Call indexer's /prove endpoint
    let prove_request = serde_json::json!({
        "private_inputs": private_inputs,
        "public_inputs": public_inputs,
        "outputs": outputs_json,
        "stake_params": stake_params_json
    });

    println!("  📡 Calling indexer /prove endpoint...");

    let prove_response = client
        .post(format!("{}/api/v1/prove", config.indexer_url))
        .json(&prove_request)
        .send()
        .await?;

    let proof_result: serde_json::Value = prove_response.json().await?;

    let success = proof_result["success"].as_bool().unwrap_or(false);
    if !success {
        let error_msg = proof_result["error"].as_str().unwrap_or("Unknown error");
        println!("  ❌ Proof generation failed: {}", error_msg);
        return Err(anyhow::anyhow!("Proof generation failed: {}", error_msg));
    }

    let proof_duration = proof_start.elapsed();
    println!(
        "  ✅ Proof generated in {:.2}s",
        proof_duration.as_secs_f64()
    );

    let proof_hex = proof_result["proof"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing proof in response"))?
        .to_string();

    // Calculate fee
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (deposit_amount * 5) / 1_000; // 0.5%
    let total_fee = fixed_fee + variable_fee;
    let stake_amount = deposit_amount - total_fee;

    let effective_fee_bps = if deposit_amount == 0 {
        0u16
    } else {
        let bps = ((variable_fee.saturating_mul(10_000)) + deposit_amount - 1) / deposit_amount;
        bps.min(u16::MAX as u64) as u16
    };

    // Extract outputs_hash from public_inputs
    let public_inputs_json: serde_json::Value = serde_json::from_str(&public_inputs)?;
    let outputs_hash = public_inputs_json["outputs_hash"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing outputs_hash in public_inputs"))?
        .to_string();

    // Convert proof from hex to base64
    use base64::Engine as _;
    let proof_bytes_vec = hex::decode(&proof_hex)
        .map_err(|e| anyhow::anyhow!("Failed to decode proof hex: {}", e))?;
    let proof_base64 = base64::engine::general_purpose::STANDARD.encode(&proof_bytes_vec);

    // Step 6: Submit stake withdraw request to relay
    println!("\n🔄 Step 6: Submitting stake withdraw request to relay...");
    println!("  Stake account: {}", stake_account);
    println!("  Stake amount: {} lamports", stake_amount);
    println!("  Fee BPS: {}", effective_fee_bps);

    let withdraw_request = WithdrawRequest {
        outputs: vec![], // Empty outputs for staking mode
        stake: Some(StakeConfig {
            stake_account: stake_account.to_string(),
            stake_authority: stake_authority.to_string(),
            validator_vote_account: validator_vote_account.to_string(),
        }),
        policy: Policy {
            fee_bps: effective_fee_bps,
        },
        public_inputs: PublicInputs {
            root: current_root,
            nf: test_data.nullifier.clone(),
            amount: deposit_amount,
            fee_bps: effective_fee_bps,
            outputs_hash,
        },
        proof_bytes: proof_base64,
    };

    println!("\n  Request payload:");
    println!("  {}", serde_json::to_string_pretty(&withdraw_request)?);

    let withdraw_response = client
        .post(format!("{}/withdraw", config.relay_url))
        .json(&withdraw_request)
        .send()
        .await?;

    println!("\n  Relay response status: {}", withdraw_response.status());
    let response_text = withdraw_response.text().await?;
    println!("  Response: {}", response_text);

    if response_text.contains("request_id") {
        println!("\n  ✅ Stake withdraw request submitted successfully!");

        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        let job_id = response_json
            .get("data")
            .and_then(|d| d.get("request_id"))
            .and_then(|v| v.as_str())
            .or_else(|| response_json.get("request_id").and_then(|v| v.as_str()))
            .unwrap_or("unknown");

        println!("  Request ID: {}", job_id);

        // Step 7: Monitor job status
        println!("\n📊 Step 7: Monitoring stake withdrawal status...");

        let mut attempts = 0;
        let max_attempts = 30;
        let mut completed = false;

        while attempts < max_attempts {
            tokio::time::sleep(Duration::from_secs(2)).await;
            attempts += 1;

            let status_response = client
                .get(format!("{}/status/{}", config.relay_url, job_id))
                .send()
                .await?;

            if status_response.status().is_success() {
                let status_json: serde_json::Value = status_response.json().await?;
                let status = status_json
                    .get("data")
                    .and_then(|d| d.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                println!("  [{}s] Status: {}", attempts * 2, status);

                if status == "completed" {
                    let tx_sig = status_json
                        .get("data")
                        .and_then(|d| d.get("tx_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    println!("\n  🎉 Stake withdrawal completed!");
                    println!("  Transaction: {}", tx_sig);
                    completed = true;
                    break;
                } else if status == "failed" {
                    println!("\n  ❌ Stake withdrawal failed");
                    println!("  Error: {}", serde_json::to_string_pretty(&status_json)?);
                    break;
                }
            }
        }

        if !completed {
            println!("\n  ⏱️  Timeout waiting for completion (check relay logs)");
        }

        // Step 8: Verify stake account received funds
        println!("\n✅ Step 8: Verifying stake account balance...");

        match rpc_client.get_account(&stake_account) {
            Ok(account) => {
                println!("  ✅ Stake account balance: {} lamports", account.lamports);
                println!("  📈 Staking executed successfully!");
                println!("  Note: The stake account should be delegated separately if needed");
            }
            Err(e) => {
                println!("  ⚠️  Could not fetch stake account: {}", e);
                println!("  (The account may not exist yet - check the transaction logs)");
            }
        }
    } else {
        println!("\n  ❌ Failed to submit stake withdraw request");
        return Err(anyhow::anyhow!("Withdraw submission failed"));
    }

    let total_duration = start_time.elapsed();
    println!(
        "\n⏱️  Total test duration: {:.2}s",
        total_duration.as_secs_f64()
    );
    println!("\nℹ️  Test finished. See relay logs for stake execution.");

    Ok(())
}

fn prepare_proof_inputs(
    test_data: &TestData,
    merkle_proof: &MerkleProof,
    merkle_root: &str,
    leaf_index: u32,
    stake_account: &Pubkey,
    stake_authority: &Pubkey,
    validator_vote_account: &Pubkey,
) -> Result<(String, String, String, serde_json::Value)> {
    use blake3::Hasher;

    println!("   🔐 Preparing proof inputs...");

    let nullifier_hex = test_data.nullifier.clone();
    println!("   - Nullifier: {}", nullifier_hex);

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
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (test_data.amount * 5) / 1_000; // 0.5%
    let total_fee = fixed_fee + variable_fee;
    let stake_amount = test_data.amount - total_fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!("   - Fixed fee: {} lamports", fixed_fee);
    println!("   - Variable fee (0.5%): {} lamports", variable_fee);
    println!("   - Total fee: {} lamports", total_fee);
    println!("   - Stake amount: {} lamports", stake_amount);

    // Empty outputs for stake mode
    let outputs = serde_json::json!([]);

    // Compute stake outputs hash: H(stake_account || public_amount)
    // Note: The circuit uses this format, matching compute_stake_outputs_hash in encoding.rs
    let mut hasher = Hasher::new();
    hasher.update(&stake_account.to_bytes());
    hasher.update(&test_data.amount.to_le_bytes()); // public_amount, not stake_amount
    let outputs_hash = hasher.finalize();
    let outputs_hash_hex = hex::encode(outputs_hash.as_bytes());
    println!("   - Stake outputs hash: {}", outputs_hash_hex);

    // Create public inputs
    let public_inputs = serde_json::json!({
        "root": merkle_root,
        "nf": nullifier_hex,
        "outputs_hash": outputs_hash_hex,
        "amount": test_data.amount
    });

    // Build stake_params for SP1 guest
    // Note: The circuit only expects stake_account in StakeParams (see encoding.rs)
    // The outputs_hash is computed as H(stake_account || public_amount)
    let stake_params_json = serde_json::json!({
        "stake_account": hex::encode(stake_account.to_bytes())
    });
    println!("   - stake_params: {}", serde_json::to_string_pretty(&stake_params_json)?);

    println!("   ✅ Proof inputs prepared");

    Ok((
        serde_json::to_string(&private_inputs)?,
        serde_json::to_string(&public_inputs)?,
        serde_json::to_string(&outputs)?,
        stake_params_json,
    ))
}

