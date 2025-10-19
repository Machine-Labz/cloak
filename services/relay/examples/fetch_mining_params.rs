//! Example: Fetch mining parameters from RPC
//!
//! Demonstrates fetching registry state and SlotHashes.
//! Note: Requires a running Solana RPC node and deployed registry.
//!
//! Run with: cargo run --package relay --example fetch_mining_params

use relay::miner::rpc::{fetch_registry, fetch_recent_slot_hash, get_current_slot};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() {
    println!("=== Fetch Mining Parameters Example ===\n");

    // Connect to RPC (devnet for this example)
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    println!("Connecting to RPC: {}", rpc_url);
    let client = RpcClient::new(rpc_url);

    // Get current slot
    println!("\n1. Fetching current slot...");
    match get_current_slot(&client) {
        Ok(slot) => println!("   Current slot: {}", slot),
        Err(e) => println!("   Error: {}", e),
    }

    // Fetch SlotHashes
    println!("\n2. Fetching recent SlotHash...");
    match fetch_recent_slot_hash(&client) {
        Ok((slot, hash)) => {
            println!("   Slot: {}", slot);
            println!("   Hash: {:x?}...", &hash[0..8]);
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Fetch registry (if program ID provided)
    if let Ok(registry_str) = std::env::var("SCRAMBLE_REGISTRY_PUBKEY") {
        println!("\n3. Fetching ScrambleRegistry...");
        match Pubkey::from_str(&registry_str) {
            Ok(registry_pubkey) => {
                match fetch_registry(&client, &registry_pubkey) {
                    Ok(registry) => {
                        println!("   ✓ Registry found!");
                        println!("   Admin: {}", registry.admin);
                        println!("   Current difficulty: {:x?}...", &registry.current_difficulty[0..4]);
                        println!("   Reveal window: {} slots", registry.reveal_window);
                        println!("   Claim window: {} slots", registry.claim_window);
                        println!("   Max batch size (k): {}", registry.max_k);
                        println!("   Fee share: {} bps ({}%)",
                            registry.fee_share_bps,
                            registry.fee_share_bps as f64 / 100.0);
                        println!("   Total claims: {}", registry.total_claims);
                        println!("   Active claims: {}", registry.active_claims);
                    }
                    Err(e) => println!("   Error: {}", e),
                }
            }
            Err(e) => println!("   Invalid pubkey: {}", e),
        }
    } else {
        println!("\n3. Skipping registry fetch (set SCRAMBLE_REGISTRY_PUBKEY env var)");
    }

    println!("\n=== Example Complete ===");
    println!("\nTo fetch registry state, run:");
    println!("  SCRAMBLE_REGISTRY_PUBKEY=<pubkey> cargo run --package relay --example fetch_mining_params");
}
