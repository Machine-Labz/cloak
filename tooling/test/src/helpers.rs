use anyhow::Result;
use blake3::Hasher;
use hex;
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

pub const SOL_TO_LAMPORTS: u64 = 1_000_000_000;

#[derive(Debug)]
pub struct ProgramAccounts {
    pub pool: Pubkey,
    pub roots_ring: Pubkey,
    pub nullifier_shard: Pubkey,
    pub treasury: Pubkey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    #[serde(rename = "pathElements")]
    pub path_elements: Vec<String>,
    #[serde(rename = "pathIndices")]
    pub path_indices: Vec<u8>,
}

#[derive(Serialize)]
pub struct DepositRequest {
    pub leaf_commit: String,
    pub encrypted_output: String,
    pub tx_signature: String,
    pub slot: u64,
}

#[derive(Deserialize)]
pub struct MerkleRootResponse {
    pub root: String,
}

/// Deploy program and return its ID
pub fn deploy_program(_client: &RpcClient, url: &str) -> Result<Pubkey> {
    println!("   Building...");
    let build = std::process::Command::new("cargo")
        .args([
            "build-sbf",
            "--manifest-path",
            "programs/shield-pool/Cargo.toml",
        ])
        .output()?;

    if !build.status.success() {
        anyhow::bail!("Build failed: {}", String::from_utf8_lossy(&build.stderr));
    }

    println!("   Deploying...");
    let deploy = std::process::Command::new("solana")
        .args([
            "program",
            "deploy",
            "--url",
            url,
            "--keypair",
            "admin-keypair.json",
            "--program-id",
            "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json",
            "target/deploy/shield_pool.so",
        ])
        .output()?;

    if !deploy.status.success() {
        anyhow::bail!("Deploy failed: {}", String::from_utf8_lossy(&deploy.stderr));
    }

    let stdout = String::from_utf8_lossy(&deploy.stdout);
    let program_id_str = stdout
        .lines()
        .find(|l| l.contains("Program Id:"))
        .and_then(|l| l.split_whitespace().nth(2))
        .ok_or_else(|| anyhow::anyhow!("Failed to parse program ID"))?;

    let program_id = program_id_str.parse::<Pubkey>()?;

    println!("   ✅ Deployed: {}", program_id);
    Ok(program_id)
}

/// Create all program accounts (pool, roots_ring, nullifier_shard, treasury)
pub fn create_program_accounts(
    client: &RpcClient,
    program_id: &Pubkey,
    admin: &Keypair,
) -> Result<ProgramAccounts> {
    use solana_sdk::system_instruction;

    let pool = Keypair::new();
    let roots_ring = Keypair::new();
    let nullifier_shard = Keypair::new();
    let treasury = Keypair::new();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &admin.pubkey(),
                &pool.pubkey(),
                client.get_minimum_balance_for_rent_exemption(0)?,
                0,
                program_id,
            ),
            system_instruction::create_account(
                &admin.pubkey(),
                &roots_ring.pubkey(),
                client.get_minimum_balance_for_rent_exemption(2056)?,
                2056,
                program_id,
            ),
            system_instruction::create_account(
                &admin.pubkey(),
                &nullifier_shard.pubkey(),
                client.get_minimum_balance_for_rent_exemption(4)?,
                4,
                program_id,
            ),
            system_instruction::create_account(
                &admin.pubkey(),
                &treasury.pubkey(),
                0,
                0,
                &solana_sdk::system_program::id(),
            ),
        ],
        Some(&admin.pubkey()),
        &[admin, &pool, &roots_ring, &nullifier_shard, &treasury],
        client.get_latest_blockhash()?,
    );

    client.send_and_confirm_transaction(&tx)?;
    println!("   ✅ Created: pool, roots_ring, nullifier_shard, treasury");

    Ok(ProgramAccounts {
        pool: pool.pubkey(),
        roots_ring: roots_ring.pubkey(),
        nullifier_shard: nullifier_shard.pubkey(),
        treasury: treasury.pubkey(),
    })
}

/// Create deposit instruction
pub fn create_deposit_instruction(
    program_id: &Pubkey,
    user: &Pubkey,
    pool: &Pubkey,
    amount: u64,
    commitment: &[u8; 32],
) -> Instruction {
    let mut data = vec![ShieldPoolInstruction::Deposit as u8];
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(commitment);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*pool, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data,
    }
}

/// Push root to program
pub fn push_root_to_program(
    client: &RpcClient,
    program_id: &Pubkey,
    roots_ring: &Pubkey,
    root: &str,
    admin: &Keypair,
) -> Result<()> {
    let root_bytes: [u8; 32] = hex::decode(root)?.try_into().unwrap();
    let mut data = vec![1u8]; // AdminPushRoot
    data.extend_from_slice(&root_bytes);

    let ix = Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(admin.pubkey(), true),
            AccountMeta::new(*roots_ring, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin.pubkey()),
        &[admin],
        client.get_latest_blockhash()?,
    );
    client.send_and_confirm_transaction(&tx)?;
    println!("   ✅ Root pushed");
    Ok(())
}

/// Reset indexer database
pub async fn reset_indexer_database(url: &str) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(&format!("{}/api/v1/admin/reset", url))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Reset failed: {}", resp.text().await?);
    }

    println!("   ✅ Indexer reset");
    Ok(())
}

/// Get merkle root from indexer
pub async fn get_merkle_root(url: &str) -> Result<String> {
    let resp: MerkleRootResponse = reqwest::Client::new()
        .get(&format!("{}/api/v1/merkle/root", url))
        .send()
        .await?
        .json()
        .await?;

    println!("   ✅ Root: {}", resp.root);
    Ok(resp.root)
}

/// Get merkle proof from indexer for a specific leaf
pub async fn get_merkle_proof(url: &str, leaf_index: u32) -> Result<MerkleProof> {
    let proof: MerkleProof = reqwest::Client::new()
        .get(&format!("{}/api/v1/merkle/proof/{}", url, leaf_index))
        .send()
        .await?
        .json()
        .await?;

    println!("   ✅ Got merkle proof for leaf {}", leaf_index);
    Ok(proof)
}

/// Get multiple merkle proofs from indexer
pub async fn get_merkle_proofs(url: &str, leaf_indices: &[u32]) -> Result<Vec<MerkleProof>> {
    let mut proofs = Vec::new();

    for &idx in leaf_indices {
        let proof = get_merkle_proof(url, idx).await?;
        proofs.push(proof);
    }

    println!("   ✅ Got {} merkle proofs total", proofs.len());
    Ok(proofs)
}

/// Compute commitment: H(amount || r || pk_spend)
pub fn compute_commitment(amount: u64, r: &[u8; 32], sk_spend: &[u8; 32]) -> [u8; 32] {
    let pk_spend = blake3::hash(sk_spend);
    let mut h = Hasher::new();
    h.update(&amount.to_le_bytes());
    h.update(r);
    h.update(pk_spend.as_bytes());
    *h.finalize().as_bytes()
}

/// Compute nullifier: H(sk_spend || leaf_index)
pub fn compute_nullifier(sk_spend: &[u8; 32], leaf_index: u32) -> [u8; 32] {
    *blake3::hash(&[&sk_spend[..], &leaf_index.to_le_bytes()[..]].concat()).as_bytes()
}

/// Calculate fee for an amount
pub fn calculate_fee(amount: u64) -> u64 {
    2_500_000 + (amount * 5) / 1_000 // 0.0025 SOL + 0.5%
}

/// Compute outputs hash: H(recipient || amount)
pub fn compute_outputs_hash(recipient: &Pubkey, amount: u64) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(&recipient.to_bytes());
    h.update(&amount.to_le_bytes());
    *h.finalize().as_bytes()
}
