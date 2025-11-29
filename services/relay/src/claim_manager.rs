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
use tracing::{debug, error, info};

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
        info!(
            "🔍 [DEBUG] Querying accounts for registry program: {}",
            self.registry_program_id
        );
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

        // Derive registry PDA (must match miner's derivation: b"registry")
        let (registry_pda, _) =
            Pubkey::find_program_address(&[b"registry"], &self.registry_program_id);

        // Verify registry account ONCE (same for all claims)
        if let Err(e) = self.verify_registry_account(&registry_pda).await {
            return Ok(None);
        }

        // Cache for verified miner accounts to avoid redundant RPC calls
        let _verified_miners: std::collections::HashSet<Pubkey> = std::collections::HashSet::new();

        // Filter for usable claims
        let mut size_filtered = 0;
        let mut parse_failed = 0;
        let mut batch_mismatch = 0;
        let mut wildcard_found = 0;
        let mut not_revealed = 0;
        let mut expired = 0;
        let mut fully_consumed = 0;
        let mut account_verification_failed = 0;

        for (pubkey, account) in &accounts {
            // Skip if too small to be a claim
            if account.data.len() < 256 {
                size_filtered += 1;
                continue;
            }

            // Check discriminator (first 8 bytes should match "claim")
            // For now, we'll just check if it looks like a claim by checking size
            if account.data.len() != 256 {
                size_filtered += 1;
                continue;
            }

            // Parse claim account
            match parse_claim_account(&account) {
                Ok(claim) => {
                    // Check if batch_hash matches (or if claim is wildcard)
                    let is_wildcard = claim.batch_hash == [0u8; 32];

                    if !is_wildcard && claim.batch_hash != *batch_hash {
                        batch_mismatch += 1;
                        continue;
                    }

                    if is_wildcard {
                        wildcard_found += 1;
                        // Log at debug level to avoid excessive logging
                        debug!(
                            "🌟 WILDCARD claim {} (status: {}, consumed: {}/{}, expires: {}, current_slot: {})",
                            pubkey, claim.status, claim.consumed_count, claim.max_consumes, claim.expires_at_slot, current_slot
                        );
                    }

                    // Check if revealed
                    if claim.status != 1 {
                        // 1 = Revealed
                        not_revealed += 1;
                        continue;
                    }

                    // Check if expired
                    if current_slot > claim.expires_at_slot {
                        expired += 1;
                        continue;
                    }

                    // Check if fully consumed
                    if claim.consumed_count >= claim.max_consumes {
                        fully_consumed += 1;
                        continue;
                    }

                    // Found a usable claim!
                    let miner_authority = claim.miner_authority;

                    // Derive miner PDA
                    let (miner_pda, _) = Pubkey::find_program_address(
                        &[b"miner", miner_authority.as_ref()],
                        &self.registry_program_id,
                    );

                    // Verify that the miner and registry accounts exist and have correct data size
                    // This prevents "invalid account data for instruction" errors
                    if let Err(e) = self.verify_accounts_exist(&miner_pda, &registry_pda).await {
                        account_verification_failed += 1;
                        continue;
                    }

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
                Err(e) => {
                    parse_failed += 1;
                }
            }
        }

        // Log summary at debug level
        if wildcard_found > 0 {
            debug!(
                "Searched {} accounts: {} wildcard claims found ({} size_filtered, {} parse_failed, {} batch_mismatch, {} not_revealed, {} expired, {} fully_consumed, {} account_verification_failed)",
                accounts.len(),
                wildcard_found,
                size_filtered,
                parse_failed,
                batch_mismatch,
                not_revealed,
                expired,
                fully_consumed,
                account_verification_failed
            );
        }

        Ok(None)
    }

    /// Verify that the registry account exists and has the correct data size
    /// This prevents "invalid account data for instruction" errors when consuming claims
    async fn verify_registry_account(&self, registry_pda: &Pubkey) -> Result<(), Error> {
        use tokio::time::{timeout, Duration};

        // Check registry account with timeout
        match timeout(
            Duration::from_secs(10),
            self.rpc_client.get_account(registry_pda),
        )
        .await
        {
            Ok(Ok(account)) => {
                // Registry account should be exactly 180 bytes
                if account.data.len() != 188 {
                    return Err(Error::ValidationError(format!(
                        "Registry account {} has invalid data size: {} bytes (expected 180)",
                        registry_pda,
                        account.data.len()
                    )));
                }
            }
            Ok(Err(e)) => {
                return Err(Error::ValidationError(format!(
                    "Registry account {} does not exist or failed to fetch: {}",
                    registry_pda, e
                )));
            }
            Err(_) => {
                return Err(Error::ValidationError(format!(
                    "Registry account {} fetch timed out after 10s",
                    registry_pda
                )));
            }
        }

        Ok(())
    }

    /// Verify that the miner and registry accounts exist and have the correct data size
    /// This prevents "invalid account data for instruction" errors when consuming claims
    async fn verify_accounts_exist(
        &self,
        miner_pda: &Pubkey,
        registry_pda: &Pubkey,
    ) -> Result<(), Error> {
        use tokio::time::{timeout, Duration};

        // Check miner account with timeout
        match timeout(
            Duration::from_secs(10),
            self.rpc_client.get_account(miner_pda),
        )
        .await
        {
            Ok(Ok(account)) => {
                // Miner account should be exactly 56 bytes
                if account.data.len() != 56 {
                    return Err(Error::ValidationError(format!(
                        "Miner account {} has invalid data size: {} bytes (expected 56)",
                        miner_pda,
                        account.data.len()
                    )));
                }
            }
            Ok(Err(e)) => {
                return Err(Error::ValidationError(format!(
                    "Miner account {} does not exist or failed to fetch: {}",
                    miner_pda, e
                )));
            }
            Err(_) => {
                return Err(Error::ValidationError(format!(
                    "Miner account {} fetch timed out after 10s",
                    miner_pda
                )));
            }
        }

        // Check registry account with timeout
        match timeout(
            Duration::from_secs(10),
            self.rpc_client.get_account(registry_pda),
        )
        .await
        {
            Ok(Ok(account)) => {
                // Registry account should be exactly 180 bytes
                if account.data.len() != 188 {
                    return Err(Error::ValidationError(format!(
                        "Registry account {} has invalid data size: {} bytes (expected 180)",
                        registry_pda,
                        account.data.len()
                    )));
                }
            }
            Ok(Err(e)) => {
                return Err(Error::ValidationError(format!(
                    "Registry account {} does not exist or failed to fetch: {}",
                    registry_pda, e
                )));
            }
            Err(_) => {
                return Err(Error::ValidationError(format!(
                    "Registry account {} fetch timed out after 10s",
                    registry_pda
                )));
            }
        }

        Ok(())
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
/// Claim layout (256 bytes total) - NO DISCRIMINATOR:
/// - miner_authority: 32 bytes (offset 0)
/// - batch_hash: 32 bytes (offset 32)
/// - slot: 8 bytes (offset 64)
/// - slot_hash: 32 bytes (offset 72)
/// - nonce: 16 bytes (offset 104)
/// - proof_hash: 32 bytes (offset 120)
/// - mined_at_slot: 8 bytes (offset 152)
/// - revealed_at_slot: 8 bytes (offset 160)
/// - consumed_count: 2 bytes (offset 168)
/// - max_consumes: 2 bytes (offset 170)
/// - expires_at_slot: 8 bytes (offset 172)
/// - status: 1 byte (offset 180)
fn parse_claim_account(account: &Account) -> Result<ParsedClaim, Error> {
    if account.data.len() < 256 {
        return Err(Error::ValidationError("Account too small".into()));
    }

    let data = &account.data;

    // Extract fields using unsafe pointer arithmetic (same as on-chain)
    unsafe {
        let miner_authority = Pubkey::new_from_array(*(data.as_ptr() as *const [u8; 32]));

        let batch_hash: [u8; 32] = *(data.as_ptr().add(32) as *const [u8; 32]);

        let slot = u64::from_le_bytes(*(data.as_ptr().add(64) as *const [u8; 8]));

        let consumed_count = u16::from_le_bytes([data[168], data[169]]);
        let max_consumes = u16::from_le_bytes([data[170], data[171]]);

        let expires_at_slot = u64::from_le_bytes(*(data.as_ptr().add(172) as *const [u8; 8]));

        let status = data[180];

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
