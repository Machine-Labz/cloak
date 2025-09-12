# Withdraw Circuit (SP1)

## Private (witness)
- `amount:u64`
- `r:32`
- `sk_spend:32`
- `leaf_index:u32`
- `merkle_path: { pathElements[], pathIndices[] }`

## Public
- `root:32`
- `nf:32`  = `H(sk_spend || leaf_index)`
- `fee_bps:u16`
- `outputs_hash:32`
- `amount:u64`

## Constraints
1. `pk_spend = H(sk_spend)`
2. `C = H(amount || r || pk_spend)`
3. `MerkleVerify(C, merkle_path) == root`
4. `nf == H(sk_spend || leaf_index)`
5. `sum(outputs) + fee(amount, fee_bps) == amount`
6. `H( serialize(outputs) ) == outputs_hash`

> Note: outputs are **public** via `outputs_hash` binding in MVP (values visible on-chain; anonymity via buckets & timing). Range proofs can hide amounts later.