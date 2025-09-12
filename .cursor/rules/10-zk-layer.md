# ZK Layer: Quick Contract

- **Commitment (leaf)**: `C = H(amount:u64 || r:32 || pk_spend:32)` with `pk_spend = H(sk_spend:32)`
- **Nullifier**: `nf = H(sk_spend:32 || leaf_index:u32)`
- **Hash `H`**: BLAKE3-256 (MVP). All sides must agree on exact byte layout.
- **Public Inputs (withdraw)**: `{ root:32, nf:32, fee_bps:u16, outputs_hash:32, amount:u64 }`
- **Outputs hash**: `H(concat(address:32 || amount: u64)...)` in exact order.
- **Circuit checks**:
  1) Merkle inclusion of `C` into `root`
  2) `nf` correctness
  3) Conservation: `sum(outputs) + fee(amount, fee_bps) == amount`
  4) `outputs_hash` matches provided outputs

