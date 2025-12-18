use anyhow::Result;
use sp1_sdk::{HashableKey, ProverClient};

fn find_guest_elf() -> Result<Vec<u8>> {
    let paths = [
        "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "../guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Ok(std::fs::read(path)?);
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

    let vk_bytes = bincode::serialize(&vk)?;
    let output_path = "groth16_vk_latest.bin";
    std::fs::write(output_path, &vk_bytes)?;

    println!("Wrote {} bytes to {}", vk_bytes.len(), output_path);
    println!("VKey hash: 0x{}", hex::encode(vk.bytes32()));

    Ok(())
}
