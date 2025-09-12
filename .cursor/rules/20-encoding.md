# Encoding & Hashing (Authoritative)

- `u64`, `u32` = **little-endian**
- `pubkey` = 32 bytes
- Concatenation = raw bytes, no delimiters, fixed order
- Hash = BLAKE3-256, 32-byte digest

**Commitment C**
```
C = H( amount:u64 || r:32 || pk_spend:32 )
pk_spend = H( sk_spend:32 )
```

**Nullifier nf**
```
nf = H( sk_spend:32 || leaf_index:u32 )
```

**Outputs**
```
output = address:32 || amount:u64
outputs_hash = H( output[0] || output[1] || ... )
```

**Encrypted output blob (off-chain)**
```
version:u8 (=1) || cipher_id:u8 (=1 xchacha20) || epk:32 || nonce:24 || tag:16 || ct:VAR
```

