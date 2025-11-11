---
title: View/Spend Key Architecture
description: Comprehensive guide to Cloak's view and spend key system for privacy-preserving note discovery and management.
---

# View/Spend Key Architecture

## Overview

Cloak implements a **view/spend key separation** scheme similar to Zcash and Monero, enabling users to:

1. **Scan** for notes sent to them without exposing spending authority
2. **Share** a public view key for receiving encrypted notes
3. **Maintain** full control over spending with separate spend keys

This separation is critical for privacy-preserving wallets where users can discover which notes belong to them without compromising the ability to spend.

## Key Hierarchy

### Master Seed (32 bytes)
- **Root secret** from which all other keys derive
- Generated from cryptographically secure randomness
- Must be backed up securely - losing it means losing all notes
- Never transmitted or stored in plaintext (except in secure backups)

### Spend Key (sk_spend, 32 bytes)
- Derived from master seed: `sk_spend = BLAKE3(master_seed || "cloak_spend_key")`
- Used to **authorize spending** of notes
- Required to compute nullifiers and prove ownership in ZK circuits
- Must remain secret - anyone with this key can spend your notes

### Public Spend Key (pk_spend, 32 bytes)
- Derived from spend key: `pk_spend = BLAKE3(sk_spend)`
- Used in **commitment computation** to bind notes to owners
- Public value stored in Merkle tree leaves
- Cannot be used to spend notes (one-way derivation)

### View Key Secret (vk_secret, 32 bytes)
- Derived from spend key: `vk_secret = BLAKE3(sk_spend || "cloak_view_key_secret")`
- Used to **decrypt** note data from the indexer
- Can view but not spend notes
- Safe to store in less secure environments (e.g., mobile wallets, watch-only wallets)

### Public View Key (pvk, 32 bytes)
- Derived from view key secret using Ed25519 → X25519 conversion
- **Public key** shared with senders to receive encrypted notes
- Used in ECDH (Elliptic Curve Diffie-Hellman) for encryption
- Safe to publish - cannot decrypt or spend notes

## Key Derivation Flow

```
Master Seed (32 bytes, random)
      ↓
      BLAKE3(seed || "cloak_spend_key")
      ↓
Spend Key (sk_spend, 32 bytes)
      ↓
      BLAKE3(sk_spend)
      ↓
Public Spend Key (pk_spend, 32 bytes) ← Used in commitments
      
Spend Key (sk_spend)
      ↓
      BLAKE3(sk_spend || "cloak_view_key_secret")
      ↓
View Key Secret (vk_secret, 32 bytes)
      ↓
      Ed25519 KeyPair Generation
      ↓
      Ed25519 → X25519 Conversion
      ↓
Public View Key (pvk, 32 bytes) ← Shared for receiving notes
```

## Note Encryption Scheme

### Encryption (Sender Side)

When depositing or sending a note to a recipient:

1. **Obtain recipient's public view key** (`pvk`)
2. **Generate ephemeral X25519 keypair** (one-time use)
3. **Compute shared secret** via ECDH: `shared_secret = ECDH(ephemeral_sk, recipient_pvk)`
4. **Serialize note data** as JSON:
   ```json
   {
     "amount": 1000000000,
     "r": "abc123...",
     "sk_spend": "def456...",
     "commitment": "789xyz..."
   }
   ```
5. **Encrypt** using XSalsa20-Poly1305 (TweetNaCl secretbox):
   - Input: `plaintext`, `nonce`, `shared_secret`
   - Output: `ciphertext` (authenticated encryption)
6. **Package encrypted note**:
   ```json
   {
     "ephemeral_pk": "...",  // hex
     "ciphertext": "...",    // hex
     "nonce": "..."          // hex
   }
   ```
7. **Encode as base64** and store in indexer

### Decryption (Recipient Side)

When scanning for notes:

1. **Fetch encrypted outputs** from indexer
2. **For each encrypted note**:
   - Parse `ephemeral_pk`, `ciphertext`, `nonce`
   - Convert view key secret to X25519 secret key
   - Compute shared secret: `shared_secret = ECDH(view_sk, ephemeral_pk)`
   - Attempt decryption: `plaintext = secretbox.open(ciphertext, nonce, shared_secret)`
   - If decryption succeeds → **note belongs to you**
   - If decryption fails → note belongs to someone else (skip)
3. **Import decrypted notes** to wallet

## Security Properties

### Confidentiality
- **Encrypted outputs** hide note data from everyone except the recipient
- Only the recipient's view key can decrypt note contents
- Indexer and external observers see only ciphertext

### Authentication
- **Poly1305 MAC** prevents tampering with ciphertexts
- Failed decryption indicates modified or incorrect note

### Forward Secrecy
- **Ephemeral keypairs** ensure each note uses a unique encryption key
- Compromising one shared secret doesn't affect other notes

### Separation of Powers
- **View key** can scan/decrypt but not spend
- **Spend key** required for withdrawal transactions
- Enables watch-only wallets and delegation scenarios

## Commitment Computation

Commitments use the **public spend key** (not view key):

```
C = BLAKE3(amount || r || pk_spend)
```

Where:
- `amount`: u64 little-endian (8 bytes)
- `r`: randomness nonce (32 bytes)
- `pk_spend`: public spend key (32 bytes)

This binds the note to the owner's spend key while maintaining privacy.

## Nullifier Computation

Nullifiers use the **spend key** (secret):

```
N = BLAKE3(sk_spend || leaf_index)
```

Where:
- `sk_spend`: secret spend key (32 bytes)
- `leaf_index`: u32 little-endian (4 bytes)

Only the owner with `sk_spend` can compute the nullifier to spend the note.

## Wallet Workflows

### Initial Setup
1. Generate or import master seed
2. Derive spend key, view key, and public keys
3. Store keys securely in browser localStorage
4. Export master seed for backup

### Receiving Notes
1. Share public view key (`pvk`) with sender
2. Sender encrypts note data with your `pvk`
3. Sender deposits and stores encrypted output in indexer

### Scanning for Notes
1. Fetch all encrypted outputs from indexer
2. Trial-decrypt each output with view key
3. Import successfully decrypted notes to wallet
4. Now you can withdraw these notes

### Spending Notes
1. Load note from wallet (already scanned)
2. Use `sk_spend` to compute nullifier
3. Generate ZK proof of ownership
4. Submit withdrawal transaction

## Key Management Best Practices

### Backup
- **Export master seed** immediately after generation
- Store in multiple secure locations (encrypted)
- Test restore process periodically
- Never store in plaintext on cloud services

### Security Levels
- **Master Seed**: Maximum security (offline, encrypted)
- **Spend Key**: High security (encrypted wallet, password-protected)
- **View Key**: Medium security (mobile device, watch-only wallets)
- **Public View Key**: No security required (can be public)

### Key Rotation
- Currently not supported (would require protocol changes)
- Plan to generate a new wallet if keys are compromised
- Transfer funds to new wallet as soon as possible

### Multi-Device Usage
- **Same keys**: Import master seed on multiple devices
- **Watch-only**: Import only view key for scanning without spending
- **Split permissions**: View key on mobile, spend key on hardware wallet

## Migration from v1.0

### Legacy Notes (v1.0)
- Used only spend key, no view key separation
- Encrypted outputs were base64-encoded JSON (not real encryption)
- Cannot be scanned automatically

### Upgrading to v2.0
1. **Old notes remain valid** - can still be withdrawn
2. **New deposits** use encrypted outputs with view keys
3. **Scanning** only works for v2.0 notes
4. **Backward compatibility**: Both schemes work simultaneously

### Migration Strategy
1. Keep existing v1.0 notes in wallet
2. Generate v2.0 keys (automatic on first use)
3. New deposits use v2.0 encryption
4. Gradually withdraw v1.0 notes and redeposit

## Technical Implementation

### Libraries Used
- **@noble/hashes**: BLAKE3 hashing
- **tweetnacl**: X25519 ECDH, XSalsa20-Poly1305 encryption
- **Web Crypto API**: Secure random number generation

### Code Locations
- Key derivation: `services/web/lib/keys.ts`
- Wallet management: `services/web/lib/note-manager.ts`
- Deposit encryption: `services/web/components/transaction/deposit-flow.tsx`
- Scanning UI: `services/web/components/wallet/scan-notes.tsx`
- Key management UI: `services/web/components/wallet/key-management.tsx`

## Future Enhancements

### Planned Features
- **Stealth addresses**: Generate unique addresses per transaction
- **Subaddresses**: Derive multiple receiving addresses from one view key
- **Hardware wallet support**: Store spend key on hardware device
- **Encrypted memo fields**: Attach messages to transactions
- **Key derivation paths**: BIP44-style hierarchical keys

### Research Areas
- **Post-quantum cryptography**: Quantum-resistant key exchange
- **Threshold signatures**: Multi-party spending authorization
- **Recursive scanning**: Efficient note discovery with sublinear complexity
- **Zero-knowledge key derivation**: Prove key ownership without revealing it

## FAQ

**Q: Can I share my view key safely?**  
A: The view key secret should not be shared. Only share the **public view key** (pvk). The view key secret can decrypt your notes but cannot spend them.

**Q: What if I lose my master seed?**  
A: Your notes become unspendable. There is no recovery mechanism. Always backup your master seed securely.

**Q: Can I use the same wallet on multiple devices?**  
A: Yes, export your master seed and import it on other devices. All devices will share the same keys and can spend notes.

**Q: How often should I scan for notes?**  
A: Scan after receiving a deposit, or periodically if you expect incoming notes. Scanning is fast and can be automated.

**Q: Is my view key safe on a mobile device?**  
A: The view key can decrypt notes but not spend them. It's safer than the spend key for mobile use, but still should be protected.

**Q: Can I change my view or spend key?**  
A: Not currently. Changing keys would require a protocol upgrade. If keys are compromised, generate a new wallet and transfer funds.

