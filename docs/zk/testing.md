# Testing Strategy

## Unit (guest)
- Merkle path verification with known vectors
- Commitment, nullifier, outputs_hash consistency
- Conservation arithmetic (edge cases, overflow)

## Property tests
- Randomized trees: inclusion holds, wrong sibling fails
- Double-spend attempts: repeated `nf` rejected

## On-chain (Anchor)
- `withdraw_ok` – pays recipients & fees, marks `nf`
- `invalid_root` – rejected
- `outputs_mismatch` – rejected
- `double_spend` – rejected

## E2E (localnet)
- Deposit → indexer updates root → FE proves → Relay withdraws → recipients credited

## Golden files
- Freeze known inputs/outputs to catch accidental encoding changes.
