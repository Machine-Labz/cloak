# shield-pool (Anchor)

**Instructions**
- `transact_deposit(encrypted_output, leaf_commit)` – emits event, Pool receives lamports
- `admin_push_root(root)` – push latest tree root (ring buffer)
- `withdraw(proofBytes, publicInputs, outputs[])` – verify, mark `nf`, pay recipients and fee

**Accounts**
- Pool (PDA), Treasury, RootsRing, NullifierShard{prefix}

**Notes**
- Keep CU budget generous (300–500k)
- Recompute `outputs_hash` on-chain to bind to proof
