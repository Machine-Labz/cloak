# On-chain Verifier & Program (Pinocchio)

Program **`shield-pool`** responsibilities:
- Store `RootsRing` (K latest).
- Store **nullifiers** in sharded PDAs.
- Verify SP1 proofs via CPI and move lamports.

## Instruction: `transact_deposit(encrypted_output, leaf_commit)`
- Accept lamports to Pool (System transfer in the same tx)
- Store deposit data for indexer retrieval via `/deposit` route

## Instruction: `withdraw(proofBytes, publicInputs, outputs[])`
1. CPI into SP1 verifier → `true/false`
2. Check `root` in `RootsRing`
3. Ensure `nf` not spent; mark as spent
4. Recompute `outputs_hash` from `outputs[]` and compare
5. Compute `fee = amount * fee_bps / 10_000`
6. Move lamports: Pool → recipients; Pool → Treasury (fee)

## Accounts
- `Pool` (PDA) – holds SOL
- `Treasury` – fee sink
- `RootsRing` – recent roots
- `NullifierShard{prefix}` – stores `nf` seen