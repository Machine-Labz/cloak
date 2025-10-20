# Encoding & Hashing Contract

**Hash function:** BLAKE3-256, digest 32 bytes (using standard `blake3` crate).

## Fee Structure

- **Deposits:** 0% fee
- **Withdrawals:** 0.5% variable + 0.0025 SOL fixed
- **Fixed fee:** 2,500,000 lamports (0.0025 SOL)
- **Variable fee:** `(amount * 5) / 1000` (0.5% = 5 basis points)

## Canonical encodings

- `u64` → 8 bytes little-endian
- `u32` → 4 bytes little-endian
- `pubkey` (Solana) → 32 bytes
- `bytes` concat → no separators, fixed-order as specified

## Commitment `C`
```

C = H( amount\:u64 || r:32 || pk\_spend:32 )
pk\_spend = H( sk\_spend:32 )

```

## Nullifier `nf`
```

nf = H( sk\_spend:32 || leaf\_index\:u32 )

```

## Outputs array
Each output:
```

output = address:32 || amount\:u64

```
`outputs_hash = H( output[0] || output[1] || ... || output[n-1] )`

> Order MUST match on FE, prover, and on-chain. No length prefixes in MVP.

## Encrypted output blob (off-chain)
```

version\:u8 (=1)
cipher\_id\:u8 (=1 for xchacha20)
epk:32 || nonce:24 || tag:16 || ct\:VAR

```

## Domain separation (optional hardening)
If desired, prepend fixed ASCII tags before hashing:
- `b"CLOAK:C|"` for commitments
- `b"CLOAK:NF|"` for nullifiers
- `b"CLOAK:OUT|"` for outputs_hash