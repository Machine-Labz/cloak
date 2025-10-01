# Task: Implement shield-pool::withdraw

Goal: Pinocchio instruction verifying SP1 proof (CPI), root ring, nullifier shards, payouts.

Deliver:
- Accounts: Pool, Treasury, RootsRing, NullifierShard{prefix}
- Instruction: withdraw(proofBytes, publicInputs, outputs[])
- Checks: proof OK, root in ring, nf unused->mark, outputs_hash matches, conservation
- Unit tests: ok, invalid_root, outputs_mismatch, double_spend

Read:
- docs/zk/onchain-verifier.md
- docs/zk/encoding.md
