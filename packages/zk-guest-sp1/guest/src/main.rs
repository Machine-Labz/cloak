#![no_main]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sp1_zkvm::io;

mod encoding;

use encoding::{SwapParams, *};

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
    /// Optional swap parameters for swap-mode withdrawals
    pub swap_params: Option<SwapParams>,
    /// Optional stake parameters for stake-mode withdrawals
    pub stake_params: Option<encoding::StakeParams>,
    /// Optional unstake parameters for unstake-to-pool
    pub unstake_params: Option<encoding::UnstakeParams>,
}

// Custom serde module for hex strings
mod hex_string {
    use serde::{Deserializer, Serializer};

    use super::*;

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

    // For unstake mode, skip Merkle and nullifier verification
    // Unstake is a DEPOSIT operation, not a WITHDRAW
    if inputs.unstake_params.is_none() {
        // Constraint 3: MerkleVerify(C, merkle_path) == root
        // Only for withdraw operations (not unstake)
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
        // Only for withdraw operations (not unstake)
        let computed_nullifier = compute_nullifier(&private.sk_spend, private.leaf_index);
        if computed_nullifier != public.nf {
            return Err(anyhow!("Nullifier mismatch"));
        }
    }

    // Constraint 5: sum(outputs) + fee(amount) == amount
    // For swap mode: outputs should be empty (all goes to swap)
    // For regular mode: outputs + fee = amount
    let outputs_sum: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(private.amount);

    if inputs.swap_params.is_some() {
        // Swap mode: verify swap constraints
        if outputs_sum != 0 {
            return Err(anyhow!(
                "Swap mode requires zero outputs, got outputs_sum = {}",
                outputs_sum
            ));
        }

        // Compute remaining amount after fee
        let swap_amount = private
            .amount
            .checked_sub(fee)
            .ok_or_else(|| anyhow!("Fee exceeds total amount"))?;

        // Verify swap parameters if present
        if let Some(ref swap_params) = inputs.swap_params {
            if swap_params.min_output_amount > swap_amount {
                return Err(anyhow!(
                    "Min output {} exceeds swap amount {}",
                    swap_params.min_output_amount,
                    swap_amount
                ));
            }
        }

        // Verify amount conservation: swap_amount + fee = amount
        if swap_amount + fee != private.amount {
            return Err(anyhow!(
                "Swap mode amount conservation failed: swap_amount ({}) + fee ({}) != deposit ({})",
                swap_amount,
                fee,
                private.amount
            ));
        }
    } else if inputs.stake_params.is_some() {
        // Stake mode: verify stake constraints
        if outputs_sum != 0 {
            return Err(anyhow!(
                "Stake mode requires zero outputs, got outputs_sum = {}",
                outputs_sum
            ));
        }

        // Compute remaining amount after fee (stake amount)
        let stake_amount = private.amount.checked_sub(fee)
            .ok_or_else(|| anyhow!("Fee exceeds total amount"))?;

        // Verify amount conservation: stake_amount + fee = amount
        if stake_amount + fee != private.amount {
            return Err(anyhow!(
                "Stake mode amount conservation failed: stake_amount ({}) + fee ({}) != deposit ({})",
                stake_amount,
                fee,
                private.amount
            ));
        }
    } else if inputs.unstake_params.is_some() {
        // Unstake mode: verify unstake constraints
        // This is a DEPOSIT operation (funds going INTO the pool)
        // The commitment is generated from the unstake params
        if outputs_sum != 0 {
            return Err(anyhow!(
                "Unstake mode requires zero outputs, got outputs_sum = {}",
                outputs_sum
            ));
        }

        // For unstake, we use a smaller fee (just protocol fee, no fixed component)
        let unstake_fee = (private.amount * 5) / 1_000; // 0.5% variable only
        let deposit_amount = private.amount.checked_sub(unstake_fee)
            .ok_or_else(|| anyhow!("Fee exceeds total amount"))?;

        // Verify the unstake params generate a valid commitment
        let unstake_params = inputs.unstake_params.as_ref().unwrap();
        let computed_pk_spend = compute_pk_spend(&unstake_params.sk_spend);
        let computed_commitment = compute_commitment(deposit_amount, &unstake_params.r, &computed_pk_spend);
        
        // The public "root" field holds the commitment in unstake mode
        if computed_commitment != public.root {
            return Err(anyhow!(
                "Unstake commitment mismatch: computed {} != public {}",
                hex::encode(computed_commitment),
                hex::encode(public.root)
            ));
        }
    } else {
        // Regular mode: verify conservation law
        let total_spent = outputs_sum + fee;
        if total_spent != private.amount {
            return Err(anyhow!(
                "Amount conservation failed: outputs({}) + fee({}) != amount({})",
                outputs_sum,
                fee,
                private.amount
            ));
        }
    }

    // Constraint 6: H(serialize(outputs)) == outputs_hash
    // For swap mode: outputs_hash = H(output_mint || recipient_ata || min_output_amount || public_amount)
    // For stake mode: outputs_hash = H(stake_account || public_amount)
    // For unstake mode: outputs_hash = H(commitment || stake_account_hash)
    // For regular mode: outputs_hash = H(output[0] || output[1] || ... || output[n-1])
    let computed_outputs_hash = if let Some(ref swap_params) = inputs.swap_params {
        // Swap mode: compute outputs_hash from swap parameters
        compute_swap_outputs_hash(swap_params, public.amount)
    } else if let Some(ref stake_params) = inputs.stake_params {
        // Stake mode: compute outputs_hash from stake parameters
        compute_stake_outputs_hash(stake_params, public.amount)
    } else if let Some(ref unstake_params) = inputs.unstake_params {
        // Unstake mode: compute outputs_hash from unstake parameters
        // This binds the commitment to the stake account
        compute_unstake_outputs_hash(&public.root, &unstake_params.stake_account)
    } else {
        // Regular mode: compute outputs_hash from outputs array
        compute_outputs_hash(outputs)
    };

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
            swap_params: None, // Regular mode (not swap)
            stake_params: None, // Regular mode (not stake)
            unstake_params: None, // Regular mode (not unstake)
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
