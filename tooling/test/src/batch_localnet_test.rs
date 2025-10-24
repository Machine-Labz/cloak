use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use hex;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use shield_pool::instructions::ShieldPoolInstruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use sp1_sdk::SP1ProofWithPublicValues;
use test_complete_flow_rust::{
    helpers::*,
    shared::{check_cluster_health, load_keypair},
};

#[derive(Debug, Serialize, Deserialize)]
struct BatchCircuitInputs {
    withdrawals: Vec<SingleWithdrawal>,
    common_root: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingleWithdrawal {
    private: PrivateInputs,
    public: PublicInputs,
    outputs: Vec<Output>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivateInputs {
    amount: u64,
    r: String,
    sk_spend: String,
    leaf_index: u32,
    merkle_path: MerklePath,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublicInputs {
    root: String,
    nf: String,
    outputs_hash: String,
    amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    address: String,
    amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MerklePath {
    path_elements: Vec<String>,
    path_indices: Vec<u8>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🚀 CLOAK BATCH WITHDRAWAL - LOCALNET E2E TEST");
    println!("==============================================\n");

    let rpc_url = "http://127.0.0.1:8899";
    let indexer_url = "http://localhost:3001";

    check_cluster_health(rpc_url)?;

    let admin_keypair = load_keypair("admin-keypair.json")?;
    let client = RpcClient::new(rpc_url);

    println!("\n📋 Step 1: Deploying Program...");
    let program_id = deploy_program(&client, rpc_url)?;

    println!("\n📋 Step 2: Creating Accounts...");
    let accounts = create_program_accounts(&client, &program_id, &admin_keypair)?;

    println!("\n🔄 Step 3: Resetting Indexer...");
    reset_indexer_database(indexer_url).await?;

    println!("\n💰 Step 4: Generating & Depositing 3 Withdrawals...");
    let batch_data = generate_and_deposit_batch(
        &client,
        &program_id,
        &accounts,
        &admin_keypair,
        indexer_url,
        3,
    )
    .await?;

    println!("\n🌳 Step 5: Getting Merkle Root from Indexer...");
    let merkle_root = get_merkle_root(indexer_url).await?;
    push_root_to_program(
        &client,
        &program_id,
        &accounts.roots_ring,
        &merkle_root,
        &admin_keypair,
    )?;

    println!("\n🔍 Step 6: Getting Merkle Proofs from Indexer...");
    let merkle_proofs = get_merkle_proofs(indexer_url, &batch_data.leaf_indices).await?;

    println!("\n🔐 Step 7: Generating Batch Proof (~2 min)...");
    let proof = generate_batch_proof(&batch_data, &merkle_proofs, &merkle_root)?;

    println!("\n💸 Step 8: Executing BatchWithdraw...");
    execute_batch_withdraw(
        &client,
        &program_id,
        &accounts,
        &proof,
        &batch_data,
        &admin_keypair,
    )?;

    println!("\n🎉 BATCH WITHDRAWAL E2E TEST SUCCESS!");
    println!("======================================");
    println!("✅ 3 deposits via indexer");
    println!("✅ Merkle tree built by indexer");
    println!("✅ 3 withdrawals in single transaction");
    println!("✅ Single batch proof verified on-chain");

    Ok(())
}

struct BatchTestData {
    withdrawals: Vec<WithdrawalData>,
    leaf_indices: Vec<u32>,
}

struct WithdrawalData {
    sk_spend: [u8; 32],
    r: [u8; 32],
    amount: u64,
    recipient: Pubkey,
}

async fn generate_and_deposit_batch(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    payer: &Keypair,
    indexer_url: &str,
    count: usize,
) -> Result<BatchTestData> {
    let mut rng = rand::thread_rng();
    let mut withdrawals = Vec::new();
    let mut leaf_indices = Vec::new();
    let http_client = reqwest::Client::new();

    for i in 0..count {
        let mut sk_spend = [0u8; 32];
        let mut r = [0u8; 32];
        rng.fill_bytes(&mut sk_spend);
        rng.fill_bytes(&mut r);

        let amount = 1_000_000_000 + (i as u64 * 100_000_000);
        let recipient = Keypair::new().pubkey();
        let commitment = compute_commitment(amount, &r, &sk_spend);
        let commitment_hex = hex::encode(commitment);

        // Deposit to indexer
        let deposit_req = DepositRequest {
            leaf_commit: commitment_hex.clone(),
            encrypted_output: general_purpose::STANDARD
                .encode(format!("Batch withdrawal {}", i + 1)),
            tx_signature: format!(
                "batch_{}_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis(),
                i
            ),
            slot: 1000 + i as u64,
        };

        let resp = http_client
            .post(&format!("{}/api/v1/deposit", indexer_url))
            .json(&deposit_req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            anyhow::bail!("Indexer deposit failed: {}", error_text);
        }

        let deposit_data: serde_json::Value = resp.json().await?;
        let actual_leaf_index = deposit_data["leafIndex"].as_u64().unwrap() as u32;
        leaf_indices.push(actual_leaf_index);

        // On-chain deposit
        let deposit_ix = create_deposit_instruction(
            program_id,
            &payer.pubkey(),
            &accounts.pool,
            amount,
            &commitment,
        );
        let tx = Transaction::new_signed_with_payer(
            &[deposit_ix],
            Some(&payer.pubkey()),
            &[payer],
            client.get_latest_blockhash()?,
        );
        client.send_and_confirm_transaction(&tx)?;

        println!(
            "   ✅ #{}: {} SOL (leaf {})",
            i + 1,
            amount / SOL_TO_LAMPORTS,
            actual_leaf_index
        );

        withdrawals.push(WithdrawalData {
            sk_spend,
            r,
            amount,
            recipient,
        });
    }

    Ok(BatchTestData {
        withdrawals,
        leaf_indices,
    })
}

struct BatchProofData {
    proof_bytes: Vec<u8>,
    public_inputs: Vec<u8>,
}

fn generate_batch_proof(
    batch_data: &BatchTestData,
    merkle_proofs: &[MerkleProof],
    merkle_root: &str,
) -> Result<BatchProofData> {
    let batch_withdrawals: Vec<SingleWithdrawal> = batch_data
        .withdrawals
        .iter()
        .enumerate()
        .map(|(i, wd)| {
            let actual_leaf_index = batch_data.leaf_indices[i];
            let fee = calculate_fee(wd.amount);
            let recipient_amount = wd.amount - fee;
            let nullifier = hex::encode(compute_nullifier(&wd.sk_spend, actual_leaf_index));
            let outputs_hash = hex::encode(compute_outputs_hash(&wd.recipient, recipient_amount));

            SingleWithdrawal {
                private: PrivateInputs {
                    amount: wd.amount,
                    r: hex::encode(wd.r),
                    sk_spend: hex::encode(wd.sk_spend),
                    leaf_index: actual_leaf_index,
                    merkle_path: MerklePath {
                        path_elements: merkle_proofs[i].path_elements.clone(),
                        path_indices: merkle_proofs[i].path_indices.clone(),
                    },
                },
                public: PublicInputs {
                    root: merkle_root.to_string(),
                    nf: nullifier,
                    outputs_hash,
                    amount: wd.amount,
                },
                outputs: vec![Output {
                    address: hex::encode(wd.recipient.to_bytes()),
                    amount: recipient_amount,
                }],
            }
        })
        .collect();

    let batch = BatchCircuitInputs {
        withdrawals: batch_withdrawals,
        common_root: merkle_root.to_string(),
    };
    std::fs::write("batch_test.json", serde_json::to_string_pretty(&batch)?)?;

    let start = std::time::Instant::now();
    let output = std::process::Command::new("./target/release/batch-prove")
        .args([
            "--batch",
            "batch_test.json",
            "--proof",
            "batch_test.bin",
            "--pubout",
            "batch_test_pub.raw",
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Proof failed:\n{}", String::from_utf8_lossy(&output.stderr));
    }

    let proof = SP1ProofWithPublicValues::load("batch_test.bin")?;
    println!("   ✅ Done in {:.0}s", start.elapsed().as_secs_f64());

    Ok(BatchProofData {
        proof_bytes: proof.bytes().to_vec(),
        public_inputs: proof.public_values.to_vec(),
    })
}

fn execute_batch_withdraw(
    client: &RpcClient,
    program_id: &Pubkey,
    accounts: &ProgramAccounts,
    proof: &BatchProofData,
    batch_data: &BatchTestData,
    admin: &Keypair,
) -> Result<()> {
    let mut data = vec![ShieldPoolInstruction::BatchWithdraw as u8];
    data.extend_from_slice(&proof.proof_bytes);
    data.extend_from_slice(&proof.public_inputs);
    data.push(batch_data.withdrawals.len() as u8);

    for wd in &batch_data.withdrawals {
        data.push(1u8);
        data.extend_from_slice(&wd.recipient.to_bytes());
        data.extend_from_slice(&(wd.amount - calculate_fee(wd.amount)).to_le_bytes());
    }

    let mut acc_metas = vec![
        AccountMeta::new(accounts.pool, false),
        AccountMeta::new(accounts.treasury, false),
        AccountMeta::new(accounts.roots_ring, false),
        AccountMeta::new(accounts.nullifier_shard, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];
    for wd in &batch_data.withdrawals {
        acc_metas.push(AccountMeta::new(wd.recipient, false));
    }

    let batch_ix = Instruction {
        program_id: *program_id,
        accounts: acc_metas,
        data,
    };

    use solana_sdk::compute_budget::ComputeBudgetInstruction;
    let tx = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_price(1_000),
            ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
            batch_ix,
        ],
        Some(&admin.pubkey()),
        &[admin],
        client.get_latest_blockhash()?,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("   ✅ Signature: {}", sig);
    Ok(())
}
