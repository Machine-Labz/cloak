# Merkle Tree Implementation & Proof Generation

This document provides a comprehensive guide to Cloak's Merkle tree implementation, including tree structure, proof generation, verification algorithms, and API specifications.

## Overview

Cloak uses a binary Merkle tree with BLAKE3-256 hashing to maintain commitments in an append-only structure. The tree provides logarithmic-sized inclusion proofs and enables efficient verification of note commitments.

## Tree Structure

### Binary Tree Properties

**Tree Configuration:**
- **Type:** Binary tree (2 children per node)
- **Height:** 32 levels (supports 2^32 ≈ 4 billion leaves)
- **Hash Function:** BLAKE3-256 (32-byte outputs)
- **Leaf Type:** Note commitments (32-byte hashes)
- **Storage:** Append-only (leaves cannot be modified)

**Tree Layout:**
```
Level 32: [root] ← Single root node
Level 31: [node_0] [node_1] ← 2 nodes
Level 30: [node_0] [node_1] [node_2] [node_3] ← 4 nodes
...
Level 1:  [node_0] ... [node_2^31-1] ← 2^31 nodes
Level 0:  [leaf_0] ... [leaf_2^32-1] ← 2^32 leaves (commitments)
```

### Node Computation

**Parent Node Formula:**
```
parent_hash = BLAKE3(left_child || right_child)
```

**Implementation:**
```rust
use blake3;

pub fn compute_parent_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(64);
    preimage.extend_from_slice(&left);   // 32 bytes
    preimage.extend_from_slice(&right);  // 32 bytes
    
    blake3::hash(&preimage).as_bytes().try_into().unwrap()
}
```

**Leaf Assignment:**
- New deposits are assigned the next available leaf index
- Leaf index starts at 0 and increments sequentially
- Each leaf contains a note commitment: `C = BLAKE3(amount || r || pk_spend)`

## Merkle Path Structure

### Path Representation

**Path Elements:**
```rust
pub struct MerklePath {
    pub path_elements: Vec<[u8; 32]>,  // Sibling hashes at each level
    pub path_indices: Vec<u32>,        // Left/right indicators (0=left, 1=right)
}
```

**Path Properties:**
- **Length:** 32 elements (one per tree level)
- **Size:** 32 × 32 + 32 × 4 = 1,152 bytes
- **Order:** Bottom-up (leaf to root)
- **Indexing:** 0-based (0=left child, 1=right child)

### Path Generation Algorithm

**Indexer Implementation:**
```rust
pub fn generate_merkle_path(
    tree: &MerkleTree,
    leaf_index: u32,
) -> Result<MerklePath, String> {
    let mut path_elements = Vec::with_capacity(32);
    let mut path_indices = Vec::with_capacity(32);
    
    let mut current_index = leaf_index;
    
    for level in 0..32 {
        let sibling_index = current_index ^ 1; // Flip last bit
        
        if let Some(sibling_hash) = tree.get_node_hash(level, sibling_index) {
            path_elements.push(sibling_hash);
            path_indices.push((current_index & 1) as u32);
        } else {
            return Err(format!("Missing sibling at level {}, index {}", level, sibling_index));
        }
        
        current_index >>= 1; // Move to parent level
    }
    
    Ok(MerklePath {
        path_elements,
        path_indices,
    })
}
```

## Path Verification Algorithm

### Verification Process

**Verification Steps:**
1. Start with leaf commitment
2. For each level, combine with sibling hash
3. Apply BLAKE3 hash to get parent
4. Continue until root is reached
5. Compare computed root with expected root

**Implementation:**
```rust
pub fn verify_merkle_path(
    leaf: [u8; 32],
    path: &MerklePath,
    leaf_index: u32,
    expected_root: [u8; 32],
) -> Result<bool, String> {
    if path.path_elements.len() != 32 {
        return Err("Invalid path length".to_string());
    }
    
    if path.path_indices.len() != 32 {
        return Err("Invalid path indices length".to_string());
    }
    
    let mut current_hash = leaf;
    let mut current_index = leaf_index;
    
    for level in 0..32 {
        let sibling_hash = path.path_elements[level];
        let is_right = path.path_indices[level] == 1;
        
        if is_right {
            // Current is left child, sibling is right
            let mut preimage = Vec::new();
            preimage.extend_from_slice(&current_hash);
            preimage.extend_from_slice(&sibling_hash);
            current_hash = blake3::hash(&preimage).as_bytes().try_into().unwrap();
        } else {
            // Current is right child, sibling is left
            let mut preimage = Vec::new();
            preimage.extend_from_slice(&sibling_hash);
            preimage.extend_from_slice(&current_hash);
            current_hash = blake3::hash(&preimage).as_bytes().try_into().unwrap();
        }
        
        current_index >>= 1; // Move to parent level
    }
    
    Ok(current_hash == expected_root)
}
```

### Circuit Integration

**SP1 Guest Implementation:**
```rust
// In SP1 guest program
fn verify_merkle_inclusion(
    leaf: [u8; 32],
    path: &MerklePath,
    leaf_index: u32,
) -> [u8; 32] {
    let mut current_hash = leaf;
    
    for (i, sibling) in path.path_elements.iter().enumerate() {
        let is_right = (leaf_index >> i) & 1 == 1;
        
        if is_right {
            // Current is left child, sibling is right
            let mut preimage = Vec::new();
            preimage.extend_from_slice(&current_hash);
            preimage.extend_from_slice(sibling);
            current_hash = blake3(&preimage);
        } else {
            // Current is right child, sibling is left
            let mut preimage = Vec::new();
            preimage.extend_from_slice(sibling);
            preimage.extend_from_slice(&current_hash);
            current_hash = blake3(&preimage);
        }
    }
    
    current_hash
}
```

## Tree Storage & Management

### Indexer Implementation

**Tree Storage:**
```rust
pub struct MerkleTree {
    pub height: u32,                    // Tree height (32)
    pub next_leaf_index: u32,            // Next available leaf index
    pub root: [u8; 32],                  // Current root hash
    pub nodes: HashMap<(u32, u32), [u8; 32]>, // (level, index) -> hash
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            height: 32,
            next_leaf_index: 0,
            root: [0u8; 32], // Empty tree root
            nodes: HashMap::new(),
        }
    }
    
    pub fn append_leaf(&mut self, commitment: [u8; 32]) -> u32 {
        let leaf_index = self.next_leaf_index;
        
        // Store leaf
        self.nodes.insert((0, leaf_index), commitment);
        
        // Update tree upward
        self.update_tree_upward(leaf_index);
        
        // Increment next leaf index
        self.next_leaf_index += 1;
        
        leaf_index
    }
    
    fn update_tree_upward(&mut self, leaf_index: u32) {
        let mut current_index = leaf_index;
        
        for level in 0..self.height {
            let left_index = current_index & !1; // Clear last bit
            let right_index = left_index + 1;
            
            let left_hash = self.nodes.get(&(level, left_index))
                .copied()
                .unwrap_or([0u8; 32]);
            let right_hash = self.nodes.get(&(level, right_index))
                .copied()
                .unwrap_or([0u8; 32]);
            
            let parent_hash = compute_parent_hash(left_hash, right_hash);
            let parent_index = current_index >> 1;
            
            self.nodes.insert((level + 1, parent_index), parent_hash);
            
            if level + 1 == self.height {
                self.root = parent_hash;
            }
            
            current_index = parent_index;
        }
    }
}
```

### Database Schema

**PostgreSQL Tables:**
```sql
-- Merkle tree nodes
CREATE TABLE merkle_nodes (
    level INTEGER NOT NULL,
    index INTEGER NOT NULL,
    hash BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (level, index)
);

-- Tree metadata
CREATE TABLE tree_metadata (
    id INTEGER PRIMARY KEY DEFAULT 1,
    height INTEGER NOT NULL DEFAULT 32,
    next_leaf_index INTEGER NOT NULL DEFAULT 0,
    root BYTEA NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Leaf commitments
CREATE TABLE leaf_commitments (
    leaf_index INTEGER PRIMARY KEY,
    commitment BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## API Specifications

### Indexer API Endpoints

**Get Current Root:**
```http
GET /api/v1/merkle/root
```

**Response:**
```json
{
  "root": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "next_index": 42,
  "height": 32
}
```

**Get Merkle Proof:**
```http
GET /api/v1/merkle/proof/{leaf_index}
```

**Response:**
```json
{
  "path_elements": [
    "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "..."
  ],
  "path_indices": [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
  "leaf": "commitment_hash_at_leaf_index",
  "root": "current_tree_root"
}
```

**Get Leaf Range:**
```http
GET /api/v1/merkle/leaves?start={start}&limit={limit}
```

**Response:**
```json
{
  "leaves": [
    {
      "index": 0,
      "commitment": "commitment_hash_0"
    },
    {
      "index": 1,
      "commitment": "commitment_hash_1"
    }
  ],
  "has_more": true,
  "total_count": 1000
}
```

## Performance Characteristics

### Computational Complexity

**Tree Operations:**
- **Append Leaf:** O(log n) - logarithmic tree height
- **Generate Proof:** O(log n) - traverse from leaf to root
- **Verify Proof:** O(log n) - hash operations per level
- **Update Root:** O(log n) - propagate changes upward

**Memory Usage:**
- **Tree Storage:** O(n) - store all nodes
- **Proof Size:** O(log n) - 32 levels × 32 bytes = 1KB
- **Root Storage:** O(1) - single 32-byte hash

### Scalability Metrics

**Tree Capacity:**
- **Maximum Leaves:** 2^32 ≈ 4 billion
- **Proof Size:** 1,024 bytes (32 levels × 32 bytes)
- **Verification Time:** ~32 hash operations
- **Storage Growth:** Linear with number of leaves

**Performance Targets:**
- **Append Time:** < 1ms (p95)
- **Proof Generation:** < 5ms (p95)
- **Proof Verification:** < 1ms (p95)
- **Root Update:** < 1ms (p95)

## Security Properties

### Cryptographic Security

**Hash Function Security:**
- **BLAKE3-256:** 128-bit collision resistance
- **Preimage Resistance:** 2^256 operations
- **Second Preimage Resistance:** 2^256 operations

**Tree Security:**
- **Append-Only:** Cannot modify existing leaves
- **Integrity:** Any modification changes root
- **Completeness:** Valid proofs always verify
- **Soundness:** Invalid proofs never verify

### Attack Resistance

**Collision Attacks:**
- **Threat:** Finding two different leaves with same hash
- **Mitigation:** BLAKE3 collision resistance
- **Security:** 2^128 operations required

**Second Preimage Attacks:**
- **Threat:** Finding different leaf with same hash as given leaf
- **Mitigation:** BLAKE3 second preimage resistance
- **Security:** 2^256 operations required

**Tree Modification Attacks:**
- **Threat:** Modifying existing tree nodes
- **Mitigation:** Append-only property, root verification
- **Security:** Any modification invalidates proofs

## Root Management

### Root Ring Buffer

**On-Chain Storage:**
```rust
pub struct RootsRing {
    pub head: u8,                    // Current position (0-63)
    pub roots: [[u8; 32]; 64],      // Ring buffer of recent roots
}
```

**Root Update Process:**
1. Indexer computes new root after appending leaf
2. Admin pushes root to on-chain ring buffer
3. Ring buffer advances head pointer
4. Recent roots available for proof verification

**Root Validation:**
```rust
pub fn validate_root_in_ring(
    ring: &RootsRing,
    root: [u8; 32],
) -> bool {
    ring.roots.iter().any(|&stored_root| stored_root == root)
}
```

### Root Freshness

**Freshness Requirements:**
- Only recent roots accepted for verification
- 64-slot window for root validity
- Stale roots rejected with `RootNotFound` error
- Prevents replay attacks with old roots

**Root Anchoring (Future):**
- Anchor roots to Solana blockchain
- Prevent indexer equivocation
- Cryptographic root binding
- Decentralized root verification

## Error Handling

### Common Errors

**Invalid Leaf Index:**
```rust
pub enum MerkleError {
    InvalidLeafIndex { index: u32, max_index: u32 },
    MissingSibling { level: u32, index: u32 },
    InvalidPathLength { expected: usize, actual: usize },
    RootNotFound { root: [u8; 32] },
    VerificationFailed { expected: [u8; 32], actual: [u8; 32] },
}
```

**Error Recovery:**
```rust
impl MerkleTree {
    pub fn recover_from_error(&mut self, error: MerkleError) -> Result<(), String> {
        match error {
            MerkleError::InvalidLeafIndex { index, max_index } => {
                if index > max_index {
                    return Err(format!("Leaf index {} exceeds maximum {}", index, max_index));
                }
            }
            MerkleError::MissingSibling { level, index } => {
                // Rebuild tree from database
                self.rebuild_tree_from_db()?;
            }
            _ => {
                // Handle other errors
            }
        }
        Ok(())
    }
}
```

## Testing & Validation

### Unit Tests

**Path Generation Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle_path_generation() {
        let mut tree = MerkleTree::new();
        
        // Add some leaves
        let commitment1 = [0x01u8; 32];
        let commitment2 = [0x02u8; 32];
        
        let index1 = tree.append_leaf(commitment1);
        let index2 = tree.append_leaf(commitment2);
        
        // Generate paths
        let path1 = tree.generate_merkle_path(index1).unwrap();
        let path2 = tree.generate_merkle_path(index2).unwrap();
        
        // Verify paths
        assert!(verify_merkle_path(commitment1, &path1, index1, tree.root).unwrap());
        assert!(verify_merkle_path(commitment2, &path2, index2, tree.root).unwrap());
    }
    
    #[test]
    fn test_merkle_path_verification() {
        let leaf = [0x01u8; 32];
        let path = MerklePath {
            path_elements: vec![[0x02u8; 32]; 32],
            path_indices: vec![0; 32],
        };
        let leaf_index = 0;
        let expected_root = [0x03u8; 32];
        
        let result = verify_merkle_path(leaf, &path, leaf_index, expected_root);
        assert!(result.is_ok());
    }
}
```

**Integration Tests:**
```rust
#[test]
fn test_end_to_end_merkle_operations() {
    let mut tree = MerkleTree::new();
    
    // Add multiple leaves
    let commitments = vec![
        [0x01u8; 32],
        [0x02u8; 32],
        [0x03u8; 32],
        [0x04u8; 32],
    ];
    
    let mut indices = Vec::new();
    for commitment in commitments {
        let index = tree.append_leaf(commitment);
        indices.push(index);
    }
    
    // Verify all paths
    for (i, &commitment) in commitments.iter().enumerate() {
        let path = tree.generate_merkle_path(indices[i]).unwrap();
        assert!(verify_merkle_path(commitment, &path, indices[i], tree.root).unwrap());
    }
    
    // Verify root consistency
    let computed_root = tree.compute_root();
    assert_eq!(tree.root, computed_root);
}
```

## Monitoring & Metrics

### Key Metrics

**Tree Metrics:**
```rust
pub struct MerkleMetrics {
    pub total_leaves: u64,
    pub tree_height: u32,
    pub root_hash: [u8; 32],
    pub append_time_ms: f64,
    pub proof_generation_time_ms: f64,
    pub proof_verification_time_ms: f64,
}
```

**Performance Monitoring:**
- Leaf append rate (leaves per second)
- Proof generation latency (p50, p95, p99)
- Proof verification latency (p50, p95, p99)
- Tree storage size (bytes)
- Root update frequency

**Health Checks:**
- Tree integrity verification
- Root consistency checks
- Database connectivity
- Memory usage monitoring

This Merkle tree implementation provides a robust foundation for Cloak's privacy-preserving protocol with strong security guarantees and efficient performance characteristics.