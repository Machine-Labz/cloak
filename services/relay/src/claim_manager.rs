//! Claim discovery for PoW-gated withdraws
//!
//! This module queries on-chain for available PoW claims from independent miners.
//! The relay acts as a CLIENT of the miner ecosystem, not a miner itself.
//!
//! Architecture:
//! - Miners run cloak-miner CLI independently
//! - Miners compete to mine claims and earn fees
//! - Relay queries on-chain for available claims
//! - Relay uses claims when building withdraw transactions

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};
use tracing::{debug, error, info, warn};

use crate::error::Error;

/// An available claim discovered on-chain
#[derive(Debug, Clone)]
pub struct AvailableClaim {
    /// Claim PDA address
    pub claim_pda: Pubkey,

    /// Miner PDA address
    pub miner_pda: Pubkey,

    /// Miner authority (receives fee share)
    pub miner_authority: Pubkey,

    /// Slot when claim was mined
    pub mined_slot: u64,

    /// Registry PDA
    pub registry_pda: Pubkey,
}

/// Discovers available PoW claims from independent miners
///
/// This queries on-chain for claims that are:
/// - Revealed (ready to use)
/// - Not expired
/// - Not fully consumed
/// - Match the required batch_hash
pub struct ClaimFinder {
    /// RPC client for querying on-chain data
    rpc_client: RpcClient,

    /// Registry program ID
    registry_program_id: Pubkey,
}

impl ClaimFinder {
    pub fn new(rpc_url: String, registry_program_id: Pubkey) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url),
            registry_program_id,
        }
    }

    /// Find an available claim for a given batch hash
    ///
    /// This queries on-chain for claims that match the batch_hash and are usable.
    /// Returns the first available claim found.
    ///
    /// # Arguments
    /// * `batch_hash` - The batch commitment hash to find a claim for
    ///
    /// # Returns
    /// * `Ok(Some(claim))` - Found an available claim
    /// * `Ok(None)` - No available claims (user should wait or try later)
    /// * `Err(_)` - RPC or parsing error
    pub async fn find_claim(&self, batch_hash: &[u8; 32]) -> Result<Option<AvailableClaim>, Error> {
        let start_time = std::time::Instant::now();
        info!(
            "🔍 [METRICS] Claim search started for batch_hash: {:?}",
            hex::encode(&batch_hash[0..8])
        );

        // Query all accounts owned by the registry program
        // In production, you'd want to filter by discriminator and use memcmp filters
        let accounts = self
            .rpc_client
            .get_program_accounts(&self.registry_program_id)
            .await
            .map_err(|e| {
                error!(
                    "❌ [METRICS] Claim query failed after {:?}: {}",
                    start_time.elapsed(),
                    e
                );
                Error::InternalServerError(format!("Failed to query claims: {}", e))
            })?;

        let query_duration = start_time.elapsed();
        let accounts_len = accounts.len(); // Save length before consuming accounts
        info!(
            "📊 [METRICS] Query complete: {} accounts found in {:?}",
            accounts_len, query_duration
        );

        // Get current slot for expiry checks
        let current_slot = self
            .rpc_client
            .get_slot()
            .await
            .map_err(|e| Error::InternalServerError(format!("Failed to get slot: {}", e)))?;

        // Derive registry PDA
        let (registry_pda, _) =
            Pubkey::find_program_address(&[b"scramble_registry"], &self.registry_program_id);

        // Filter for usable claims
        for (pubkey, account) in &accounts {
            // Skip if too small to be a claim
            if account.data.len() < 256 {
                continue;
            }

            // Check discriminator (first 8 bytes should match "claim")
            // For now, we'll just check if it looks like a claim by checking size
            if account.data.len() != 256 {
                continue;
            }

            // Parse claim account
            if let Ok(claim) = parse_claim_account(&account) {
                // Check if batch_hash matches (or if claim is wildcard)
                let is_wildcard = claim.batch_hash == [0u8; 32];

                if !is_wildcard && claim.batch_hash != *batch_hash {
                    debug!("Claim {} batch_hash mismatch (not wildcard)", pubkey);
                    continue;
                }

                if is_wildcard {
                    debug!(
                        "Found wildcard claim {} (can be used for any batch)",
                        pubkey
                    );
                }

                // Check if revealed
                if claim.status != 1 {
                    // 1 = Revealed
                    debug!(
                        "⏭️  Claim {} not revealed (status: {})",
                        pubkey, claim.status
                    );
                    continue;
                }

                // Check if expired
                if current_slot > claim.expires_at_slot {
                    debug!(
                        "⏰ Claim {} expired (expires: {}, current: {})",
                        pubkey, claim.expires_at_slot, current_slot
                    );
                    continue;
                }

                // Check if fully consumed
                if claim.consumed_count >= claim.max_consumes {
                    debug!(
                        "💯 Claim {} fully consumed ({}/{})",
                        pubkey, claim.consumed_count, claim.max_consumes
                    );
                    continue;
                }

                // Found a usable claim!
                let miner_authority = claim.miner_authority;

                // Derive miner PDA
                let (miner_pda, _) = Pubkey::find_program_address(
                    &[b"miner", miner_authority.as_ref()],
                    &self.registry_program_id,
                );

                let total_duration = start_time.elapsed();
                info!(
                    "✅ [METRICS] Found available claim: {} (consumed {}/{}, expires at slot {}, search took {:?})",
                    pubkey, claim.consumed_count, claim.max_consumes, claim.expires_at_slot, total_duration
                );

                return Ok(Some(AvailableClaim {
                    claim_pda: *pubkey,
                    miner_pda,
                    miner_authority,
                    mined_slot: claim.slot,
                    registry_pda,
                }));
            }
        }

        let total_duration = start_time.elapsed();
        warn!("❌ [METRICS] No available claims found for batch_hash: {:?} (searched {} accounts in {:?})", 
              hex::encode(&batch_hash[0..8]), accounts.len(), total_duration);
        Ok(None)
    }
}

/// Parsed claim data from on-chain account
#[derive(Debug)]
struct ParsedClaim {
    miner_authority: Pubkey,
    batch_hash: [u8; 32],
    slot: u64,
    status: u8,
    consumed_count: u16,
    max_consumes: u16,
    expires_at_slot: u64,
}

/// Parse a claim account from raw bytes
///
/// Claim layout (256 bytes total):
/// - discriminator: 8 bytes
/// - miner_authority: 32 bytes (offset 8)
/// - batch_hash: 32 bytes (offset 40)
/// - slot: 8 bytes (offset 72)
/// - slot_hash: 32 bytes (offset 80)
/// - nonce: 16 bytes (offset 112)
/// - proof_hash: 32 bytes (offset 128)
/// - mined_at_slot: 8 bytes (offset 160)
/// - revealed_at_slot: 8 bytes (offset 168)
/// - consumed_count: 2 bytes (offset 176)
/// - max_consumes: 2 bytes (offset 178)
/// - expires_at_slot: 8 bytes (offset 180)
/// - status: 1 byte (offset 188)
fn parse_claim_account(account: &Account) -> Result<ParsedClaim, Error> {
    if account.data.len() < 256 {
        return Err(Error::ValidationError("Account too small".into()));
    }

    let data = &account.data;

    // Extract fields using unsafe pointer arithmetic (same as on-chain)
    unsafe {
        let miner_authority = Pubkey::new_from_array(*(data.as_ptr().add(8) as *const [u8; 32]));

        let batch_hash: [u8; 32] = *(data.as_ptr().add(40) as *const [u8; 32]);

        let slot = u64::from_le_bytes(*(data.as_ptr().add(72) as *const [u8; 8]));

        let consumed_count = u16::from_le_bytes([data[176], data[177]]);
        let max_consumes = u16::from_le_bytes([data[178], data[179]]);

        let expires_at_slot = u64::from_le_bytes(*(data.as_ptr().add(180) as *const [u8; 8]));

        let status = data[188];

        Ok(ParsedClaim {
            miner_authority,
            batch_hash,
            slot,
            status,
            consumed_count,
            max_consumes,
            expires_at_slot,
        })
    }
}

/// Helper to compute batch hash from job ID (k=1 for MVP)
pub fn compute_batch_hash(job_id: &str) -> [u8; 32] {
    use blake3::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(job_id.as_bytes());
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_batch_hash_determinism() {
        let job_id = "test-job-123";
        let hash1 = compute_batch_hash(job_id);
        let hash2 = compute_batch_hash(job_id);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_batch_hash_uniqueness() {
        let hash1 = compute_batch_hash("job-001");
        let hash2 = compute_batch_hash("job-002");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_parse_claim_account() {
        // Create a mock claim account with proper layout
        let mut data = vec![0u8; 256];

        // discriminator (8 bytes) - skip

        // miner_authority (32 bytes at offset 8)
        let miner_authority = Pubkey::new_unique();
        data[8..40].copy_from_slice(miner_authority.as_ref());

        // batch_hash (32 bytes at offset 40)
        let batch_hash = [0xAB; 32];
        data[40..72].copy_from_slice(&batch_hash);

        // slot (8 bytes at offset 72)
        data[72..80].copy_from_slice(&1000u64.to_le_bytes());

        // consumed_count (2 bytes at offset 176)
        data[176..178].copy_from_slice(&5u16.to_le_bytes());

        // max_consumes (2 bytes at offset 178)
        data[178..180].copy_from_slice(&10u16.to_le_bytes());

        // expires_at_slot (8 bytes at offset 180)
        data[180..188].copy_from_slice(&2000u64.to_le_bytes());

        // status (1 byte at offset 188) - 1 = Revealed
        data[188] = 1;

        let account = Account {
            lamports: 1_000_000,
            data,
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        let parsed = parse_claim_account(&account).expect("Failed to parse");

        assert_eq!(parsed.miner_authority, miner_authority);
        assert_eq!(parsed.batch_hash, batch_hash);
        assert_eq!(parsed.slot, 1000);
        assert_eq!(parsed.status, 1);
        assert_eq!(parsed.consumed_count, 5);
        assert_eq!(parsed.max_consumes, 10);
        assert_eq!(parsed.expires_at_slot, 2000);
    }

    // Note: find_claim() requires live RPC connection, so it's tested in integration tests
}
