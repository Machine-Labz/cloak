use anyhow::{anyhow, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// BLAKE3-256 hash function returning 32 bytes
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

/// Serialize u64 to little-endian bytes
pub fn serialize_u64_le(value: u64) -> [u8; 8] {
    value.to_le_bytes()
}

/// Serialize u32 to little-endian bytes
pub fn serialize_u32_le(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Compute commitment: C = H(amount:u64 || r:32 || pk_spend:32)
pub fn compute_commitment(amount: u64, r: &[u8; 32], pk_spend: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(&serialize_u64_le(amount));
    hasher.update(r);
    hasher.update(pk_spend);
    hasher.finalize().into()
}

/// Compute pk_spend: pk_spend = H(sk_spend:32)
pub fn compute_pk_spend(sk_spend: &[u8; 32]) -> [u8; 32] {
    hash_blake3(sk_spend)
}

/// Compute nullifier: nf = H(sk_spend:32 || leaf_index:u32)
pub fn compute_nullifier(sk_spend: &[u8; 32], leaf_index: u32) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(sk_spend);
    hasher.update(&serialize_u32_le(leaf_index));
    hasher.finalize().into()
}

/// Compute outputs hash: H(output[0] || output[1] || ... || output[n-1])
/// where output = address:32 || amount:u64
pub fn compute_outputs_hash(outputs: &[Output]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    for output in outputs {
        hasher.update(&output.address);
        hasher.update(&serialize_u64_le(output.amount));
    }
    hasher.finalize().into()
}

/// Calculate fee: fee = fixed_fee + (amount * variable_rate) / 1000
pub fn calculate_fee(amount: u64, fee_bps: u16) -> u64 {
    let fixed_fee = 2_500_000; // 0.0025 SOL
    let variable_fee = (amount * fee_bps as u64) / 1_000; // 0.5% = 5/1000
    fixed_fee + variable_fee
}

/// Merkle path verification
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
        current = hasher.finalize().into();
    }

    current == *root
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

/// Parse base58 or hex string to 32-byte address
pub fn parse_address(addr_str: &str) -> Result<[u8; 32]> {
    // Try base58 first
    if let Ok(decoded) = base58::FromBase58::from_base58(addr_str) {
        if decoded.len() == 32 {
            let mut result = [0u8; 32];
            result.copy_from_slice(&decoded);
            return Ok(result);
        }
    }

    // Fall back to hex
    parse_hex32(addr_str)
}

// Custom serde for hex arrays
mod hex_array_serde {
    use serde::{Deserializer, Serializer};

    use super::*;

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

// Custom serde for address field to handle both base58 and hex
mod address_serde {
    use serde::{Deserializer, Serializer};

    use super::*;

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

/// Output structure for withdraw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    #[serde(with = "address_serde")]
    pub address: [u8; 32],
    pub amount: u64,
}

/// Private inputs for the circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInputs {
    pub amount: u64,
    pub r: [u8; 32],
    pub sk_spend: [u8; 32],
    pub leaf_index: u32,
    pub merkle_path: MerklePath,
}

/// Public inputs for the circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    pub root: [u8; 32],
    pub nf: [u8; 32],
    pub fee_bps: u16,
    pub outputs_hash: [u8; 32],
    pub amount: u64,
}

/// Merkle path structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePath {
    #[serde(with = "hex_array_serde")]
    pub path_elements: Vec<[u8; 32]>,
    pub path_indices: Vec<u8>,
}

/// Complete inputs for the circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInputs {
    pub private: PrivateInputs,
    pub public: PublicInputs,
    pub outputs: Vec<Output>,
}
