//! An end-to-end example of using the SP1 SDK to generate a proof of a withdrawal authorization program.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs;

// Define the public values struct directly here
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalProofStruct {
    pub user_address: [u8; 20],
    pub pool_id: u64,
    pub user_balance: u64,
    pub withdrawal_amount: u64,
    pub pool_liquidity: u64,
    pub timestamp: u64,
    pub is_valid: bool,
}

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const WITHDRAWAL_PROOF_ELF: &[u8] = include_elf!("withdrawal-proof-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long)]
    verify: bool,

    #[arg(long, default_value = "1000000")]
    user_balance: u64,

    #[arg(long, default_value = "100000")]
    withdrawal_amount: u64,

    #[arg(long, default_value = "5000000")]
    pool_liquidity: u64,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    let mode_count = [args.execute, args.prove, args.verify]
        .iter()
        .filter(|&&x| x)
        .count();
    if mode_count != 1 {
        eprintln!("Error: You must specify exactly one of --execute, --prove, or --verify");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Create test data for withdrawal proof
    let user_address = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99, 0xaa, 0xbb, 0xcc,
    ];
    let pool_id = 12345u64;
    let user_balance = args.user_balance;
    let withdrawal_amount = args.withdrawal_amount;
    let pool_liquidity = args.pool_liquidity;
    let user_signature = vec![0x01; 65]; // Mock signature
    let pool_signature = vec![0x02; 65]; // Mock signature
    let timestamp = 1700000000u64; // Mock timestamp

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&user_address);
    stdin.write(&pool_id);
    stdin.write(&user_balance);
    stdin.write(&withdrawal_amount);
    stdin.write(&pool_liquidity);
    stdin.write(&user_signature);
    stdin.write(&pool_signature);
    stdin.write(&timestamp);

    println!("Withdrawal Proof Test:");
    println!("User Address: {:?}", user_address);
    println!("Pool ID: {}", pool_id);
    println!("User Balance: {}", user_balance);
    println!("Withdrawal Amount: {}", withdrawal_amount);
    println!("Pool Liquidity: {}", pool_liquidity);
    println!("Timestamp: {}", timestamp);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(WITHDRAWAL_PROOF_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output - the program outputs the struct as raw bytes
        // We need to deserialize it manually since we're not using alloy_sol_types
        let decoded: WithdrawalProofStruct = bincode::deserialize(&output.to_vec()).unwrap();
        let WithdrawalProofStruct {
            user_address: out_address,
            pool_id: out_pool_id,
            user_balance: out_user_balance,
            withdrawal_amount: out_withdrawal_amount,
            pool_liquidity: out_pool_liquidity,
            timestamp: out_timestamp,
            is_valid,
        } = decoded;

        println!("Output:");
        println!("User Address: {:?}", out_address);
        println!("Pool ID: {}", out_pool_id);
        println!("User Balance: {}", out_user_balance);
        println!("Withdrawal Amount: {}", out_withdrawal_amount);
        println!("Pool Liquidity: {}", out_pool_liquidity);
        println!("Timestamp: {}", out_timestamp);
        println!("Is Valid: {}", is_valid);

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else if args.prove {
        // Setup the program for proving.
        let (pk, vk) = client.setup(WITHDRAWAL_PROOF_ELF);

        // Generate the compressed proof (perfect for off-chain verification)
        let proof = client
            .prove(&pk, &stdin)
            .compressed()
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated compressed proof!");

        // Save the proof to the proofs directory using manual serialization
        let proof_file = "../proofs/withdrawal_proof.bin";
        let proof_bytes = bincode::serialize(&proof).expect("failed to serialize proof");
        std::fs::write(proof_file, &proof_bytes).expect("failed to write proof file");
        println!(
            "Proof saved to: {} ({} bytes)",
            proof_file,
            proof_bytes.len()
        );

        // Save the verifying key to the proofs directory
        let vk_file = "../proofs/withdrawal_vk.bin";
        let vk_bytes = bincode::serialize(&vk).expect("failed to serialize vk");
        std::fs::write(vk_file, &vk_bytes).expect("failed to write vk file");
        println!(
            "Verifying key saved to: {} ({} bytes)",
            vk_file,
            vk_bytes.len()
        );

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");

        // Print proof details
        println!("\nProof Details:");
        println!("- Proof size: {} bytes", proof_bytes.len());
        println!("- Verifying key size: {} bytes", vk_bytes.len());
    } else if args.verify {
        // Load and verify an existing proof
        println!("Loading and verifying existing proof...");

        let proof_bytes = fs::read("withdrawal_proof.bin").expect("Failed to read proof file");
        let vk_bytes = fs::read("withdrawal_vk.bin").expect("Failed to read vk file");

        let proof: sp1_sdk::SP1ProofWithPublicValues =
            bincode::deserialize(&proof_bytes).expect("Failed to deserialize proof");
        let vk: sp1_sdk::SP1VerifyingKey =
            bincode::deserialize(&vk_bytes).expect("Failed to deserialize verifying key");

        match client.verify(&proof, &vk) {
            Ok(_) => {
                println!("✅ Proof verification successful!");
                println!(
                    "Proof file: withdrawal_proof.bin ({} bytes)",
                    proof_bytes.len()
                );
                println!(
                    "Verifying key: withdrawal_vk.bin ({} bytes)",
                    vk_bytes.len()
                );
            }
            Err(e) => {
                println!("❌ Proof verification failed: {}", e);
            }
        }
    }
}
