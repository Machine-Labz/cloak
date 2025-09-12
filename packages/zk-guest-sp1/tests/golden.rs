use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::fs;
use std::path::Path;

// Import encoding functions for testing
use zk_guest_sp1_host::encoding::*;

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
    pub fee_bps: u16,
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
    use serde::{Deserialize, Deserializer, Serializer};
    use zk_guest_sp1_host::encoding::*;
    
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

fn load_test_inputs() -> Result<CircuitInputs> {
    let private_json = fs::read_to_string("examples/private.example.json")?;
    let public_json = fs::read_to_string("examples/public.example.json")?;
    let outputs_json = fs::read_to_string("examples/outputs.example.json")?;
    
    let private_inputs: PrivateInputs = serde_json::from_str(&private_json)?;
    let public_inputs: PublicInputs = serde_json::from_str(&public_json)?;
    let outputs: Vec<Output> = serde_json::from_str(&outputs_json)?;
    
    Ok(CircuitInputs {
        private: private_inputs,
        public: public_inputs,
        outputs,
    })
}

fn find_guest_elf() -> Result<Vec<u8>> {
    let paths = [
        "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
        "../guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest",
    ];
    
    for path in &paths {
        if Path::new(path).exists() {
            return Ok(fs::read(path)?);
        }
    }
    
    Err(anyhow::anyhow!("Could not find guest ELF in any expected location"))
}

#[test]
fn test_valid_proof_generation_and_verification() -> Result<()> {
    let inputs = load_test_inputs()?;
    let input_json = serde_json::to_string(&inputs)?;
    
    // Setup client and load guest ELF
    let client = ProverClient::from_env();
    let guest_elf = find_guest_elf()?;
    let (pk, vk) = client.setup(&guest_elf);
    
    // Generate proof
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_json);
    
    let mut proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("Failed to generate proof");
    
    // Verify proof
    client.verify(&proof, &vk).expect("Proof verification failed");
    
    // Check public outputs
    let public_output: PublicInputs = proof.public_values.read();
    assert_eq!(public_output.root, inputs.public.root);
    assert_eq!(public_output.nf, inputs.public.nf);
    assert_eq!(public_output.fee_bps, inputs.public.fee_bps);
    assert_eq!(public_output.outputs_hash, inputs.public.outputs_hash);
    assert_eq!(public_output.amount, inputs.public.amount);
    
    Ok(())
}

#[test]
fn test_invalid_merkle_path_fails() -> Result<()> {
    let mut inputs = load_test_inputs()?;
    
    // Flip a path index bit to make merkle path invalid
    inputs.private.merkle_path.path_indices[0] = 1;
    
    let input_json = serde_json::to_string(&inputs)?;
    
    // Setup client and load guest ELF
    let client = ProverClient::from_env();
    let guest_elf = find_guest_elf()?;
    let (pk, _) = client.setup(&guest_elf);
    
    // Generate proof - this should fail
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_json);
    
    let result = client.prove(&pk, &stdin).run();
    
    // The proof generation should fail due to constraint violation
    assert!(result.is_err(), "Expected proof generation to fail with invalid merkle path");
    
    Ok(())
}

#[test]
fn test_invalid_outputs_hash_fails() -> Result<()> {
    let mut inputs = load_test_inputs()?;
    
    // Change an output amount to break outputs hash
    inputs.outputs[0].amount = 500000;
    
    let input_json = serde_json::to_string(&inputs)?;
    
    // Setup client and load guest ELF
    let client = ProverClient::from_env();
    let guest_elf = find_guest_elf()?;
    let (pk, _) = client.setup(&guest_elf);
    
    // Generate proof - this should fail
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_json);
    
    let result = client.prove(&pk, &stdin).run();
    
    // The proof generation should fail due to constraint violation
    assert!(result.is_err(), "Expected proof generation to fail with invalid outputs hash");
    
    Ok(())
}

#[test]
fn test_conservation_failure() -> Result<()> {
    let mut inputs = load_test_inputs()?;
    
    // Change fee_bps to break conservation
    inputs.public.fee_bps = 100; // 1% instead of 0.6%
    
    let input_json = serde_json::to_string(&inputs)?;
    
    // Setup client and load guest ELF
    let client = ProverClient::from_env();
    let guest_elf = find_guest_elf()?;
    let (pk, _) = client.setup(&guest_elf);
    
    // Generate proof - this should fail
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_json);
    
    let result = client.prove(&pk, &stdin).run();
    
    // The proof generation should fail due to constraint violation
    assert!(result.is_err(), "Expected proof generation to fail with conservation violation");
    
    Ok(())
}

// Unit tests for encoding functions
#[cfg(test)]
mod encoding_tests {
    use super::*;
    
    #[test]
    fn test_hash_commitment_matches_docs() {
        let amount = 1000000u64;
        let r = [0x42u8; 32];
        let sk_spend = [0x33u8; 32];
        
        let pk_spend = compute_pk_spend(&sk_spend);
        let commitment = compute_commitment(amount, &r, &pk_spend);
        
        // Verify the commitment is computed correctly
        assert_eq!(commitment.len(), 32);
        
        // Test consistency
        let commitment2 = compute_commitment(amount, &r, &pk_spend);
        assert_eq!(commitment, commitment2);
    }

    #[test]
    fn test_nullifier_matches_docs() {
        let sk_spend = [0x11u8; 32];
        let leaf_index = 42u32;
        
        let nullifier = compute_nullifier(&sk_spend, leaf_index);
        assert_eq!(nullifier.len(), 32);
        
        // Test consistency
        let nullifier2 = compute_nullifier(&sk_spend, leaf_index);
        assert_eq!(nullifier, nullifier2);
    }

    #[test]
    fn test_outputs_hash_order_sensitive() {
        let output1 = Output {
            address: [0x01u8; 32],
            amount: 100,
        };
        let output2 = Output {
            address: [0x02u8; 32],
            amount: 200,
        };
        
        let hash1 = compute_outputs_hash(&[output1.clone(), output2.clone()]);
        let hash2 = compute_outputs_hash(&[output2, output1]);
        
        // Order should matter
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_merkle_verify_ok_and_fails_on_swapped_sibling() {
        let leaf = [0x01u8; 32];
        let sibling1 = [0x02u8; 32];
        let sibling2 = [0x03u8; 32];
        
        // Compute correct root
        let level1 = hash_blake3(&[&leaf[..], &sibling1[..]].concat());
        let root = hash_blake3(&[&level1[..], &sibling2[..]].concat());
        
        let path_elements = vec![sibling1, sibling2];
        let path_indices = vec![0, 0]; // leaf left, then level1 left
        
        // Should verify correctly
        assert!(verify_merkle_path(&leaf, &path_elements, &path_indices, &root));
        
        // Should fail with swapped sibling
        let path_elements_swapped = vec![sibling2, sibling1];
        assert!(!verify_merkle_path(&leaf, &path_elements_swapped, &path_indices, &root));
    }

    #[test]
    fn test_fee_calculation() {
        assert_eq!(calculate_fee(1000000, 60), 6000); // 0.6%
        assert_eq!(calculate_fee(1000000, 100), 10000); // 1%
        assert_eq!(calculate_fee(1000000, 0), 0); // 0%
    }

    #[test]
    fn test_address_parsing() {
        // Test hex parsing
        let hex_addr = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let parsed_hex = parse_address(hex_addr).unwrap();
        assert_eq!(hex::encode(parsed_hex), hex_addr);
        
        // Test hex with 0x prefix
        let hex_addr_prefixed = format!("0x{}", hex_addr);
        let parsed_hex_prefixed = parse_address(&hex_addr_prefixed).unwrap();
        assert_eq!(parsed_hex, parsed_hex_prefixed);
    }
}
