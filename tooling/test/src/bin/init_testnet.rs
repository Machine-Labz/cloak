use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

const PROGRAM_ID: &str = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp";
const ROOTS_RING_SIZE: usize = 2056; // 8 + 64 * 32
const NULLIFIER_SHARD_SIZE: usize = 4; // Start with just count field

fn main() -> Result<()> {
    println!("🚀 Initializing Cloak Program on Testnet");
    println!("==========================================\n");

    // Connect to testnet
    let rpc_url = "https://api.testnet.solana.com";
    let client = RpcClient::new(rpc_url);

    // Load admin keypair
    let admin_keypair = load_keypair("admin-keypair.json")?;
    println!("Admin pubkey: {}", admin_keypair.pubkey());

    // Get admin balance
    let balance = client.get_balance(&admin_keypair.pubkey())?;
    println!("Admin balance: {} SOL\n", balance as f64 / 1_000_000_000.0);

    if balance < 1_000_000_000 {
        println!("⚠️  Admin balance is low. Please fund the admin account first.");
        println!(
            "   Run: solana airdrop 2 {} --url {}",
            admin_keypair.pubkey(),
            rpc_url
        );
        return Ok(());
    }

    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    // Derive PDAs
    let (pool_pda, pool_bump) = Pubkey::find_program_address(&[b"pool"], &program_id);
    let (roots_ring_pda, roots_ring_bump) =
        Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let (nullifier_shard_pda, nullifier_shard_bump) =
        Pubkey::find_program_address(&[b"nullifier_shard"], &program_id);
    let (treasury_pda, treasury_bump) = Pubkey::find_program_address(&[b"treasury"], &program_id);

    println!("📋 Program PDAs:");
    println!("  Pool:            {} (bump: {})", pool_pda, pool_bump);
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

    // Check if accounts already exist
    if client.get_account(&pool_pda).is_ok() {
        println!("✅ Pool account already exists!");
        println!("   Account: {}", pool_pda);
        println!("\n⚠️  Accounts appear to be already initialized.");
        println!(
            "   If you need to reinitialize, you'll need to close the existing accounts first."
        );
        return Ok(());
    }

    println!("❌ CANNOT CREATE PDA ACCOUNTS FROM HERE\n");
    println!("═══════════════════════════════════════════════════════════════");
    println!("PDAs (Program Derived Addresses) can only be created by the");
    println!("program itself using invoke_signed, not from external clients.");
    println!("");
    println!("The shield-pool program does NOT currently expose an Initialize");
    println!("instruction, so these accounts cannot be created without either:");
    println!("");
    println!("  1. Temporarily adding Initialize instruction and redeploying");
    println!("  2. Using a separate helper program");
    println!("  3. Waiting to see if program creates them on first use");
    println!("═══════════════════════════════════════════════════════════════");
    println!("");
    println!("📋 Required PDA Addresses:");
    println!("  Pool:            {}", pool_pda);
    println!("  Roots Ring:      {}", roots_ring_pda);
    println!("  Nullifier Shard: {}", nullifier_shard_pda);
    println!("  Treasury:        {}", treasury_pda);
    println!("");
    println!("📖 See INITIALIZE_SHIELD_POOL.md for detailed instructions.");
    println!("");

    Ok(())
}

fn load_keypair(path: &str) -> Result<Keypair> {
    let keypair_data = std::fs::read(path)?;

    // Try to parse as JSON first (array of numbers)
    if let Ok(json_data) = serde_json::from_slice::<Vec<u8>>(&keypair_data) {
        return Ok(Keypair::from_bytes(&json_data)?);
    }

    // Otherwise try as raw bytes
    Ok(Keypair::from_bytes(&keypair_data)?)
}
