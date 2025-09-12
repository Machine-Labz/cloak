//! Simple script to get the VKey hash for our withdrawal proof program

use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const WITHDRAWAL_PROOF_ELF: &[u8] = include_elf!("withdrawal-proof-program");

fn main() {
    // Setup the prover client
    let client = ProverClient::from_env();
    
    // Setup the program for proving to get the verification key
    let (pk, vk) = client.setup(WITHDRAWAL_PROOF_ELF);
    
    // Get the VKey hash
    let vkey_hash = vk.bytes32();
    
    println!("VKey Hash: 0x{}", hex::encode(vkey_hash));
    println!("VKey Hash (with 0x prefix): 0x{}", hex::encode(vkey_hash));
    
    // Also print the VKey bytes for reference
    println!("VKey bytes: {:?}", vk.bytes());
}
