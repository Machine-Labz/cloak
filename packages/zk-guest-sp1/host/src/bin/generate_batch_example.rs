use anyhow::Result;
use blake3::Hasher;
use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Parser)]
#[command(name = "generate-batch-example")]
#[command(about = "Generate example batch inputs for testing")]
struct Cli {
    /// Number of withdrawals in the batch
    #[arg(short, long, default_value = "3")]
    count: usize,

    /// Output file for batch inputs
    #[arg(short, long, default_value = "batch_example.json")]
    output: String,
}

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🔨 Generating Batch Example");
    println!("============================\n");
    println!("Generating {} withdrawals...", cli.count);

    let mut rng = rand::thread_rng();
    let mut withdrawals = Vec::new();

    // We'll compute a valid common root after creating all commitments
    let mut commitments = Vec::new();

    for i in 0..cli.count {
        println!("  Withdrawal {}...", i + 1);

        // Generate random secret data
        let mut sk_spend = [0u8; 32];
        let mut r = [0u8; 32];
        rng.fill(&mut sk_spend);
        rng.fill(&mut r);

        let amount: u64 = 1_000_000_000 + rng.gen_range(0..500_000_000); // 1-1.5 SOL
        let leaf_index: u32 = i as u32;

        // Compute pk_spend = H(sk_spend)
        let pk_spend = blake3::hash(&sk_spend);

        // Compute commitment = H(amount || r || pk_spend)
        let commitment = {
            let mut hasher = Hasher::new();
            hasher.update(&amount.to_le_bytes());
            hasher.update(&r);
            hasher.update(pk_spend.as_bytes());
            hasher.finalize()
        };

        // Compute nullifier = H(sk_spend || leaf_index)
        let nullifier = {
            let mut hasher = Hasher::new();
            hasher.update(&sk_spend);
            hasher.update(&leaf_index.to_le_bytes());
            hasher.finalize()
        };

        // Store commitment for later root calculation
        commitments.push(*commitment.as_bytes());

        // We'll fill in the merkle path later after computing the common root
        let merkle_path = MerklePath {
            path_elements: vec![],
            path_indices: vec![],
        };

        // Calculate fee and recipient amount
        let fee = {
            let fixed_fee = 2_500_000; // 0.0025 SOL
            let variable_fee = (amount * 5) / 1_000; // 0.5%
            fixed_fee + variable_fee
        };
        let recipient_amount = amount - fee;

        // Generate random recipient address
        let recipient_address = {
            let mut addr = [0u8; 32];
            rng.fill(&mut addr);
            hex::encode(addr)
        };

        // Create output
        let outputs = vec![Output {
            address: recipient_address.clone(),
            amount: recipient_amount,
        }];

        // Compute outputs hash
        let outputs_hash = {
            let mut hasher = Hasher::new();
            let addr_bytes = hex::decode(&recipient_address)?;
            hasher.update(&addr_bytes);
            hasher.update(&recipient_amount.to_le_bytes());
            hex::encode(hasher.finalize().as_bytes())
        };

        let withdrawal = SingleWithdrawal {
            private: PrivateInputs {
                amount,
                r: hex::encode(r),
                sk_spend: hex::encode(sk_spend),
                leaf_index,
                merkle_path, // Will be filled in below
            },
            public: PublicInputs {
                root: String::new(), // Will be filled in below
                nf: hex::encode(nullifier.as_bytes()),
                outputs_hash,
                amount,
            },
            outputs,
        };

        withdrawals.push(withdrawal);
    }

    // Build a simple merkle tree from all commitments
    println!("\n🌳 Building merkle tree...");

    // Build a simple binary merkle tree
    // We'll track the tree structure to compute paths for each leaf
    let mut tree_levels: Vec<Vec<[u8; 32]>> = vec![commitments.clone()];

    // Build tree level by level, bottom-up
    let mut current_level = commitments.clone();
    while current_level.len() > 1 {
        let mut next_level = Vec::new();

        // Process pairs
        for chunk in current_level.chunks(2) {
            let parent = if chunk.len() == 2 {
                // Both children exist
                blake3::hash(&[&chunk[0][..], &chunk[1][..]].concat())
            } else {
                // Odd one - duplicate it
                blake3::hash(&[&chunk[0][..], &chunk[0][..]].concat())
            };
            next_level.push(*parent.as_bytes());
        }

        tree_levels.push(next_level.clone());
        current_level = next_level;
    }

    let common_root = hex::encode(tree_levels.last().unwrap()[0]);
    println!("  Common root: {}", common_root);

    // Now compute merkle path for each commitment
    for (leaf_idx, withdrawal) in withdrawals.iter_mut().enumerate() {
        let mut path_elements = Vec::new();
        let mut path_indices = Vec::new();
        let mut idx = leaf_idx;

        // Go up the tree level by level
        for level in 0..(tree_levels.len() - 1) {
            let current_level = &tree_levels[level];

            // Find sibling
            let sibling_idx = if idx % 2 == 0 {
                // We're on the left, sibling is on the right
                idx + 1
            } else {
                // We're on the right, sibling is on the left
                idx - 1
            };

            // Get sibling (or duplicate if it doesn't exist)
            let sibling = if sibling_idx < current_level.len() {
                current_level[sibling_idx]
            } else {
                current_level[idx] // Duplicate ourselves if no sibling
            };

            path_elements.push(hex::encode(sibling));
            path_indices.push(if idx % 2 == 0 { 0 } else { 1 });

            // Move to parent index
            idx = idx / 2;
        }

        withdrawal.public.root = common_root.clone();
        withdrawal.private.merkle_path = MerklePath {
            path_elements,
            path_indices,
        };
    }

    let batch = BatchCircuitInputs {
        withdrawals,
        common_root,
    };

    // Write to file
    println!("\n📝 Writing to {}...", cli.output);
    let json = serde_json::to_string_pretty(&batch)?;
    fs::write(&cli.output, json)?;

    println!("✅ Batch example generated successfully!");
    println!("\n📊 Summary:");
    println!("  - Withdrawals: {}", cli.count);
    println!("  - Common root: {}", batch.common_root);
    println!("  - Output file: {}", cli.output);

    println!("\n💡 Next steps:");
    println!("  1. Review the generated file: {}", cli.output);
    println!("  2. Generate proof with:");
    println!("     cargo run --release --bin batch-prove -- \\");
    println!("       --batch {} \\", cli.output);
    println!("       --proof batch_proof.bin \\");
    println!("       --pubout batch_public.raw");

    Ok(())
}
