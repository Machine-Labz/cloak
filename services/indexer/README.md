# Indexer service

- Runs event ingestion; persists tree (e.g., RocksDB/Postgres)
- HTTP API as per `docs/zk/api-contracts.md`
- Artifact hosting (guest ELF, vk) with SHA-256 in responses
