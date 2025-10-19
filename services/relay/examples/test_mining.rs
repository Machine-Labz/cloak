//! Simple example demonstrating the mining engine
//!
//! Run with: cargo run --package relay --example test_mining

use relay::miner::{MiningEngine};
use solana_sdk::pubkey::Pubkey;

fn main() {
    // Set up tracing
    tracing_subscriber::fmt::init();

    println!("=== PoW Mining Engine Test ===\n");

    // Test 1: Very easy difficulty (should succeed immediately)
    println!("Test 1: Easy difficulty (all 0xFF)");
    let easy_engine = MiningEngine::new(
        [0xFF; 32],  // Very easy difficulty
        12345,
        [0x42; 32],
        Pubkey::new_unique(),
        [0x88; 32],
    );

    match easy_engine.mine() {
        Ok(solution) => {
            println!("✓ Found solution: nonce={}", solution.nonce);
            println!("  Hash: {:x?}...\n", &solution.proof_hash[0..8]);
        }
        Err(e) => {
            println!("✗ Mining failed: {}\n", e);
        }
    }

    // Test 2: Moderate difficulty (first byte must be 0x00)
    println!("Test 2: Moderate difficulty (first byte < 0x01)");
    let mut moderate_difficulty = [0xFF; 32];
    moderate_difficulty[0] = 0x01;  // First byte must be 0x00

    let moderate_engine = MiningEngine::new(
        moderate_difficulty,
        67890,
        [0x33; 32],
        Pubkey::new_unique(),
        [0x77; 32],
    );

    match moderate_engine.mine() {
        Ok(solution) => {
            println!("✓ Found solution: nonce={}", solution.nonce);
            println!("  Hash: {:x?}...", &solution.proof_hash[0..8]);
            println!("  First byte: 0x{:02x} (< 0x01) ✓\n", solution.proof_hash[0]);
        }
        Err(e) => {
            println!("✗ Mining failed: {}\n", e);
        }
    }

    // Test 3: With timeout (harder difficulty)
    println!("Test 3: Mining with timeout (first 2 bytes < 0x0001)");
    let mut hard_difficulty = [0xFF; 32];
    hard_difficulty[0] = 0x01;
    hard_difficulty[1] = 0x00;  // First 2 bytes must be 0x0000

    let hard_engine = MiningEngine::new(
        hard_difficulty,
        11111,
        [0xAA; 32],
        Pubkey::new_unique(),
        [0xBB; 32],
    );

    let timeout = std::time::Duration::from_secs(5);
    match hard_engine.mine_with_timeout(timeout) {
        Ok(solution) => {
            println!("✓ Found solution: nonce={}", solution.nonce);
            println!("  Hash: {:x?}...", &solution.proof_hash[0..8]);
            println!("  First 2 bytes: 0x{:02x}{:02x} (< 0x0001) ✓\n",
                solution.proof_hash[0], solution.proof_hash[1]);
        }
        Err(e) => {
            println!("✗ Mining timed out (expected): {}\n", e);
        }
    }

    println!("=== Mining Engine Test Complete ===");
}
