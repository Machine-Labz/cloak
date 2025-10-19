//! Example: Build PoW instructions
//!
//! Demonstrates building mine_claim and reveal_claim instructions.
//!
//! Run with: cargo run --package relay --example build_instructions

use cloak_miner::{
    build_mine_and_reveal_instructions, derive_claim_pda, derive_miner_pda, derive_registry_pda,
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() {
    println!("=== Build PoW Instructions Example ===\n");

    // Example program ID (replace with actual deployed program)
    let program_id = Pubkey::from_str("ScRaMbLePoWReg1StRyPRoGRam111111111111111")
        .unwrap_or_else(|_| Pubkey::new_unique());

    let miner_authority = Pubkey::new_unique();

    println!("Program ID: {}", program_id);
    println!("Miner authority: {}\n", miner_authority);

    // 1. Derive PDAs
    println!("1. PDA Derivation:");
    let (registry_pda, registry_bump) = derive_registry_pda(&program_id);
    println!(
        "   Registry PDA: {} (bump: {})",
        registry_pda, registry_bump
    );

    let (miner_pda, miner_bump) = derive_miner_pda(&program_id, &miner_authority);
    println!("   Miner PDA:    {} (bump: {})", miner_pda, miner_bump);

    let batch_hash = [0x88; 32];
    let slot = 12345u64;
    let (claim_pda, claim_bump) =
        derive_claim_pda(&program_id, &miner_authority, &batch_hash, slot);
    println!("   Claim PDA:    {} (bump: {})\n", claim_pda, claim_bump);

    // 2. Build instructions
    println!("2. Building mine + reveal instructions:");

    let slot_hash = [0x42; 32];
    let nonce = 98765u128;
    let proof_hash = [0xAA; 32];
    let max_consumes = 1u16;

    match build_mine_and_reveal_instructions(
        &program_id,
        &miner_authority,
        slot,
        slot_hash,
        batch_hash,
        nonce,
        proof_hash,
        max_consumes,
    ) {
        Ok((mine_ix, reveal_ix)) => {
            println!("   ✓ Mine instruction:");
            println!("      Program: {}", mine_ix.program_id);
            println!("      Accounts: {}", mine_ix.accounts.len());
            println!("      Data size: {} bytes", mine_ix.data.len());
            println!("      Discriminator: {}", mine_ix.data[0]);

            println!("\n   ✓ Reveal instruction:");
            println!("      Program: {}", reveal_ix.program_id);
            println!("      Accounts: {}", reveal_ix.accounts.len());
            println!("      Data size: {} bytes", reveal_ix.data.len());
            println!("      Discriminator: {}", reveal_ix.data[0]);

            println!("\n   These can be submitted:");
            println!("   - Together in one transaction (atomic)");
            println!("   - Separately (mine first, then reveal)");
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nNext steps:");
    println!("1. Sign instructions with miner authority keypair");
    println!("2. Submit mine_claim transaction");
    println!("3. Wait for confirmation");
    println!("4. Submit reveal_claim transaction");
    println!("5. Claim is now ready to consume!");
}
