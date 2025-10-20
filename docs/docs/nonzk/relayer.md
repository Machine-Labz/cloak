# Relay (non-ZK component)

**Goal:** Accept a withdraw request with `proofBytes + publicInputs`, submit one Solana tx calling `shield-pool::withdraw`, return `txid`.

## Responsibilities
- Basic payload validation (sum(outputs), outputs_hash, fee math)
- Queue + single-flight executor (no Jito in MVP)
- Submit, confirm, and return `txid`
- Optional: mint receipt NFT with AppData

## Done criteria
- At least-once submission with idempotency keys
- Clear error mapping for proof invalid / root mismatch / doublespend
