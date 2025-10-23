# ZK Design Principles & Architecture

This document outlines the high-level design principles, architectural decisions, and implementation rationale for Cloak's zero-knowledge privacy layer.

## Design Philosophy

### Core Principles

**1. Privacy-First Architecture**
- Zero-knowledge proofs enable private withdrawals without revealing transaction links
- Cryptographic guarantees ensure mathematical privacy, not just obfuscation
- User secrets never leave their local environment

**2. Trustless Verification**
- On-chain proof verification eliminates need for trusted intermediaries
- SP1 Groth16 proofs provide cryptographic soundness guarantees
- Public verification key enables anyone to verify proof validity

**3. Economic Sustainability**
- Fee structure balances privacy costs with protocol sustainability
- PoW mining creates economic incentives for network participation
- Transparent fee calculation prevents hidden costs

**4. Practical Usability**
- Simple deposit/withdraw interface abstracts complex cryptography
- Reasonable proof generation times (60-90s local, 2-3min TEE)
- Clear error messages and troubleshooting guidance

## Architectural Overview

### High-Level Flow

```text
┌────────────────────────────────────────────────────────────────┐
│                    ZK DESIGN ARCHITECTURE                      │
├────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Deposit   │    │   Storage    │    │   Withdrawal    │    │
│  │             │    │              │    │                 │    │
│  │ • Generate  │───►│ • Merkle     │───►│ • Generate      │    │
│  │   secrets   │    │   Tree       │    │   Proof         │    │
│  │ • Compute   │    │ • Encrypted  │    │ • Verify        │    │
│  │   commitment│    │   Outputs    │    │ • Execute       │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
│         │                   │                   │              │
│         │                   │                   │              │
│         ▼                   ▼                   ▼              │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Client    │    │   Indexer    │    │   Relay         │    │
│  │   Wallet    │    │   Service    │    │   Service       │    │
│  │             │    │              │    │                 │    │
│  │ • Key Mgmt  │    │ • Tree       │    │ • Job Queue     │    │
│  │ • Encryption│    │   Updates    │    │ • Proof Gen     │    │
│  │ • UI/UX     │    │ • Proof      │    │ • Submission    │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
└────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**Client Layer:**
- Secret key generation and management
- Commitment computation and encryption
- Proof generation coordination
- User interface and experience

**Storage Layer:**
- Merkle tree maintenance and updates
- Encrypted output storage
- Proof generation artifacts
- State synchronization

**Verification Layer:**
- On-chain proof verification
- Nullifier tracking and validation
- Amount conservation enforcement
- Transaction execution

## Cryptographic Design

### Hash Function Selection: BLAKE3-256

**Rationale:**
- **Performance:** Superior speed compared to SHA-256
- **Security:** 128-bit security level with collision resistance
- **Simplicity:** Single hash function for all operations
- **Compatibility:** Well-supported across platforms

**Usage Patterns:**
```rust
// Commitment computation
C = BLAKE3(amount || r || pk_spend)

// Nullifier generation  
nf = BLAKE3(sk_spend || leaf_index)

// Merkle tree nodes
parent = BLAKE3(left_child || right_child)

// Outputs hash binding
outputs_hash = BLAKE3(serialize(outputs))
```

### Commitment Scheme

**Design Goals:**
- **Binding:** Each commitment uniquely identifies a note
- **Hiding:** Commitment reveals no information about inputs
- **Collision Resistance:** Prevents commitment reuse attacks

**Implementation:**
```rust
pub fn compute_commitment(amount: u64, r: [u8; 32], pk_spend: [u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&amount.to_le_bytes());  // 8 bytes
    preimage.extend_from_slice(&r);                     // 32 bytes
    preimage.extend_from_slice(&pk_spend);              // 32 bytes
    blake3(&preimage)  // Returns 32 bytes
}
```

**Security Properties:**
- **Preimage Resistance:** Cannot recover inputs from commitment
- **Second Preimage Resistance:** Cannot find different inputs with same commitment
- **Collision Resistance:** Cannot find two different inputs with same commitment

### Nullifier Scheme

**Design Goals:**
- **Uniqueness:** Each note generates unique nullifier
- **Unlinkability:** Nullifiers don't reveal note relationships
- **One-way:** Cannot recover secret key from nullifier

**Implementation:**
```rust
pub fn compute_nullifier(sk_spend: [u8; 32], leaf_index: u32) -> [u8; 32] {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&sk_spend);              // 32 bytes
    preimage.extend_from_slice(&leaf_index.to_le_bytes()); // 4 bytes
    blake3(&preimage)  // Returns 32 bytes
}
```

**Security Properties:**
- **Double-Spend Prevention:** Same nullifier cannot be used twice
- **Privacy Preservation:** Nullifiers don't link to specific notes
- **Forward Security:** Compromised nullifier doesn't affect other notes

## Circuit Design Rationale

### Constraint System Design

**Six Core Constraints:**

1. **Spend Key Derivation** - Ensures public key consistency
2. **Commitment Recomputation** - Validates note authenticity
3. **Merkle Inclusion Proof** - Proves note exists in tree
4. **Nullifier Generation** - Prevents double-spending
5. **Amount Conservation** - Enforces value preservation
6. **Outputs Hash Verification** - Binds recipients to proof

**Design Trade-offs:**
- **Completeness:** All valid proofs verify correctly
- **Soundness:** Invalid proofs never verify (negligible probability)
- **Zero-Knowledge:** Proofs reveal no witness information
- **Efficiency:** Minimal constraint count for fast proving

### Public Input Structure

**104-byte Public Input Layout:**
```
Offset  Size    Field           Purpose
0       32      root            Merkle tree root for verification
32      32      nullifier       Spending nullifier for uniqueness
64      32      outputs_hash    Hash of output recipients
96      8       amount          Total amount being spent
```

**Design Rationale:**
- **Fixed Size:** Enables efficient on-chain parsing
- **Minimal Information:** Only essential public data exposed
- **Binding:** Cryptographically commits to transaction details
- **Verification:** Enables efficient on-chain validation

## Privacy Model

### Privacy Guarantees

**Strong Privacy (Cryptographically Guaranteed):**
- **Transaction Unlinkability:** Deposits and withdrawals cannot be linked
- **Amount Hiding:** Note amounts are hidden in commitments
- **Recipient Privacy:** Output recipients are cryptographically bound
- **Nullifier Uniqueness:** Prevents double-spending without linking

**Weak Privacy (Operational):**
- **Timing Privacy:** Mitigated by batching and relay delays
- **Network Privacy:** IP addresses may be visible to network observers
- **Metadata Privacy:** Transaction metadata may leak information

### Privacy Limitations (MVP)

**Amount Visibility:**
- Output amounts are public in withdrawal transactions
- Mitigation: Use amount buckets and timing obfuscation
- Future: Range proofs for complete amount privacy

**Timing Analysis:**
- Deposit and withdrawal timing may be correlated
- Mitigation: Relay batching and random delays
- Future: Cryptographic timing obfuscation

**Network Analysis:**
- IP addresses and network patterns may be visible
- Mitigation: Use VPNs and Tor for network privacy
- Future: Decentralized relay network

## Security Model

### Cryptographic Assumptions

**Hash Function Security:**
- BLAKE3-256 provides 128-bit security level
- Collision resistance: 2^128 operations
- Preimage resistance: 2^256 operations

**Zero-Knowledge Proof Security:**
- Groth16 provides computational soundness
- Discrete logarithm assumption in elliptic curve groups
- Random oracle model for hash functions

**Merkle Tree Security:**
- Tree structure provides logarithmic proof sizes
- Append-only property prevents history modification
- Root anchoring prevents equivocation attacks

### Attack Vectors & Mitigations

**Double-Spending Attacks:**
- **Attack:** Reusing nullifiers to spend same note twice
- **Mitigation:** On-chain nullifier tracking and validation
- **Security:** Cryptographically prevents with negligible probability

**Merkle Tree Attacks:**
- **Attack:** Using stale roots or invalid inclusion proofs
- **Mitigation:** Ring buffer of recent roots, on-chain validation
- **Security:** Only recent valid roots accepted

**Commitment Collision Attacks:**
- **Attack:** Finding two different notes with same commitment
- **Mitigation:** Collision-resistant hash function (BLAKE3)
- **Security:** 2^128 operations required for collision

**Nullifier Linking Attacks:**
- **Attack:** Linking nullifiers to specific notes or users
- **Mitigation:** One-way hash function, random secret keys
- **Security:** Computationally infeasible to reverse

## Performance Design

### Proof Generation Optimization

**Local Proving:**
- Multi-threaded SP1 execution
- Optimized BLAKE3 implementation
- Cached Merkle tree operations
- Target: 60-90 seconds (p95)

**TEE Proving:**
- Hardware-accelerated execution
- Parallel processing capabilities
- Reduced memory requirements
- Target: 2-3 minutes

**On-Chain Verification:**
- Efficient verification key storage
- Optimized constraint checking
- Minimal compute unit consumption
- Target: < 200ms verification time

### Scalability Considerations

**Merkle Tree Scaling:**
- 32-level tree supports 2^32 ≈ 4 billion notes
- Logarithmic proof size (32 levels × 32 bytes = 1KB)
- Efficient append-only operations

**Nullifier Scaling:**
- Sharded storage for large nullifier sets
- Linear scan complexity (O(N) per shard)
- Capacity limits prevent unbounded growth

**Proof Verification Scaling:**
- Constant-time verification regardless of tree size
- Parallel verification of multiple proofs
- Efficient batch verification capabilities

## Implementation Design

### SP1 Integration

**Guest Program Design:**
- RISC-V target for SP1 zkVM
- Minimal dependencies for security
- Deterministic execution for reproducibility
- Clear constraint implementation

**Host Program Design:**
- Witness preparation and validation
- Proof generation coordination
- Error handling and recovery
- Performance monitoring

**On-Chain Integration:**
- SP1 Solana verifier integration
- Efficient verification key management
- CPI call optimization
- Error code standardization

### Data Flow Design

**Deposit Flow:**
```
Client → Generate Secrets → Compute Commitment → Submit Deposit
Indexer → Update Merkle Tree → Store Encrypted Output → Log Event
```

**Withdrawal Flow:**
```
Client → Discover Notes → Generate Proof → Submit Withdrawal
Relay → Validate Proof → Execute Transaction → Update State
```

**Proof Generation Flow:**
```
Witness → SP1 Guest → Constraints → Groth16 → Proof Bundle
Host → Extract Proof → Parse Inputs → Submit Transaction
```

## Future Design Considerations

### Enhanced Privacy

**Range Proofs:**
- Hide exact amounts in withdrawals
- Maintain amount conservation
- Increase proof size and generation time
- Trade-off: Privacy vs. Performance

**Multi-Asset Support:**
- Support for SPL tokens
- Cross-asset privacy pools
- Asset-specific commitment schemes
- Increased complexity in circuit design

### Scalability Improvements

**Tree Compression:**
- ZK-compressed Merkle trees
- Reduce proof size and verification time
- Increase circuit complexity
- Trade-off: Simplicity vs. Efficiency

**Batch Verification:**
- Verify multiple proofs simultaneously
- Reduce per-proof verification cost
- Increase transaction throughput
- Trade-off: Latency vs. Throughput

### Security Enhancements

**Post-Quantum Security:**
- Quantum-resistant hash functions
- Post-quantum zero-knowledge proofs
- Increased proof sizes and generation times
- Future-proofing considerations

**Formal Verification:**
- Mathematical proof of circuit correctness
- Automated constraint verification
- Reduced implementation bugs
- Increased development complexity

## Design Decisions Summary

### Key Architectural Choices

1. **BLAKE3-256:** Chosen for performance and security balance
2. **Groth16:** Selected for efficient on-chain verification
3. **SP1 zkVM:** Enables Rust-based circuit development
4. **Merkle Trees:** Provides logarithmic proof sizes
5. **Ring Buffer:** Balances security and efficiency for root storage
6. **Sharded Nullifiers:** Enables scalable double-spend prevention

### Trade-off Analysis

**Privacy vs. Performance:**
- Current: Public amounts for fast proving
- Future: Range proofs for complete privacy

**Security vs. Usability:**
- Current: Strong cryptographic guarantees
- Future: Enhanced user experience features

**Decentralization vs. Efficiency:**
- Current: Centralized relay for simplicity
- Future: Decentralized relay network

**Simplicity vs. Features:**
- Current: Core privacy functionality
- Future: Advanced features and optimizations

This design provides a solid foundation for privacy-preserving transactions while maintaining practical usability and strong security guarantees.