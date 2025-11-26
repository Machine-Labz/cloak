use std::fs;

use anyhow::Result;
use serde_json;

mod encoding {
    pub use zk_guest_sp1_host::encoding::*;
}

use encoding::*;

#[derive(serde::Serialize, serde::Deserialize)]
struct PrivateInputs {
    pub amount: u64,
    #[serde(with = "hex_string")]
    pub r: [u8; 32],
    #[serde(with = "hex_string")]
    pub sk_spend: [u8; 32],
    pub leaf_index: u32,
    pub merkle_path: ExampleMerklePath,
}

#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
struct ExampleMerklePath {
    #[serde(with = "hex_array")]
    pub path_elements: Vec<[u8; 32]>,
    pub path_indices: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ExampleOutput {
    #[serde(with = "hex_string")]
    pub address: [u8; 32],
    pub amount: u64,
}

// Helper module for arrays of hex strings
mod hex_array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = bytes.iter().map(|b| hex::encode(b)).collect();
        hex_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {
            let bytes = super::encoding::parse_hex32(&hex_str).map_err(serde::de::Error::custom)?;
            result.push(bytes);
        }
        Ok(result)
    }
}

mod hex_string {
    use serde::{Deserializer, Serializer};

    use super::encoding::*;

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
        use serde::Deserialize;
        let s = String::deserialize(deserializer)?;
        parse_hex32(&s).map_err(serde::de::Error::custom)
    }
}

fn main() -> Result<()> {
    // Create example data
    let sk_spend = [0x11u8; 32];
    let r = [0x22u8; 32];
    let amount = 1000000u64;
    let leaf_index = 42u32;
    let fee_bps = 60u16; // 0.6%

    let pk_spend = compute_pk_spend(&sk_spend);
    let commitment = compute_commitment(amount, &r, &pk_spend);
    let nullifier = compute_nullifier(&sk_spend, leaf_index);

    // Create a simple merkle path (single level for testing)
    let sibling = [0x33u8; 32];
    let root = hash_blake3(&[&commitment[..], &sibling[..]].concat());

    let outputs = vec![
        ExampleOutput {
            address: [0x01u8; 32],
            amount: 400000,
        },
        ExampleOutput {
            address: [0x02u8; 32],
            amount: 594000, // 1000000 - 6000 (fee) = 994000, so 400000 + 594000 = 994000
        },
    ];

    // Convert to host Output format for hash computation
    let host_outputs: Vec<Output> = outputs
        .iter()
        .map(|o| Output {
            address: o.address,
            amount: o.amount,
        })
        .collect();
    let outputs_hash = compute_outputs_hash(&host_outputs);

    let private_inputs = PrivateInputs {
        amount,
        r,
        sk_spend,
        leaf_index,
        merkle_path: ExampleMerklePath {
            path_elements: vec![sibling],
            path_indices: vec![0], // commitment is left, sibling is right
        },
    };

    let public_inputs = PublicInputs {
        root,
        nf: nullifier,
        fee_bps,
        outputs_hash,
        amount,
    };

    // Write example files
    fs::create_dir_all("examples")?;

    let private_json = serde_json::to_string_pretty(&private_inputs)?;
    fs::write("examples/private.example.json", private_json)?;

    let public_json = serde_json::to_string_pretty(&public_inputs)?;
    fs::write("examples/public.example.json", public_json)?;

    let outputs_json = serde_json::to_string_pretty(&outputs)?;
    fs::write("examples/outputs.example.json", outputs_json)?;

    println!("Generated example files:");
    println!("- examples/private.example.json");
    println!("- examples/public.example.json");
    println!("- examples/outputs.example.json");

    Ok(())
}
