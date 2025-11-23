---
title: Withdraw Workflow
description: Detailed sequence for generating proofs, submitting withdraw jobs, and executing Solana transactions.
---

# Withdraw Workflow

The withdraw workflow consumes a commitment created during deposit and releases SOL to arbitrary recipients while preserving privacy.

## 1. Discover Spendable Notes

- Query the indexer for encrypted outputs: `GET /api/v1/notes/range?start=...&limit=...`.
- Decrypt notes locally. Identify one with sufficient amount and unused nullifier (`nf`).
- Fetch the Merkle proof from `GET /api/v1/merkle/proof/:index`.

## 2. Prepare Witness & Public Inputs

| Input | Source |
| --- | --- |
| `amount` | Stored in note payload. |
| `r`, `sk_spend` | Retrieved from note secret store. |
| `outputs[]` | Planned recipients and amounts. |
| `fee_bps` | Policy (default 50 basis points) or job-specific config. |
| `merkle_path` | Indexer response (`pathElements`, `pathIndices`). |

Derive public values:

- `root` – from proof endpoint.
- `nf = BLAKE3(sk_spend || leaf_index)`.
- `outputs_hash = BLAKE3(serialize(outputs))`.

## 3. Generate Proof (SP1)

Run the SP1 host CLI:

```bash
cargo run -p zk-guest-sp1-host -- prove \
  --private examples/private.example.json \
  --public examples/public.example.json \
  --outputs examples/outputs.example.json \
  --proof out/proof.bin \
  --pubout out/public.json
```

Replace the example JSON files with the values from step 2. The command outputs:

- `proof.bin` – SP1 bundle containing Groth16 proof and public inputs.
- `public.json` – canonicalised public inputs for submission.

If running remote proving, POST to the relay's `/jobs/:job_id/prove-local` endpoint instead.

## 4. Submit Withdraw Job

Send a request to the relay:

```bash
curl -X POST http://localhost:3002/withdraw \
  -H 'Content-Type: application/json' \
  -d '{
    "outputs": [...],
    "policy": {"fee_bps": 50},
    "public_inputs": {
      "root": "...",
      "nf": "...",
      "amount": 1000000,
      "fee_bps": 50,
      "outputs_hash": "..."
    },
    "proof_bytes": "<base64-proof.bin>"
  }'
```

The relay validates shapes, stores the job in PostgreSQL, and enqueues it in Redis.

Monitor job state:

```bash
curl http://localhost:3002/status/<request_id>
```

## 5. Worker Processing

1. Pop job from Redis queue.
2. Reconstruct Solana instruction data, recomputing outputs hash and conservation checks.
3. Fetch wildcard claims (if enabled) with `ClaimFinder`.
4. Simulate the transaction to obtain compute unit usage.
5. Submit via
   - `sendTransaction` (default RPC), and optionally
   - Jito bundle relay if configured.
6. Update job status to `completed` with the confirmed signature, or `failed` with error context.

## 6. On-Chain Execution

`shield-pool` executes the following invariants:

1. `verify_proof()` via `sp1-solana` using the embedded verification key hash.
2. Root containment in the 64-slot ring buffer.
3. Nullifier uniqueness enforced by the shard account.
4. Outputs hash recomputation matches public input.
5. `∑outputs + fee == amount`.
6. Transfers lamports to recipients and treasury PDAs.

## Observability

- Relay logs include `[WORKER]` and `[METRICS]` entries for queue times, claim discovery, and RPC failures.
- Postgres tables `jobs`, `job_attempts`, and `nullifiers` provide audit trails.
- Use `solana confirm -v <tx>` to inspect compute unit usage and result logs.

Continue to the [PoW Withdraw Enhancements](./pow-withdraw.md) for wildcard claims and miner coordination.
