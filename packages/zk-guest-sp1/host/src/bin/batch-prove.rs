use anyhow::Result;
use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1ProofWithPublicValues, SP1Stdin};
use std::fs;

const ELF: &[u8] = include_elf!("zk-guest-sp1-guest");

#[derive(Parser)]
#[command(name = "batch-prove")]
#[command(about = "Generate batch ZK proof for multiple withdrawals")]
struct Cli {
    /// Path to batch inputs JSON file
    #[arg(short, long)]
    batch: String,

    /// Path to write the proof binary
    #[arg(short, long)]
    proof: String,

    /// Path to write the public inputs
    #[arg(short = 'o', long)]
    pubout: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🔨 Generating Batch ZK Proof");
    println!("=============================\n");

    // Setup SP1 client
    println!("Setting up SP1 prover client...");
    let client = ProverClient::from_env();
    let (pk, _vk) = client.setup(ELF);

    // Read batch input file
    println!("Reading batch inputs from: {}", cli.batch);
    let batch_json = fs::read_to_string(&cli.batch)?;

    // Parse to count withdrawals
    let batch_value: serde_json::Value = serde_json::from_str(&batch_json)?;
    let num_withdrawals = batch_value["withdrawals"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    println!("Batch contains {} withdrawals", num_withdrawals);

    // Generate proof
    println!("\n⏳ Generating proof (this may take a while)...");
    let start = std::time::Instant::now();

    let mut stdin = SP1Stdin::new();
    stdin.write(&batch_json);

    let proof_result = client.prove(&pk, &stdin).groth16().run()?;

    let elapsed = start.elapsed();
    println!("✅ Proof generated in {:.2}s", elapsed.as_secs_f64());

    // Save proof
    println!("\n📝 Saving proof to: {}", cli.proof);
    proof_result.save(&cli.proof)?;

    // Save public inputs
    println!("📝 Saving public inputs to: {}", cli.pubout);
    let sp1_proof_with_public_values = SP1ProofWithPublicValues::load(&cli.proof)?;
    let public_inputs = sp1_proof_with_public_values.public_values.to_vec();
    fs::write(&cli.pubout, &public_inputs)?;

    // Summary
    println!("\n📊 Summary:");
    println!("  - Withdrawals in batch: {}", num_withdrawals);
    println!("  - Proof size: {} bytes", proof_result.bytes().len());
    println!("  - Public inputs size: {} bytes", public_inputs.len());
    println!(
        "  - Public inputs per withdrawal: {} bytes",
        public_inputs.len() / num_withdrawals.max(1)
    );
    println!(
        "  - Average time per withdrawal: {:.2}s",
        elapsed.as_secs_f64() / num_withdrawals.max(1) as f64
    );

    println!("\n🎉 Batch proof generation complete!");

    Ok(())
}
