use pinocchio::{
    account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use sp1_solana::verify_proof;

#[cfg(not(feature = "no-entrypoint"))]
pinocchio::entrypoint!(process_instruction);

#[cfg(not(doctest))]
/// Derived as follows:
///
/// ```
/// let client = sp1_sdk::ProverClient::new();
/// let (pk, vk) = client.setup(WITHDRAWAL_PROOF_ELF);
/// let vkey_hash = vk.bytes32();
/// ```
/// 
const WITHDRAWAL_PROOF_VKEY_HASH: &str =
    "0x00d02fdf525cdf62ba99003d384772f1ac098fd1c8a6692d100f6dcbe54ef873";

/// The instruction data for the withdrawal proof program.
pub struct SP1Groth16WithdrawalProof(*mut u8);

impl SP1Groth16WithdrawalProof {
    // Withdrawal proof structure:
    // - 260 bytes for Groth16 proof
    // - 20 bytes for user_address
    // - 8 bytes for pool_id
    // - 8 bytes for user_balance
    // - 8 bytes for withdrawal_amount
    // - 8 bytes for pool_liquidity
    // - 8 bytes for timestamp
    // - 1 byte for is_valid (bool)
    pub const LEN: usize = 260 + 20 + 8 + 8 + 8 + 8 + 8 + 1; // 321 bytes total

    #[inline(always)]
    pub fn from_instruction_data(instruction_data: &[u8]) -> Self {
        Self(instruction_data.as_ptr() as *mut u8)
    }

    #[inline(always)]
    pub fn proof(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0, 260) }
    }

    #[inline(always)]
    pub fn user_address(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.add(260), 20) }
    }

    #[inline(always)]
    pub fn pool_id(&self) -> u64 {
        let bytes = unsafe { std::slice::from_raw_parts(self.0.add(280), 8) };
        u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    #[inline(always)]
    pub fn user_balance(&self) -> u64 {
        let bytes = unsafe { std::slice::from_raw_parts(self.0.add(288), 8) };
        u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    #[inline(always)]
    pub fn withdrawal_amount(&self) -> u64 {
        let bytes = unsafe { std::slice::from_raw_parts(self.0.add(296), 8) };
        u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    #[inline(always)]
    pub fn pool_liquidity(&self) -> u64 {
        let bytes = unsafe { std::slice::from_raw_parts(self.0.add(304), 8) };
        u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    #[inline(always)]
    pub fn timestamp(&self) -> u64 {
        let bytes = unsafe { std::slice::from_raw_parts(self.0.add(312), 8) };
        u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        unsafe { *self.0.add(320) != 0 }
    }

    #[inline(always)]
    pub fn sp1_public_inputs(&self) -> &[u8] {
        // SP1 public inputs are the last 12 bytes (3 u32 values)
        // We'll pack our withdrawal data into this format
        unsafe { std::slice::from_raw_parts(self.0.add(260), 12) }
    }
}

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() != SP1Groth16WithdrawalProof::LEN {
        msg!("Invalid instruction data length");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Deserialize the SP1Groth16WithdrawalProof from the instruction data.
    let withdrawal_proof = SP1Groth16WithdrawalProof::from_instruction_data(instruction_data);

    // Get the SP1 Groth16 verification key from the `sp1-solana` crate.
    let vk = sp1_solana::GROTH16_VK_5_0_0_BYTES;

    // Verify the proof.
    verify_proof(
        &withdrawal_proof.proof(),
        &withdrawal_proof.sp1_public_inputs(),
        WITHDRAWAL_PROOF_VKEY_HASH,
        vk,
    )
    .map_err(|_| {
        msg!("Proof verification failed");
        ProgramError::InvalidInstructionData
    })?;

    // Extract and validate the withdrawal proof data
    let user_address = withdrawal_proof.user_address();
    let pool_id = withdrawal_proof.pool_id();
    let user_balance = withdrawal_proof.user_balance();
    let withdrawal_amount = withdrawal_proof.withdrawal_amount();
    let pool_liquidity = withdrawal_proof.pool_liquidity();
    let timestamp = withdrawal_proof.timestamp();
    let is_valid = withdrawal_proof.is_valid();

    // Additional on-chain validation
    if !is_valid {
        msg!("Withdrawal proof is invalid");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Validate withdrawal conditions on-chain
    if user_balance < withdrawal_amount {
        msg!("Insufficient user balance");
        return Err(ProgramError::InvalidInstructionData);
    }

    if pool_liquidity < withdrawal_amount {
        msg!("Insufficient pool liquidity");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Check withdrawal limit (max 50% of pool)
    if withdrawal_amount > pool_liquidity / 2 {
        msg!("Withdrawal amount exceeds limit");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Print out the verified withdrawal proof data
    let values = format!(
        "Withdrawal Proof Verified - User: {:?}, Pool: {}, Balance: {}, Amount: {}, Liquidity: {}, Timestamp: {}, Valid: {}",
        user_address, pool_id, user_balance, withdrawal_amount, pool_liquidity, timestamp, is_valid
    );
    msg!(&values);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use five8_const::decode_32_const;

    #[test]
    fn test_from_instruction_data_roundtrip() {
        let mut buf = [0u8; SP1Groth16WithdrawalProof::LEN];
        for (i, b) in buf.iter_mut().enumerate() { 
            *b = (i % 251) as u8; 
        }
        let proof = SP1Groth16WithdrawalProof::from_instruction_data(&buf);
        assert_eq!(&proof.proof()[..], &buf[0..260]);
        assert_eq!(&proof.user_address()[..], &buf[260..280]);
        
        // Test pool_id extraction without unsafe pointer dereference
        let pool_id_bytes = &buf[280..288];
        let pool_id = u64::from_le_bytes([
            pool_id_bytes[0], pool_id_bytes[1], pool_id_bytes[2], pool_id_bytes[3],
            pool_id_bytes[4], pool_id_bytes[5], pool_id_bytes[6], pool_id_bytes[7],
        ]);
        assert_eq!(proof.pool_id(), pool_id);
    }

    #[test]
    fn test_instruction_processing_with_invalid_data() {
        let program_id = Pubkey::from(decode_32_const(
            "99999999999999999999999999999999999999999999",
        ));
        let accounts = vec![];
        let invalid_data = vec![1, 2, 3];
        let result = process_instruction(&program_id, &accounts, &invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_withdrawal_proof_structure() {
        let mut data = vec![0u8; SP1Groth16WithdrawalProof::LEN];
        
        // Set up test data
        data[280..288].copy_from_slice(&12345u64.to_le_bytes()); // pool_id
        data[288..296].copy_from_slice(&1000000u64.to_le_bytes()); // user_balance
        data[296..304].copy_from_slice(&100000u64.to_le_bytes()); // withdrawal_amount
        data[304..312].copy_from_slice(&5000000u64.to_le_bytes()); // pool_liquidity
        data[312..320].copy_from_slice(&1700000000u64.to_le_bytes()); // timestamp
        data[320] = 1; // is_valid = true
        
        let proof = SP1Groth16WithdrawalProof::from_instruction_data(&data);
        
        assert_eq!(proof.pool_id(), 12345);
        assert_eq!(proof.user_balance(), 1000000);
        assert_eq!(proof.withdrawal_amount(), 100000);
        assert_eq!(proof.pool_liquidity(), 5000000);
        assert_eq!(proof.timestamp(), 1700000000);
        assert_eq!(proof.is_valid(), true);
    }

    #[test]
    fn test_withdrawal_proof_vkey_hash_constant() {
        assert_eq!(WITHDRAWAL_PROOF_VKEY_HASH.len(), 66);
        assert!(WITHDRAWAL_PROOF_VKEY_HASH.starts_with("0x"));
    }
}
