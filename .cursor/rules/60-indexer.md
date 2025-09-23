# Indexer (HTTP) – Rules

**Status:** ✅ **IMPLEMENTED & WORKING**  
**Service:** PostgreSQL-backed Merkle tree management

**Ingest** ✅ COMPLETE
- ✅ `/deposit` route - read `leaf_commit`, append to tree, persist `root`, `nextIndex`

**Serve** ✅ COMPLETE
- ✅ `/merkle/root`  
- ✅ `/merkle/proof/:index`  
- ✅ `/notes/range?start&end` (no sender filter)
- ✅ `/artifacts/withdraw/:version` (guest ELF + vk + SHA-256)

**Performance** ✅ ACHIEVED
- ✅ Deterministic proofs for any index
- ✅ Paginate ranges; no address linkage leakage
- ✅ Real-time Merkle tree updates
- ✅ 31-level tree with PostgreSQL persistence

**Current Implementation**
- **File:** `services/indexer/src/`
- **Merkle Tree:** `lib/merkle.ts` - BLAKE3-256 implementation
- **API Routes:** `api/routes.ts` - HTTP endpoints
- **Database:** PostgreSQL with proper indexing
- **Docker:** `docker-compose.yml` for easy deployment

