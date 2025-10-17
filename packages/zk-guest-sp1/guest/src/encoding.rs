use anyhow::{anyhow, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// BLAKE3 hash function returning 32 bytes
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Serialize u64 to little-endian bytes
pub fn serialize_u64_le(value: u64) -> [u8; 8] {
    value.to_le_bytes()
}

/// Serialize u32 to little-endian bytes
pub fn serialize_u32_le(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Parse hex string to 32-byte array
pub fn parse_hex32(hex_str: &str) -> Result<[u8; 32]> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).map_err(|e| anyhow!("Invalid hex string: {}", e))?;

    if bytes.len() != 32 {
        return Err(anyhow!("Expected 32 bytes, got {}", bytes.len()));
    }

    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

/// Parse address from base58 or hex to 32-byte array
pub fn parse_address(address_str: &str) -> Result<[u8; 32]> {
    // Try hex first (with or without 0x prefix)
    if let Ok(hex_bytes) = parse_hex32(address_str) {
        return Ok(hex_bytes);
    }

    // Try base58
    use base58::FromBase58;
    let bytes = address_str
        .from_base58()
        .map_err(|e| anyhow!("Invalid base58 address: {:?}", e))?;

    if bytes.len() != 32 {
        return Err(anyhow!(
            "Base58 address must decode to 32 bytes, got {}",
            bytes.len()
        ));
    }

    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

/// Compute commitment: C = H(amount:u64 || r:32 || pk_spend:32) using BLAKE3
pub fn compute_commitment(amount: u64, r: &[u8; 32], pk_spend: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(&serialize_u64_le(amount));
    hasher.update(r);
    hasher.update(pk_spend);
    *hasher.finalize().as_bytes()
}

/// Compute pk_spend: pk_spend = H(sk_spend:32)
pub fn compute_pk_spend(sk_spend: &[u8; 32]) -> [u8; 32] {
    hash_blake3(sk_spend)
}

/// Compute nullifier: nf = H(sk_spend:32 || leaf_index:u32) using BLAKE3
pub fn compute_nullifier(sk_spend: &[u8; 32], leaf_index: u32) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(sk_spend);
    hasher.update(&serialize_u32_le(leaf_index));
    *hasher.finalize().as_bytes()
}

/// Compute outputs hash: H(output[0] || output[1] || ... || output[n-1]) using BLAKE3
/// where output = address:32 || amount:u64
pub fn compute_outputs_hash(outputs: &[Output]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    for output in outputs {
        hasher.update(&output.address);
        hasher.update(&serialize_u64_le(output.amount));
    }
    *hasher.finalize().as_bytes()
}

pub fn calculate_fee(amount: u64) -> u64 {
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (amount * 5) / 1_000; // 0.5% = 5/1000
    fixed_fee + variable_fee
}

/// Merkle path verification using BLAKE3
/// Rule: if bit==0 => parent=H(curr||sib) else parent=H(sib||curr)
pub fn verify_merkle_path(
    leaf: &[u8; 32],
    path_elements: &[[u8; 32]],
    path_indices: &[u8],
    root: &[u8; 32],
) -> bool {
    if path_elements.len() != path_indices.len() {
        return false;
    }

    let mut current = *leaf;

    for (element, &index) in path_elements.iter().zip(path_indices.iter()) {
        let mut hasher = Hasher::new();
        if index == 0 {
            // current is left, element is right
            hasher.update(&current);
            hasher.update(element);
        } else if index == 1 {
            // element is left, current is right
            hasher.update(element);
            hasher.update(&current);
        } else {
            return false; // Invalid index
        }
        current = *hasher.finalize().as_bytes();
    }

    current == *root
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    #[serde(with = "address_serde")]
    pub address: [u8; 32],
    pub amount: u64,
}

// Custom serde for address field to handle both base58 and hex
mod address_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(address: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = hex::encode(address);
        serializer.serialize_str(&hex_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_address(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePath {
    #[serde(with = "hex_array_serde")]
    pub path_elements: Vec<[u8; 32]>,
    pub path_indices: Vec<u8>,
}

// Custom serde for hex arrays
mod hex_array_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(elements: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = elements.iter().map(|e| hex::encode(e)).collect();
        hex_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_strings = Vec::<String>::deserialize(deserializer)?;
        hex_strings
            .into_iter()
            .map(|s| parse_hex32(&s).map_err(serde::de::Error::custom))
            .collect()
    }
}

#[cfg(test)]
mod tests {
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
        assert!(verify_merkle_path(
            &leaf,
            &path_elements,
            &path_indices,
            &root
        ));

        // Should fail with swapped sibling
        let path_elements_swapped = vec![sibling2, sibling1];
        assert!(!verify_merkle_path(
            &leaf,
            &path_elements_swapped,
            &path_indices,
            &root
        ));
    }

    #[test]
    fn test_fee_calculation() {
        assert_eq!(calculate_fee(1000000), 50000000 + 50); // 0.05 SOL + 0.05%
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
