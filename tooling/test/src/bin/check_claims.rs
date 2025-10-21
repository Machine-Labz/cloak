#!/usr/bin/env cargo
//! Quick test to check for available wildcard claims on testnet
//!
//! Usage:
//!   cargo run --bin check_claims

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Checking for Available Wildcard Claims on Testnet");
    println!("====================================================\n");

    // Testnet configuration
    let rpc_url = "https://api.testnet.solana.com";
    let registry_program_id = Pubkey::from_str("EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6")?;

    println!("📡 Connecting to: {}", rpc_url);
    println!("📋 Registry Program: {}\n", registry_program_id);

    let client = RpcClient::new(rpc_url);

    // Step 1: Query all accounts owned by scramble-registry
    println!("🔎 Step 1: Querying program accounts...");
    let accounts = client.get_program_accounts(&registry_program_id)?;
    println!("   Found {} total accounts\n", accounts.len());

    // Step 2: Filter for Claim accounts
    println!("🔎 Step 2: Filtering for Claim accounts...");
    let mut claim_count = 0;
    let mut wildcard_count = 0;
    let mut revealed_wildcards = Vec::new();

    for (pubkey, account) in &accounts {
        // Claim accounts are 256 bytes (Pinocchio - no discriminator)
        if account.data.len() != 256 {
            continue;
        }

        claim_count += 1;

        // Parse claim data (Pinocchio format - no discriminator)
        // Layout (from scramble-registry/src/state/mod.rs Claim struct):
        // 0-31: miner_authority
        // 32-63: batch_hash
        // 64-71: slot
        // 72-103: slot_hash (32 bytes)
        // 104-119: nonce (16 bytes)
        // 120-151: proof_hash (32 bytes)
        // 152-159: mined_at_slot
        // 160-167: revealed_at_slot
        // 168-169: consumed_count
        // 170-171: max_consumes
        // 172-179: expires_at_slot
        // 180: status
        let miner_authority = parse_pubkey(&account.data[0..32]);
        let batch_hash = &account.data[32..64];
        let slot = u64::from_le_bytes(account.data[64..72].try_into()?);
        let consumed_count = u16::from_le_bytes(account.data[168..170].try_into()?);
        let max_consumes = u16::from_le_bytes(account.data[170..172].try_into()?);
        let expires_at_slot = u64::from_le_bytes(account.data[172..180].try_into()?);
        let status = account.data[180];

        // Check if wildcard
        let is_wildcard = batch_hash == &[0u8; 32];

        if is_wildcard {
            wildcard_count += 1;

            let status_str = match status {
                0 => "Hidden",
                1 => "Revealed",
                2 => "Consumed",
                _ => "Unknown",
            };

            println!("\n   📍 Wildcard Claim: {}", pubkey);
            println!("      Miner: {}", miner_authority);
            println!("      Status: {} ({})", status_str, status);
            println!("      Slot: {}", slot);
            println!("      Consumed: {}/{}", consumed_count, max_consumes);
            println!("      Expires at slot: {}", expires_at_slot);

            // Check if usable
            let current_slot = client.get_slot()?;
            let is_expired = current_slot > expires_at_slot;
            let is_fully_consumed = consumed_count >= max_consumes;
            let is_revealed = status == 1;

            let usable = is_revealed && !is_expired && !is_fully_consumed;

            println!("      Current slot: {}", current_slot);
            println!("      Expired: {}", is_expired);
            println!("      Fully consumed: {}", is_fully_consumed);

            if usable {
                println!("      ✅ USABLE FOR WITHDRAW!");
                revealed_wildcards.push((pubkey.clone(), miner_authority));
            } else {
                println!("      ❌ Not usable");
            }
        }
    }

    // Step 3: Summary
    println!("\n📊 Summary");
    println!("==========");
    println!("Total accounts: {}", accounts.len());
    println!("Total claims: {}", claim_count);
    println!("Wildcard claims: {}", wildcard_count);
    println!("Usable wildcard claims: {}\n", revealed_wildcards.len());

    if revealed_wildcards.is_empty() {
        println!("❌ No usable wildcard claims found!");
        println!("   Ensure the miner is running and has revealed claims.");
        println!("   Miner should be: 2E6otmfdfWZJSx9dXfpqWNwq8hBa58ENGM3AryJNLZQn");
    } else {
        println!(
            "✅ Found {} usable wildcard claim(s)!",
            revealed_wildcards.len()
        );
        println!("\n🎯 These claims can be used for withdraws:");
        for (i, (claim_pda, miner_authority)) in revealed_wildcards.iter().enumerate() {
            println!("   {}. Claim: {}", i + 1, claim_pda);
            println!("      Miner: {}", miner_authority);
            println!("      → Miner will earn fees when this claim is consumed!\n");
        }
    }

    // Step 4: Check miner balance
    if !revealed_wildcards.is_empty() {
        println!("💰 Checking miner balances:");
        for (claim_pda, miner_authority) in &revealed_wildcards {
            match client.get_balance(&miner_authority) {
                Ok(balance) => {
                    println!(
                        "   Miner {} (claim {}): {} SOL",
                        miner_authority,
                        claim_pda,
                        balance as f64 / 1_000_000_000.0
                    );
                }
                Err(e) => {
                    println!(
                        "   ⚠️  Failed to get balance for {}: {}",
                        miner_authority, e
                    );
                }
            }
        }
    }

    println!("\n✅ Check complete!");

    Ok(())
}

fn parse_pubkey(bytes: &[u8]) -> Pubkey {
    let mut array = [0u8; 32];
    array.copy_from_slice(bytes);
    Pubkey::new_from_array(array)
}
