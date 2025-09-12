//! Generate JSON proof data for withdrawal WASM verifier

use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin, HashableKey};

// Define the public values struct for withdrawal proof
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalProofStruct {
    pub user_address: [u8; 20],
    pub pool_id: u64,
    pub user_balance: u64,
    pub withdrawal_amount: u64,
    pub pool_liquidity: u64,
    pub timestamp: u64,
    pub is_valid: bool,
}

// JSON proof data structure for WASM verifier
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WithdrawalProofData {
    pub proof: String,           // Hex-encoded proof bytes
    pub public_inputs: String,   // Hex-encoded public inputs
    pub vkey_hash: String,       // Verification key hash
    pub mode: String,            // "groth16" or "plonk"
    pub user_address: String,    // Hex-encoded user address
    pub pool_id: u64,
    pub user_balance: u64,
    pub withdrawal_amount: u64,
    pub pool_liquidity: u64,
    pub timestamp: u64,
    pub is_valid: bool,
}

// We'll load the ELF from the existing proof instead of using include_elf

#[derive(Parser)]
#[command(name = "withdrawal-json-generator")]
#[command(about = "Generate JSON proof data for withdrawal WASM verifier")]
struct Cli {
    #[arg(long, default_value = "groth16")]
    mode: String,
    
    #[arg(long)]
    prove: bool,
    
    #[arg(long, default_value = "1000000")]
    user_balance: u64,
    
    #[arg(long, default_value = "100000")]
    withdrawal_amount: u64,
    
    #[arg(long, default_value = "5000000")]
    pool_liquidity: u64,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    
    let args = Cli::parse();
    
    // Validate mode
    if args.mode != "groth16" && args.mode != "plonk" {
        eprintln!("Error: mode must be either 'groth16' or 'plonk'");
        std::process::exit(1);
    }
    
    // Create test data for withdrawal proof
    let user_address = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99, 0xaa, 0xbb, 0xcc,
    ];
    let pool_id = 12345u64;
    let user_balance = args.user_balance;
    let withdrawal_amount = args.withdrawal_amount;
    let pool_liquidity = args.pool_liquidity;
    let user_signature = vec![0x01; 65]; // Mock signature
    let pool_signature = vec![0x02; 65]; // Mock signature
    let timestamp = 1700000000u64; // Mock timestamp
    
    // Setup the inputs
    let mut stdin = SP1Stdin::new();
    stdin.write(&user_address);
    stdin.write(&pool_id);
    stdin.write(&user_balance);
    stdin.write(&withdrawal_amount);
    stdin.write(&pool_liquidity);
    stdin.write(&user_signature);
    stdin.write(&pool_signature);
    stdin.write(&timestamp);
    
    println!("Generating withdrawal proof data...");
    println!("Mode: {}", args.mode);
    println!("User Address: {:?}", user_address);
    println!("Pool ID: {}", pool_id);
    println!("User Balance: {}", user_balance);
    println!("Withdrawal Amount: {}", withdrawal_amount);
    println!("Pool Liquidity: {}", pool_liquidity);
    
    // For now, we'll just work with existing proofs since Groth16 generation has Docker issues
    println!("Note: This generator works with existing compressed proofs");
    println!("To generate new Groth16/PLONK proofs, use the withdrawal-proof script");
    
    // Load existing proof
    let proof_path = format!("../withdrawal_proof.bin");
    let vk_path = format!("../withdrawal_vk.bin");
    
    if !std::path::Path::new(&proof_path).exists() {
        eprintln!("Error: Proof file not found at {}", proof_path);
        eprintln!("Run the withdrawal-proof script first to generate a proof");
        std::process::exit(1);
    }
    
    let proof_bytes = std::fs::read(&proof_path).expect("Failed to read proof file");
    let vk_bytes = std::fs::read(&vk_path).expect("Failed to read vk file");
    
    // For compressed proofs, we'll create a mock structure since they can't be directly verified
    // In a real implementation, you would need to implement compressed proof verification
    let proof_data = WithdrawalProofData {
        proof: hex::encode(&proof_bytes), // Use the raw proof bytes
        public_inputs: hex::encode(&[0u8; 32]), // Mock public inputs for compressed proofs
        vkey_hash: format!("0x{}", "0".repeat(64)), // Mock vkey hash for compressed proofs
        mode: "compressed".to_string(),
        user_address: hex::encode(user_address),
        pool_id,
        user_balance,
        withdrawal_amount,
        pool_liquidity,
        timestamp,
        is_valid: true,
    };
    
    // Save to JSON file
    let json_path = format!("../withdrawal-json/withdrawal_compressed_proof.json");
    std::fs::create_dir_all("../withdrawal-json").expect("Failed to create json directory");
    
    let json_data = serde_json::to_string_pretty(&proof_data).expect("Failed to serialize proof data");
    std::fs::write(&json_path, json_data).expect("Failed to write JSON file");
    
    println!("Proof data saved to: {}", json_path);
    println!("Note: Using compressed proof (not directly verifiable in WASM)");
}
