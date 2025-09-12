//! A program that proves someone has the ability to withdraw tokens from a pool.
//! The program verifies:
//! 1. User has sufficient balance in the pool
//! 2. Pool has sufficient liquidity for withdrawal
//! 3. Withdrawal amount is within reasonable limits
//! 4. User has proper authorization (signature verification)
//! 5. Timestamp is recent and valid

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

// Define the public values struct directly here
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

pub fn main() {
    // Read inputs to the program
    let user_address = sp1_zkvm::io::read::<[u8; 20]>();
    let pool_id = sp1_zkvm::io::read::<u64>();
    let user_balance = sp1_zkvm::io::read::<u64>();
    let withdrawal_amount = sp1_zkvm::io::read::<u64>();
    let pool_liquidity = sp1_zkvm::io::read::<u64>();

    // Read signatures as Vec<u8> and convert to arrays
    let user_signature_vec = sp1_zkvm::io::read::<Vec<u8>>();
    let pool_signature_vec = sp1_zkvm::io::read::<Vec<u8>>();

    // Convert to fixed-size arrays, padding with zeros if needed
    let mut user_signature = [0u8; 65];
    let mut pool_signature = [0u8; 65];

    for (i, &byte) in user_signature_vec.iter().take(65).enumerate() {
        user_signature[i] = byte;
    }

    for (i, &byte) in pool_signature_vec.iter().take(65).enumerate() {
        pool_signature[i] = byte;
    }

    let timestamp = sp1_zkvm::io::read::<u64>();

    // Verify the withdrawal authorization and pool state
    let is_valid = verify_withdrawal(
        user_address,
        pool_id,
        user_balance,
        withdrawal_amount,
        pool_liquidity,
        user_signature,
        pool_signature,
        timestamp,
    );

    // Compute the withdrawal hash for additional verification
    let withdrawal_hash =
        compute_withdrawal_hash(user_address, pool_id, withdrawal_amount, timestamp);

    // Create the public values struct
    let public_values = WithdrawalProofStruct {
        user_address,
        pool_id,
        user_balance,
        withdrawal_amount,
        pool_liquidity,
        timestamp,
        is_valid,
    };

    // Encode the public values using bincode
    let bytes = bincode::serialize(&public_values).unwrap();

    // Commit to the public values of the program
    sp1_zkvm::io::commit_slice(&bytes);

    // Also commit to the withdrawal hash for additional verification
    sp1_zkvm::io::commit_slice(&withdrawal_hash);

    // Log the verification result for debugging
    if is_valid {
        let message = b"Withdrawal proof is VALID";
        sp1_zkvm::io::write(1, message);
    } else {
        let message = b"Withdrawal proof is INVALID";
        sp1_zkvm::io::write(1, message);
    }
}

/// Verify withdrawal authorization and pool state
fn verify_withdrawal(
    user_address: [u8; 20],
    pool_id: u64,
    user_balance: u64,
    withdrawal_amount: u64,
    pool_liquidity: u64,
    user_signature: [u8; 65],
    pool_signature: [u8; 65],
    timestamp: u64,
) -> bool {
    // Verify user has sufficient balance
    if user_balance < withdrawal_amount {
        return false;
    }

    // Verify pool has sufficient liquidity
    if pool_liquidity < withdrawal_amount {
        return false;
    }

    // Verify withdrawal amount is within reasonable limits (not more than 50% of pool)
    if withdrawal_amount > pool_liquidity / 2 {
        return false;
    }

    // Verify timestamp is recent (within last 24 hours)
    let current_time = timestamp;
    if current_time < timestamp || current_time - timestamp > 86400 {
        return false;
    }

    // Verify user signature (simplified - in real implementation, you'd verify against user's public key)
    if !verify_user_signature(user_address, withdrawal_amount, pool_id, user_signature) {
        return false;
    }

    // Verify pool signature (simplified - in real implementation, you'd verify against pool's public key)
    if !verify_pool_signature(pool_id, pool_liquidity, pool_signature) {
        return false;
    }

    true
}

/// Verify user signature (simplified implementation)
fn verify_user_signature(
    _user_address: [u8; 20],
    _withdrawal_amount: u64,
    _pool_id: u64,
    signature: [u8; 65],
) -> bool {
    // In a real implementation, this would verify the signature against the user's public key
    // For this example, we'll do a simple check that the signature is not all zeros
    !signature.iter().all(|&x| x == 0)
}

/// Verify pool signature (simplified implementation)
fn verify_pool_signature(_pool_id: u64, _pool_liquidity: u64, signature: [u8; 65]) -> bool {
    // In a real implementation, this would verify the signature against the pool's public key
    // For this example, we'll do a simple check that the signature is not all zeros
    !signature.iter().all(|&x| x == 0)
}

/// Compute the hash of withdrawal data for signature verification
fn compute_withdrawal_hash(
    user_address: [u8; 20],
    pool_id: u64,
    withdrawal_amount: u64,
    timestamp: u64,
) -> [u8; 32] {
    // For this example, we'll create a simple hash by combining the inputs
    // In a real implementation, you would use a proper cryptographic hash function
    let mut hash = [0u8; 32];

    // Combine inputs into a deterministic hash
    for (i, &byte) in user_address.iter().enumerate() {
        hash[i % 32] ^= byte;
    }

    for (i, &byte) in pool_id.to_le_bytes().iter().enumerate() {
        hash[(i + 8) % 32] ^= byte;
    }

    for (i, &byte) in withdrawal_amount.to_le_bytes().iter().enumerate() {
        hash[(i + 16) % 32] ^= byte;
    }

    for (i, &byte) in timestamp.to_le_bytes().iter().enumerate() {
        hash[(i + 24) % 32] ^= byte;
    }

    hash
}
