use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

const PROGRAM_ID: &str = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp";
const ROOTS_RING_SIZE: usize = 2056; // 8 + 64 * 32
const NULLIFIER_SHARD_SIZE: usize = 4; // Start with just count field

fn main() -> Result<()> {
    println!("🚀 Initializing Cloak Program on Localnet");
    println!("==========================================\n");

    // Connect to localnet
    let rpc_url = "http://127.0.0.1:8899";
    let client = RpcClient::new(rpc_url);

    // Load admin keypair
    let admin_keypair = load_keypair("admin-keypair.json")?;
    println!("Admin pubkey: {}", admin_keypair.pubkey());

    // Get admin balance
    let balance = client.get_balance(&admin_keypair.pubkey())?;
    println!("Admin balance: {} SOL\n", balance as f64 / 1_000_000_000.0);

    if balance < 1_000_000_000 {
        println!("⚠️  Admin balance is low. Requesting airdrop...");
        let signature = client.request_airdrop(&admin_keypair.pubkey(), 10_000_000_000)?;
        client.confirm_transaction(&signature)?;
        println!("✅ Airdrop confirmed\n");
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
    println!("🔍 Checking if accounts exist...");
    let pool_exists = client.get_account(&pool_pda).is_ok();
    let roots_ring_exists = client.get_account(&roots_ring_pda).is_ok();
    let nullifier_shard_exists = client.get_account(&nullifier_shard_pda).is_ok();
    let treasury_exists = client.get_account(&treasury_pda).is_ok();

    println!(
        "  Pool:            {}",
        if pool_exists {
            "✅ exists"
        } else {
            "❌ missing"
        }
    );
    println!(
        "  Roots Ring:      {}",
        if roots_ring_exists {
            "✅ exists"
        } else {
            "❌ missing"
        }
    );
    println!(
        "  Nullifier Shard: {}",
        if nullifier_shard_exists {
            "✅ exists"
        } else {
            "❌ missing"
        }
    );
    println!(
        "  Treasury:        {}\n",
        if treasury_exists {
            "✅ exists"
        } else {
            "❌ missing"
        }
    );

    if pool_exists && roots_ring_exists && nullifier_shard_exists && treasury_exists {
        println!("✅ All accounts already initialized!");
        return Ok(());
    }

    println!("🔨 Creating missing accounts...\n");

    // Create pool account if needed
    if !pool_exists {
        println!("Creating pool account...");
        let pool_rent = client.get_minimum_balance_for_rent_exemption(0)?;
        let create_pool_ix = system_instruction::create_account_with_seed(
            &admin_keypair.pubkey(),
            &pool_pda,
            &admin_keypair.pubkey(),
            "pool",
            pool_rent,
            0,
            &program_id,
        );

        let mut tx = Transaction::new_with_payer(&[create_pool_ix], Some(&admin_keypair.pubkey()));
        let blockhash = client.get_latest_blockhash()?;
        tx.sign(&[&admin_keypair], blockhash);

        match client.send_and_confirm_transaction(&tx) {
            Ok(_) => println!("  ✅ Pool account created"),
            Err(e) => println!("  ⚠️  Pool account creation failed: {}", e),
        }
    }

    // Create roots ring account if needed
    if !roots_ring_exists {
        println!("Creating roots ring account...");
        let roots_ring_rent = client.get_minimum_balance_for_rent_exemption(ROOTS_RING_SIZE)?;
        let create_roots_ring_ix = system_instruction::create_account_with_seed(
            &admin_keypair.pubkey(),
            &roots_ring_pda,
            &admin_keypair.pubkey(),
            "roots_ring",
            roots_ring_rent,
            ROOTS_RING_SIZE as u64,
            &program_id,
        );

        let mut tx =
            Transaction::new_with_payer(&[create_roots_ring_ix], Some(&admin_keypair.pubkey()));
        let blockhash = client.get_latest_blockhash()?;
        tx.sign(&[&admin_keypair], blockhash);

        match client.send_and_confirm_transaction(&tx) {
            Ok(_) => println!("  ✅ Roots ring account created"),
            Err(e) => println!("  ⚠️  Roots ring creation failed: {}", e),
        }
    }

    // Create nullifier shard account if needed
    if !nullifier_shard_exists {
        println!("Creating nullifier shard account...");
        let nullifier_shard_rent =
            client.get_minimum_balance_for_rent_exemption(NULLIFIER_SHARD_SIZE)?;
        let create_nullifier_shard_ix = system_instruction::create_account_with_seed(
            &admin_keypair.pubkey(),
            &nullifier_shard_pda,
            &admin_keypair.pubkey(),
            "nullifier_shard",
            nullifier_shard_rent,
            NULLIFIER_SHARD_SIZE as u64,
            &program_id,
        );

        let mut tx = Transaction::new_with_payer(
            &[create_nullifier_shard_ix],
            Some(&admin_keypair.pubkey()),
        );
        let blockhash = client.get_latest_blockhash()?;
        tx.sign(&[&admin_keypair], blockhash);

        match client.send_and_confirm_transaction(&tx) {
            Ok(_) => println!("  ✅ Nullifier shard account created"),
            Err(e) => println!("  ⚠️  Nullifier shard creation failed: {}", e),
        }
    }

    // Create treasury account if needed
    if !treasury_exists {
        println!("Creating treasury account...");
        let treasury_rent = client.get_minimum_balance_for_rent_exemption(0)?;
        let create_treasury_ix = system_instruction::create_account_with_seed(
            &admin_keypair.pubkey(),
            &treasury_pda,
            &admin_keypair.pubkey(),
            "treasury",
            treasury_rent,
            0,
            &solana_sdk::system_program::id(),
        );

        let mut tx =
            Transaction::new_with_payer(&[create_treasury_ix], Some(&admin_keypair.pubkey()));
        let blockhash = client.get_latest_blockhash()?;
        tx.sign(&[&admin_keypair], blockhash);

        match client.send_and_confirm_transaction(&tx) {
            Ok(_) => println!("  ✅ Treasury account created"),
            Err(e) => println!("  ⚠️  Treasury creation failed: {}", e),
        }
    }

    println!("\n✅ Localnet initialization complete!");
    println!("\n📋 Account Summary:");
    println!("  Pool PDA:            {}", pool_pda);
    println!("  Roots Ring PDA:      {}", roots_ring_pda);
    println!("  Nullifier Shard PDA: {}", nullifier_shard_pda);
    println!("  Treasury PDA:        {}", treasury_pda);

    Ok(())
}

fn load_keypair(path: &str) -> Result<Keypair> {
    let keypair_bytes = std::fs::read(path)?;
    let keypair_json: Vec<u8> = serde_json::from_slice(&keypair_bytes)?;
    Ok(Keypair::from_bytes(&keypair_json)?)
}
