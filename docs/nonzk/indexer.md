# Indexer (non-ZK component)

**Goal:** Maintain append-only Merkle tree; serve roots, proofs, and encrypted outputs.

## Responsibilities
- Handle `/deposit` route requests with `leaf_commit`
- Append `leaf_commit` to tree, persist `root` and `nextIndex`
- Expose:
  - `/merkle/root`
  - `/merkle/proof/:index`
  - `/notes/range?start&end`
  - `/artifacts/withdraw/:version` (SP1 guest ELF + vk + hashes)

## Done criteria
- Deterministic proofs for any index
- Start/end pagination without address filters
- Hashes for artifacts are stable and documented