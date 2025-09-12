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
- **fee_bps:** protocol fee in basis points.