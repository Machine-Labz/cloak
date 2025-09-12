# SP1 Prover (Guest & Host)

## Guest (Rust)
Implements the constraints in `circuit-withdraw.md`. Produces `proofBytes` and re-outputs the public inputs.

**Inputs to guest:**
- Private: amount, r, sk_spend, leaf_index, merkle_path
- Public: root, fee_bps, outputs_hash

**Output:**
- `proofBytes` (Groth16) + `PublicInputs` (serialized bytes used by on-chain verifier)

## Artifact bundle (served by Indexer/API)
```

GET /artifacts/withdraw/\:version
{
"guestElfUrl": "ipfs\://...",
"vk": "<hex or base64>",
"sha256": { "elf": "<hex>", "vk": "<hex>" },
"sp1Version": "x.y.z"
}

```

## Host flow (FE)
1. Fetch `root`, scan notes (decrypt blobs).
2. Fetch `/merkle/proof/:leaf_index`.
3. Build `publicInputs` and run guest to get `proofBytes`.
4. Call Relay `/withdraw`.

## Determinism
- No timestamps or RNG inside guest except derived from inputs.
- Use the exact `encoding.md` layouts or proofs won’t verify.