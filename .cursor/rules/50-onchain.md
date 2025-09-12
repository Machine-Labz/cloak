# On-chain Program (Anchor) – shield-pool

## Instructions
- `transact_deposit(encrypted_output, leaf_commit)`  
  Accept lamports to Pool (same tx) + emit event `{leaf_commit, blob_ref?}`

- `admin_push_root(root)`  
  Maintain ring of K recent roots

- `withdraw(proofBytes, publicInputs, outputs[])`
  1) CPI to SP1 verifier (Groth16)  
  2) Check `root` in ring  
  3) Check `nf` unused; mark as spent (sharded PDA by prefix)  
  4) Recompute `outputs_hash` from `outputs[]` and compare  
  5) Compute `fee` = `amount * fee_bps / 10_000` (integer)  
  6) Pay recipients; send fee to Treasury (all from Pool)

## Notes
- CU budget target: ≤ ~450k CU
- Strict, canonical encoding (see `docs/zk/encoding.md`)

