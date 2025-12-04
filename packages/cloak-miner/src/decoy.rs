//! Decoy Transaction Module - Creates indistinguishable decoy transactions
//!
//! This module enables miners to create decoy deposits and withdrawals that
//! are indistinguishable from real user transactions on-chain.
//!
//! Flow:
//! 1. Decoy Deposit: Generate note secrets, deposit to shield-pool
//! 2. Decoy Withdraw: Reveal note secrets, withdraw from shield-pool (no ZK proof needed)

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Domain tag for commitment computation (empty to match ZK circuit)
#[allow(dead_code)]
const COMMITMENT_DOMAIN: &[u8] = b"";

/// A deposited note that can be withdrawn later
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoyNote {
    /// Spending key secret (32 bytes)
    pub sk_spend: [u8; 32],
    /// Randomness (32 bytes)
    pub r: [u8; 32],
    /// Deposit amount in lamports
    pub amount: u64,
    /// Leaf index in Merkle tree (set after deposit confirmed)
    pub leaf_index: Option<u32>,
    /// Commitment hash
    pub commitment: [u8; 32],
    /// Deposit transaction signature
    pub deposit_signature: Option<String>,
    /// Timestamp when deposited
    pub deposited_at: u64,
    /// Whether this note has been spent
    pub spent: bool,
}

impl DecoyNote {
    /// Generate a new decoy note with random secrets
    pub fn generate(amount: u64) -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();

        let mut sk_spend = [0u8; 32];
        let mut r = [0u8; 32];
        rng.fill_bytes(&mut sk_spend);
        rng.fill_bytes(&mut r);

        // Compute pk_spend = BLAKE3(sk_spend)
        let pk_spend = blake3::hash(&sk_spend);

        // Compute commitment = BLAKE3(amount || r || pk_spend)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&amount.to_le_bytes());
        hasher.update(&r);
        hasher.update(pk_spend.as_bytes());
        let commitment = *hasher.finalize().as_bytes();

        Self {
            sk_spend,
            r,
            amount,
            leaf_index: None,
            commitment,
            deposit_signature: None,
            deposited_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            spent: false,
        }
    }

    /// Compute the nullifier for this note (requires leaf_index)
    pub fn compute_nullifier(&self) -> Result<[u8; 32]> {
        let leaf_index = self
            .leaf_index
            .ok_or_else(|| anyhow!("Note has no leaf_index yet"))?;

        // nullifier = BLAKE3(sk_spend || leaf_index)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.sk_spend);
        hasher.update(&leaf_index.to_le_bytes());
        Ok(*hasher.finalize().as_bytes())
    }

    /// Get pk_spend derived from sk_spend
    pub fn pk_spend(&self) -> [u8; 32] {
        *blake3::hash(&self.sk_spend).as_bytes()
    }
}

/// Storage for decoy notes
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NoteStorage {
    /// Notes indexed by commitment hex
    notes: HashMap<String, DecoyNote>,
    /// Path to storage file
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}

impl NoteStorage {
    /// Create new note storage with file path
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            notes: HashMap::new(),
            storage_path: Some(storage_path),
        }
    }

    /// Load from file or create new
    pub fn load_or_create(storage_path: PathBuf) -> Result<Self> {
        if storage_path.exists() {
            let file = File::open(&storage_path)
                .with_context(|| format!("Failed to open {}", storage_path.display()))?;
            let reader = BufReader::new(file);
            let mut storage: NoteStorage = serde_json::from_reader(reader)
                .with_context(|| format!("Failed to parse {}", storage_path.display()))?;
            storage.storage_path = Some(storage_path);
            Ok(storage)
        } else {
            Ok(Self::new(storage_path))
        }
    }

    /// Save to file
    pub fn save(&self) -> Result<()> {
        let path = self
            .storage_path
            .as_ref()
            .ok_or_else(|| anyhow!("No storage path set"))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file =
            File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Add a new note
    pub fn add_note(&mut self, note: DecoyNote) -> Result<()> {
        let key = hex::encode(note.commitment);
        self.notes.insert(key, note);
        self.save()
    }

    /// Update note with leaf index after deposit confirmed
    pub fn set_leaf_index(&mut self, commitment: &[u8; 32], leaf_index: u32) -> Result<()> {
        let key = hex::encode(commitment);
        if let Some(note) = self.notes.get_mut(&key) {
            note.leaf_index = Some(leaf_index);
            self.save()?;
        }
        Ok(())
    }

    /// Mark note as spent
    pub fn mark_spent(&mut self, commitment: &[u8; 32]) -> Result<()> {
        let key = hex::encode(commitment);
        if let Some(note) = self.notes.get_mut(&key) {
            note.spent = true;
            self.save()?;
        }
        Ok(())
    }

    /// Get all unspent notes that have leaf_index set
    pub fn get_withdrawable_notes(&self) -> Vec<&DecoyNote> {
        self.notes
            .values()
            .filter(|n| !n.spent && n.leaf_index.is_some())
            .collect()
    }

    /// Get all pending notes (deposited but no leaf_index yet)
    pub fn get_pending_notes(&self) -> Vec<&DecoyNote> {
        self.notes
            .values()
            .filter(|n| !n.spent && n.leaf_index.is_none())
            .collect()
    }

    /// Get total deposited amount (unspent notes)
    pub fn get_total_deposited(&self) -> u64 {
        self.notes
            .values()
            .filter(|n| !n.spent)
            .map(|n| n.amount)
            .sum()
    }

    /// Get note by commitment
    pub fn get_note(&self, commitment: &[u8; 32]) -> Option<&DecoyNote> {
        let key = hex::encode(commitment);
        self.notes.get(&key)
    }

    /// Get note count
    pub fn count(&self) -> usize {
        self.notes.len()
    }

    /// Get unspent count
    pub fn unspent_count(&self) -> usize {
        self.notes.values().filter(|n| !n.spent).count()
    }
}

/// Shield Pool program ID (hardcoded)
pub const SHIELD_POOL_PROGRAM_ID: &str = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp";

/// Derive miner escrow PDA
pub fn derive_miner_escrow_pda(
    scramble_program_id: &Pubkey,
    miner_authority: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"miner_escrow", miner_authority.as_ref()],
        scramble_program_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_generation() {
        let amount = 1_000_000_000u64; // 1 SOL
        let note = DecoyNote::generate(amount);

        assert_eq!(note.amount, amount);
        assert!(!note.spent);
        assert!(note.leaf_index.is_none());

        // Commitment should be deterministic from secrets
        let pk_spend = blake3::hash(&note.sk_spend);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&amount.to_le_bytes());
        hasher.update(&note.r);
        hasher.update(pk_spend.as_bytes());
        let expected_commitment = *hasher.finalize().as_bytes();

        assert_eq!(note.commitment, expected_commitment);
    }

    #[test]
    fn test_nullifier_computation() {
        let mut note = DecoyNote::generate(1_000_000_000);
        note.leaf_index = Some(42);

        let nullifier = note.compute_nullifier().unwrap();
        assert_eq!(nullifier.len(), 32);

        // Should be deterministic
        let nullifier2 = note.compute_nullifier().unwrap();
        assert_eq!(nullifier, nullifier2);
    }

    #[test]
    fn test_note_storage() {
        let mut storage = NoteStorage::default();

        let note1 = DecoyNote::generate(1_000_000_000);
        let note2 = DecoyNote::generate(2_000_000_000);

        let commitment1 = note1.commitment;

        storage.notes.insert(hex::encode(note1.commitment), note1);
        storage.notes.insert(hex::encode(note2.commitment), note2);

        assert_eq!(storage.count(), 2);
        assert_eq!(storage.unspent_count(), 2);
        assert_eq!(storage.get_total_deposited(), 3_000_000_000);

        // No withdrawable notes yet (no leaf_index)
        assert_eq!(storage.get_withdrawable_notes().len(), 0);
        assert_eq!(storage.get_pending_notes().len(), 2);

        // Set leaf index
        storage
            .notes
            .get_mut(&hex::encode(commitment1))
            .unwrap()
            .leaf_index = Some(10);
        assert_eq!(storage.get_withdrawable_notes().len(), 1);
        assert_eq!(storage.get_pending_notes().len(), 1);
    }
}
