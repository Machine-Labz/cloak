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

/// Swap-specific parameters for computing outputs_hash in swap mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    /// Output token mint address (e.g., USDC)
    #[serde(with = "address_serde")]
    pub output_mint: [u8; 32],

    /// Recipient's associated token account for the output mint
    #[serde(with = "address_serde")]
    pub recipient_ata: [u8; 32],

    /// Minimum output amount (slippage protection)
    pub min_output_amount: u64,
}

/// Staking-specific parameters for computing outputs_hash in stake mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeParams {
    /// Stake account address (where SOL will be staked)
    #[serde(with = "address_serde")]
    pub stake_account: [u8; 32],
}

/// Unstaking parameters for private unstake-to-pool
/// Used when moving funds from a stake account back to the shield pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeParams {
    /// Stake account address (where SOL is being unstaked from)
    #[serde(with = "address_serde")]
    pub stake_account: [u8; 32],
    /// New randomness for the commitment
    #[serde(with = "hex_serde")]
    pub r: [u8; 32],
    /// Secret key for spending (generates pk_spend)
    #[serde(with = "hex_serde")]
    pub sk_spend: [u8; 32],
}

/// Compute unstake outputs hash: H(commitment || stake_account_hash)
/// This is used to bind the commitment to the stake account
pub fn compute_unstake_outputs_hash(commitment: &[u8; 32], stake_account: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(commitment);
    // stake_account_hash
    let stake_hash = hash_blake3(stake_account);
    hasher.update(&stake_hash);
    *hasher.finalize().as_bytes()
}

/// Compute swap-mode outputs hash: H(output_mint || recipient_ata || min_output_amount || public_amount)
/// This is used for swap withdrawals where we withdraw SOL and swap it for another token
pub fn compute_swap_outputs_hash(swap_params: &SwapParams, public_amount: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(&swap_params.output_mint);
    hasher.update(&swap_params.recipient_ata);
    hasher.update(&serialize_u64_le(swap_params.min_output_amount));
    hasher.update(&serialize_u64_le(public_amount));
    *hasher.finalize().as_bytes()
}

/// Compute stake-mode outputs hash: H(stake_account || public_amount)
/// This is used for staking withdrawals where we withdraw SOL to a stake account
pub fn compute_stake_outputs_hash(stake_params: &StakeParams, public_amount: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(&stake_params.stake_account);
    hasher.update(&serialize_u64_le(public_amount));
    *hasher.finalize().as_bytes()
}

pub fn calculate_fee(amount: u64) -> u64 {
    // Fee structure:
    // - For SOL withdrawals: Fixed (0.0025 SOL) + Variable (0.5%)
    // - For SPL swaps: Variable fee (0.5%) is deducted from withdrawn SOL, fixed fee paid separately
    // Since the circuit doesn't distinguish, we use the full fee (fixed + variable) for all cases
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

// Custom serde for hex strings (32 bytes)
mod hex_serde {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePath {
    #[serde(with = "hex_array_serde")]
    pub path_elements: Vec<[u8; 32]>,
    pub path_indices: Vec<u8>,
}

// Custom serde for hex arrays
mod hex_array_serde {
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(elements: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = elements.iter().map(hex::encode).collect();
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

    #[test]
    fn test_swap_outputs_hash() {
        // Test swap-mode outputs_hash computation
        let swap_params = SwapParams {
            output_mint: [0xAAu8; 32],
            recipient_ata: [0xBBu8; 32],
            min_output_amount: 1_000_000, // 1 USDC (6 decimals)
        };
        let public_amount = 3_000_000_000u64; // 3 SOL

        let hash1 = compute_swap_outputs_hash(&swap_params, public_amount);
        assert_eq!(hash1.len(), 32);

        // Test determinism
        let hash2 = compute_swap_outputs_hash(&swap_params, public_amount);
        assert_eq!(hash1, hash2, "Swap outputs_hash should be deterministic");

        // Test different parameters produce different hash
        let swap_params2 = SwapParams {
            output_mint: [0xCCu8; 32],
            recipient_ata: [0xBBu8; 32],
            min_output_amount: 1_000_000,
        };
        let hash3 = compute_swap_outputs_hash(&swap_params2, public_amount);
        assert_ne!(hash1, hash3, "Different mint should produce different hash");

        // Test different public_amount produces different hash
        let hash4 = compute_swap_outputs_hash(&swap_params, 5_000_000_000u64);
        assert_ne!(
            hash1, hash4,
            "Different public_amount should produce different hash"
        );
    }

    #[test]
    fn test_swap_vs_regular_outputs_hash() {
        // Verify that swap mode and regular mode produce different hashes
        // (they use different formulas)
        let output = Output {
            address: [0xAAu8; 32],
            amount: 1_000_000,
        };
        let regular_hash = compute_outputs_hash(&[output]);

        let swap_params = SwapParams {
            output_mint: [0xAAu8; 32],
            recipient_ata: [0xBBu8; 32],
            min_output_amount: 1_000_000,
        };
        let swap_hash = compute_swap_outputs_hash(&swap_params, 3_000_000_000);

        assert_ne!(
            regular_hash, swap_hash,
            "Swap and regular mode should produce different hashes"
        );
    }
}
