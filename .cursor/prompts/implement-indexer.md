# Task: Implement Indexer MVP

Goal: Append-only Merkle indexer + HTTP API.

Deliver:
- Node/TS service in `services/indexer/`
- Routes:
  - GET /merkle/root
  - GET /merkle/proof/:index
  - GET /notes/range?start&end
  - GET /artifacts/withdraw/:version
- Storage: simple RocksDB or Postgres (your pick), tree height 32
- Ingestion stub: accept a CLI or webhook that feeds `{leaf_commit, encrypted_output}` events for now
- Unit tests: proof recomputation, range pagination

Read:
- docs/zk/merkle.md
- docs/zk/api-contracts.md
- docs/zk/encoding.md
