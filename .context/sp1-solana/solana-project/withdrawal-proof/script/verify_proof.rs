//! A simple script to load and verify a saved SP1 proof

use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1VerifyingKey};
use std::fs;

fn main() {
    // Setup the logger
    sp1_sdk::utils::setup_logger();
    
    // Load the proof and verifying key from files
    let proof_bytes = fs::read("withdrawal_proof.bin").expect("Failed to read proof file");
    let vk_bytes = fs::read("withdrawal_vk.bin").expect("Failed to read vk file");
    
    // Deserialize the proof and verifying key
    let proof: SP1ProofWithPublicValues = bincode::deserialize(&proof_bytes)
        .expect("Failed to deserialize proof");
    let vk: SP1VerifyingKey = bincode::deserialize(&vk_bytes)
        .expect("Failed to deserialize verifying key");
    
    // Setup the prover client
    let client = ProverClient::from_env();
    
    // Verify the proof
    match client.verify(&proof, &vk) {
        Ok(_) => {
            println!("✅ Proof verification successful!");
            println!("Proof file: withdrawal_proof.bin ({} bytes)", proof_bytes.len());
            println!("Verifying key: withdrawal_vk.bin ({} bytes)", vk_bytes.len());
        }
        Err(e) => {
            println!("❌ Proof verification failed: {}", e);
        }
    }
}
