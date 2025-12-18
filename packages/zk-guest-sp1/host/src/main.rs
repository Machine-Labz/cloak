use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1Stdin};

mod encoding;
use encoding::*;

// We'll load the ELF at runtime since the build path can vary
// const GUEST_ELF: &[u8] = include_bytes!("../target/elf-compilation/zk-guest-sp1-guest");

#[derive(Parser)]
#[command(name = "cloak-zk")]
#[command(about = "Cloak ZK proof generation and verification CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a ZK proof
    Prove {
        /// Path to private inputs JSON file
        #[arg(long, short = 'r')]
        private: PathBuf,
        /// Path to public inputs JSON file
        #[arg(long, short = 'u')]
        public: PathBuf,
        /// Path to outputs JSON file
        #[arg(long, short = 'o')]
        outputs: PathBuf,
        /// Path to write the proof binary
        #[arg(long, short = 'f')]
        proof: PathBuf,
        /// Path to write the public inputs JSON
        #[arg(long, short = 't')]
        pubout: PathBuf,
    },
    /// Verify a ZK proof
    Verify {
        /// Path to proof binary file
        #[arg(long)]
        proof: PathBuf,
        /// Path to public inputs JSON file
        #[arg(long)]
        public: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivateInputs {
    pub amount: u64,
    #[serde(with = "hex_string")]
    pub r: [u8; 32],
    #[serde(with = "hex_string")]
    pub sk_spend: [u8; 32],
    pub leaf_index: u32,
    pub merkle_path: MerklePath,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublicInputs {
    #[serde(with = "hex_string")]
    pub root: [u8; 32],
    #[serde(with = "hex_string")]
    pub nf: [u8; 32],
    pub fee_bps: u16,
    #[serde(with = "hex_string")]
    pub outputs_hash: [u8; 32],
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CircuitInputs {
    pub private: PrivateInputs,
    pub public: PublicInputs,
    pub outputs: Vec<Output>,
}

// Custom serde module for hex strings
mod hex_string {
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = hex::encode(bytes);
        serializer.serialize_str(&hex_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_hex32(&s).map_err(serde::de::Error::custom)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove {
            private,
            public,
            outputs,
            proof,
            pubout,
        } => {
            prove_command(private, public, outputs, proof, pubout)?;
        }
        Commands::Verify { proof, public } => {
            verify_command(proof, public)?;
        }
    }

    Ok(())
}

fn prove_command(
    private_path: PathBuf,
    public_path: PathBuf,
    outputs_path: PathBuf,
    proof_path: PathBuf,
    pubout_path: PathBuf,
) -> Result<()> {
    println!("Loading inputs...");

    // Load inputs
    let private_json = fs::read_to_string(&private_path)
        .map_err(|e| anyhow!("Failed to read private inputs: {}", e))?;
    let public_json = fs::read_to_string(&public_path)
        .map_err(|e| anyhow!("Failed to read public inputs: {}", e))?;
    let outputs_json =
        fs::read_to_string(&outputs_path).map_err(|e| anyhow!("Failed to read outputs: {}", e))?;

    let private_inputs: PrivateInputs = serde_json::from_str(&private_json)?;
    let public_inputs: PublicInputs = serde_json::from_str(&public_json)?;
    let outputs: Vec<Output> = serde_json::from_str(&outputs_json)?;

    // Create combined input for guest
    let circuit_inputs = CircuitInputs {
        private: private_inputs,
        public: public_inputs,
        outputs,
    };

    let input_json = serde_json::to_string(&circuit_inputs)?;

    println!("Generating proof...");

    // Setup prover
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_json);

    // Generate proof
    let guest_elf = std::fs::read(
        "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
    )
    .or_else(|_| {
        std::fs::read("../guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest")
    })
    .or_else(|_| std::fs::read("target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest"))
    .or_else(|_| {
        std::fs::read("guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest")
    })
    .map_err(|e| anyhow!("Failed to load guest ELF: {}", e))?;
    let (pk, vk) = client.setup(&guest_elf);
    let mut proof = client
        .prove(&pk, &stdin)
        .groth16()
        .run()
        .map_err(|e| anyhow!("Failed to generate proof: {}", e))?;

    println!("Verifying proof...");

    // Verify the proof
    client
        .verify(&proof, &vk)
        .map_err(|e| anyhow!("Proof verification failed: {}", e))?;

    println!("Proof verified successfully!");

    // Create output directory if it doesn't exist
    if let Some(parent) = proof_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = pubout_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Save proof
    let proof_bytes = bincode::serialize(&proof)?;
    fs::write(&proof_path, proof_bytes)?;

    // Save public inputs (from guest commitment)
    let public_output: PublicInputs = proof.public_values.read();
    let public_json = serde_json::to_string_pretty(&public_output)?;
    fs::write(&pubout_path, public_json)?;

    println!("Proof saved to: {}", proof_path.display());
    println!("Public inputs saved to: {}", pubout_path.display());

    Ok(())
}

fn verify_command(proof_path: PathBuf, public_path: PathBuf) -> Result<()> {
    println!("Loading proof and public inputs...");

    // Load proof
    let proof_bytes = fs::read(&proof_path).map_err(|e| anyhow!("Failed to read proof: {}", e))?;
    let mut proof = bincode::deserialize(&proof_bytes)?;

    // Load expected public inputs
    let public_json = fs::read_to_string(&public_path)
        .map_err(|e| anyhow!("Failed to read public inputs: {}", e))?;

    println!("Verifying proof...");

    // Setup verifier
    let client = ProverClient::from_env();
    let guest_elf = std::fs::read(
        "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
    )
    .or_else(|_| {
        std::fs::read("../guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest")
    })
    .or_else(|_| std::fs::read("target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest"))
    .or_else(|_| {
        std::fs::read("guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest")
    })
    .map_err(|e| anyhow!("Failed to load guest ELF: {}", e))?;
    let (_, vk) = client.setup(&guest_elf);

    // Verify the proof
    client
        .verify(&proof, &vk)
        .map_err(|e| anyhow!("Proof verification failed: {}", e))?;

    // Verify public inputs match what guest output
    let actual_public: PublicInputs = proof.public_values.read();
    let expected_public: PublicInputs = serde_json::from_str(&public_json)?;

    if expected_public.root != actual_public.root
        || expected_public.nf != actual_public.nf
        || expected_public.fee_bps != actual_public.fee_bps
        || expected_public.outputs_hash != actual_public.outputs_hash
        || expected_public.amount != actual_public.amount
    {
        return Err(anyhow!("Public inputs mismatch"));
    }

    println!("✅ Proof verified successfully!");
    println!("Public inputs match expected values.");

    Ok(())
}
