use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

use super::Output;
use crate::error::Error;

/// Build a withdraw instruction for the shield-pool program
pub fn build_withdraw_instruction(
    program_id: &Pubkey,
    proof_bytes: &[u8],
    public_inputs: &[u8],
    outputs: &[Output],
    recent_blockhash: Hash,
) -> Result<Transaction, Error> {
    // For now, this is a placeholder implementation
    // In a real implementation, this would need to:
    // 1. Create proper accounts based on the shield-pool program interface
    // 2. Serialize proof and public inputs according to the program's expected format
    // 3. Handle all the program-specific account derivations

    // Convert outputs to account metas
    let mut accounts = Vec::new();
    
    // Add system accounts that are typically needed
    accounts.push(AccountMeta::new_readonly(system_program::id(), false));
    
    // Add output recipient accounts
    for output in outputs {
        let recipient_pubkey = output.to_pubkey()?;
        accounts.push(AccountMeta::new(recipient_pubkey, false));
    }

    // Create instruction data
    // This would need to match the actual shield-pool program interface
    let mut instruction_data = Vec::new();
    
    // Instruction discriminator (8 bytes for anchor programs)
    // This would be the hash of "global:withdraw"
    instruction_data.extend_from_slice(&[0u8; 8]); // Placeholder
    
    // Proof bytes length + proof
    instruction_data.extend_from_slice(&(proof_bytes.len() as u32).to_le_bytes());
    instruction_data.extend_from_slice(proof_bytes);
    
    // Public inputs length + data
    instruction_data.extend_from_slice(&(public_inputs.len() as u32).to_le_bytes());
    instruction_data.extend_from_slice(public_inputs);
    
    // Outputs count
    instruction_data.extend_from_slice(&(outputs.len() as u32).to_le_bytes());
    
    // Serialize outputs
    for output in outputs {
        let recipient_pubkey = output.to_pubkey()?;
        instruction_data.extend_from_slice(recipient_pubkey.as_ref());
        instruction_data.extend_from_slice(&output.amount.to_le_bytes());
    }

    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    };

    // Create transaction (unsigned for now)
    let transaction = Transaction::new_unsigned(
        recent_blockhash,
        &[instruction],
    );

    Ok(transaction)
}

/// Parse public inputs according to the circuit specification
pub fn parse_public_inputs(public_inputs: &[u8]) -> Result<PublicInputs, Error> {
    if public_inputs.len() != 64 {
        return Err(Error::ValidationError(
            "Public inputs must be exactly 64 bytes".to_string()
        ));
    }

    // Based on docs/zk/circuit-withdraw.md:
    // Public inputs: root:32, nf:32, fee_bps:u16, outputs_hash:32, amount:u64
    // But our validation shows 64 bytes total, so let's parse accordingly
    
    let mut offset = 0;
    
    // Root hash (32 bytes)
    let root = public_inputs[offset..offset + 32].try_into()
        .map_err(|_| Error::ValidationError("Invalid root hash".to_string()))?;
    offset += 32;
    
    // Nullifier (32 bytes) 
    let nullifier = public_inputs[offset..offset + 32].try_into()
        .map_err(|_| Error::ValidationError("Invalid nullifier".to_string()))?;

    // For a 64-byte public input, we'd need to adjust this based on actual circuit layout
    // This is a placeholder that assumes the layout might be different
    
    Ok(PublicInputs {
        root,
        nullifier,
        // These would need to be extracted from the remaining bytes based on actual layout
        fee_bps: 0,
        outputs_hash: [0u8; 32],
        amount: 0,
    })
}

#[derive(Debug, Clone)]
pub struct PublicInputs {
    pub root: [u8; 32],
    pub nullifier: [u8; 32],
    pub fee_bps: u16,
    pub outputs_hash: [u8; 32],
    pub amount: u64,
}

impl PublicInputs {
    /// Calculate outputs hash from a list of outputs
    pub fn calculate_outputs_hash(outputs: &[Output]) -> Result<[u8; 32], Error> {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        
        for output in outputs {
            let recipient_pubkey = output.to_pubkey()?;
            // Serialize according to docs/zk/encoding.md
            hasher.update(recipient_pubkey.as_ref()); // address:32
            hasher.update(&output.amount.to_le_bytes()); // amount:u64
        }
        
        let hash = hasher.finalize();
        Ok(*hash.as_bytes())
    }
    
    /// Validate that the outputs hash matches the provided outputs
    pub fn validate_outputs_hash(&self, outputs: &[Output]) -> Result<(), Error> {
        let calculated_hash = Self::calculate_outputs_hash(outputs)?;
        
        if calculated_hash != self.outputs_hash {
            return Err(Error::ValidationError(
                "Outputs hash mismatch".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate conservation: sum(outputs) + fee == amount
    pub fn validate_conservation(&self, outputs: &[Output]) -> Result<(), Error> {
        let outputs_sum: u64 = outputs.iter().map(|o| o.amount).sum();
        let fee = self.amount * self.fee_bps as u64 / 10_000;
        let expected_total = outputs_sum + fee;
        
        if expected_total != self.amount {
            return Err(Error::ValidationError(format!(
                "Conservation check failed: {} + {} != {}",
                outputs_sum, fee, self.amount
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_public_inputs() {
        let public_inputs = vec![0u8; 64];
        let parsed = parse_public_inputs(&public_inputs);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_invalid_public_inputs_length() {
        let public_inputs = vec![0u8; 63]; // Wrong length
        let parsed = parse_public_inputs(&public_inputs);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_outputs_hash_calculation() {
        let outputs = vec![
            Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 1000000,
            },
            Output {
                recipient: "11111111111111111111111111111113".to_string(),
                amount: 2000000,
            },
        ];

        let hash1 = PublicInputs::calculate_outputs_hash(&outputs);
        let hash2 = PublicInputs::calculate_outputs_hash(&outputs);
        
        assert!(hash1.is_ok());
        assert!(hash2.is_ok());
        assert_eq!(hash1.unwrap(), hash2.unwrap()); // Should be deterministic
    }

    #[test]
    fn test_conservation_validation() {
        let public_inputs = PublicInputs {
            root: [0u8; 32],
            nullifier: [0u8; 32],
            fee_bps: 100, // 1%
            outputs_hash: [0u8; 32],
            amount: 1000000, // 1 SOL
        };

        let outputs = vec![
            Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 990000, // 0.99 SOL (1% fee = 10,000 lamports)
            },
        ];

        assert!(public_inputs.validate_conservation(&outputs).is_ok());

        // Test invalid conservation
        let invalid_outputs = vec![
            Output {
                recipient: "11111111111111111111111111111112".to_string(),
                amount: 1000000, // Too much - doesn't account for fee
            },
        ];

        assert!(public_inputs.validate_conservation(&invalid_outputs).is_err());
    }
} 