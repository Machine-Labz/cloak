# Threat Model (ZK Layer)

## What ZK guarantees (MVP)
- Unlinkability of which leaf was spent (membership + nullifier uniqueness)
- Integrity of payouts (`outputs_hash` + on-chain recompute)
- Conservation of value

## Not guaranteed (MVP)
- Amount privacy (values public; mitigate via buckets)
- Timing privacy (mitigated by batching/relay, but not cryptographically)

## Attacks & mitigations
- **Root staleness:** ring buffer + client refresh
- **Nullifier DoS:** sharded storage + size caps
- **Encoding drift:** golden tests + artifact hashing
- **Indexer equivocation:** program only accepts roots the admin pushed (or future: root anchoring / zk-compression)
