//! Verify mining logic with actual hash values
//!
//! Run with: cargo run --package relay --example verify_mining

use cloak_miner::MiningEngine;
use solana_sdk::pubkey::Pubkey;

fn main() {
    println!("=== Mining Logic Verification ===\n");

    // Test with harder difficulty where we expect multiple attempts
    println!("Test: First byte must be 0x00");
    let mut difficulty = [0xFF; 32];
    difficulty[0] = 0x00; // VERY strict: first byte must be exactly 0x00
    difficulty[1] = 0x10; // And second byte < 0x10

    println!("Difficulty target: {:02x?}...\n", &difficulty[0..4]);

    let engine = MiningEngine::new(
        difficulty,
        99999,
        [
            0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
        Pubkey::new_unique(),
        [
            0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
    );

    let timeout = std::time::Duration::from_secs(10);
    match engine.mine_with_timeout(timeout) {
        Ok(solution) => {
            println!("✓ Solution found!");
            println!("  Nonce: {}", solution.nonce);
            println!("  Hash:   {:02x?}...", &solution.proof_hash[0..8]);
            println!("  Target: {:02x?}...", &difficulty[0..8]);
            println!("\n  Verification:");
            println!(
                "    Hash[0]={:02x} < Target[0]={:02x}? {}",
                solution.proof_hash[0],
                difficulty[0],
                solution.proof_hash[0] < difficulty[0]
            );
            if solution.proof_hash[0] == difficulty[0] {
                println!(
                    "    Hash[1]={:02x} < Target[1]={:02x}? {}",
                    solution.proof_hash[1],
                    difficulty[1],
                    solution.proof_hash[1] < difficulty[1]
                );
            }
        }
        Err(e) => {
            println!("✗ Mining failed/timed out: {}", e);
        }
    }
}
