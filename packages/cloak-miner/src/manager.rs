//! Claim Manager - Manages PoW claim lifecycle
//!
//! Handles:
//! - Mining new claims when needed
//! - Tracking active claims
//! - Submitting mine/reveal transactions
//! - Expiry monitoring

use std::{collections::HashMap, str::FromStr, time::Duration};

use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use super::{
    engine::MiningEngine,
    instructions::{build_mine_and_reveal_instructions, derive_claim_pda, derive_registry_pda},
    rpc::{fetch_recent_slot_hash, fetch_registry, get_current_slot},
};

/// Helper to compute batch hash from job ID (k=1 for MVP)
fn compute_batch_hash(job_id: &str) -> [u8; 32] {
    use blake3::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(job_id.as_bytes());
    *hasher.finalize().as_bytes()
}

/// Active claim state
#[derive(Debug, Clone)]
pub struct ClaimState {
    pub pda: Pubkey,
    pub batch_hash: [u8; 32],
    pub slot: u64, // Add slot for unique key
    pub revealed_at_slot: u64,
    pub expires_at_slot: u64,
    pub consumed_count: u16,
    pub max_consumes: u16,
}

/// Unique key for claim tracking (batch_hash + slot for wildcards)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClaimKey {
    batch_hash: [u8; 32],
    slot: u64,
}

/// Claim manager
pub struct ClaimManager {
    /// RPC client for on-chain communication
    rpc_client: RpcClient,
    /// Miner keypair for signing transactions
    miner_keypair: Keypair,
    /// Scramble registry program ID
    program_id: Pubkey,
    /// Mining timeout
    mining_timeout: Duration,
    /// Active claims ((batch_hash, slot) -> ClaimState)
    active_claims: HashMap<ClaimKey, ClaimState>,
}

impl ClaimManager {
    /// Create new claim manager
    pub fn new(
        rpc_url: String,
        miner_keypair: Keypair,
        program_id_str: &str,
        mining_timeout_seconds: u64,
    ) -> Result<Self> {
        let program_id =
            Pubkey::from_str(program_id_str).map_err(|e| anyhow!("Invalid program ID: {}", e))?;

        Ok(Self {
            rpc_client: RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed()),
            miner_keypair,
            program_id,
            mining_timeout: Duration::from_secs(mining_timeout_seconds),
            active_claims: HashMap::new(),
        })
    }

    /// Get or mine a claim for a single job
    ///
    /// Returns the claim PDA and mining solution that can be consumed.
    /// Uses the actual batch_hash computed from the job_id to match relay expectations.
    pub async fn get_claim_for_job(
        &mut self,
        job_id: &str,
    ) -> Result<(Pubkey, super::engine::MiningSolution)> {
        // Compute batch_hash from job_id to match what the relay expects
        let batch_hash = compute_batch_hash(job_id);

        tracing::debug!(
            "Mining claim for job '{}' with batch_hash: {:x?}...",
            job_id,
            &batch_hash[0..8]
        );

        // Always mine a new claim for each job to ensure fresh PoW work
        tracing::info!(
            "Mining new claim for batch_hash {:x?}...",
            &batch_hash[0..8]
        );
        self.mine_and_reveal(batch_hash).await
    }

    /// Get or mine a claim for batch_hash
    ///
    /// 1. Check if we have a usable claim
    /// 2. If not, mine and reveal a new one
    /// 3. Return claim PDA
    pub async fn get_claim(&mut self, batch_hash: [u8; 32]) -> Result<Pubkey> {
        let (claim_pda, _) = self.get_claim_with_solution(batch_hash).await?;
        Ok(claim_pda)
    }

    /// Get or mine a claim for batch_hash with solution details
    ///
    /// 1. Check if we have a usable claim
    /// 2. If not, mine and reveal a new one
    /// 3. Return claim PDA and mining solution
    pub async fn get_claim_with_solution(
        &mut self,
        batch_hash: [u8; 32],
    ) -> Result<(Pubkey, super::engine::MiningSolution)> {
        // Check if we have a usable claim (search by batch_hash)
        if let Some((key, state)) = self
            .active_claims
            .iter()
            .find(|(k, _)| k.batch_hash == batch_hash)
        {
            if self.is_claim_usable(state).await? {
                tracing::debug!(
                    "Using existing claim: {} ({}/{} consumed)",
                    state.pda,
                    state.consumed_count,
                    state.max_consumes
                );
                // Return a dummy solution for existing claims
                let dummy_solution = super::engine::MiningSolution {
                    nonce: 0,
                    proof_hash: [0; 32],
                    attempts: 0,
                    mining_time: std::time::Duration::from_secs(0),
                };
                return Ok((state.pda.clone(), dummy_solution));
            } else {
                tracing::info!("Existing claim expired or fully consumed, mining new one");
                let key_to_remove = key.clone();
                self.active_claims.remove(&key_to_remove);
            }
        }

        // Mine and reveal new claim
        tracing::info!(
            "Mining new claim for batch_hash {:x?}...",
            &batch_hash[0..8]
        );
        self.mine_and_reveal(batch_hash).await
    }

    /// Mine and reveal a new claim
    ///
    /// Forces mining of a fresh claim, bypassing the cache.
    /// Used in continuous mining mode to create unique claims with different slots.
    pub async fn mine_and_reveal(
        &mut self,
        batch_hash: [u8; 32],
    ) -> Result<(Pubkey, super::engine::MiningSolution)> {
        // 1. Fetch registry state
        let (registry_pda, _) = derive_registry_pda(&self.program_id);
        let registry = fetch_registry(&self.rpc_client, &registry_pda)?;

        tracing::info!(
            "Registry state: difficulty={:x?}..., reveal_window={}, claim_window={}",
            &registry.current_difficulty[0..4],
            registry.reveal_window,
            registry.claim_window
        );

        // 2. Fetch recent slot hash
        let (slot, slot_hash) = fetch_recent_slot_hash(&self.rpc_client)?;
        tracing::info!("Using slot {} with hash {:x?}...", slot, &slot_hash[0..8]);

        // 3. Run mining engine
        tracing::info!(
            "Starting PoW mining (timeout: {:?})...",
            self.mining_timeout
        );
        let engine = MiningEngine::new(
            registry.current_difficulty,
            slot,
            slot_hash,
            self.miner_keypair.pubkey(),
            batch_hash,
        );

        let solution = engine
            .mine_with_timeout(self.mining_timeout)
            .map_err(|e| anyhow!("Mining failed: {}", e))?;

        tracing::info!(
            "Mining SUCCESS: nonce={}, attempts={}, time={:.2}s, hash_rate={:.0} H/s",
            solution.nonce,
            solution.attempts,
            solution.mining_time.as_secs_f64(),
            solution.attempts as f64 / solution.mining_time.as_secs_f64()
        );

        // 4. Build and submit mine + reveal in a SINGLE transaction
        // This avoids reveal window expiry issues caused by delays between separate transactions
        let max_consumes = 1u16; // For now, k=1 (single job per claim)

        let (mine_ix, reveal_ix) = build_mine_and_reveal_instructions(
            &self.program_id,
            &self.miner_keypair.pubkey(),
            slot,
            slot_hash,
            batch_hash,
            solution.nonce,
            solution.proof_hash,
            max_consumes,
        )?;

        // Derive claim PDA
        let (claim_pda, _) = derive_claim_pda(
            &self.program_id,
            &self.miner_keypair.pubkey(),
            &batch_hash,
            slot,
        );

        // Submit BOTH mine and reveal in a single transaction
        // This ensures reveal happens immediately after mine, avoiding reveal window expiry
        tracing::info!("Submitting mine+reveal transaction (combined)...");
        let combined_tx = Transaction::new_signed_with_payer(
            &[mine_ix, reveal_ix],
            Some(&self.miner_keypair.pubkey()),
            &[&self.miner_keypair],
            self.rpc_client.get_latest_blockhash()?,
        );

        let sig = self.rpc_client.send_and_confirm_transaction(&combined_tx)?;
        tracing::info!("Mine+reveal transaction confirmed: {}", sig);

        // 5. Get current slot and calculate expiry
        let current_slot = get_current_slot(&self.rpc_client)?;
        let expires_at_slot = current_slot + registry.claim_window;

        // 6. Track claim with unique key (batch_hash + slot)
        let claim_state = ClaimState {
            pda: claim_pda,
            batch_hash,
            slot,
            revealed_at_slot: current_slot,
            expires_at_slot,
            consumed_count: 0,
            max_consumes,
        };

        let key = ClaimKey { batch_hash, slot };
        self.active_claims.insert(key, claim_state);

        tracing::info!(
            "Claim ready: {} (expires at slot {})",
            claim_pda,
            expires_at_slot
        );

        Ok((claim_pda, solution))
    }

    /// Check if claim is still usable
    async fn is_claim_usable(&self, state: &ClaimState) -> Result<bool> {
        let current_slot = get_current_slot(&self.rpc_client)?;

        // Check not expired
        if current_slot > state.expires_at_slot {
            tracing::debug!(
                "Claim {} expired (current slot {} > expiry {})",
                state.pda,
                current_slot,
                state.expires_at_slot
            );
            return Ok(false);
        }

        // Check not fully consumed
        if state.consumed_count >= state.max_consumes {
            tracing::debug!(
                "Claim {} fully consumed ({}/{})",
                state.pda,
                state.consumed_count,
                state.max_consumes
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Get the number of active claims (checks validity)
    pub async fn get_active_claims_count(&mut self) -> Result<usize> {
        // Clean up expired/consumed claims first
        let current_slot = get_current_slot(&self.rpc_client)?;

        self.active_claims.retain(|_, state| {
            let not_expired = current_slot <= state.expires_at_slot;
            let not_fully_consumed = state.consumed_count < state.max_consumes;
            not_expired && not_fully_consumed
        });

        Ok(self.active_claims.len())
    }

    /// Check if we have a usable claim for the given batch_hash
    ///
    /// Returns true if a claim exists and is still valid
    pub fn has_usable_claim(&self, batch_hash: &[u8; 32]) -> bool {
        // Search for any claim with matching batch_hash
        self.active_claims.iter().any(|(k, state)| {
            k.batch_hash == *batch_hash && state.consumed_count < state.max_consumes
        })
    }

    /// Record that a claim was consumed
    ///
    /// Called after successful withdraw to track claim usage.
    pub fn record_consume(&mut self, batch_hash: &[u8; 32]) {
        // Find the claim key by searching for matching batch_hash
        if let Some(key) = self
            .active_claims
            .keys()
            .find(|k| k.batch_hash == *batch_hash)
            .cloned()
        {
            if let Some(state) = self.active_claims.get_mut(&key) {
                state.consumed_count += 1;
                tracing::debug!(
                    "Claim {} consumed: {}/{}",
                    state.pda,
                    state.consumed_count,
                    state.max_consumes
                );

                // Remove if fully consumed
                if state.consumed_count >= state.max_consumes {
                    tracing::info!("Claim {} fully consumed, removing from cache", state.pda);
                    self.active_claims.remove(&key);
                }
            }
        }
    }

    /// Clean up expired claims
    ///
    /// Removes claims that have expired or are fully consumed
    pub async fn cleanup_expired_claims(&mut self) -> Result<usize> {
        let current_slot = get_current_slot(&self.rpc_client)?;
        let before = self.active_claims.len();

        self.active_claims.retain(|_, state| {
            let not_expired = current_slot <= state.expires_at_slot;
            let not_fully_consumed = state.consumed_count < state.max_consumes;
            not_expired && not_fully_consumed
        });

        Ok(before - self.active_claims.len())
    }

    /// Get miner public key
    pub fn miner_pubkey(&self) -> Pubkey {
        self.miner_keypair.pubkey()
    }

    /// Get program ID
    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_state_tracking() {
        let pda = Pubkey::new_unique();
        let batch_hash = [0x42; 32];

        let state = ClaimState {
            pda,
            batch_hash,
            slot: 1000,
            revealed_at_slot: 1000,
            expires_at_slot: 1300,
            consumed_count: 0,
            max_consumes: 5,
        };

        // Initially usable
        assert_eq!(state.consumed_count, 0);

        // Can track consumption
        let mut count = state.consumed_count;
        count += 1;
        assert_eq!(count, 1);
    }

    #[test]
    fn test_batch_hash_key() {
        let batch_hash = [0x88; 32];
        let key = batch_hash.to_vec();

        let mut map: HashMap<Vec<u8>, u16> = HashMap::new();
        map.insert(key.clone(), 42);

        assert_eq!(map.get(&key), Some(&42));
    }
}
