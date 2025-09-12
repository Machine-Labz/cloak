# Task: Wire Relay API (no Jito)

Goal: Minimal relay with POST /withdraw and GET /status/:id.

Deliver:
- `services/relay/` (Fastify/Express)
- Validate payloads (zod), submit tx via @solana/web3.js, wait for confirmation
- Map errors: ProofInvalid, InvalidRoot, DoubleSpend, OutputsMismatch
- Return txid/rootUsed/nf/receiptAsset?

Read:
- docs/zk/api-contracts.md
- docs/nonzk/relay.md
