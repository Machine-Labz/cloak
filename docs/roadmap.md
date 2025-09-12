# Roadmap (ZK-first)

**M0 – Merkle & Indexer (2–4 days)**
- Build append-only tree, `/merkle/root`, `/merkle/proof/:index`, `/notes/range?start&end`
- Event ingestion from deposit txs

**M1 – Deposit Path (2–3 days)**
- `transact_deposit` instruction + event (leaf_commit, encrypted_output)
- FE shows “Private balance” via local scan

**M2 – SP1 Withdraw Circuit (5–7 days)**
- Circuit: inclusion, nullifier, conservation, outputs_hash
- Local prove/verify harness + golden tests

**M3 – On-chain Verifier + Program (5–7 days)**
- Anchor `shield-pool::withdraw` + CPI to SP1 verifier
- Roots ring, nullifier shards, payouts & fee

**M4 – Relay + API (2–4 days)**
- `POST /withdraw`, `GET /status/:id`, queue (no Jito)
- Mint receipt (optional)

**M5 – Hardening (1 sprint)**
- Encoding invariants, rate limits, metrics, threat-model doc