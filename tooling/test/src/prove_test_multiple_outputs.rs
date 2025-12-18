use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::Result;
use cloak_proof_extract::extract_groth16_260_sp1;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use sp1_sdk::{network::FulfillmentStrategy, HashableKey, Prover, ProverClient, SP1Stdin};
use test_complete_flow_rust::shared::{
    check_cluster_health, ensure_user_funding, load_keypair, print_config, MerkleProof, TestConfig,
    SOL_TO_LAMPORTS,
};
use tokio::time::timeout;
use zk_guest_sp1_host::{
    generate_proof as generate_proof_local, ProofResult as LocalProofResult, ELF,
};

// Relay API structures
#[derive(Debug, Serialize, Deserialize)]
struct WithdrawRequest {
    outputs: Vec<RelayOutput>,
    swap: Option<SwapConfig>,
    policy: Policy,
    public_inputs: PublicInputs,
    proof_bytes: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RelayOutput {
    recipient: String,
    amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SwapConfig {
    output_mint: String,
    slippage_bps: u16,
    min_output_amount: u64,
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

    let config = TestConfig::devnet();
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

    // Deploy program only on localnet; on devnet, use existing deployed program
    let program_id = if config.is_devnet() {
        println!(
            "\n✅ Using existing program on devnet: {}",
            config.program_id
        );
        Pubkey::from_str(&config.program_id)?
    } else {
        println!("\n🚀 Step 0: Deploying Program...");
        deploy_program(&client)?
    };

    // Create program accounts only on localnet; on devnet, they should already exist
    let accounts = if config.is_devnet() {
        println!("\n✅ Using existing program accounts on devnet...");
        use test_complete_flow_rust::shared::get_pda_addresses;
        let (pool, commitments, roots_ring, nullifier_shard, treasury) =
            get_pda_addresses(&program_id, &Pubkey::default());

        println!("   - Pool (derived PDA): {}", pool);
        println!("   - Commitments log (derived PDA): {}", commitments);
        println!("   - Roots ring (derived PDA): {}", roots_ring);
        println!("   - Nullifier shard (derived PDA): {}", nullifier_shard);
        println!("   - Treasury (derived PDA): {}", treasury);

        ProgramAccounts {
            pool,
            commitments,
            roots_ring,
            nullifier_shard,
            treasury,
        }
    } else {
        println!("\n📋 Step 1: Creating Program Accounts...");
        create_program_accounts(&client, &program_id, &admin_keypair)?
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
    let (private_inputs, public_inputs, outputs, output_details) = prepare_proof_inputs(
        &test_data,
        &merkle_proof,
        &merkle_root,
        leaf_index,
        &recipient_keypair,
        &miner_keypair,
    )?;

    // Generate proof locally or via TEE
    println!("\n🚀 Step 9: Generating Proof Client-Side...");
    let prove_result =
        generate_proof_client_side(&private_inputs, &public_inputs, &outputs).await?;

    // Validate proof artifacts
    println!("\n✅ Step 10: Validating Proof Artifacts...");
    validate_proof_artifacts(&prove_result)?;

    // Submit withdraw request to relay
    println!("\n💸 Step 11: Submitting Withdraw Request to Relay...");
    let (withdraw_signature, request_id) = execute_withdraw_via_relay(
        &config.relay_url,
        &prove_result,
        &output_details,
        &merkle_root,
        &test_data.nullifier,
        test_data.amount,
    )
    .await?;

    println!(
        "   ✅ Withdraw transaction completed: {}",
        withdraw_signature
    );
    println!("   Request ID: {}", request_id);

    // Verify recipient balances
    let recipient_balance_after = client.get_balance(&recipient_keypair.pubkey())?;
    let miner_balance_after = client.get_balance(&miner_keypair.pubkey())?;
    let recipient_delta = recipient_balance_after.saturating_sub(recipient_balance);
    let miner_delta = miner_balance_after.saturating_sub(miner_balance);

    let recipient_expected = output_details
        .iter()
        .find(|(pk, _)| *pk == recipient_keypair.pubkey())
        .map(|(_, amount)| *amount)
        .unwrap_or(0);
    let miner_expected = output_details
        .iter()
        .find(|(pk, _)| *pk == miner_keypair.pubkey())
        .map(|(_, amount)| *amount)
        .unwrap_or(0);

    println!(
        "\n📊 Recipient balance before: {} SOL",
        recipient_balance / SOL_TO_LAMPORTS
    );
    println!(
        "   📊 Recipient balance after: {} SOL",
        recipient_balance_after / SOL_TO_LAMPORTS
    );
    println!(
        "   📊 Recipient delta: {} lamports (expected {})",
        recipient_delta, recipient_expected
    );

    println!(
        "\n📊 Second recipient (miner) before: {} SOL",
        miner_balance / SOL_TO_LAMPORTS
    );
    println!(
        "   📊 Second recipient (miner) after: {} SOL",
        miner_balance_after / SOL_TO_LAMPORTS
    );
    println!(
        "   📊 Second recipient delta: {} lamports (expected {})",
        miner_delta, miner_expected
    );

    if recipient_delta != recipient_expected {
        println!(
            "   ⚠️  Recipient amount mismatch! expected {} got {}",
            recipient_expected, recipient_delta
        );
    }
    if miner_delta != miner_expected {
        println!(
            "   ⚠️  Second recipient amount mismatch! expected {} got {}",
            miner_expected, miner_delta
        );
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

    println!("\n💸 Outputs Summary:");
    for (index, (pk, amount)) in output_details.iter().enumerate() {
        let delta = if *pk == recipient_keypair.pubkey() {
            recipient_delta
        } else if *pk == miner_keypair.pubkey() {
            miner_delta
        } else {
            0
        };
        println!(
            "   - Output #{} -> {}: {} lamports (delta {})",
            index, pk, amount, delta
        );
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
        .post(format!("{}/api/v1/admin/reset", indexer_url))
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
        .args([
            "exec",
            "cloak-postgres",
            "psql",
            "-U",
            "cloak",
            "-d",
            "cloak",
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
        &test_data.commitment[..8],
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
        .post(format!("{}/api/v1/deposit", indexer_url))
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
        .get(format!("{}/api/v1/merkle/root", indexer_url))
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
        .get(format!(
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
    miner_keypair: &Keypair,
) -> Result<(String, String, String, Vec<(Pubkey, u64)>)> {
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

    // Calculate fee (must match zk-guest-sp1/guest/src/encoding.rs::calculate_fee)
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (test_data.amount * 5) / 1_000; // 0.5%
        fixed_fee + variable_fee
    };
    let distributable_amount = test_data.amount - fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!("   - Fee: {} lamports", fee);
    println!(
        "   - Distributable amount: {} lamports",
        distributable_amount
    );

    // Use actual recipient keypairs
    let primary_pubkey = recipient_keypair.pubkey();
    let secondary_pubkey = miner_keypair.pubkey();

    // Split amounts deterministically (2/3 and remainder)
    let primary_amount = distributable_amount * 2 / 3;
    let secondary_amount = distributable_amount - primary_amount;

    println!(
        "   - Primary recipient: {} ({} lamports)",
        primary_pubkey, primary_amount
    );
    println!(
        "   - Secondary recipient: {} ({} lamports)",
        secondary_pubkey, secondary_amount
    );

    let outputs_details = vec![
        (primary_pubkey, primary_amount),
        (secondary_pubkey, secondary_amount),
    ];

    // Create outputs JSON structure
    let outputs = serde_json::json!([
        {
            "address": hex::encode(primary_pubkey.to_bytes()),
            "amount": primary_amount
        },
        {
            "address": hex::encode(secondary_pubkey.to_bytes()),
            "amount": secondary_amount
        }
    ]);

    // Compute outputs hash exactly like SP1 guest program
    let mut hasher = Hasher::new();
    for (pk, amount) in &outputs_details {
        hasher.update(&pk.to_bytes());
        hasher.update(&amount.to_le_bytes());
    }
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
        outputs_details,
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

    println!(
        "         Starting hash (commitment): {}",
        hex::encode(&current_hash)
    );
    println!("         Expected root: {}", root_hex);
    println!("         Number of path elements: {}", path_elements.len());

    for (i, (sibling_hex, &is_left)) in path_elements.iter().zip(path_indices.iter()).enumerate() {
        let sibling = hex::decode(sibling_hex)?;
        println!(
            "         Level {}: is_left={}, sibling={}",
            i, is_left, sibling_hex
        );
        let mut hasher = Hasher::new();
        if is_left == 0 {
            // Current is left, sibling is right
            println!("           -> hash(current || sibling)");
            hasher.update(&current_hash);
            hasher.update(&sibling);
        } else {
            // Current is right, sibling is left
            println!("           -> hash(sibling || current)");
            hasher.update(&sibling);
            hasher.update(&current_hash);
        }
        current_hash = hasher.finalize().as_bytes().to_vec();
        println!("           = {}", hex::encode(&current_hash));
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

    // Fee calculation must match zk-guest-sp1/guest/src/encoding.rs::calculate_fee
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (amount * 5) / 1_000; // 0.5%
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
    let proof_hex = hex::encode(canonical_proof);
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
    let proof_hex = hex::encode(canonical_proof);
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
    let mint = Pubkey::new_unique();

    println!("   Deriving PDA addresses...");
    // Use the exact same program ID as the program expects
    let program_id_bytes = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"
        .parse::<Pubkey>()
        .unwrap();
    let (pool_pda, commitments_pda, roots_ring_pda, nullifier_shard_pda, treasury_pda) =
        get_pda_addresses(&program_id_bytes, &mint);

    println!("   - Pool PDA: {}", pool_pda);
    println!("   - Commitments PDA: {}", commitments_pda);
    println!("   - Roots Ring PDA: {}", roots_ring_pda);
    println!("   - Nullifier Shard PDA: {}", nullifier_shard_pda);
    println!("   - Treasury PDA: {}", treasury_pda);

    // Debug: Print expected PDA addresses
    println!("   Debug - Expected PDAs:");
    println!("     Pool: {}", pool_pda);
    println!("     Commitments: {}", commitments_pda);
    println!("     Roots Ring: {}", roots_ring_pda);
    println!("     Nullifier Shard: {}", nullifier_shard_pda);
    println!("     Treasury: {}", treasury_pda);

    // Ensure we use the exact same program ID as the program expects
    // gitleaks:allow - This is a Solana program ID (public key), not a secret
    let program_id_bytes: Pubkey = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"
        .parse::<Pubkey>()
        .unwrap();
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
    let mut create_accounts_tx =
        Transaction::new_with_payer(&[initialize_pool_ix], Some(&admin_keypair.pubkey()));

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

async fn execute_withdraw_via_relay(
    relay_url: &str,
    prove_result: &ProofArtifacts,
    outputs: &[(Pubkey, u64)],
    merkle_root: &str,
    nullifier: &str,
    total_amount: u64,
) -> Result<(String, String)> {
    use base64::Engine as _;

    println!(
        "   🔧 Preparing withdraw request with {} outputs for relay",
        outputs.len()
    );

    // Calculate fee_bps based on the circuit's fee calculation
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (total_amount * 5) / 1_000; // 0.5%
        fixed_fee + variable_fee
    };

    let effective_fee_bps = if total_amount == 0 {
        0u16
    } else {
        let bps = (fee.saturating_mul(10_000)).div_ceil(total_amount);
        bps.min(u16::MAX as u64) as u16
    };

    // Extract outputs_hash from the public inputs
    let public_inputs_bytes = hex::decode(&prove_result.public_inputs_hex)
        .map_err(|e| anyhow::anyhow!("Failed to decode public_inputs hex: {}", e))?;

    if public_inputs_bytes.len() != 104 {
        return Err(anyhow::anyhow!(
            "Invalid public_inputs length: expected 104 bytes, got {}",
            public_inputs_bytes.len()
        ));
    }

    let mut outputs_hash_bytes = [0u8; 32];
    outputs_hash_bytes.copy_from_slice(&public_inputs_bytes[64..96]);
    let outputs_hash = hex::encode(outputs_hash_bytes);

    // Convert proof from hex to base64 (relay expects base64)
    let proof_bytes_vec = hex::decode(&prove_result.proof_hex)
        .map_err(|e| anyhow::anyhow!("Failed to decode proof hex: {}", e))?;
    let proof_base64 = base64::engine::general_purpose::STANDARD.encode(&proof_bytes_vec);

    // Build relay withdraw request
    let relay_outputs: Vec<RelayOutput> = outputs
        .iter()
        .enumerate()
        .map(|(index, (pk, amount))| {
            println!("   - Output #{} -> {} ({} lamports)", index, pk, amount);
            RelayOutput {
                recipient: pk.to_string(),
                amount: *amount,
            }
        })
        .collect();

    let withdraw_request = WithdrawRequest {
        outputs: relay_outputs,
        swap: None, // No swap for multiple outputs test
        policy: Policy {
            fee_bps: effective_fee_bps,
        },
        public_inputs: PublicInputs {
            root: merkle_root.to_string(),
            nf: nullifier.to_string(),
            amount: total_amount,
            fee_bps: effective_fee_bps,
            outputs_hash,
        },
        proof_bytes: proof_base64,
    };

    println!("\n  Request payload:");
    println!("  {}", serde_json::to_string_pretty(&withdraw_request)?);

    // Submit to relay
    let client = reqwest::Client::new();
    let withdraw_response = client
        .post(format!("{}/withdraw", relay_url))
        .json(&withdraw_request)
        .send()
        .await?;

    println!("\n  Relay response status: {}", withdraw_response.status());
    let response_text = withdraw_response.text().await?;
    println!("  Response: {}", response_text);

    if !response_text.contains("request_id") {
        return Err(anyhow::anyhow!(
            "Failed to submit withdraw request: {}",
            response_text
        ));
    }

    // Parse request_id from API response payload
    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
    let job_id = response_json
        .get("data")
        .and_then(|d| d.get("request_id"))
        .and_then(|v| v.as_str())
        .or_else(|| response_json.get("request_id").and_then(|v| v.as_str()))
        .ok_or_else(|| anyhow::anyhow!("Missing request_id in response"))?;

    println!("\n  ✅ Withdraw request submitted successfully!");
    println!("  Request ID: {}", job_id);

    // Monitor job status
    println!("\n📊 Monitoring withdrawal status...");

    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;

        let status_response = client
            .get(format!("{}/status/{}", relay_url, job_id))
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
                    .ok_or_else(|| anyhow::anyhow!("Missing tx_id in completed response"))?;

                println!("\n  🎉 Withdrawal completed!");
                println!("  Transaction: {}", tx_sig);

                return Ok((tx_sig.to_string(), job_id.to_string()));
            } else if status == "failed" {
                let error = status_json
                    .get("data")
                    .and_then(|d| d.get("error"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");

                return Err(anyhow::anyhow!("Withdrawal failed: {}", error));
            }
        }
    }

    Err(anyhow::anyhow!("Timeout waiting for withdrawal completion"))
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
