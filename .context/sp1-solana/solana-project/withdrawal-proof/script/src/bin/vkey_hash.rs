//! Script to get the VKey hash for the withdrawal proof program

use sp1_sdk::{include_elf, HashableKey, ProverClient};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const WITHDRAWAL_PROOF_ELF: &[u8] = include_elf!("withdrawal-proof-program");

fn main() {
    // Setup the logger
    sp1_sdk::utils::setup_logger();

    // Setup the prover client
    let client = ProverClient::from_env();

    // Setup the program for proving to get the verification key
    let (_pk, vk) = client.setup(WITHDRAWAL_PROOF_ELF);

    // Get the VKey hash
    let vkey_hash = vk.bytes32();

    println!("=== Withdrawal Proof VKey Hash ===");
    println!("VKey Hash: {}", vkey_hash);
    println!("VKey Hash (for Pinocchio): {}", vkey_hash);
    println!("VKey bytes length: {}", vkey_hash.len());

    // Also save to file for easy copying
    std::fs::write("vkey_hash.txt", &vkey_hash).expect("Failed to write vkey hash file");
    println!("VKey hash saved to: vkey_hash.txt");
}
