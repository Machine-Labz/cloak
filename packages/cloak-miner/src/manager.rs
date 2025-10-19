//! Claim Manager - Manages PoW claim lifecycle
//!
//! Handles:
//! - Mining new claims when needed
//! - Tracking active claims
//! - Submitting mine/reveal transactions
//! - Expiry monitoring

use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use super::{
    batch::compute_single_job_hash,
    engine::MiningEngine,
    instructions::{
        build_mine_and_reveal_instructions, derive_claim_pda, derive_miner_pda,
        derive_registry_pda,
    },
    rpc::{fetch_recent_slot_hash, fetch_registry, get_current_slot},
};

/// Active claim state
#[derive(Debug, Clone)]
pub struct ClaimState {
    pub pda: Pubkey,
    pub batch_hash: [u8; 32],
    pub revealed_at_slot: u64,
    pub expires_at_slot: u64,
    pub consumed_count: u16,
    pub max_consumes: u16,
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
    /// Active claims (batch_hash -> ClaimState)
    active_claims: HashMap<Vec<u8>, ClaimState>,
}

impl ClaimManager {
    /// Create new claim manager
    pub fn new(
        rpc_url: String,
        miner_keypair: Keypair,
        program_id_str: &str,
        mining_timeout_seconds: u64,
    ) -> Result<Self> {
        let program_id = Pubkey::from_str(program_id_str)
            .map_err(|e| anyhow!("Invalid program ID: {}", e))?;

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
    /// Returns the claim PDA that can be consumed.
    pub async fn get_claim_for_job(&mut self, job_id: &str) -> Result<Pubkey> {
        let batch_hash = compute_single_job_hash(job_id);
        self.get_claim(batch_hash).await
    }

    /// Get or mine a claim for batch_hash
    ///
    /// 1. Check if we have a usable claim
    /// 2. If not, mine and reveal a new one
    /// 3. Return claim PDA
    pub async fn get_claim(&mut self, batch_hash: [u8; 32]) -> Result<Pubkey> {
        // Check if we have a usable claim
        if let Some(state) = self.active_claims.get(&batch_hash.to_vec()) {
            if self.is_claim_usable(state).await? {
                tracing::debug!(
                    "Using existing claim: {} ({}/{} consumed)",
                    state.pda,
                    state.consumed_count,
                    state.max_consumes
                );
                return Ok(state.pda);
            } else {
                tracing::info!("Existing claim expired or fully consumed, mining new one");
                self.active_claims.remove(&batch_hash.to_vec());
            }
        }

        // Mine and reveal new claim
        tracing::info!("Mining new claim for batch_hash {:x?}...", &batch_hash[0..8]);
        self.mine_and_reveal(batch_hash).await
    }

    /// Mine and reveal a new claim
    async fn mine_and_reveal(&mut self, batch_hash: [u8; 32]) -> Result<Pubkey> {
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
        tracing::info!("Starting PoW mining (timeout: {:?})...", self.mining_timeout);
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
            "Mining SUCCESS: nonce={}, proof_hash={:x?}...",
            solution.nonce,
            &solution.proof_hash[0..8]
        );

        // 4. Build and submit mine + reveal transactions
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

        // Submit mine_claim transaction
        tracing::info!("Submitting mine_claim transaction...");
        let mine_tx = Transaction::new_signed_with_payer(
            &[mine_ix],
            Some(&self.miner_keypair.pubkey()),
            &[&self.miner_keypair],
            self.rpc_client.get_latest_blockhash()?,
        );

        let mine_sig = self.rpc_client.send_and_confirm_transaction(&mine_tx)?;
        tracing::info!("Mine transaction confirmed: {}", mine_sig);

        // Submit reveal_claim transaction
        tracing::info!("Submitting reveal_claim transaction...");
        let reveal_tx = Transaction::new_signed_with_payer(
            &[reveal_ix],
            Some(&self.miner_keypair.pubkey()),
            &[&self.miner_keypair],
            self.rpc_client.get_latest_blockhash()?,
        );

        let reveal_sig = self.rpc_client.send_and_confirm_transaction(&reveal_tx)?;
        tracing::info!("Reveal transaction confirmed: {}", reveal_sig);

        // 5. Get current slot and calculate expiry
        let current_slot = get_current_slot(&self.rpc_client)?;
        let expires_at_slot = current_slot + registry.claim_window;

        // 6. Track claim
        let claim_state = ClaimState {
            pda: claim_pda,
            batch_hash,
            revealed_at_slot: current_slot,
            expires_at_slot,
            consumed_count: 0,
            max_consumes,
        };

        self.active_claims
            .insert(batch_hash.to_vec(), claim_state);

        tracing::info!(
            "Claim ready: {} (expires at slot {})",
            claim_pda,
            expires_at_slot
        );

        Ok(claim_pda)
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

    /// Record that a claim was consumed
    ///
    /// Called after successful withdraw to track claim usage.
    pub fn record_consume(&mut self, batch_hash: &[u8; 32]) {
        if let Some(state) = self.active_claims.get_mut(&batch_hash.to_vec()) {
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
                self.active_claims.remove(&batch_hash.to_vec());
            }
        }
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
