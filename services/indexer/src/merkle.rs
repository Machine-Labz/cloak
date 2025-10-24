use crate::error::{IndexerError, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub path_elements: Vec<String>,
    pub path_indices: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTreeState {
    pub root: String,
    pub next_index: u64,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub level: u32,
    pub index: u64,
    pub value: String,
}

/// Interface for tree storage operations
#[async_trait::async_trait]
pub trait TreeStorage: Send + Sync {
    /// Store a tree node at the specified level and index
    async fn store_node(&self, level: u32, index: u64, value: &str) -> Result<()>;

    /// Retrieve a tree node by level and index
    async fn get_node(&self, level: u32, index: u64) -> Result<Option<String>>;

    /// Get the maximum leaf index (used to initialize next_index on startup)
    async fn get_max_leaf_index(&self) -> Result<u64>;
}

/// Merkle tree implementation for Cloak privacy protocol
/// Uses BLAKE3 hashing with configurable height
pub struct MerkleTree {
    height: usize,
    zero_values: Vec<String>,
    next_index: u64,
}

impl MerkleTree {
    /// Create a new Merkle tree with the specified height and zero value
    pub fn new(height: usize, zero_value: &str) -> Result<Self> {
        if height == 0 || height > 64 {
            return Err(IndexerError::merkle_tree(
                "Tree height must be between 1 and 64",
            ));
        }

        if zero_value.len() != 64 {
            return Err(IndexerError::merkle_tree(
                "Zero value must be a 64-character hex string (32 bytes)",
            ));
        }

        let zero_values = Self::compute_zero_values(zero_value, height)?;

        tracing::info!(
            height = height,
            zero_value = zero_value,
            zero_values_computed = zero_values.len(),
            "Initialized Merkle Tree"
        );

        Ok(MerkleTree {
            height,
            zero_values,
            next_index: 0,
        })
    }

    /// Compute zero values for each level of the tree
    fn compute_zero_values(zero_value: &str, height: usize) -> Result<Vec<String>> {
        let mut zeros = vec![zero_value.to_lowercase()];

        for i in 1..height {
            let prev_zero = &zeros[i - 1];
            let hash = Self::hash_pair(prev_zero, prev_zero)?;
            zeros.push(hash);
        }

        Ok(zeros)
    }

    /// Hash two values using BLAKE3-256
    /// Inputs should be hex strings without 0x prefix
    /// Returns hex string without 0x prefix
    fn hash_pair(left: &str, right: &str) -> Result<String> {
        // Remove 0x prefix if present and ensure lowercase
        let left_clean = left.strip_prefix("0x").unwrap_or(left).to_lowercase();
        let right_clean = right.strip_prefix("0x").unwrap_or(right).to_lowercase();

        // Convert hex to bytes
        let left_bytes = hex::decode(&left_clean)?;
        let right_bytes = hex::decode(&right_clean)?;

        if left_bytes.len() != 32 || right_bytes.len() != 32 {
            return Err(IndexerError::merkle_tree(format!(
                "Invalid hash input length: left={}, right={}",
                left_bytes.len(),
                right_bytes.len()
            )));
        }

        // Concatenate and hash
        let mut hasher = Hasher::new();
        hasher.update(&left_bytes);
        hasher.update(&right_bytes);
        let hash = hasher.finalize();

        Ok(hex::encode(hash.as_bytes()))
    }

    /// Insert a new leaf into the tree and return the new root
    pub async fn insert_leaf(
        &mut self,
        leaf_value: &str,
        storage: &dyn TreeStorage,
    ) -> Result<(String, u64)> {
        let leaf_index = self.next_index;
        tracing::info!(
            leaf_value = leaf_value,
            leaf_index = leaf_index,
            "Inserting leaf"
        );

        // Validate leaf value
        let clean_leaf = leaf_value
            .strip_prefix("0x")
            .unwrap_or(leaf_value)
            .to_lowercase();
        if clean_leaf.len() != 64 {
            return Err(IndexerError::merkle_tree(
                "Leaf value must be a 64-character hex string (32 bytes)",
            ));
        }

        // Store the leaf at level 0
        storage.store_node(0, leaf_index, &clean_leaf).await?;

        // Compute and store internal nodes bottom-up
        let mut current_index = leaf_index;
        let mut current_value = clean_leaf;

        for level in 0..(self.height - 1) {
            let is_left_child = current_index % 2 == 0;
            let parent_index = current_index / 2;

            tracing::info!(
                level = level,
                current_index = current_index,
                is_left_child = is_left_child,
                parent_index = parent_index,
                leaf_index = leaf_index,
                "Processing tree level"
            );

            let (left_child, right_child) = if is_left_child {
                // Current value is left child
                let right_sibling = storage
                    .get_node(level as u32, current_index + 1)
                    .await?
                    .unwrap_or_else(|| self.zero_values[level].clone());

                tracing::debug!(
                    level = level,
                    current_index = current_index,
                    "Left child processing"
                );

                (current_value, right_sibling)
            } else {
                // Current value is right child, get left sibling
                tracing::info!(
                    level = level,
                    current_index = current_index,
                    leaf_index = leaf_index,
                    "Right child processing - looking for left sibling at index {}",
                    current_index - 1
                );
                let left_sibling = storage.get_node(level as u32, current_index - 1).await?;
                if left_sibling.is_none() {
                    // If this is the very first leaf (index 0) and we're at level 0,
                    // this is expected - we're inserting the first leaf
                    if leaf_index == 0 && level == 0 {
                        tracing::debug!(
                            level = level,
                            current_index = current_index,
                            leaf_index = leaf_index,
                            "First leaf insertion - no left sibling expected"
                        );
                        // For the first leaf, we use the zero value as the left sibling
                        let left_sibling = self.zero_values[level].clone();
                        let parent_value = Self::hash_pair(&left_sibling, &current_value)?;
                        storage
                            .store_node((level + 1) as u32, parent_index, &parent_value)
                            .await?;
                        current_index = parent_index;
                        current_value = parent_value;
                        continue;
                    } else {
                        tracing::warn!(
                            level = level,
                            current_index = current_index,
                            leaf_index = leaf_index,
                            "Missing left sibling detected, this indicates a tree inconsistency"
                        );
                        return Err(IndexerError::merkle_tree(format!(
                            "Missing left sibling at level {}, index {}. Tree is in an inconsistent state.",
                            level,
                            current_index - 1
                        )));
                    }
                }

                tracing::debug!(
                    level = level,
                    current_index = current_index,
                    left_sibling_exists = true,
                    "Right child processing"
                );

                (left_sibling.unwrap(), current_value)
            };

            // Compute parent hash
            let parent_value = Self::hash_pair(&left_child, &right_child)?;

            // Store parent node (this will update if it already exists)
            storage
                .store_node((level + 1) as u32, parent_index, &parent_value)
                .await?;

            // Move up the tree
            current_index = parent_index;
            current_value = parent_value;
        }

        self.next_index += 1;
        let root_value = current_value;

        tracing::info!(
            leaf_index = leaf_index,
            root_value = %root_value,
            next_index = self.next_index,
            "Leaf inserted successfully"
        );

        Ok((root_value, leaf_index))
    }

    /// Generate a Merkle proof for a given leaf index
    pub async fn generate_proof(
        &self,
        leaf_index: u64,
        storage: &dyn TreeStorage,
    ) -> Result<MerkleProof> {
        if leaf_index >= self.next_index {
            return Err(IndexerError::merkle_tree(format!(
                "Leaf index {} does not exist (next_index: {})",
                leaf_index, self.next_index
            )));
        }

        let mut path_elements = Vec::new();
        let mut path_indices = Vec::new();
        let mut current_index = leaf_index;

        for level in 0..(self.height - 1) {
            let is_left_child = current_index % 2 == 0;
            path_indices.push(if is_left_child { 0 } else { 1 });

            let sibling_index = if is_left_child {
                current_index + 1
            } else {
                current_index - 1
            };

            // Get sibling from storage or use zero value
            let sibling_value = storage
                .get_node(level as u32, sibling_index)
                .await?
                .unwrap_or_else(|| self.zero_values[level].clone());

            path_elements.push(sibling_value);
            current_index /= 2;
        }

        tracing::debug!(
            leaf_index = leaf_index,
            path_elements_count = path_elements.len(),
            path_indices = ?path_indices,
            "Generated Merkle proof"
        );

        Ok(MerkleProof {
            path_elements,
            path_indices,
        })
    }

    /// Verify a Merkle proof
    pub fn verify_proof(
        &self,
        leaf_value: &str,
        leaf_index: u64,
        proof: &MerkleProof,
        expected_root: &str,
    ) -> Result<bool> {
        if proof.path_elements.len() != self.height - 1 {
            tracing::error!(
                expected = self.height - 1,
                actual = proof.path_elements.len(),
                "Invalid proof length"
            );
            return Ok(false);
        }

        let clean_leaf = leaf_value
            .strip_prefix("0x")
            .unwrap_or(leaf_value)
            .to_lowercase();
        let clean_root = expected_root
            .strip_prefix("0x")
            .unwrap_or(expected_root)
            .to_lowercase();

        let mut current_value = clean_leaf;
        let mut _current_index = leaf_index;

        for i in 0..proof.path_elements.len() {
            let path_element = &proof.path_elements[i];
            let is_left_child = proof.path_indices[i] == 0;

            current_value = if is_left_child {
                // Current value is left child
                Self::hash_pair(&current_value, path_element)?
            } else {
                // Current value is right child
                Self::hash_pair(path_element, &current_value)?
            };

            _current_index /= 2;
        }

        let is_valid = current_value == clean_root;

        tracing::debug!(
            leaf_index = leaf_index,
            computed_root = %current_value,
            expected_root = %clean_root,
            is_valid = is_valid,
            "Proof verification result"
        );

        Ok(is_valid)
    }

    /// Get current tree state
    pub async fn get_tree_state(&self, storage: &dyn TreeStorage) -> Result<MerkleTreeState> {
        let root = if self.next_index == 0 {
            // Empty tree - root is zero value at max level
            self.zero_values[self.height - 1].clone()
        } else {
            // Get current root from storage
            storage
                .get_node((self.height - 1) as u32, 0)
                .await?
                .ok_or_else(|| IndexerError::merkle_tree("Root not found in storage"))?
        };

        Ok(MerkleTreeState {
            root,
            next_index: self.next_index,
        })
    }

    /// Set the next index (used during initialization from storage)
    pub fn set_next_index(&mut self, index: u64) {
        self.next_index = index;
        tracing::info!(next_index = self.next_index, "Set next index");
    }

    /// Reset the tree state (useful after database reset)
    pub fn reset_state(&mut self) {
        self.next_index = 0;
        tracing::info!("Reset Merkle tree state - next_index set to 0");
    }

    /// Get tree height
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get zero values
    pub fn zero_values(&self) -> &[String] {
        &self.zero_values
    }
}

// Tests can be added here when needed
