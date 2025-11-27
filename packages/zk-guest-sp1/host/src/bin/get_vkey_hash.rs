use anyhow::Result;
use sha2::{Digest, Sha256};
use sp1_sdk::{HashableKey, ProverClient};
use std::fs;
use std::path::Path;

fn find_guest_elf() -> Result<(Vec<u8>, String)> {
    let paths = [
        "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "../guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "../.artifacts/zk-guest-sp1-guest",
    ];

    for path in &paths {
        if Path::new(path).exists() {
            let elf_bytes = fs::read(path)?;
            return Ok((elf_bytes, path.to_string()));
        }
    }

    Err(anyhow::anyhow!(
        "Could not find guest ELF in any expected location"
    ))
}

fn main() -> Result<()> {
    let (guest_elf, elf_path) = find_guest_elf()?;
    
    // Compute SHA256 hash of ELF for verification
    let mut hasher = Sha256::new();
    hasher.update(&guest_elf);
    let elf_hash = hasher.finalize();
    
    println!("=== VKey Hash Diagnostic ===");
    println!("ELF Location: {}", elf_path);
    println!("ELF SHA256: {}", hex::encode(elf_hash));
    println!("ELF Size: {} bytes", guest_elf.len());
    
    let client = ProverClient::from_env();
    let (_, vk) = client.setup(&guest_elf);
    let vkey_hash = vk.bytes32();

    println!("\nSP1 Withdraw Circuit VKey Hash: {}", vkey_hash);

    Ok(())
}
