//! RPC helpers for fetching PoW mining parameters
//!
//! Provides functions to fetch:
//! - ScrambleRegistry state (difficulty, windows, parameters)
//! - SlotHashes sysvar (recent slot + hash)
//! - Current slot

use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, sysvar};

/// ScrambleRegistry state (matches on-chain struct)
#[derive(Debug, Clone)]
pub struct RegistryState {
    pub admin: Pubkey,
    pub current_difficulty: [u8; 32],
    pub last_retarget_slot: u64,
    pub solutions_observed: u64,
    pub target_interval_slots: u64,
    pub fee_share_bps: u16,
    pub reveal_window: u64,
    pub claim_window: u64,
    pub max_k: u16,
    pub min_difficulty: [u8; 32],
    pub max_difficulty: [u8; 32],
    pub total_claims: u64,
    pub active_claims: u64,
}

/// Fetch ScrambleRegistry account and deserialize
///
/// Returns the registry state including current difficulty and windows.
pub fn fetch_registry(client: &RpcClient, registry_pubkey: &Pubkey) -> Result<RegistryState> {
    let account = client
        .get_account(registry_pubkey)
        .map_err(|e| anyhow!("Failed to fetch registry account: {}", e))?;

    if account.data.is_empty() {
        return Err(anyhow!("Registry account has no data"));
    }

    deserialize_registry(&account.data)
}

/// Deserialize ScrambleRegistry account data
///
/// Layout matches programs/scramble-registry/src/state/registry.rs:
/// - discriminator: [u8; 8]
/// - admin: Pubkey (32)
/// - current_difficulty: [u8; 32]
/// - last_retarget_slot: u64
/// - solutions_observed: u64
/// - target_interval_slots: u64
/// - fee_share_bps: u16
/// - reveal_window: u64
/// - claim_window: u64
/// - max_k: u16
/// - min_difficulty: [u8; 32]
/// - max_difficulty: [u8; 32]
/// - total_claims: u64
/// - active_claims: u64
fn deserialize_registry(data: &[u8]) -> Result<RegistryState> {
    if data.len() < 196 {
        return Err(anyhow!("Registry data too short: {} bytes", data.len()));
    }

    let mut offset = 0;

    // Skip discriminator (8 bytes)
    offset += 8;

    // Admin (32 bytes)
    let admin = Pubkey::try_from(&data[offset..offset + 32])
        .map_err(|e| anyhow!("Failed to parse admin pubkey: {}", e))?;
    offset += 32;

    // Current difficulty (32 bytes)
    let current_difficulty: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| anyhow!("Failed to parse current_difficulty"))?;
    offset += 32;

    // last_retarget_slot (8 bytes LE)
    let last_retarget_slot = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse last_retarget_slot"))?,
    );
    offset += 8;

    // solutions_observed (8 bytes LE)
    let solutions_observed = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse solutions_observed"))?,
    );
    offset += 8;

    // target_interval_slots (8 bytes LE)
    let target_interval_slots = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse target_interval_slots"))?,
    );
    offset += 8;

    // fee_share_bps (2 bytes LE)
    let fee_share_bps = u16::from_le_bytes(
        data[offset..offset + 2]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse fee_share_bps"))?,
    );
    offset += 2;

    // reveal_window (8 bytes LE)
    let reveal_window = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse reveal_window"))?,
    );
    offset += 8;

    // claim_window (8 bytes LE)
    let claim_window = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse claim_window"))?,
    );
    offset += 8;

    // max_k (2 bytes LE)
    let max_k = u16::from_le_bytes(
        data[offset..offset + 2]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse max_k"))?,
    );
    offset += 2;

    // min_difficulty (32 bytes)
    let min_difficulty: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| anyhow!("Failed to parse min_difficulty"))?;
    offset += 32;

    // max_difficulty (32 bytes)
    let max_difficulty: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| anyhow!("Failed to parse max_difficulty"))?;
    offset += 32;

    // total_claims (8 bytes LE)
    let total_claims = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse total_claims"))?,
    );
    offset += 8;

    // active_claims (8 bytes LE)
    let active_claims = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse active_claims"))?,
    );

    Ok(RegistryState {
        admin,
        current_difficulty,
        last_retarget_slot,
        solutions_observed,
        target_interval_slots,
        fee_share_bps,
        reveal_window,
        claim_window,
        max_k,
        min_difficulty,
        max_difficulty,
        total_claims,
        active_claims,
    })
}

/// Fetch SlotHashes sysvar and get most recent slot hash
///
/// Returns (slot, slot_hash) tuple for the most recent slot.
pub fn fetch_recent_slot_hash(client: &RpcClient) -> Result<(u64, [u8; 32])> {
    let slot_hashes_pubkey = sysvar::slot_hashes::id();
    let account = client
        .get_account(&slot_hashes_pubkey)
        .map_err(|e| anyhow!("Failed to fetch SlotHashes sysvar: {}", e))?;

    if account.data.is_empty() {
        return Err(anyhow!("SlotHashes sysvar has no data"));
    }

    parse_slot_hashes(&account.data)
}

/// Parse SlotHashes sysvar data
///
/// SlotHashes layout:
/// - count: u64 LE (number of entries)
/// - entries: [(slot: u64 LE, hash: [u8; 32])] * count
///
/// Returns the first (most recent) entry.
fn parse_slot_hashes(data: &[u8]) -> Result<(u64, [u8; 32])> {
    if data.len() < 8 {
        return Err(anyhow!("SlotHashes data too short"));
    }

    // Parse count
    let count = u64::from_le_bytes(
        data[0..8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse SlotHashes count"))?,
    );

    if count == 0 {
        return Err(anyhow!("No slot hashes available"));
    }

    // Each entry is 8 (slot) + 32 (hash) = 40 bytes
    let expected_len = 8 + (count as usize * 40);
    if data.len() < expected_len {
        return Err(anyhow!(
            "SlotHashes data malformed: expected {} bytes, got {}",
            expected_len,
            data.len()
        ));
    }

    // Parse first entry (most recent)
    let offset = 8;
    let slot = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse slot"))?,
    );

    let hash: [u8; 32] = data[offset + 8..offset + 40]
        .try_into()
        .map_err(|_| anyhow!("Failed to parse slot hash"))?;

    Ok((slot, hash))
}

/// Get current slot from RPC
pub fn get_current_slot(client: &RpcClient) -> Result<u64> {
    client
        .get_slot()
        .map_err(|e| anyhow!("Failed to get current slot: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_registry() {
        // Build mock registry data (196 bytes minimum)
        let mut data = Vec::new();

        // Discriminator (8 bytes)
        data.extend_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);

        // Admin pubkey (32 bytes)
        let admin = Pubkey::new_unique();
        data.extend_from_slice(admin.as_ref());

        // Current difficulty (32 bytes)
        let difficulty = [0xFF; 32];
        data.extend_from_slice(&difficulty);

        // last_retarget_slot: 1000
        data.extend_from_slice(&1000u64.to_le_bytes());

        // solutions_observed: 42
        data.extend_from_slice(&42u64.to_le_bytes());

        // target_interval_slots: 600
        data.extend_from_slice(&600u64.to_le_bytes());

        // fee_share_bps: 2500
        data.extend_from_slice(&2500u16.to_le_bytes());

        // reveal_window: 150
        data.extend_from_slice(&150u64.to_le_bytes());

        // claim_window: 300
        data.extend_from_slice(&300u64.to_le_bytes());

        // max_k: 100
        data.extend_from_slice(&100u16.to_le_bytes());

        // min_difficulty (32 bytes)
        let min_diff = [0x01; 32];
        data.extend_from_slice(&min_diff);

        // max_difficulty (32 bytes)
        let max_diff = [0xFE; 32];
        data.extend_from_slice(&max_diff);

        // total_claims: 999
        data.extend_from_slice(&999u64.to_le_bytes());

        // active_claims: 50
        data.extend_from_slice(&50u64.to_le_bytes());

        // Deserialize
        let registry = deserialize_registry(&data).expect("Should deserialize");

        assert_eq!(registry.admin, admin);
        assert_eq!(registry.current_difficulty, difficulty);
        assert_eq!(registry.last_retarget_slot, 1000);
        assert_eq!(registry.solutions_observed, 42);
        assert_eq!(registry.target_interval_slots, 600);
        assert_eq!(registry.fee_share_bps, 2500);
        assert_eq!(registry.reveal_window, 150);
        assert_eq!(registry.claim_window, 300);
        assert_eq!(registry.max_k, 100);
        assert_eq!(registry.min_difficulty, min_diff);
        assert_eq!(registry.max_difficulty, max_diff);
        assert_eq!(registry.total_claims, 999);
        assert_eq!(registry.active_claims, 50);
    }

    #[test]
    fn test_parse_slot_hashes() {
        // Build mock SlotHashes data
        let count = 3u64;
        let mut data = Vec::new();

        // Count
        data.extend_from_slice(&count.to_le_bytes());

        // Entry 1 (most recent)
        data.extend_from_slice(&100u64.to_le_bytes());
        data.extend_from_slice(&[0x42; 32]);

        // Entry 2
        data.extend_from_slice(&99u64.to_le_bytes());
        data.extend_from_slice(&[0x43; 32]);

        // Entry 3
        data.extend_from_slice(&98u64.to_le_bytes());
        data.extend_from_slice(&[0x44; 32]);

        let (slot, hash) = parse_slot_hashes(&data).expect("Should parse");

        assert_eq!(slot, 100);
        assert_eq!(hash, [0x42; 32]);
    }

    #[test]
    fn test_parse_slot_hashes_empty() {
        let mut data = Vec::new();
        data.extend_from_slice(&0u64.to_le_bytes()); // count = 0

        let result = parse_slot_hashes(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No slot hashes"));
    }
}
