# Frontend (non-ZK component)

**Goal:** Provide UX to Top Up and Withdraw privately.

## Responsibilities
- Generate `encrypted_output` and call deposit tx
- Scan `/notes/range` and decrypt blobs to show private balance
- Build publicInputs (`root,nf,amount,fee_bps,outputs_hash`)
- Call SP1 WASM prover to get `proofBytes`
- POST `/withdraw` and display result

## Done criteria
- Byte-level encoding matches `docs/zk/encoding.md`
- Smooth UX for scanning and proof generation (<~1.5s target per proof on a laptop)
