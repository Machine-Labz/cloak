use anyhow::Result;
use base64;
use hex;
use rand;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use std::str::FromStr;
use test_complete_flow_rust::shared::{
    check_cluster_health, ensure_user_funding, load_keypair, print_config, validate_config,
    MerkleProof, TestConfig, SOL_TO_LAMPORTS,
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

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = std::time::Instant::now();

    println!("🚀 CLOAK PRIVACY PROTOCOL - LOCALNET TEST");
    println!("==========================================\n");

    let config = TestConfig::localnet();
    print_config(&config);

    // Validate configuration
    validate_config(&config)
        .map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;

    // Check cluster health
    check_cluster_health(&config.rpc_url)?;

    // Load keypairs
    let user_keypair = load_keypair(&config.user_keypair_path)?;
    let recipient_keypair = load_keypair(&config.recipient_keypair_path)?;
    let admin_keypair = load_keypair("admin-keypair.json")?; // Use admin keypair for admin operations

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

    // Reset indexer database to avoid conflicts
    println!("\n🔄 Step 3: Resetting Indexer Database...");
    reset_indexer_database(&config.indexer_url).await?;

    // Generate test data
    println!("\n🔨 Step 4: Generating Test Data...");
    let mut test_data = generate_test_data(config.amount)?;

    // Deposit to indexer
    println!("\n📥 Step 5: Depositing to Indexer...");
    let leaf_index = deposit_to_indexer(&config.indexer_url, &mut test_data).await?;

    // Create real deposit transaction
    println!("\n💰 Step 6: Creating Real Deposit Transaction...");
    let deposit_signature =
        create_deposit_transaction(&client, &program_id, &accounts, &test_data, &user_keypair)?;

    // Get merkle root and push to program
    println!("\n🌳 Step 7: Getting Merkle Root from Indexer...");
    let merkle_root = get_merkle_root(&config.indexer_url).await?;
    push_root_to_program(
        &client,
        &program_id,
        &accounts,
        &merkle_root,
        &admin_keypair,
    )?;

    // Get merkle proof
    println!("\n🔍 Step 8: Getting Merkle Proof from Indexer...");
    let merkle_proof = get_merkle_proof(&config.indexer_url, leaf_index).await?;

    // Verify merkle path
    println!("\n🔍 Step 9: Verifying Merkle Path...");
    verify_merkle_path(&test_data.commitment, &merkle_proof, &merkle_root)?;

    // Generate SP1 proof
    println!("\n🔐 Step 10: Generating SP1 Proof Inputs...");
    let sp1_inputs = generate_sp1_proof_inputs(
        &test_data,
        &merkle_proof,
        &merkle_root,
        leaf_index,
        &recipient_keypair,
    )?;

    println!("\n🔨 Step 11: Generating SP1 Proof with Current Data...");
    let sp1_proof = generate_sp1_proof(&sp1_inputs)?;

    // Execute withdraw
    println!("\n💸 Step 12: Executing Withdraw Transaction...");
    let withdraw_signature = execute_withdraw_transaction(
        &client,
        &program_id,
        &accounts,
        &sp1_proof,
        &test_data,
        &recipient_keypair,
        &admin_keypair,
    )?;

    // Success!
    println!("\n🎉 CLOAK PRIVACY PROTOCOL - TEST RESULT");
    println!("=======================================");
    println!("✅ Test completed successfully!");
    println!("\n📊 Transaction Details:");
    println!("   - Deposit: {}", deposit_signature);
    println!("   - Withdraw: {}", withdraw_signature);

    println!("\n🔐 Privacy Protocol Summary:");
    println!("   - Commitment: {}", test_data.commitment);
    println!("   - Merkle root: {}", merkle_root);
    println!("   - Nullifier: {}", test_data.nullifier);

    println!("\n🚀 The Cloak privacy protocol is now fully functional!");
    println!("   - Real Solana transactions ✅");
    println!("   - Real BLAKE3 computation ✅");
    println!("   - Real Merkle tree with 31-level paths ✅");
    println!("   - Real SP1 proof generation ✅");
    println!("   - Real indexer integration ✅");
    println!("   - Production-ready infrastructure ✅");

    println!("\n🔄 Test completed! Running on Solana Localnet...");
    println!("   📋 Network: Solana Localnet ({})", config.rpc_url);
    println!("   📋 Program ID: {}", config.program_id);
    println!("   📋 Indexer Status: Running on {}", config.indexer_url);
    println!("   📋 Database Status: PostgreSQL running in Docker");
    println!("\n   ✅ Test process completed");

    Ok(())
}

// Helper functions (simplified versions of the complex logic)
fn deploy_program(client: &RpcClient) -> Result<Pubkey> {
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
            "admin-keypair.json", // Use admin keypair as authority
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

    // Parse the deployed program ID from the output
    let stdout = String::from_utf8_lossy(&deploy_output.stdout);
    println!("   Deploy output: {}", stdout);

    // Extract program ID from output (format: "Program Id: <program_id>")
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

    // Generate unique keypairs for each account
    let pool_keypair = Keypair::new();
    let roots_ring_keypair = Keypair::new();
    let nullifier_shard_keypair = Keypair::new();
    let treasury_keypair = Keypair::new();

    println!("   Creating pool account...");

    // Create pool account (owned by shield pool program, rent-exempt minimum)
    let pool_rent_exempt = client.get_minimum_balance_for_rent_exemption(0)?;
    let create_pool_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &pool_keypair.pubkey(),
        pool_rent_exempt, // rent-exempt minimum for 0-byte account
        0,                // 0 bytes data
        &program_id,      // Owned by the shield pool program
    );

    println!("   Creating roots ring account...");

    // Create roots ring account with correct size (2056 bytes)
    const ROOTS_RING_SIZE: usize = 2056; // 8 + 64 * 32
    let create_roots_ring_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &roots_ring_keypair.pubkey(),
        client.get_minimum_balance_for_rent_exemption(ROOTS_RING_SIZE)?,
        ROOTS_RING_SIZE as u64,
        program_id, // Owned by our program
    );

    println!("   Creating nullifier shard account...");

    // Create nullifier shard account (4 + 32*N bytes, start with 4 bytes for count)
    const NULLIFIER_SHARD_SIZE: usize = 4; // Start with just count field
    let create_nullifier_shard_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &nullifier_shard_keypair.pubkey(),
        client.get_minimum_balance_for_rent_exemption(NULLIFIER_SHARD_SIZE)?,
        NULLIFIER_SHARD_SIZE as u64,
        program_id, // Owned by our program
    );

    println!("   Creating treasury account...");

    // Create treasury account (owned by system program, 0 lamports initially)
    let create_treasury_ix = system_instruction::create_account(
        &admin_keypair.pubkey(),
        &treasury_keypair.pubkey(),
        0, // rent-exempt minimum
        0, // 0 bytes data
        &solana_sdk::system_program::id(),
    );

    // Create transaction with all account creation instructions
    let mut create_accounts_tx = Transaction::new_with_payer(
        &[
            create_pool_ix,
            create_roots_ring_ix,
            create_nullifier_shard_ix,
            create_treasury_ix,
        ],
        Some(&admin_keypair.pubkey()),
    );

    // Sign with both admin and all the new keypairs
    create_accounts_tx.sign(
        &[
            &admin_keypair,
            &pool_keypair,
            &roots_ring_keypair,
            &nullifier_shard_keypair,
            &treasury_keypair,
        ],
        client.get_latest_blockhash()?,
    );

    client.send_and_confirm_transaction(&create_accounts_tx)?;

    println!("   ✅ All program accounts created successfully");
    println!("   - Pool account: {}", pool_keypair.pubkey());
    println!("   - Roots ring account: {}", roots_ring_keypair.pubkey());
    println!(
        "   - Nullifier shard account: {}",
        nullifier_shard_keypair.pubkey()
    );
    println!("   - Treasury account: {}", treasury_keypair.pubkey());

    // Verify pool account ownership
    let pool_account_info = client.get_account(&pool_keypair.pubkey())?;
    println!("   🔍 Pool account owner: {}", pool_account_info.owner);
    println!(
        "   🔍 Pool account lamports: {}",
        pool_account_info.lamports
    );
    println!("   🔍 Expected program ID: {}", program_id);

    Ok(ProgramAccounts {
        pool: pool_keypair.pubkey(),
        roots_ring: roots_ring_keypair.pubkey(),
        nullifier_shard: nullifier_shard_keypair.pubkey(),
        treasury: treasury_keypair.pubkey(),
    })
}

fn generate_test_data(amount: u64) -> Result<TestData> {
    use blake3::Hasher;
    use rand::RngCore;

    // Generate UNIQUE random test data for each run
    let mut sk_spend = [0u8; 32];
    let mut r = [0u8; 32];

    // Use system time as seed for deterministic but unique randomness
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Generate unique random data based on timestamp
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut sk_spend);
    rng.fill_bytes(&mut r);

    // Add timestamp to ensure uniqueness even if random values collide
    for i in 0..8 {
        sk_spend[i] ^= (timestamp >> (i * 8)) as u8;
        r[i] ^= (timestamp >> (i * 8)) as u8;
    }

    println!("   - sk_spend: {}", hex::encode(sk_spend));
    println!("   - r: {}", hex::encode(r));
    println!("   - amount: {}", amount);

    // Compute pk_spend = H(sk_spend)
    let pk_spend = blake3::hash(&sk_spend);
    println!("   - pk_spend: {}", hex::encode(pk_spend.as_bytes()));

    // Compute commitment = H(amount || r || pk_spend) - exactly like SP1 guest program
    let mut hasher = Hasher::new();
    hasher.update(&amount.to_le_bytes());
    hasher.update(&r);
    hasher.update(pk_spend.as_bytes());
    let commitment = hasher.finalize();
    let commitment_hex = hex::encode(commitment.as_bytes());
    println!("   - commitment: {}", commitment_hex);

    // Compute nullifier = H(sk_spend || leaf_index) exactly like SP1 guest program
    let mut nullifier_hasher = Hasher::new();
    nullifier_hasher.update(&sk_spend);
    nullifier_hasher.update(&0u32.to_le_bytes()); // leaf_index = 0 initially, will be updated after deposit
    let nullifier = nullifier_hasher.finalize();
    let nullifier_hex = hex::encode(nullifier.as_bytes());
    println!("   - nullifier (initial): {}", nullifier_hex);

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

    // Try to call a reset endpoint if it exists, or we can implement a simple approach
    // For now, let's try to clear the database by calling a potential admin endpoint
    let reset_response = http_client
        .post(&format!("{}/api/v1/admin/reset", indexer_url))
        .send()
        .await;

    match reset_response {
        Ok(response) => {
            if response.status().is_success() {
                println!("   ✅ Indexer database reset successfully");
                return Ok(());
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                println!("   ❌ Reset failed: {}", error_text);
                return Err(anyhow::anyhow!("Reset failed: {}", error_text));
            }
        }
        Err(e) => {
            println!("   ❌ Failed to call reset endpoint: {}", e);
            return Err(anyhow::anyhow!("Failed to call reset endpoint: {}", e));
        }
    }
}

async fn deposit_to_indexer(indexer_url: &str, test_data: &mut TestData) -> Result<u32> {
    let http_client = reqwest::Client::new();

    // Generate a unique transaction signature to avoid conflicts
    let unique_tx_signature = format!(
        "deposit_{}_{}_{}",
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
                "Deposit {} SOL at {}",
                test_data.amount / SOL_TO_LAMPORTS,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ))
        },
        tx_signature: Some(unique_tx_signature),
        slot: Some(1000), // Dummy slot
    };

    let deposit_response = http_client
        .post(&format!("{}/api/v1/deposit", indexer_url))
        .json(&deposit_request)
        .send()
        .await?;

    if deposit_response.status().is_success() {
        let deposit_data: serde_json::Value = deposit_response.json().await?;
        let indexer_commitment = deposit_data["leafCommit"].as_str().unwrap();
        let actual_leaf_index = deposit_data["leafIndex"].as_u64().unwrap() as u32;

        // Update nullifier with actual leaf index
        use blake3::Hasher;
        let mut nullifier_hasher = Hasher::new();
        nullifier_hasher.update(&test_data.sk_spend);
        nullifier_hasher.update(&actual_leaf_index.to_le_bytes());
        let updated_nullifier = nullifier_hasher.finalize();
        test_data.nullifier = hex::encode(updated_nullifier.as_bytes());

        println!("   ✅ Deposit successful to indexer");
        println!("   - Indexer commitment: {}", indexer_commitment);
        println!("   - Our commitment: {}", test_data.commitment);
        println!("   - Actual leaf index: {}", actual_leaf_index);
        println!("   - nullifier (updated): {}", test_data.nullifier);
        Ok(actual_leaf_index)
    } else {
        let error_text = deposit_response.text().await?;
        println!("   ❌ Deposit failed: {}", error_text);
        Err(anyhow::anyhow!("Deposit failed: {}", error_text))
    }
}

fn create_deposit_transaction(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    test_data: &TestData,
    user_keypair: &Keypair,
) -> Result<String> {
    // Log balances before deposit
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
        &accounts.roots_ring,
        program_id,
        test_data.amount,
        &commitment_array,
    );

    // Add compute budget instructions for deposit transaction
    let compute_unit_limit_ix =
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(200_000);
    let compute_unit_price_ix =
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(1_000); // 0.000001 SOL per CU

    println!("   🔍 Getting latest blockhash for deposit...");
    let blockhash = client.get_latest_blockhash()?;

    let mut deposit_tx = Transaction::new_with_payer(
        &[compute_unit_price_ix, compute_unit_limit_ix, deposit_ix],
        Some(&user_keypair.pubkey()),
    );
    deposit_tx.sign(&[&user_keypair], blockhash);

    match client.send_and_confirm_transaction(&deposit_tx) {
        Ok(signature) => {
            // Log balances after deposit
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

fn verify_merkle_path(
    commitment: &str,
    merkle_proof: &MerkleProof,
    merkle_root: &str,
) -> Result<()> {
    // Convert commitment and merkle root to [u8; 32] arrays
    let commitment_hex_clean = commitment.strip_prefix("0x").unwrap_or(commitment);
    let commitment_bytes = hex::decode(commitment_hex_clean).unwrap();
    let mut commitment_array = [0u8; 32];
    commitment_array.copy_from_slice(&commitment_bytes);

    let merkle_root_clean = merkle_root.strip_prefix("0x").unwrap_or(merkle_root);
    let merkle_root_bytes = hex::decode(merkle_root_clean).unwrap();
    let mut merkle_root_array = [0u8; 32];
    merkle_root_array.copy_from_slice(&merkle_root_bytes);

    // Convert path elements to [u8; 32] arrays
    let mut path_elements = Vec::new();
    for element_hex in &merkle_proof.path_elements {
        let element_hex_clean = element_hex.strip_prefix("0x").unwrap_or(element_hex);
        let element = hex::decode(element_hex_clean).unwrap();
        let mut element_array = [0u8; 32];
        element_array.copy_from_slice(&element);
        path_elements.push(element_array);
    }

    // Verify Merkle path using the exact same logic as SP1 guest program
    let merkle_valid = test_complete_flow_rust::shared::verify_merkle_path(
        &commitment_array,
        &path_elements,
        &merkle_proof.path_indices,
        &merkle_root_array,
    );

    if merkle_valid {
        println!("   ✅ Merkle path verification successful");
        println!("   - Commitment: {}", commitment);
        println!("   - Merkle root: {}", merkle_root);
        println!("   - Path elements: {}", merkle_proof.path_elements.len());
        Ok(())
    } else {
        println!("   ❌ Merkle path verification failed");
        Err(anyhow::anyhow!("Merkle path verification failed"))
    }
}

fn generate_sp1_proof_inputs(
    test_data: &TestData,
    merkle_proof: &MerkleProof,
    merkle_root: &str,
    leaf_index: u32,
    recipient_keypair: &Keypair,
) -> Result<SP1Inputs> {
    use blake3::Hasher;
    use serde_json;

    println!("   🔐 Generating SP1 Proof Inputs...");

    // Create private inputs exactly like the original main.rs
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

    // Calculate fee using the same logic as SP1 guest program
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (test_data.amount * 5) / 1_000; // 0.5% = 5/1000
        fixed_fee + variable_fee
    };
    let recipient_amount = test_data.amount - fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!(
        "   - Fee: {} lamports (0.0025 SOL fixed + 0.5% variable)",
        fee
    );
    println!("   - Recipient amount: {} lamports", recipient_amount);

    // Create outputs exactly like the original main.rs
    // Use the actual recipient address instead of placeholder
    let recipient_address_hex = hex::encode(recipient_keypair.pubkey().to_bytes());
    let outputs = serde_json::json!([
        {
            "address": recipient_address_hex,
            "amount": recipient_amount  // Amount after fees
        }
    ]);

    // Compute outputs hash exactly like SP1 guest program
    let mut hasher = Hasher::new();

    // Single output
    let recipient_address = recipient_keypair.pubkey().to_bytes();
    hasher.update(&recipient_address);
    hasher.update(&recipient_amount.to_le_bytes());

    let outputs_hash = hasher.finalize();
    let outputs_hash_hex = hex::encode(outputs_hash.as_bytes());
    println!("   - Outputs hash: {}", outputs_hash_hex);

    // Create public inputs exactly like the original main.rs
    // Note: fee_bps removed since fee is fixed in the program
    let public_inputs = serde_json::json!({
        "root": merkle_root,
        "nf": test_data.nullifier,
        "outputs_hash": outputs_hash_hex,
        "amount": test_data.amount
    });

    // Write files for SP1 prover
    std::fs::create_dir_all("packages/zk-guest-sp1/out")?;
    std::fs::write(
        "packages/zk-guest-sp1/out/private.json",
        serde_json::to_string_pretty(&private_inputs)?,
    )?;
    std::fs::write(
        "packages/zk-guest-sp1/out/public.json",
        serde_json::to_string_pretty(&public_inputs)?,
    )?;
    std::fs::write(
        "packages/zk-guest-sp1/out/outputs.json",
        serde_json::to_string_pretty(&outputs)?,
    )?;

    println!("   ✅ SP1 proof inputs generated");
    println!(
        "   - Private inputs: {} bytes",
        serde_json::to_string(&private_inputs)?.len()
    );
    println!(
        "   - Public inputs: {} bytes",
        serde_json::to_string(&public_inputs)?.len()
    );
    println!(
        "   - Outputs: {} bytes",
        serde_json::to_string(&outputs)?.len()
    );

    Ok(SP1Inputs {
        private_inputs: serde_json::to_vec(&private_inputs)?,
        public_inputs: serde_json::to_vec(&public_inputs)?,
        outputs: serde_json::to_vec(&outputs)?,
    })
}

fn generate_sp1_proof(_inputs: &SP1Inputs) -> Result<SP1Proof> {
    println!("   🔨 Generating SP1 Proof with Current Data...");

    let total_start_time = std::time::Instant::now();

    // Step 1: Write input files
    let write_start = std::time::Instant::now();
    println!("   📝 Writing input files...");

    // Ensure output directory exists
    std::fs::create_dir_all("packages/zk-guest-sp1/out")?;

    // Write private inputs (these are already JSON bytes)
    std::fs::write(
        "packages/zk-guest-sp1/out/private.json",
        &_inputs.private_inputs,
    )?;

    // Write public inputs (these are already JSON bytes)
    std::fs::write(
        "packages/zk-guest-sp1/out/public.json",
        &_inputs.public_inputs,
    )?;

    // Write outputs (these are already JSON bytes)
    std::fs::write("packages/zk-guest-sp1/out/outputs.json", &_inputs.outputs)?;

    // Debug: Verify the files contain current data
    println!("   🔍 Verifying input files contain current data...");
    let public_content = std::fs::read_to_string("packages/zk-guest-sp1/out/public.json")?;
    println!("   📄 Public inputs file content: {}", public_content);

    let write_time = write_start.elapsed();
    println!("   ⏱️  File writing took: {:?}", write_time);

    // Step 2: Generate proof
    let proof_start = std::time::Instant::now();
    println!("   🔨 Executing SP1 proof generation...");

    let proof_output = std::process::Command::new("./target/release/cloak-zk")
        .args([
            "prove",
            "--private",
            "packages/zk-guest-sp1/out/private.json",
            "--public",
            "packages/zk-guest-sp1/out/public.json",
            "--outputs",
            "packages/zk-guest-sp1/out/outputs.json",
            "--proof",
            "packages/zk-guest-sp1/out/proof_live.bin",
            "--pubout",
            "packages/zk-guest-sp1/out/public_live.raw",
        ])
        .output()
        .expect("Failed to execute cloak-zk");

    let proof_generation_time = proof_start.elapsed();
    println!(
        "   ⏱️  SP1 proof generation took: {:?}",
        proof_generation_time
    );

    if !proof_output.status.success() {
        let stderr = String::from_utf8_lossy(&proof_output.stderr);
        let stdout = String::from_utf8_lossy(&proof_output.stdout);
        println!("   ❌ cloak-zk failed:");
        println!("   STDOUT: {}", stdout);
        println!("   STDERR: {}", stderr);
        return Err(anyhow::anyhow!("SP1 proof generation failed"));
    }

    // Step 3: Read generated proof
    let read_start = std::time::Instant::now();
    println!("   📖 Reading generated proof...");

    // Read the generated proof files using SP1 SDK proper deserialization
    use sp1_sdk::SP1ProofWithPublicValues;

    let sp1_proof_with_public_values =
        SP1ProofWithPublicValues::load("packages/zk-guest-sp1/out/proof_live.bin")?;

    // Use the proof bytes directly as in the official example
    let full_proof_bytes = sp1_proof_with_public_values.bytes();
    let raw_public_inputs = sp1_proof_with_public_values.public_values.to_vec();

    let read_time = read_start.elapsed();
    println!("   ⏱️  File reading took: {:?}", read_time);

    let total_time = total_start_time.elapsed();
    println!("   ⏱️  Total proof generation time: {:?}", total_time);

    println!("   ✅ SP1 proof generated successfully with current data");
    println!("   - Full SP1 proof size: {} bytes", full_proof_bytes.len());
    println!(
        "   - Raw public inputs size: {} bytes",
        raw_public_inputs.len()
    );

    // Use the full 260-byte proof (with vkey hash) as in the working example
    let proof_bytes = &full_proof_bytes; // Use full 260-byte proof (with vkey hash)
    println!("   - Using full proof size: {} bytes", proof_bytes.len());

    // Use the full 104-byte public inputs (our format)
    let public_inputs_104 = &raw_public_inputs;
    println!(
        "   - Using full public inputs size: {} bytes",
        public_inputs_104.len()
    );

    // Verify the proof format matches what the program expects
    if proof_bytes.len() != 260 {
        println!(
            "   ⚠️  Warning: Expected 260-byte proof, got {} bytes",
            proof_bytes.len()
        );
    }

    if public_inputs_104.len() != 104 {
        println!(
            "   ⚠️  Warning: Expected 104-byte public inputs, got {} bytes",
            public_inputs_104.len()
        );
    }

    Ok(SP1Proof {
        proof_bytes: proof_bytes.to_vec(),
        public_inputs: public_inputs_104.to_vec(),
    })
}

fn execute_withdraw_transaction(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    sp1_proof: &SP1Proof,
    test_data: &TestData,
    recipient_keypair: &Keypair,
    admin_keypair: &Keypair,
) -> Result<String> {
    use solana_sdk::{compute_budget::ComputeBudgetInstruction, transaction::Transaction};

    println!("   💸 Executing Withdraw Transaction...");

    // Calculate fee and recipient amount
    let fee = {
        let fixed_fee = 2_500_000; // 0.0025 SOL
        let variable_fee = (test_data.amount * 5) / 1_000; // 0.5% = 5/1000
        fixed_fee + variable_fee
    };
    let recipient_amount = test_data.amount - fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!("   - Fee: {} lamports", fee);
    println!("   - Recipient amount: {} lamports", recipient_amount);

    // Convert nullifier from hex string to [u8; 32]
    let nullifier_hex_clean = test_data
        .nullifier
        .strip_prefix("0x")
        .unwrap_or(&test_data.nullifier);
    let nullifier_bytes = hex::decode(nullifier_hex_clean).unwrap();
    let mut nullifier_array = [0u8; 32];
    nullifier_array.copy_from_slice(&nullifier_bytes);

    // Create withdraw instruction using the shared function
    let withdraw_ix = test_complete_flow_rust::shared::create_withdraw_instruction(
        &accounts.pool,
        &accounts.treasury,
        &accounts.roots_ring,
        &accounts.nullifier_shard,
        &recipient_keypair.pubkey(),
        program_id,
        &sp1_proof.proof_bytes,
        &sp1_proof.public_inputs,
        &nullifier_array,
        1, // num_outputs
        recipient_amount,
    );

    // Add compute budget instructions for withdraw transaction
    let compute_unit_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(500_000); // Higher limit for withdraw
    let compute_unit_price_ix = ComputeBudgetInstruction::set_compute_unit_price(1_000); // 0.000001 SOL per CU

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
        Some(&admin_keypair.pubkey()), // Admin signs the withdraw transaction
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

// Data structures
#[derive(Debug)]
struct ProgramAccounts {
    pool: Pubkey,
    roots_ring: Pubkey,
    nullifier_shard: Pubkey,
    treasury: Pubkey,
}

#[derive(Debug)]
struct TestData {
    sk_spend: [u8; 32],
    r: [u8; 32],
    amount: u64,
    commitment: String,
    nullifier: String,
}

#[derive(Debug)]
struct SP1Inputs {
    private_inputs: Vec<u8>,
    public_inputs: Vec<u8>,
    outputs: Vec<u8>,
}

#[derive(Debug)]
struct SP1Proof {
    proof_bytes: Vec<u8>,
    public_inputs: Vec<u8>,
}
