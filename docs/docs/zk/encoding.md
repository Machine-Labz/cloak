# Data Encoding & Serialization Specifications

This document defines the canonical data encoding formats, serialization standards, and byte-level specifications used throughout Cloak's zero-knowledge privacy layer.

## Overview

Cloak uses consistent encoding standards across all components to ensure compatibility between client applications, proof generation, on-chain verification, and service APIs. All encoding follows little-endian byte order and uses fixed-size formats where possible.

## Core Encoding Standards

### Byte Order
- **All multi-byte integers:** Little-endian (LE)
- **All byte arrays:** Big-endian (network byte order)
- **All concatenations:** No separators, fixed order

### Integer Encoding

**u64 (8 bytes):**
```rust
let value: u64 = 0x123456789ABCDEF0;
let encoded = value.to_le_bytes();
// Result: [0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]
```

**u32 (4 bytes):**
```rust
let value: u32 = 0x12345678;
let encoded = value.to_le_bytes();
// Result: [0x78, 0x56, 0x34, 0x12]
```

**u16 (2 bytes):**
```rust
let value: u16 = 0x1234;
let encoded = value.to_le_bytes();
// Result: [0x34, 0x12]
```

**u8 (1 byte):**
```rust
let value: u8 = 0x12;
let encoded = value;
// Result: 0x12
```

### Public Key Encoding

**Solana Public Key (32 bytes):**
```rust
let pubkey: Pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
let encoded = pubkey.to_bytes();
// Result: [0x00, 0x00, 0x00, ..., 0x12] (32 bytes)
```

**Byte Array (32 bytes):**
```rust
let bytes: [u8; 32] = [0x12; 32];
let encoded = bytes;
// Result: [0x12, 0x12, ..., 0x12] (32 bytes)
```

## Hash Function Specifications

### BLAKE3-256 Configuration

**Hash Function:** BLAKE3-256
**Output Size:** 32 bytes
**Security Level:** 128-bit collision resistance
**Implementation:** Standard `blake3` crate

**Usage Pattern:**
```rust
use blake3;

fn hash_data(data: &[u8]) -> [u8; 32] {
    let hash = blake3::hash(data);
    hash.as_bytes().try_into().unwrap()
}
```

## Commitment Encoding

### Commitment Computation

**Formula:** `C = BLAKE3(amount || r || pk_spend)`

**Data Layout:**
```
Preimage Structure:
├── amount: u64 LE     (8 bytes)
├── r: [u8; 32]        (32 bytes) - randomness
└── pk_spend: [u8; 32] (32 bytes) - public spend key
Total: 72 bytes
```

**Implementation:**
```rust
pub fn compute_commitment(amount: u64, r: [u8; 32], pk_spend: [u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(72);
    preimage.extend_from_slice(&amount.to_le_bytes());  // 8 bytes
    preimage.extend_from_slice(&r);                     // 32 bytes
    preimage.extend_from_slice(&pk_spend);              // 32 bytes
    
    blake3::hash(&preimage).as_bytes().try_into().unwrap()
}
```

**Example:**
```rust
let amount = 1_000_000u64;
let r = [0x01u8; 32];
let pk_spend = [0x02u8; 32];

let commitment = compute_commitment(amount, r, pk_spend);
// Result: 32-byte commitment hash
```

## Nullifier Encoding

### Nullifier Computation

**Formula:** `nf = BLAKE3(sk_spend || leaf_index)`

**Data Layout:**
```
Preimage Structure:
├── sk_spend: [u8; 32]    (32 bytes) - secret spend key
└── leaf_index: u32 LE    (4 bytes) - Merkle tree leaf index
Total: 36 bytes
```

**Implementation:**
```rust
pub fn compute_nullifier(sk_spend: [u8; 32], leaf_index: u32) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(36);
    preimage.extend_from_slice(&sk_spend);              // 32 bytes
    preimage.extend_from_slice(&leaf_index.to_le_bytes()); // 4 bytes
    
    blake3::hash(&preimage).as_bytes().try_into().unwrap()
}
```

**Example:**
```rust
let sk_spend = [0x03u8; 32];
let leaf_index = 42u32;

let nullifier = compute_nullifier(sk_spend, leaf_index);
// Result: 32-byte nullifier hash
```

## Merkle Tree Encoding

### Merkle Path Structure

**Path Elements:**
```rust
pub struct MerklePath {
    pub path_elements: Vec<[u8; 32]>,  // Sibling hashes at each level
    pub path_indices: Vec<u32>,        // Left/right indicators (0=left, 1=right)
}
```

**Path Verification:**
```rust
pub fn verify_merkle_path(
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
            current_hash = blake3::hash(&preimage).as_bytes().try_into().unwrap();
        } else {
            // Current is right child, sibling is left
            let mut preimage = Vec::new();
            preimage.extend_from_slice(sibling);
            preimage.extend_from_slice(&current_hash);
            current_hash = blake3::hash(&preimage).as_bytes().try_into().unwrap();
        }
    }
    
    current_hash
}
```

### Tree Node Encoding

**Parent Node Computation:**
```rust
pub fn compute_parent_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(64);
    preimage.extend_from_slice(&left);   // 32 bytes
    preimage.extend_from_slice(&right);  // 32 bytes
    
    blake3::hash(&preimage).as_bytes().try_into().unwrap()
}
```

## Output Encoding

### Output Structure

**Individual Output:**
```rust
pub struct Output {
    pub address: [u8; 32],  // Recipient public key
    pub amount: u64,        // Amount in lamports
}
```

**Output Serialization:**
```rust
pub fn serialize_output(output: &Output) -> Vec<u8> {
    let mut serialized = Vec::with_capacity(40);
    serialized.extend_from_slice(&output.address);     // 32 bytes
    serialized.extend_from_slice(&output.amount.to_le_bytes()); // 8 bytes
    serialized
}
```

### Outputs Hash Computation

**Formula:** `outputs_hash = BLAKE3(serialize(outputs))`

**Serialization Order:**
```
Outputs Array:
├── output[0]: address:32 || amount:8
├── output[1]: address:32 || amount:8
├── ...
└── output[n-1]: address:32 || amount:8
```

**Implementation:**
```rust
pub fn compute_outputs_hash(outputs: &[Output]) -> [u8; 32] {
    let mut serialized = Vec::new();
    
    for output in outputs {
        serialized.extend_from_slice(&output.address);     // 32 bytes
        serialized.extend_from_slice(&output.amount.to_le_bytes()); // 8 bytes
    }
    
    blake3::hash(&serialized).as_bytes().try_into().unwrap()
}
```

**Example:**
```rust
let outputs = vec![
    Output { address: [0x01u8; 32], amount: 500_000 },
    Output { address: [0x02u8; 32], amount: 300_000 },
];

let outputs_hash = compute_outputs_hash(&outputs);
// Result: 32-byte hash of serialized outputs
```

## Public Input Encoding

### Public Input Structure

**104-byte Public Input Layout:**
```rust
pub struct PublicInputs {
    pub root: [u8; 32],           // Merkle tree root (32 bytes)
    pub nullifier: [u8; 32],      // Spending nullifier (32 bytes)
    pub outputs_hash: [u8; 32],   // Hash of output recipients (32 bytes)
    pub amount: u64,              // Total amount being spent (8 bytes)
}
```

**Serialization:**
```rust
pub fn serialize_public_inputs(inputs: &PublicInputs) -> [u8; 104] {
    let mut serialized = [0u8; 104];
    
    // Root (32 bytes)
    serialized[0..32].copy_from_slice(&inputs.root);
    
    // Nullifier (32 bytes)
    serialized[32..64].copy_from_slice(&inputs.nullifier);
    
    // Outputs hash (32 bytes)
    serialized[64..96].copy_from_slice(&inputs.outputs_hash);
    
    // Amount (8 bytes)
    serialized[96..104].copy_from_slice(&inputs.amount.to_le_bytes());
    
    serialized
}
```

**Deserialization:**
```rust
pub fn deserialize_public_inputs(data: &[u8; 104]) -> PublicInputs {
    PublicInputs {
        root: data[0..32].try_into().unwrap(),
        nullifier: data[32..64].try_into().unwrap(),
        outputs_hash: data[64..96].try_into().unwrap(),
        amount: u64::from_le_bytes(data[96..104].try_into().unwrap()),
    }
}
```

## Fee Structure Encoding

### Fee Calculation

**Variable Fee:** `(amount * 5) / 1000` (0.5%)
**Fixed Fee:** `2,500,000` lamports (0.0025 SOL)
**Total Fee:** `variable_fee + fixed_fee`

**Implementation:**
```rust
pub fn calculate_fee(amount: u64) -> u64 {
    let variable_fee = (amount * 5) / 1000;  // 0.5%
    let fixed_fee = 2_500_000;               // 0.0025 SOL
    variable_fee + fixed_fee
}
```

**Fee Validation:**
```rust
pub fn validate_amount_conservation(
    input_amount: u64,
    outputs: &[Output],
) -> Result<(), String> {
    let total_outputs: u64 = outputs.iter().map(|o| o.amount).sum();
    let fee = calculate_fee(input_amount);
    
    if total_outputs + fee == input_amount {
        Ok(())
    } else {
        Err(format!(
            "Amount conservation failed: {} != {} + {}",
            input_amount, total_outputs, fee
        ))
    }
}
```

## Encrypted Output Encoding

### Encryption Format

**Encrypted Output Structure:**
```
Encrypted Output:
├── version: u8        (1 byte) - Version identifier (=1)
├── cipher_id: u8      (1 byte) - Cipher identifier (=1 for XChaCha20)
├── epk: [u8; 32]      (32 bytes) - Ephemeral public key
├── nonce: [u8; 24]    (24 bytes) - Nonce for encryption
├── tag: [u8; 16]      (16 bytes) - Authentication tag
└── ciphertext: VAR    (variable) - Encrypted payload
```

**Payload Structure:**
```
Plaintext Payload:
├── amount: u64 LE     (8 bytes)
├── r: [u8; 32]        (32 bytes) - randomness
├── pk_spend: [u8; 32] (32 bytes) - public spend key
└── outputs: VAR       (variable) - output recipients
```

**Encryption Implementation:**
```rust
use chacha20poly1305::{XChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};

pub fn encrypt_output(
    payload: &[u8],
    recipient_pk: &[u8; 32],
) -> Result<Vec<u8>, String> {
    // Generate ephemeral key pair
    let (epk, esk) = generate_ephemeral_keypair();
    
    // Derive shared secret
    let shared_secret = derive_shared_secret(&esk, recipient_pk);
    
    // Generate nonce
    let nonce = generate_nonce();
    
    // Encrypt payload
    let cipher = XChaCha20Poly1305::new(&Key::from_slice(&shared_secret));
    let ciphertext = cipher.encrypt(&Nonce::from_slice(&nonce), payload)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Build encrypted output
    let mut encrypted = Vec::new();
    encrypted.push(1); // version
    encrypted.push(1); // cipher_id
    encrypted.extend_from_slice(&epk);
    encrypted.extend_from_slice(&nonce);
    encrypted.extend_from_slice(&ciphertext[..16]); // tag
    encrypted.extend_from_slice(&ciphertext[16..]); // ciphertext
    
    Ok(encrypted)
}
```

## Domain Separation (Optional)

### Hardened Encoding

For additional security, domain separation can be applied:

**Commitment Domain:**
```rust
pub fn compute_commitment_with_domain(
    amount: u64, 
    r: [u8; 32], 
    pk_spend: [u8; 32]
) -> [u8; 32] {
    let domain = b"CLOAK:C|";
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(&amount.to_le_bytes());
    preimage.extend_from_slice(&r);
    preimage.extend_from_slice(&pk_spend);
    
    blake3::hash(&preimage).as_bytes().try_into().unwrap()
}
```

**Nullifier Domain:**
```rust
pub fn compute_nullifier_with_domain(
    sk_spend: [u8; 32], 
    leaf_index: u32
) -> [u8; 32] {
    let domain = b"CLOAK:NF|";
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(&sk_spend);
    preimage.extend_from_slice(&leaf_index.to_le_bytes());
    
    blake3::hash(&preimage).as_bytes().try_into().unwrap()
}
```

**Outputs Hash Domain:**
```rust
pub fn compute_outputs_hash_with_domain(outputs: &[Output]) -> [u8; 32] {
    let domain = b"CLOAK:OUT|";
    let mut serialized = Vec::new();
    serialized.extend_from_slice(domain);
    
    for output in outputs {
        serialized.extend_from_slice(&output.address);
        serialized.extend_from_slice(&output.amount.to_le_bytes());
    }
    
    blake3::hash(&serialized).as_bytes().try_into().unwrap()
}
```

## API Encoding Standards

### JSON Encoding

**Hex Encoding:**
- All byte arrays encoded as hex strings
- No `0x` prefix
- Lowercase letters

**Example:**
```json
{
  "root": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "nullifier": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
  "amount": "1000000"
}
```

**Base64 Encoding:**
- Used for binary data in API responses
- Standard base64 encoding
- No padding issues

**Example:**
```json
{
  "proof_bytes": "base64_encoded_proof_data",
  "public_inputs": "base64_encoded_public_inputs"
}
```

### Error Encoding

**Error Response Format:**
```json
{
  "error": {
    "code": "INVALID_PROOF",
    "message": "Proof verification failed",
    "details": {
      "constraint": "merkle_inclusion",
      "expected": "a1b2c3d4...",
      "actual": "fedcba09..."
    }
  }
}
```

## Validation Rules

### Input Validation

**Amount Validation:**
```rust
pub fn validate_amount(amount: u64) -> Result<(), String> {
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    if amount > u64::MAX / 2 {
        return Err("Amount too large".to_string());
    }
    
    Ok(())
}
```

**Public Key Validation:**
```rust
pub fn validate_pubkey(pubkey: &[u8; 32]) -> Result<(), String> {
    if pubkey == &[0u8; 32] {
        return Err("Invalid public key: zero key".to_string());
    }
    
    Ok(())
}
```

**Output Validation:**
```rust
pub fn validate_outputs(outputs: &[Output]) -> Result<(), String> {
    if outputs.is_empty() {
        return Err("At least one output required".to_string());
    }
    
    if outputs.len() > 10 {
        return Err("Too many outputs".to_string());
    }
    
    for output in outputs {
        validate_amount(output.amount)?;
        validate_pubkey(&output.address)?;
    }
    
    Ok(())
}
```

## Testing & Validation

### Encoding Tests

**Round-trip Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commitment_roundtrip() {
        let amount = 1_000_000u64;
        let r = [0x01u8; 32];
        let pk_spend = [0x02u8; 32];
        
        let commitment = compute_commitment(amount, r, pk_spend);
        
        // Verify deterministic output
        let commitment2 = compute_commitment(amount, r, pk_spend);
        assert_eq!(commitment, commitment2);
    }
    
    #[test]
    fn test_public_inputs_serialization() {
        let inputs = PublicInputs {
            root: [0x01u8; 32],
            nullifier: [0x02u8; 32],
            outputs_hash: [0x03u8; 32],
            amount: 1_000_000,
        };
        
        let serialized = serialize_public_inputs(&inputs);
        let deserialized = deserialize_public_inputs(&serialized);
        
        assert_eq!(inputs.root, deserialized.root);
        assert_eq!(inputs.nullifier, deserialized.nullifier);
        assert_eq!(inputs.outputs_hash, deserialized.outputs_hash);
        assert_eq!(inputs.amount, deserialized.amount);
    }
}
```

**Cross-Platform Tests:**
```rust
#[test]
fn test_cross_platform_encoding() {
    // Test that encoding is consistent across platforms
    let test_cases = vec![
        (0u64, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (1u64, [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (0x123456789ABCDEF0u64, [0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]),
    ];
    
    for (value, expected) in test_cases {
        let encoded = value.to_le_bytes();
        assert_eq!(encoded, expected);
    }
}
```

This encoding specification ensures consistent data handling across all Cloak components and provides a solid foundation for interoperability and security.