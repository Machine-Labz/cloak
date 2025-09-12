# Frontend – Rules

**Deposit**
- Build `C` and `encrypted_output`
- Tx: `System::transfer(lamports) -> Pool` + `transact_deposit(encrypted_output, C)`

**Balance**
- Scan `/notes/range` windows; decrypt locally to identify owned notes

**Withdraw**
- Fetch `root`, `proof/:leaf_index`
- Build `publicInputs` and run SP1 WASM prover (Groth16)
- POST `/withdraw` to relay and display txid/receipt

**Non-goals (MVP)**
- No Jito/bundling
- Amount privacy via buckets/UX, not cryptographic range proofs (yet)

