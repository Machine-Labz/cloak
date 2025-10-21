use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

fn main() -> Result<()> {
    println!("🚀 Calling Initialize Instruction on Shield Pool");
    println!("================================================\n");

    let rpc_url = "https://api.testnet.solana.com";
    let rpc = RpcClient::new(rpc_url);

    // Load admin keypair from ~/.config/solana/id.json
    let home = std::env::var("HOME")?;
    let admin_path = format!("{}/.config/solana/id.json", home);
    println!("Loading admin keypair from: {}", admin_path);
    let admin = read_keypair_file(&admin_path)
        .map_err(|e| anyhow::anyhow!("Failed to load admin keypair: {}", e))?;
    println!("Admin pubkey: {}\n", admin.pubkey());

    // Program ID
    let program_id = Pubkey::from_str("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp")?;

    // Derive PDAs
    let (pool_pda, pool_bump) = Pubkey::find_program_address(&[b"pool"], &program_id);
    let (commitments_pda, commitments_bump) =
        Pubkey::find_program_address(&[b"commitments"], &program_id);
    let (roots_ring_pda, roots_ring_bump) =
        Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let (nullifier_shard_pda, nullifier_shard_bump) =
        Pubkey::find_program_address(&[b"nullifier_shard"], &program_id);
    let (treasury_pda, treasury_bump) = Pubkey::find_program_address(&[b"treasury"], &program_id);

    println!("Program PDAs:");
    println!("  Pool:            {} (bump: {})", pool_pda, pool_bump);
    println!(
        "  Commitments:     {} (bump: {})",
        commitments_pda, commitments_bump
    );
    println!(
        "  Roots Ring:      {} (bump: {})",
        roots_ring_pda, roots_ring_bump
    );
    println!(
        "  Nullifier Shard: {} (bump: {})",
        nullifier_shard_pda, nullifier_shard_bump
    );
    println!(
        "  Treasury:        {} (bump: {})\n",
        treasury_pda, treasury_bump
    );

    // Check if pool PDA already exists as a proxy for initialization
    if let Ok(account) = rpc.get_account(&pool_pda) {
        println!("⚠️  Pool PDA already exists!");
        println!("   Owner: {}", account.owner);
        println!("   Lamports: {}", account.lamports);
        println!("   Data length: {} bytes\n", account.data.len());
        println!("Initialization may have already been completed.");
        return Ok(());
    }

    // Create Initialize instruction
    // Instruction discriminant = 3
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),       // admin (signer)
            AccountMeta::new(pool_pda, false),            // pool PDA
            AccountMeta::new(commitments_pda, false),     // commitments PDA
            AccountMeta::new(roots_ring_pda, false),      // roots ring PDA
            AccountMeta::new(nullifier_shard_pda, false), // nullifier shard PDA
            AccountMeta::new(treasury_pda, false),        // treasury PDA
            AccountMeta::new_readonly(system_program::id(), false), // system_program
        ],
        data: vec![3], // Initialize discriminant
    };

    println!("Building transaction...");
    let mut tx = Transaction::new_with_payer(&[ix], Some(&admin.pubkey()));

    let blockhash = rpc.get_latest_blockhash()?;
    tx.sign(&[&admin], blockhash);

    println!("Sending transaction...");
    let sig = rpc.send_and_confirm_transaction(&tx)?;

    println!("\n✅ Initialize transaction confirmed!");
    println!("   Signature: {}", sig);
    println!("   Pool PDA:            {}", pool_pda);
    println!("   Commitments PDA:     {}", commitments_pda);
    println!("   Roots Ring PDA:      {}", roots_ring_pda);
    println!("   Nullifier Shard PDA: {}", nullifier_shard_pda);
    println!("   Treasury PDA:        {}\n", treasury_pda);

    // Verify accounts were created
    for (label, pubkey) in [
        ("Pool", pool_pda),
        ("Commitments", commitments_pda),
        ("Roots Ring", roots_ring_pda),
        ("Nullifier Shard", nullifier_shard_pda),
        ("Treasury", treasury_pda),
    ] {
        match rpc.get_account(&pubkey) {
            Ok(account) => {
                println!("✅ {} account verification:", label);
                println!("   Owner: {}", account.owner);
                println!("   Lamports: {}", account.lamports);
                println!("   Data length: {} bytes", account.data.len());
            }
            Err(e) => {
                println!("⚠️  Could not verify {} account: {}", label, e);
            }
        }
    }

    Ok(())
}
