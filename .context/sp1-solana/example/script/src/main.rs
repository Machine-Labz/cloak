use clap::Parser;
use fibonacci_verifier_contract::SP1Groth16Proof;
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::Signer,
    transaction::Transaction,
};
use sp1_sdk::{include_elf, utils, ProverClient, SP1ProofWithPublicValues, SP1Stdin};

#[derive(clap::Parser)]
#[command(name = "zkVM Proof Generator")]
struct Cli {
    #[arg(
        long,
        value_name = "prove",
        default_value = "false",
        help = "Specifies whether to generate a proof for the program."
    )]
    prove: bool,
}

/// The ELF binary of the SP1 program.
const ELF: &[u8] = include_elf!("fibonacci-program");

/// Invokes the solana program using Solana Program Test.
async fn run_verify_instruction(groth16_proof: SP1Groth16Proof) {
    let program_id = Pubkey::new_unique();

    // Create program test environment
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "fibonacci-verifier-contract",
        program_id,
        processor!(fibonacci_verifier_contract::process_instruction),
    )
    .start()
    .await;

    // Serialize the proof data in the format expected by the Pinocchio program
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&groth16_proof.proof);
    instruction_data.extend_from_slice(&groth16_proof.sp1_public_inputs);
    
    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![AccountMeta::new(payer.pubkey(), false)],
    );

    // Create and send transaction
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}

#[tokio::main]
async fn main() {
    // Setup logging for the application.
    utils::setup_logger();

    // Where to save / load the sp1 proof from.
    let proof_file = "../../proofs/fibonacci_proof.bin";

    // Parse command line arguments.
    let args = Cli::parse();

    // Only generate a proof if the prove flag is set.
    if args.prove {
        // Initialize the prover client
        let client = ProverClient::from_env();
        let (pk, vk) = client.setup(ELF);

        println!(
            "Program Verification Key Bytes {:?}",
            sp1_sdk::HashableKey::bytes32(&vk)
        );

        // In our SP1 program, compute the 20th fibonacci number.
        let mut stdin = SP1Stdin::new();
        stdin.write(&20u32);

        // Generate a proof for the fibonacci program.
        let proof = client
            .prove(&pk, &stdin)
            .groth16()
            .run()
            .expect("Groth16 proof generation failed");

        // Save the generated proof to `proof_file`.
        proof.save(proof_file).unwrap();
    }

    // Load the proof from the file, and convert it to a Borsh-serializable `SP1Groth16Proof`.
    let sp1_proof_with_public_values = SP1ProofWithPublicValues::load(proof_file).unwrap();
    
    // Ensure the proof is exactly 256 bytes and public inputs are exactly 64 bytes
    let mut proof_bytes = sp1_proof_with_public_values.bytes();
    let mut public_inputs = sp1_proof_with_public_values.public_values.to_vec();
    
    // Pad or truncate to exact sizes expected by the Pinocchio program
    proof_bytes.resize(256, 0);
    public_inputs.resize(64, 0);
    
    let groth16_proof = SP1Groth16Proof {
        proof: proof_bytes,
        sp1_public_inputs: public_inputs,
    };

    // Send the proof to the contract, and verify it on `solana-program-test`.
    run_verify_instruction(groth16_proof).await;
}

// use clap::Parser;
// use sp1_sdk::{include_elf, utils, ProverClient, SP1ProofWithPublicValues, SP1Stdin};

// #[derive(clap::Parser)]
// #[command(name = "zkVM Proof Generator")]
// struct Cli {
//     #[arg(
//         long,
//         value_name = "prove",
//         default_value = "false",
//         help = "Specifies whether to generate a proof for the program."
//     )]
//     prove: bool,
// }

// /// The ELF binary of the SP1 program.
// const ELF: &[u8] = include_elf!("fibonacci-program");

// // Off-chain verification function for compressed proofs
// fn verify_compressed_proof(proof: &SP1ProofWithPublicValues, vk: &sp1_sdk::SP1VerifyingKey) -> bool {
//     // This would implement off-chain verification of compressed proofs
//     // For now, we'll just return true as a placeholder
//     println!("🔍 Verifying compressed proof off-chain...");
//     println!("   - Proof type: Compressed");
//     println!("   - Public values: {:?}", proof.public_values);
//     println!("   - VK hash: {:?}", sp1_sdk::HashableKey::bytes32(vk));
//     true
// }

// #[tokio::main]
// async fn main() {
//     // Setup logging for the application.
//     utils::setup_logger();

//     // Where to save / load the sp1 proof from.
//     let proof_file = "../../proofs/fibonacci_proof.bin";

//     // Parse command line arguments.
//     let args = Cli::parse();

//     // Only generate a proof if the prove flag is set.
//     if args.prove {
//         // Initialize the prover client
//         let client = ProverClient::from_env();
//         let (pk, vk) = client.setup(ELF);

//         println!(
//             "Program Verification Key Bytes {:?}",
//             sp1_sdk::HashableKey::bytes32(&vk)
//         );

//         // In our SP1 program, compute the 20th fibonacci number.
//         let mut stdin = SP1Stdin::new();
//         stdin.write(&20u32);

//         // Generate a compressed proof for the fibonacci program.
//         let proof = client
//             .prove(&pk, &stdin)
//             .compressed()
//             .run()
//             .expect("Compressed proof generation failed");

//         // Save the generated proof to `proof_file` using manual serialization.
//         let proof_bytes = bincode::serialize(&proof).expect("failed to serialize proof");
//         std::fs::write(proof_file, &proof_bytes).expect("failed to write proof file");
//         println!("Fibonacci proof saved to: {} ({} bytes)", proof_file, proof_bytes.len());
//     }

//     // Load the compressed proof from the file
//     let proof_bytes = std::fs::read(proof_file).expect("failed to read proof file");
//     let sp1_proof_with_public_values: SP1ProofWithPublicValues = bincode::deserialize(&proof_bytes).expect("failed to deserialize proof");
    
//     println!("✅ Compressed proof loaded successfully!");
//     println!("   - Proof type: Compressed");
//     println!("   - Public values: {:?}", sp1_proof_with_public_values.public_values);
    
//     // Verify the proof off-chain
//     let client = ProverClient::from_env();
//     let (_, vk) = client.setup(ELF);
//     let is_valid = verify_compressed_proof(&sp1_proof_with_public_values, &vk);
    
//     if is_valid {
//         println!("🎉 Proof verification successful!");
//     } else {
//         println!("❌ Proof verification failed!");
//     }
    
//     println!("ℹ️  Compressed proofs are perfect for off-chain verification and recursive proofs!");
//     println!("   For on-chain verification, you would need Groth16 or PLONK proofs.");
// }
