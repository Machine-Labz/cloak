use anyhow::Result;
use sp1_sdk::{HashableKey, ProverClient};
use std::fs;

fn find_guest_elf() -> Result<Vec<u8>> {
    let paths = [
        "packages/zk-guest-sp1/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "packages/zk-guest-sp1/guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Ok(fs::read(path)?);
        }
    }

    Err(anyhow::anyhow!(
        "Could not find guest ELF in any expected location"
    ))
}

fn main() -> Result<()> {
    let client = ProverClient::from_env();
    let guest_elf = find_guest_elf()?;
    let (_, vk) = client.setup(&guest_elf);
    let vkey_hash = vk.bytes32();

    let vkey_hash_hex = vkey_hash;
    println!("SP1 Withdraw Circuit VKey Hash: {}", vkey_hash_hex);

    // Write to file in target directory
    let target_dir = "target";
    std::fs::create_dir_all(target_dir)?;
    let output_file = format!("{}/vkey_hash.txt", target_dir);
    fs::write(&output_file, &vkey_hash_hex)?;

    println!("VKey hash written to: {}", output_file);

    Ok(())
}
