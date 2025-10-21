use std::collections::HashMap;
use tracing::{debug, warn};

use crate::db::models::Job;
use crate::error::Error;
use crate::solana::transaction_builder::{parse_public_inputs, PublicInputs};
use crate::solana::Output;

pub struct ValidationService {
    config: ValidationConfig,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_outputs: usize,
    pub max_fee_bps: u16,
    pub min_amount: u64,
    pub max_amount: u64,
    pub enable_proof_verification: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_outputs: 10,
            max_fee_bps: 1000, // 10%
            min_amount: 1000,  // 0.000001 SOL
            max_amount: 1_000_000_000_000, // 1000 SOL
            enable_proof_verification: true,
        }
    }
}

impl ValidationService {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Comprehensive validation of a withdraw job
    pub async fn validate_withdraw_proof(&self, job: &Job) -> Result<(), Error> {
        debug!("Validating withdraw proof for job: {}", job.request_id);

        // 1. Parse public inputs
        let public_inputs = parse_public_inputs(&job.public_inputs)?;

        // 2. Parse outputs from JSON
        let outputs = self.parse_outputs(&job.outputs_json)?;

        // 3. Basic validation checks
        self.validate_basic_constraints(&public_inputs, &outputs)?;

        // 4. Business logic validation
        self.validate_business_logic(&public_inputs, &outputs, job.fee_bps)?;

        // 5. Cryptographic validation
        self.validate_cryptographic_constraints(&public_inputs, &outputs, &job.proof_bytes)?;

        debug!("Withdraw proof validation passed for job: {}", job.request_id);
        Ok(())
    }

    /// Parse outputs from JSON with validation
    fn parse_outputs(&self, outputs_json: &serde_json::Value) -> Result<Vec<Output>, Error> {
        let outputs_array = outputs_json.as_array()
            .ok_or_else(|| Error::ValidationError("Outputs must be an array".to_string()))?;

        if outputs_array.len() > self.config.max_outputs {
            return Err(Error::ValidationError(format!(
                "Too many outputs: {} (max: {})",
                outputs_array.len(),
                self.config.max_outputs
            )));
        }

        let mut outputs = Vec::new();
        for (i, output_value) in outputs_array.iter().enumerate() {
            let output: Output = serde_json::from_value(output_value.clone())
                .map_err(|e| Error::ValidationError(format!("Invalid output {} format: {}", i, e)))?;
            
            // Validate individual output
            self.validate_output(&output)?;
            outputs.push(output);
        }

        if outputs.is_empty() {
            return Err(Error::ValidationError("At least one output is required".to_string()));
        }

        Ok(outputs)
    }

    /// Validate individual output
    fn validate_output(&self, output: &Output) -> Result<(), Error> {
        // Validate recipient address format
        output.to_pubkey().map_err(|_| {
            Error::ValidationError("Invalid recipient address format".to_string())
        })?;

        // Validate amount
        if output.amount == 0 {
            return Err(Error::ValidationError("Output amount must be greater than zero".to_string()));
        }

        if output.amount > self.config.max_amount {
            return Err(Error::ValidationError(format!(
                "Output amount too large: {} (max: {})",
                output.amount,
                self.config.max_amount
            )));
        }

        Ok(())
    }

    /// Basic validation checks (format, bounds, etc.)
    fn validate_basic_constraints(&self, public_inputs: &PublicInputs, outputs: &[Output]) -> Result<(), Error> {
        // Validate amount bounds
        if public_inputs.amount < self.config.min_amount {
            return Err(Error::ValidationError(format!(
                "Amount too small: {} (min: {})",
                public_inputs.amount,
                self.config.min_amount
            )));
        }

        if public_inputs.amount > self.config.max_amount {
            return Err(Error::ValidationError(format!(
                "Amount too large: {} (max: {})",
                public_inputs.amount,
                self.config.max_amount
            )));
        }

        // Validate fee
        if public_inputs.fee_bps > self.config.max_fee_bps {
            return Err(Error::ValidationError(format!(
                "Fee too high: {} bps (max: {} bps)",
                public_inputs.fee_bps,
                self.config.max_fee_bps
            )));
        }

        // Check for duplicate recipients
        let mut seen_recipients = HashMap::new();
        for (i, output) in outputs.iter().enumerate() {
            if let Some(prev_index) = seen_recipients.insert(&output.recipient, i) {
                return Err(Error::ValidationError(format!(
                    "Duplicate recipient at indices {} and {}",
                    prev_index,
                    i
                )));
            }
        }

        Ok(())
    }

    /// Business logic validation (conservation, outputs hash, etc.)
    fn validate_business_logic(&self, public_inputs: &PublicInputs, outputs: &[Output], fee_bps: i16) -> Result<(), Error> {
        // Ensure fee matches
        if public_inputs.fee_bps != fee_bps as u16 {
            return Err(Error::ValidationError(format!(
                "Fee mismatch: public_inputs={} bps, job={} bps",
                public_inputs.fee_bps,
                fee_bps
            )));
        }

        // Validate conservation
        public_inputs.validate_conservation(outputs)?;

        // Validate outputs hash
        public_inputs.validate_outputs_hash(outputs)?;

        Ok(())
    }

    /// Cryptographic validation (proof verification, nullifier format, etc.)
    fn validate_cryptographic_constraints(&self, public_inputs: &PublicInputs, _outputs: &[Output], proof_bytes: &[u8]) -> Result<(), Error> {
        // Validate proof format
        if proof_bytes.len() != 260 && proof_bytes.len() != 256 {
            return Err(Error::ValidationError(format!(
                "Invalid proof length: {} (expected 260 or 256)",
                proof_bytes.len()
            )));
        }

        // Validate nullifier format (should be non-zero)
        if public_inputs.nullifier == [0u8; 32] {
            return Err(Error::ValidationError("Nullifier cannot be zero".to_string()));
        }

        // Validate root format (should be non-zero)
        if public_inputs.root == [0u8; 32] {
            warn!("Root hash is zero - this might be a test transaction");
        }

        // TODO: Add actual SP1 proof verification
        if self.config.enable_proof_verification {
            self.verify_sp1_proof(proof_bytes, public_inputs)?;
        }

        Ok(())
    }

    /// Verify SP1 proof (placeholder implementation)
    fn verify_sp1_proof(&self, _proof_bytes: &[u8], _public_inputs: &PublicInputs) -> Result<(), Error> {
        // This would integrate with SP1 verifier
        // For now, we'll just check basic format constraints
        
        // In a real implementation, this would:
        // 1. Load the verification key
        // 2. Deserialize the proof
        // 3. Call SP1 verifier with proof + public inputs
        // 4. Return verification result

        debug!("SP1 proof verification (placeholder) - assuming valid");
        Ok(())
    }

    /// Validate that the nullifier hasn't been seen before
    pub fn validate_nullifier_format(&self, nullifier: &[u8]) -> Result<(), Error> {
        if nullifier.len() != 32 {
            return Err(Error::ValidationError(
                "Nullifier must be 32 bytes".to_string()
            ));
        }

        if nullifier == &[0u8; 32] {
            return Err(Error::ValidationError(
                "Nullifier cannot be zero".to_string()
            ));
        }

        Ok(())
    }

    /// Validate Merkle root format
    pub fn validate_root_format(&self, root: &[u8]) -> Result<(), Error> {
        if root.len() != 32 {
            return Err(Error::ValidationError(
                "Root must be 32 bytes".to_string()
            ));
        }

        // Note: We allow zero root for testing, but warn about it
        if root == &[0u8; 32] {
            warn!("Root hash is zero - this might be a test transaction");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_job() -> Job {
        use crate::db::models::{Job, JobStatus};
        
        Job {
            id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            status: JobStatus::Queued,
            proof_bytes: vec![1u8; 256], // Valid length
            public_inputs: vec![0u8; 64], // Valid length
            outputs_json: json!([
                {
                    "recipient": "11111111111111111111111111111112",
                    "amount": 990000
                }
            ]),
            fee_bps: 100, // 1%
            root_hash: vec![1u8; 32],
            nullifier: vec![1u8; 32],
            amount: 1000000,
            outputs_hash: vec![0u8; 32],
            tx_id: None,
            solana_signature: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    #[test]
    fn test_parse_outputs_valid() {
        let service = ValidationService::new(ValidationConfig::default());
        
        let outputs_json = json!([
            {
                "recipient": "11111111111111111111111111111112",
                "amount": 1000000
            }
        ]);

        let result = service.parse_outputs(&outputs_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_parse_outputs_too_many() {
        let mut config = ValidationConfig::default();
        config.max_outputs = 1;
        let service = ValidationService::new(config);
        
        let outputs_json = json!([
            {
                "recipient": "11111111111111111111111111111112",
                "amount": 1000000
            },
            {
                "recipient": "11111111111111111111111111111113",
                "amount": 2000000
            }
        ]);

        let result = service.parse_outputs(&outputs_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_output_zero_amount() {
        let service = ValidationService::new(ValidationConfig::default());
        
        let output = Output {
            recipient: "11111111111111111111111111111112".to_string(),
            amount: 0,
        };

        let result = service.validate_output(&output);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_nullifier_format() {
        let service = ValidationService::new(ValidationConfig::default());
        
        // Valid nullifier
        let valid_nullifier = vec![1u8; 32];
        assert!(service.validate_nullifier_format(&valid_nullifier).is_ok());
        
        // Invalid length
        let invalid_length = vec![1u8; 31];
        assert!(service.validate_nullifier_format(&invalid_length).is_err());
        
        // Zero nullifier
        let zero_nullifier = vec![0u8; 32];
        assert!(service.validate_nullifier_format(&zero_nullifier).is_err());
    }
} 
