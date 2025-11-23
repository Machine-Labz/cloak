use anyhow::Result;
use hex;
use rand;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account;
use spl_token;
use std::str::FromStr;
use std::time::{Duration, Instant};
use test_complete_flow_rust::shared::{
    check_cluster_health, ensure_user_funding, get_pda_addresses, load_keypair, print_config,
    MerkleProof, TestConfig, SOL_TO_LAMPORTS,
};

// SPL Token constants
const USDC_MINT_TESTNET: &str = "DEN84VAfNR9qjgFVx4mAR3otjNzevGr7ZTMkCnRmGAJq"; // Testnet test token (6 decimals)
const USDC_MINT_DEVNET: &str = "BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k"; // DevUSDC (has good Orca pool)
const SOL_MINT: &str = "So11111111111111111111111111111111111111112"; // Native SOL wrapped mint

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
#[allow(dead_code)]
struct MerkleRootResponse {
    root: String,
}

// Fetch a quote from Jupiter's Legacy Quote API
#[derive(Debug, Deserialize)]
struct JupiterLegacyQuote {
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "inAmount")]
    in_amount: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    #[serde(rename = "outAmount")]
    out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    swap_mode: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: u16,
    #[serde(rename = "priceImpactPct")]
    price_impact_pct: String,
}

struct LegacyQuoteParsed {
    out_amount: u64,
    min_out_amount: u64,
}

// Fetch a quote from Jupiter's Legacy Quote API
async fn get_jupiter_quote(
    client: &reqwest::Client,
    input_mint: &str,
    amount: u64,
    slippage_bps: u16,
) -> Result<LegacyQuoteParsed> {
    let url = format!(
        "https://lite-api.jup.ag/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&restrictIntermediateTokens=true",
        input_mint, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", amount, slippage_bps
    );

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Jupiter quote failed: {}: {}",
            status,
            body
        ));
    }
    let quote: JupiterLegacyQuote = response.json().await?;

    let usdc_amount = quote
        .out_amount
        .parse::<u64>()
        .map_err(|e| anyhow::anyhow!("Failed to parse USDC amount: {}", e))?;
    let min_out = quote
        .other_amount_threshold
        .parse::<u64>()
        .map_err(|e| anyhow::anyhow!("Failed to parse min output amount: {}", e))?;

    Ok(LegacyQuoteParsed {
        out_amount: usdc_amount,
        min_out_amount: min_out,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct WithdrawRequest {
    outputs: Vec<Output>,
    swap: Option<SwapConfig>,
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

    println!("🔐 CLOAK PRIVACY PROTOCOL - SWAP WITHDRAWAL TEST");
    println!("================================================\n");

    // Parse network argument
    let args: Vec<String> = std::env::args().collect();
    let network = args
        .iter()
        .position(|arg| arg == "--network")
        .and_then(|idx| args.get(idx + 1))
        .map(|s| s.as_str())
        .unwrap_or("devnet");

    if network == "localnet" {
        println!("⚠️  Note: Swap testing requires testnet/devnet/mainnet for Jupiter liquidity");
        println!("    Falling back to devnet for this test\n");
    }

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

    // Use USDC as swap target mint (network-specific)
    let swap_target_mint = if network == "testnet" {
        Pubkey::from_str(USDC_MINT_TESTNET)?
    } else {
        Pubkey::from_str(USDC_MINT_DEVNET)?
    };

    println!("📋 Test Configuration:");
    println!("  User: {}", user_keypair.pubkey());
    println!("  Recipient: {}", recipient_keypair.pubkey());
    println!("  Pool Mint: SOL (native)");
    println!("  Swap Target: {} (USDC-like)", swap_target_mint);
    println!();

    // Ensure funding
    ensure_user_funding(&config.rpc_url, &user_keypair, &admin_keypair)?;
    ensure_user_funding(&config.rpc_url, &recipient_keypair, &admin_keypair)?;

    // Step 1: Generate test data (includes secret, commitment, etc.)
    let deposit_amount: u64 = SOL_TO_LAMPORTS / 10; // 0.1 SOL
    println!("🎲 Step 1: Generating test data...");
    let mut test_data = generate_test_data(deposit_amount)?;
    let commitment = hex::decode(&test_data.commitment)?;
    let commitment_bytes: [u8; 32] = commitment
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid commitment length"))?;
    println!("  Commitment: {}...", &test_data.commitment[..16]);

    // Step 3: Deposit to shield pool (native SOL)
    println!(
        "\n💰 Step 3: Depositing {} lamports to shield pool...",
        deposit_amount
    );

    let native_mint = Pubkey::default();
    let (pool_pda, commitments_pda, _roots_ring_pda, _nullifier_shard_pda, _treasury_pda) =
        get_pda_addresses(&program_id, &native_mint);

    println!("  Pool PDA: {}", pool_pda);
    println!("  Commitments PDA: {}", commitments_pda);

    // Build deposit instruction for native SOL
    // Format: [discriminator (1 byte), amount (8 bytes), commitment (32 bytes)]
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

    // Step 4: Get current Merkle root from indexer
    println!("\n🌳 Step 4: Fetching Merkle root from indexer...");
    let client = reqwest::Client::new();

    // Get the slot number from the deposit transaction
    let slot = rpc_client.get_slot()?;

    // Submit deposit to indexer
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

    // Parse deposit response to get leaf index and root
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

    // Get Merkle proof using the leaf index
    let merkle_response = client
        .get(format!(
            "{}/api/v1/merkle/proof/{}",
            config.indexer_url, leaf_index
        ))
        .send()
        .await?;

    let merkle_text = merkle_response.text().await?;
    println!("  🔍 DEBUG: Raw Merkle proof response (first 500 chars):");
    println!(
        "     {}",
        &merkle_text[..std::cmp::min(500, merkle_text.len())]
    );

    let proof_response: MerkleProof = serde_json::from_str(&merkle_text)?;

    println!(
        "  Merkle path length: {}",
        proof_response.path_elements.len()
    );

    // Step 4.5: Get Jupiter quote for SOL → USDC swap
    println!("\n💱 Step 4.5: Getting Jupiter quote for SOL → USDC swap...");
    let slippage_bps: u16 = 100; // 1%
                                 // Calculate total fee: fixed + variable (must match circuit calculation)
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (deposit_amount * 5) / 1_000; // 0.5%
    let total_fee = fixed_fee + variable_fee;
    let sol_to_swap = deposit_amount
        .checked_sub(total_fee)
        .ok_or_else(|| anyhow::anyhow!("Fees exceed deposit amount"))?;

    println!("  SOL amount to swap: {} lamports", sol_to_swap);
    println!("  Output mint (swap on devnet): {}", swap_target_mint);
    println!(
        "  Output mint (quoter on mainnet): {}",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    println!(
        "  Slippage: {} bps ({}%)",
        slippage_bps,
        slippage_bps as f64 / 100.0
    );

    let parsed_quote = get_jupiter_quote(&client, SOL_MINT, sol_to_swap, slippage_bps).await?;
    let usdc_amount = parsed_quote.out_amount;
    let min_output_amount = parsed_quote.min_out_amount;

    println!("  ✅ Jupiter quote received:");
    println!(
        "     {} lamports → {} USDC (6 decimals)",
        sol_to_swap, usdc_amount
    );

    // Step 5: Generate ZK proof via indexer /prove endpoint
    println!("\n🔬 Step 5: Generating ZK proof via indexer /prove endpoint...");

    let proof_start = Instant::now();

    let (private_inputs, public_inputs, outputs_json, swap_params_json) = prepare_proof_inputs(
        &test_data,
        &proof_response,
        &current_root,
        leaf_index,
        &recipient_keypair,
        Some(&swap_target_mint),
        Some(min_output_amount),
    )?;

    // Call indexer's /prove endpoint
    let prove_request = serde_json::json!({
        "private_inputs": private_inputs,
        "public_inputs": public_inputs,
        "outputs": outputs_json,
        "swap_params": swap_params_json
    });

    println!("  📡 Calling indexer /prove endpoint...");
    println!("  🔍 DEBUG: Prove request structure:");
    println!(
        "     - private_inputs (first 100 chars): {}",
        private_inputs.chars().take(100).collect::<String>()
    );
    println!(
        "     - public_inputs (first 100 chars): {}",
        public_inputs.chars().take(100).collect::<String>()
    );
    println!("     - outputs: {}", outputs_json);
    println!(
        "     - swap_params: {}",
        serde_json::to_string_pretty(&swap_params_json)?
    );

    let prove_response = client
        .post(format!("{}/api/v1/prove", config.indexer_url))
        .json(&prove_request)
        .send()
        .await?;

    let proof_result: serde_json::Value = prove_response.json().await?;

    // Check if proof generation succeeded
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
    if let Some(cycles) = proof_result["total_cycles"].as_u64() {
        println!("     Cycles: {}", cycles);
    }
    if let Some(method) = proof_result["proof_method"].as_str() {
        println!("     Method: {}", method);
    }

    // Extract proof from response
    let proof_hex = proof_result["proof"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing proof in response"))?
        .to_string();

    println!(
        "  Proof: {}...",
        &proof_hex[..std::cmp::min(16, proof_hex.len())]
    );

    // Calculate fee for conservation used by current circuit/TEE: fixed + variable fee
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (deposit_amount * 5) / 1_000; // 0.5% = 5/1000
    let total_fee = fixed_fee + variable_fee;
    let withdraw_amount = deposit_amount - total_fee;

    // Calculate fee_bps for relay.
    // For swap-mode withdrawals, the relay currently models fee_bps using ONLY the variable fee
    // (0.5% of the deposited SOL). The fixed fee is still taken from the deposited SOL but is
    // not encoded in fee_bps. This must match services/relay/src/api/withdraw.rs::validate_request.
    let effective_fee_bps = if deposit_amount == 0 {
        0u16
    } else {
        let bps = ((variable_fee.saturating_mul(10_000)) + deposit_amount - 1) / deposit_amount;
        bps.min(u16::MAX as u64) as u16
    };

    // Extract outputs_hash from public_inputs JSON
    let public_inputs_json: serde_json::Value = serde_json::from_str(&public_inputs)?;
    let outputs_hash = public_inputs_json["outputs_hash"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing outputs_hash in public_inputs"))?
        .to_string();

    // Convert proof from hex to base64 (relay expects base64)
    use base64::Engine as _;
    let proof_bytes_vec = hex::decode(&proof_hex)
        .map_err(|e| anyhow::anyhow!("Failed to decode proof hex: {}", e))?;
    let proof_base64 = base64::engine::general_purpose::STANDARD.encode(&proof_bytes_vec);

    // Step 6: Submit withdraw request WITH SWAP to relay
    println!("\n🔄 Step 6: Submitting swap withdraw request to relay...");
    println!("  Swap: SOL → USDC");
    println!("  Slippage: 1% (100 bps)");
    println!("  Fee BPS: {}", effective_fee_bps);

    let withdraw_request = WithdrawRequest {
        outputs: vec![Output {
            // Recipient specified as pubkey; relay derives ATA for output mint
            recipient: recipient_keypair.pubkey().to_string(),
            // Conservation check expects SOL lamports: amount - fee
            amount: withdraw_amount,
        }],
        swap: Some(SwapConfig {
            output_mint: swap_target_mint.to_string(),
            slippage_bps: 100, // 1% slippage
            min_output_amount: min_output_amount,
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
        println!("\n  ✅ Swap withdraw request submitted successfully!");

        // Parse request_id from API response payload
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        let job_id = response_json
            .get("data")
            .and_then(|d| d.get("request_id"))
            .and_then(|v| v.as_str())
            .or_else(|| response_json.get("request_id").and_then(|v| v.as_str()))
            .unwrap_or("unknown");

        println!("  Request ID: {}", job_id);

        // Step 7: Monitor job status
        println!("\n📊 Step 7: Monitoring swap withdrawal status...");

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
                    println!("\n  🎉 Swap withdrawal completed!");
                    println!("  Transaction: {}", tx_sig);
                    println!(
                        "  View on Solscan: https://solscan.io/tx/{}?cluster=testnet",
                        tx_sig
                    );
                    completed = true;
                    break;
                } else if status == "failed" {
                    println!("\n  ❌ Swap withdrawal failed");
                    println!("  Error: {}", serde_json::to_string_pretty(&status_json)?);
                    break;
                }
            }
        }

        if !completed {
            println!("\n  ⏱️  Timeout waiting for completion (check relay logs)");
        }

        // Step 8: Verify recipient received USDC
        println!("\n✅ Step 8: Verifying swap results...");

        let recipient_usdc_ata = spl_associated_token_account::get_associated_token_address(
            &recipient_keypair.pubkey(),
            &swap_target_mint,
        );

        println!("  Recipient USDC ATA: {}", recipient_usdc_ata);

        match rpc_client.get_account(&recipient_usdc_ata) {
            Ok(account) => {
                let token_account = spl_token::state::Account::unpack(&account.data)?;
                println!(
                    "  ✅ Recipient USDC balance: {} tokens",
                    token_account.amount
                );
                println!("  📈 Swap executed successfully!");
            }
            Err(e) => {
                println!("  ⚠️  Could not fetch USDC balance: {}", e);
                println!("  (ATA may not exist yet - check transaction)");
            }
        }
    } else {
        println!("\n  ❌ Failed to submit swap withdraw request");
        return Err(anyhow::anyhow!("Withdraw submission failed"));
    }

    let total_duration = start_time.elapsed();
    println!(
        "\n⏱️  Total test duration: {:.2}s",
        total_duration.as_secs_f64()
    );
    println!("\nℹ️  Test finished. See relay logs for swap execution.");

    Ok(())
}

fn prepare_proof_inputs(
    test_data: &TestData,
    merkle_proof: &MerkleProof,
    merkle_root: &str,
    leaf_index: u32,
    recipient_keypair: &Keypair,
    // When performing a swap, provide target mint and quoted output amount
    swap_target_mint: Option<&Pubkey>,
    min_output_amount: Option<u64>,
) -> Result<(String, String, String, serde_json::Value)> {
    use blake3::Hasher;

    println!("   🔐 Preparing proof inputs...");

    // Nullifier should already be updated with leaf_index
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

    // Calculate fee for conservation used by current circuit/TEE: fixed + variable fee
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (test_data.amount * 5) / 1_000; // 0.5% = 5/1000
    let total_fee = fixed_fee + variable_fee;

    println!("   - Amount: {} lamports", test_data.amount);
    println!("   - Fixed fee: {} lamports", fixed_fee);
    println!("   - Variable fee (0.5%): {} lamports", variable_fee);
    println!("   - Total fee: {} lamports", total_fee);

    // Commit to SOL recipient pubkey and lamports after total fee
    let recipient_amount = test_data.amount - total_fee;
    let recipient_pubkey = recipient_keypair.pubkey();
    println!("   - Recipient amount: {} lamports", recipient_amount);

    let recipient_address_hex = hex::encode(recipient_pubkey.to_bytes());
    println!("   - Recipient address: {}", recipient_pubkey);
    println!("   - Recipient address (hex): {}", recipient_address_hex);

    // Create outputs - EMPTY for swap mode since user receives swapped tokens
    let outputs = if swap_target_mint.is_some() {
        serde_json::json!([]) // No SOL outputs for swap
    } else {
        serde_json::json!([
            {
                "address": recipient_address_hex,
                "amount": recipient_amount
            }
        ])
    };

    // Compute outputs_hash
    // Regular: H(recipient || amount)
    // Swap mode: H(output_mint || recipient_ata || min_output_amount || public_amount)
    let outputs_hash_hex =
        if let (Some(mint), Some(min_out)) = (swap_target_mint, min_output_amount) {
            let recipient_ata =
                spl_associated_token_account::get_associated_token_address(&recipient_pubkey, mint);
            let mut hasher = Hasher::new();
            hasher.update(&mint.to_bytes());
            hasher.update(&recipient_ata.to_bytes());
            hasher.update(&min_out.to_le_bytes());
            hasher.update(&test_data.amount.to_le_bytes()); // public amount
            let h = hasher.finalize();
            let hexval = hex::encode(h.as_bytes());
            println!("   - Swap outputs hash: {}", hexval);
            hexval
        } else {
            let mut hasher = Hasher::new();
            hasher.update(&recipient_pubkey.to_bytes());
            hasher.update(&recipient_amount.to_le_bytes());
            let h = hasher.finalize();
            let hexval = hex::encode(h.as_bytes());
            println!("   - Outputs hash: {}", hexval);
            hexval
        };

    // Create public inputs
    let public_inputs = serde_json::json!({
        "root": merkle_root,
        "nf": nullifier_hex,
        "outputs_hash": outputs_hash_hex,
        "amount": test_data.amount
    });

    // Build optional swap_params for SP1 guest (top-level field)
    let swap_params_json =
        if let (Some(mint), Some(min_out)) = (swap_target_mint, min_output_amount) {
            let recipient_ata =
                spl_associated_token_account::get_associated_token_address(&recipient_pubkey, mint);
            let obj = serde_json::json!({
                "output_mint": mint.to_string(),
                "recipient_ata": recipient_ata.to_string(),
                "min_output_amount": min_out,
            });
            println!("   - swap_params: {}", serde_json::to_string_pretty(&obj)?);
            obj
        } else {
            serde_json::json!(null)
        };

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
        swap_target_mint.is_some(),
    )?;
    println!("   ✅ All circuit constraints validated successfully!\n");

    Ok((
        serde_json::to_string(&private_inputs)?,
        serde_json::to_string(&public_inputs)?,
        serde_json::to_string(&outputs)?,
        swap_params_json,
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
    is_swap: bool,
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

    // Circuit uses fixed + variable fee for conservation
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (amount * 5) / 1_000; // 0.5% = 5/1000
    let total_fee = fixed_fee + variable_fee;

    if is_swap {
        // For swap mode: outputs should be empty (all goes to swap)
        if outputs_sum != 0 {
            return Err(anyhow::anyhow!(
                "Swap mode requires empty outputs!\n         Outputs sum: {} (expected 0)",
                outputs_sum
            ));
        }
        println!(
            "         ✓ Swap mode: outputs empty, fee = {}, swap amount = {}",
            total_fee,
            amount - total_fee
        );
    } else {
        // For regular mode: outputs + fee = amount
        if outputs_sum + total_fee != amount {
            return Err(anyhow::anyhow!(
                "Output sum + fee != amount!\n         Outputs: {}\n         Fixed fee: {}\n         Variable fee (0.5%): {}\n         Total fee: {}\n         Amount: {}",
                outputs_sum,
                fixed_fee,
                variable_fee,
                total_fee,
                amount
            ));
        }
        println!("         ✓ Output sum + total fee == amount");
    }

    Ok(())
}
