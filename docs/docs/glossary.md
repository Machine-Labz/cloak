---
title: Glossary
description: Core protocol terms, hashes, and fee definitions used by Cloak components.
---

# Glossary

- **Note:** spendable record (amount + secrets).
- **Commitment `C`:** `H(amount || r || pk_spend)`, leaf in the Merkle tree.
- **sk_spend:** secret spend key (private, per-note recommended).
- **pk_spend:** `H(sk_spend)`, public handle used inside `C`.
- **Nullifier `nf`:** `H(sk_spend || leaf_index)`, public anti-double-spend tag.
- **H:** BLAKE3-256 (MVP). All parties must use identical byte layouts.
- **Merkle `root`:** tree root over all `C` leaves (append-only).
- **Merkle proof:** pathElements + pathIndices proving leaf→root inclusion.
- **outputs_hash:** `H( canonical_serialize(outputs[]) )`, binds the circuit to the actual recipients on-chain.
- **fee_bps:** protocol fee in basis points (currently 0% for deposits, 0.5% for withdrawals).
- **fixed_fee:** fixed protocol fee in lamports (currently 0.0025 SOL = 2,500,000 lamports).
- **variable_fee:** percentage-based fee calculated as (amount * 5) / 1000 (0.5%).
- **total_fee:** combined variable and fixed fees (variable_fee + fixed_fee).
