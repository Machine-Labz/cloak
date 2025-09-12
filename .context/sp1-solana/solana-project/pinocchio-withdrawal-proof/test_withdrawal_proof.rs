//! Test script for the Pinocchio withdrawal proof verifier

use pinocchio_withdrawal_proof_verifier_contract::{
    process_instruction, SP1Groth16WithdrawalProof, WITHDRAWAL_PROOF_VKEY_HASH,
};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
};
use five8_const::decode_32_const;

fn main() {
    println!("Testing Pinocchio Withdrawal Proof Verifier");
    println!("===========================================");

    // Test 1: Valid withdrawal proof
    test_valid_withdrawal_proof();
    
    // Test 2: Invalid instruction data length
    test_invalid_data_length();
    
    // Test 3: Insufficient balance
    test_insufficient_balance();
    
    // Test 4: Excessive withdrawal amount
    test_excessive_withdrawal();
    
    println!("\nAll tests completed!");
}

fn test_valid_withdrawal_proof() {
    println!("\nTest 1: Valid withdrawal proof");
    
    let program_id = Pubkey::from(decode_32_const(
        "99999999999999999999999999999999999999999999",
    ));
    let accounts = vec![];
    
    // Create valid withdrawal proof data
    let mut instruction_data = vec![0u8; SP1Groth16WithdrawalProof::LEN];
    
    // Mock Groth16 proof (260 bytes of zeros for testing)
    // In real usage, this would be a valid Groth16 proof
    
    // Set withdrawal data
    let user_address = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 
                       0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 
                       0x99, 0xaa, 0xbb, 0xcc];
    let pool_id = 12345u64;
    let user_balance = 1000000u64;
    let withdrawal_amount = 100000u64;
    let pool_liquidity = 5000000u64;
    let timestamp = 1700000000u64;
    let is_valid = true;
    
    // Pack the data
    instruction_data[260..280].copy_from_slice(&user_address);
    instruction_data[280..288].copy_from_slice(&pool_id.to_le_bytes());
    instruction_data[288..296].copy_from_slice(&user_balance.to_le_bytes());
    instruction_data[296..304].copy_from_slice(&withdrawal_amount.to_le_bytes());
    instruction_data[304..312].copy_from_slice(&pool_liquidity.to_le_bytes());
    instruction_data[312..320].copy_from_slice(&timestamp.to_le_bytes());
    instruction_data[320] = is_valid as u8;
    
    // Pack SP1 public inputs (12 bytes for 3 u32 values)
    // We'll use pool_id, user_balance, and withdrawal_amount as u32
    instruction_data[260..264].copy_from_slice(&(pool_id as u32).to_le_bytes());
    instruction_data[264..268].copy_from_slice(&(user_balance as u32).to_le_bytes());
    instruction_data[268..272].copy_from_slice(&(withdrawal_amount as u32).to_le_bytes());
    
    let result = process_instruction(&program_id, &accounts, &instruction_data);
    
    match result {
        Ok(_) => println!("✅ Valid withdrawal proof test passed"),
        Err(e) => println!("❌ Valid withdrawal proof test failed: {:?}", e),
    }
}

fn test_invalid_data_length() {
    println!("\nTest 2: Invalid instruction data length");
    
    let program_id = Pubkey::from(decode_32_const(
        "99999999999999999999999999999999999999999999",
    ));
    let accounts = vec![];
    let invalid_data = vec![1, 2, 3]; // Too short
    
    let result = process_instruction(&program_id, &accounts, &invalid_data);
    
    match result {
        Ok(_) => println!("❌ Invalid data length test failed - should have errored"),
        Err(ProgramError::InvalidInstructionData) => {
            println!("✅ Invalid data length test passed - correctly rejected");
        }
        Err(e) => println!("❌ Invalid data length test failed with unexpected error: {:?}", e),
    }
}

fn test_insufficient_balance() {
    println!("\nTest 3: Insufficient balance");
    
    let program_id = Pubkey::from(decode_32_const(
        "99999999999999999999999999999999999999999999",
    ));
    let accounts = vec![];
    
    let mut instruction_data = vec![0u8; SP1Groth16WithdrawalProof::LEN];
    
    // Set up data with insufficient balance
    let user_balance = 50000u64;  // Less than withdrawal amount
    let withdrawal_amount = 100000u64;
    let pool_liquidity = 5000000u64;
    let is_valid = true; // Even if SP1 says valid, on-chain validation should catch this
    
    instruction_data[280..288].copy_from_slice(&12345u64.to_le_bytes()); // pool_id
    instruction_data[288..296].copy_from_slice(&user_balance.to_le_bytes());
    instruction_data[296..304].copy_from_slice(&withdrawal_amount.to_le_bytes());
    instruction_data[304..312].copy_from_slice(&pool_liquidity.to_le_bytes());
    instruction_data[312..320].copy_from_slice(&1700000000u64.to_le_bytes()); // timestamp
    instruction_data[320] = is_valid as u8;
    
    // Pack SP1 public inputs
    instruction_data[260..264].copy_from_slice(&12345u32.to_le_bytes());
    instruction_data[264..268].copy_from_slice(&(user_balance as u32).to_le_bytes());
    instruction_data[268..272].copy_from_slice(&(withdrawal_amount as u32).to_le_bytes());
    
    let result = process_instruction(&program_id, &accounts, &instruction_data);
    
    match result {
        Ok(_) => println!("❌ Insufficient balance test failed - should have errored"),
        Err(ProgramError::InvalidInstructionData) => {
            println!("✅ Insufficient balance test passed - correctly rejected");
        }
        Err(e) => println!("❌ Insufficient balance test failed with unexpected error: {:?}", e),
    }
}

fn test_excessive_withdrawal() {
    println!("\nTest 4: Excessive withdrawal amount");
    
    let program_id = Pubkey::from(decode_32_const(
        "99999999999999999999999999999999999999999999",
    ));
    let accounts = vec![];
    
    let mut instruction_data = vec![0u8; SP1Groth16WithdrawalProof::LEN];
    
    // Set up data with excessive withdrawal (more than 50% of pool)
    let user_balance = 10000000u64;
    let withdrawal_amount = 3000000u64;  // More than 50% of pool
    let pool_liquidity = 5000000u64;
    let is_valid = true;
    
    instruction_data[280..288].copy_from_slice(&12345u64.to_le_bytes()); // pool_id
    instruction_data[288..296].copy_from_slice(&user_balance.to_le_bytes());
    instruction_data[296..304].copy_from_slice(&withdrawal_amount.to_le_bytes());
    instruction_data[304..312].copy_from_slice(&pool_liquidity.to_le_bytes());
    instruction_data[312..320].copy_from_slice(&1700000000u64.to_le_bytes()); // timestamp
    instruction_data[320] = is_valid as u8;
    
    // Pack SP1 public inputs
    instruction_data[260..264].copy_from_slice(&12345u32.to_le_bytes());
    instruction_data[264..268].copy_from_slice(&(user_balance as u32).to_le_bytes());
    instruction_data[268..272].copy_from_slice(&(withdrawal_amount as u32).to_le_bytes());
    
    let result = process_instruction(&program_id, &accounts, &instruction_data);
    
    match result {
        Ok(_) => println!("❌ Excessive withdrawal test failed - should have errored"),
        Err(ProgramError::InvalidInstructionData) => {
            println!("✅ Excessive withdrawal test passed - correctly rejected");
        }
        Err(e) => println!("❌ Excessive withdrawal test failed with unexpected error: {:?}", e),
    }
}
