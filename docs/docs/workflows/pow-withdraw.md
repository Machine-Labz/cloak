---
title: PoW-Enhanced Withdrawals
description: How wildcard proof-of-work claims are mined, discovered, and consumed by the relay.
---

# PoW-Enhanced Withdrawals

Wildcard claims remove the need to pre-compute batch hashes for every withdraw job. Miners mine claims with a batch hash of all zeros, and the relay can attach any job-specific hash when consuming the claim.

## Actors

- **Scramble Registry Program** – Tracks miners, claims, reveal deadlines, and consumption counts.
- **`cloak-miner` CLI** – Mines wildcard claims, submits `mine_claim`/`reveal_claim` transactions, monitors registry state.
- **Relay ClaimFinder** – Queries the registry for available claims and selects the best candidate per job.
- **Shield Pool** – Verifies claims via CPI on withdraw execution (`consume_claim`).

## Claim Lifecycle

1. **Difficulty Fetch:** Miner reads on-chain difficulty + slot hash to seed the preimage.
2. **Mining:**
   - Preimage layout: `"CLOAK:SCRAMBLE:v1" ∥ slot ∥ slot_hash ∥ miner_pubkey ∥ batch_hash ∥ nonce`.
   - For wildcard mode, `batch_hash = [0u8; 32]`.
   - A valid claim satisfies `BLAKE3(preimage) < difficulty_target`.
3. **Commit (`mine_claim`):** Miner submits transaction committing to the hash.
4. **Reveal (`reveal_claim`):** Once reveal window opens, miner reveals the preimage.
5. **Consume (`consume_claim` CPI):** Relay references the claim when building the withdraw transaction. The scramble registry ensures consumption limits and expiry checks.

## Relay Integration

- The relay loads `SCRAMBLE_REGISTRY_PROGRAM_ID` from configuration.
- `ClaimFinder` performs `get_program_accounts` queries, filters expired/fully consumed claims, and prefers wildcards.
- When a job enters `processing`, the worker requests a claim:

```rust
if let Some(claim) = claim_finder.find_claim(&batch_hash).await? {
    info!("Found wildcard claim", claim_pda = %claim.claim_pda);
    tx_builder.attach_pow(claim)?;
} else {
    return Err(Error::NoClaimsAvailable);
}
```

- Metrics emitted by the relay (`[METRICS]`) track claim search duration, matches, and failure modes.

## Failure Modes

| Scenario | Mitigation |
| --- | --- |
| No wildcard claims available | Alert miners, fall back to legacy batching, or surface job error to caller. |
| Claim expired between discovery and submission | Worker restarts search; log includes claim slot + expiry. |
| Batch hash mismatch (non-wildcard claim) | ClaimFinder excludes non-matching claims. Wildcards skip hash equality. |
| Consume failure on-chain | Scramble registry error surfaces in simulation logs; transaction is aborted and job retried. |

## Monitoring

- Review [`METRICS_GUIDE.md`](../operations/metrics-guide.md) for grep patterns and thresholds.
- Workers log successful claims with miner pubkeys and expiry slots.
- Use the CLI `cloak-miner status` to inspect active claims (`revealed`, `consumed`, `expires_at`).

For additional context, refer to the [Wildcard Mining Overview](../pow/overview.md) and [Integration Guide](../POW_INTEGRATION_GUIDE.md).
