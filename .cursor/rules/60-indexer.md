# Indexer (HTTP) – Rules

**Ingest**
-  `/deposit` route - read `leaf_commit`, append to tree, persist `root`, `nextIndex`

**Serve**
- `/merkle/root`  
- `/merkle/proof/:index`  
- `/notes/range?start&end` (no sender filter)
- `/artifacts/withdraw/:version` (guest ELF + vk + SHA-256)

**Performance**
- Deterministic proofs for any index
- Paginate ranges; no address linkage leakage

