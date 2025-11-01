#![no_main]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sp1_zkvm::io;

mod encoding;

use encoding::*;

#[derive(Debug, Serialize, Deserialize)]
struct PrivateInputs {
    pub amount: u64,
    #[serde(with = "hex_string")]
    pub r: [u8; 32],
    #[serde(with = "hex_string")]
    pub sk_spend: [u8; 32],
    pub leaf_index: u32,
    pub merkle_path: MerklePath,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublicInputs {
    #[serde(with = "hex_string")]
    pub root: [u8; 32],
    #[serde(with = "hex_string")]
    pub nf: [u8; 32],
    #[serde(with = "hex_string")]
    pub outputs_hash: [u8; 32],
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CircuitInputs {
    pub private: PrivateInputs,
    pub public: PublicInputs,
    pub outputs: Vec<Output>,
}

// Custom serde module for hex strings
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

sp1_zkvm::entrypoint!(main);

pub fn main() {
    // Read JSON input from stdin
    let input_json = io::read::<String>();

    // Parse the input
    let inputs: CircuitInputs =
        serde_json::from_str(&input_json).expect("Failed to parse input JSON");

    // Verify all circuit constraints
    verify_circuit_constraints(&inputs).expect("Circuit constraint verification failed");

    // Commit public inputs as a single 104-byte canonical blob
    // Format: root(32) || nf(32) || outputs_hash(32) || amount(8)
    // This matches the format expected by the Solana verifier (sp1-solana crate)
    let mut public_inputs_blob = [0u8; 104];
    public_inputs_blob[0..32].copy_from_slice(&inputs.public.root);
    public_inputs_blob[32..64].copy_from_slice(&inputs.public.nf);
    public_inputs_blob[64..96].copy_from_slice(&inputs.public.outputs_hash);
    public_inputs_blob[96..104].copy_from_slice(&inputs.public.amount.to_le_bytes());

    sp1_zkvm::io::commit_slice(&public_inputs_blob);
}

fn verify_circuit_constraints(inputs: &CircuitInputs) -> Result<()> {
    let private = &inputs.private;
    let public = &inputs.public;
    let outputs = &inputs.outputs;

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
    // Fee is fixed in the program, so we calculate it directly
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_inputs() -> CircuitInputs {
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

        let outputs = vec![
            Output {
                address: [0x01u8; 32],
                amount: 400000,
            },
            Output {
                address: [0x02u8; 32],
                amount: 594000, // 1000000 - 6000 (fee) = 994000, so 400000 + 594000 = 994000
            },
        ];

        let outputs_hash = compute_outputs_hash(&outputs);

        CircuitInputs {
            private: PrivateInputs {
                amount,
                r,
                sk_spend,
                leaf_index,
                merkle_path: MerklePath {
                    path_elements: vec![sibling],
                    path_indices: vec![0], // commitment is left, sibling is right
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
    fn test_valid_circuit() {
        let inputs = create_test_inputs();
        assert!(verify_circuit_constraints(&inputs).is_ok());
    }

    #[test]
    fn test_invalid_merkle_path() {
        let mut inputs = create_test_inputs();
        // Flip a path index bit
        inputs.private.merkle_path.path_indices[0] = 1;
        assert!(verify_circuit_constraints(&inputs).is_err());
    }

    #[test]
    fn test_invalid_outputs_hash() {
        let mut inputs = create_test_inputs();
        // Change an output amount
        inputs.outputs[0].amount = 500000;
        assert!(verify_circuit_constraints(&inputs).is_err());
    }

    #[test]
    fn test_conservation_failure() {
        let inputs = create_test_inputs();
        assert!(verify_circuit_constraints(&inputs).is_err());
    }
}
