use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::encoding::*;

/// Batch circuit inputs for multiple withdrawals
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCircuitInputs {
    pub withdrawals: Vec<SingleWithdrawal>,
    #[serde(with = "hex_string")]
    pub common_root: [u8; 32],
}

/// Single withdrawal data within a batch
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleWithdrawal {
    pub private: PrivateInputs,
    pub public: PublicInputs,
    pub outputs: Vec<Output>,
}

/// Private inputs for a single withdrawal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateInputs {
    pub amount: u64,
    #[serde(with = "hex_string")]
    pub r: [u8; 32],
    #[serde(with = "hex_string")]
    pub sk_spend: [u8; 32],
    pub leaf_index: u32,
    pub merkle_path: MerklePath,
}

/// Public inputs for a single withdrawal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicInputs {
    #[serde(with = "hex_string")]
    pub root: [u8; 32],
    #[serde(with = "hex_string")]
    pub nf: [u8; 32],
    #[serde(with = "hex_string")]
    pub outputs_hash: [u8; 32],
    pub amount: u64,
}

/// Verify a single withdrawal's constraints
pub fn verify_single_withdrawal(withdrawal: &SingleWithdrawal) -> Result<()> {
    let private = &withdrawal.private;
    let public = &withdrawal.public;
    let outputs = &withdrawal.outputs;

    // Constraint 1: pk_spend = H(sk_spend)
    let pk_spend = compute_pk_spend(&private.sk_spend);

    // Constraint 2: C = H(amount || r || pk_spend)
    let commitment = compute_commitment(private.amount, &private.r, &pk_spend);

    // Constraint 3: MerkleVerify(C, merkle_path) == root
    let merkle_valid = verify_merkle_path(
        &commitment,
        &private.merkle_path.path_elements,
        &private.merkle_path.path_indices,
        &public.root,
    );
    if !merkle_valid {
        return Err(anyhow!("Merkle path verification failed"));
    }

    // Constraint 4: nf == H(sk_spend || leaf_index)
    let computed_nullifier = compute_nullifier(&private.sk_spend, private.leaf_index);
    if computed_nullifier != public.nf {
        return Err(anyhow!("Nullifier mismatch"));
    }

    // Constraint 5: sum(outputs) + fee(amount) == amount
    let outputs_sum: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(private.amount);
    let total_spent = outputs_sum + fee;

    if total_spent != private.amount {
        return Err(anyhow!(
            "Amount conservation failed: outputs({}) + fee({}) != amount({})",
            outputs_sum,
            fee,
            private.amount
        ));
    }

    // Constraint 6: H(serialize(outputs)) == outputs_hash
    let computed_outputs_hash = compute_outputs_hash(outputs);
    if computed_outputs_hash != public.outputs_hash {
        return Err(anyhow!("Outputs hash mismatch"));
    }

    // Additional consistency checks
    if private.amount != public.amount {
        return Err(anyhow!("Private and public amounts must match"));
    }

    Ok(())
}

/// Verify all withdrawals in a batch
pub fn verify_batch_constraints(inputs: &BatchCircuitInputs) -> Result<()> {
    if inputs.withdrawals.is_empty() {
        return Err(anyhow!("Batch cannot be empty"));
    }

    // Verify each withdrawal uses the common root
    for (i, withdrawal) in inputs.withdrawals.iter().enumerate() {
        if withdrawal.public.root != inputs.common_root {
            return Err(anyhow!(
                "Withdrawal {} has mismatched root (expected {}, got {})",
                i,
                hex::encode(inputs.common_root),
                hex::encode(withdrawal.public.root)
            ));
        }

        // Verify the withdrawal's constraints
        verify_single_withdrawal(withdrawal)
            .map_err(|e| anyhow!("Withdrawal {} verification failed: {}", i, e))?;
    }

    Ok(())
}

// Custom serde for hex strings (reused from main.rs)
mod hex_string {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = hex::encode(bytes);
        serializer.serialize_str(&hex_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_hex32(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_withdrawal() -> SingleWithdrawal {
        let sk_spend = [0x11u8; 32];
        let r = [0x22u8; 32];
        let amount = 1000000u64;
        let leaf_index = 42u32;

        let pk_spend = compute_pk_spend(&sk_spend);
        let commitment = compute_commitment(amount, &r, &pk_spend);
        let nullifier = compute_nullifier(&sk_spend, leaf_index);

        // Create a simple merkle path (single level for testing)
        let sibling = [0x33u8; 32];
        let root = hash_blake3(&[&commitment[..], &sibling[..]].concat());

        let fee = calculate_fee(amount);
        let outputs = vec![Output {
            address: [0x01u8; 32],
            amount: amount - fee,
        }];

        let outputs_hash = compute_outputs_hash(&outputs);

        SingleWithdrawal {
            private: PrivateInputs {
                amount,
                r,
                sk_spend,
                leaf_index,
                merkle_path: MerklePath {
                    path_elements: vec![sibling],
                    path_indices: vec![0],
                },
            },
            public: PublicInputs {
                root,
                nf: nullifier,
                outputs_hash,
                amount,
            },
            outputs,
        }
    }

    #[test]
    fn test_single_withdrawal_verification() {
        let withdrawal = create_test_withdrawal();
        assert!(verify_single_withdrawal(&withdrawal).is_ok());
    }

    #[test]
    fn test_batch_verification_single() {
        let withdrawal = create_test_withdrawal();
        let batch = BatchCircuitInputs {
            withdrawals: vec![withdrawal.clone()],
            common_root: withdrawal.public.root,
        };

        assert!(verify_batch_constraints(&batch).is_ok());
    }

    #[test]
    fn test_batch_verification_multiple() {
        let withdrawal1 = create_test_withdrawal();
        let mut withdrawal2 = create_test_withdrawal();

        // Make withdrawal2 slightly different but use same root
        withdrawal2.private.sk_spend = [0x44u8; 32];
        withdrawal2.private.leaf_index = 43;

        // Recalculate for withdrawal2
        let pk_spend2 = compute_pk_spend(&withdrawal2.private.sk_spend);
        let _commitment2 = compute_commitment(
            withdrawal2.private.amount,
            &withdrawal2.private.r,
            &pk_spend2,
        );
        withdrawal2.public.nf = compute_nullifier(
            &withdrawal2.private.sk_spend,
            withdrawal2.private.leaf_index,
        );

        // Use same root
        withdrawal2.public.root = withdrawal1.public.root;

        let batch = BatchCircuitInputs {
            withdrawals: vec![withdrawal1.clone(), withdrawal2],
            common_root: withdrawal1.public.root,
        };

        assert!(verify_batch_constraints(&batch).is_ok());
    }

    #[test]
    fn test_batch_verification_fails_on_mismatched_root() {
        let withdrawal1 = create_test_withdrawal();
        let mut withdrawal2 = create_test_withdrawal();
        withdrawal2.public.root = [0xFFu8; 32]; // Different root

        let batch = BatchCircuitInputs {
            withdrawals: vec![withdrawal1.clone(), withdrawal2],
            common_root: withdrawal1.public.root,
        };

        assert!(verify_batch_constraints(&batch).is_err());
    }
}
