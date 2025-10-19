//! End-to-end PoW mining example
//!
//! Demonstrates the complete flow:
//! 1. Initialize ClaimManager
//! 2. Mine and reveal a claim
//! 3. Use the claim
//!
//! NOTE: Requires deployed scramble-registry program on devnet/localnet
//!
//! Run with:
//!   SOLANA_RPC_URL=<url> \
//!   SCRAMBLE_PROGRAM_ID=<program_id> \
//!   MINER_KEYPAIR_PATH=<path> \
//!   cargo run --package relay --example end_to_end_mining

use relay::miner::ClaimManager;
use solana_sdk::signature::read_keypair_file;

#[tokio::main]
async fn main() {
    // Setup tracing
    tracing_subscriber::fmt::init();

    println!("=== End-to-End PoW Mining Example ===\n");

    // Read configuration from environment
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8899".to_string());

    let program_id = std::env::var("SCRAMBLE_PROGRAM_ID")
        .unwrap_or_else(|_| {
            eprintln!("SCRAMBLE_PROGRAM_ID not set, using placeholder");
            "11111111111111111111111111111111".to_string()
        });

    let keypair_path = std::env::var("MINER_KEYPAIR_PATH")
        .unwrap_or_else(|_| {
            eprintln!("MINER_KEYPAIR_PATH not set, using default");
            "~/.config/solana/id.json".to_string()
        });

    println!("Configuration:");
    println!("  RPC URL: {}", rpc_url);
    println!("  Program ID: {}", program_id);
    println!("  Keypair: {}\n", keypair_path);

    // Load miner keypair
    let miner_keypair = match read_keypair_file(&keypair_path) {
        Ok(kp) => {
            println!("✓ Loaded miner keypair: {}", kp.pubkey());
            kp
        }
        Err(e) => {
            eprintln!("✗ Failed to load keypair: {}", e);
            eprintln!("\nGenerate a keypair with:");
            eprintln!("  solana-keygen new -o {}", keypair_path);
            return;
        }
    };

    // Initialize ClaimManager
    println!("\n1. Initializing ClaimManager...");
    let mut manager = match ClaimManager::new(
        rpc_url.clone(),
        miner_keypair,
        &program_id,
        30, // 30 second mining timeout
    ) {
        Ok(m) => {
            println!("   ✓ ClaimManager initialized");
            println!("   Miner pubkey: {}", m.miner_pubkey());
            m
        }
        Err(e) => {
            eprintln!("   ✗ Failed to initialize: {}", e);
            return;
        }
    };

    // Mine claim for a test job
    let job_id = "test-withdraw-12345";
    println!("\n2. Mining claim for job: {}", job_id);
    println!("   This will:");
    println!("   - Fetch registry difficulty");
    println!("   - Fetch recent SlotHash");
    println!("   - Mine for valid nonce");
    println!("   - Submit mine_claim transaction");
    println!("   - Submit reveal_claim transaction");
    println!("   (This may take 10-30 seconds depending on difficulty)\n");

    match manager.get_claim_for_job(job_id).await {
        Ok(claim_pda) => {
            println!("\n   ✓ Claim ready!");
            println!("   Claim PDA: {}", claim_pda);
            println!("\n3. Claim can now be consumed in withdraw transaction");
            println!("   The shield-pool withdraw instruction will:");
            println!("   - Include consume_claim CPI");
            println!("   - Verify claim is valid and not expired");
            println!("   - Decrement consumed_count");
        }
        Err(e) => {
            eprintln!("\n   ✗ Mining failed: {}", e);
            eprintln!("\nPossible causes:");
            eprintln!("- RPC node not reachable");
            eprintln!("- Program not deployed");
            eprintln!("- Registry not initialized");
            eprintln!("- Insufficient SOL for transaction fees");
        }
    }

    println!("\n=== Example Complete ===");
}
