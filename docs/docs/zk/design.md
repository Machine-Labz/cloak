# ZK Design (High Level)

- **Deposits**
  - User sends SOL to Pool and submits `encrypted_output` + `leaf_commit = C`
  - Program emits event; Indexer appends `C` to tree and updates `root`

- **Withdraws**
  - Client scans `encrypted_outputs`, selects input note
  - Fetches `root` and `/merkle/proof/:leaf_index`
  - Runs **SP1 guest** to produce `proofBytes` over public inputs:
    `root, nf, amount, fee_bps, outputs_hash`
  - Relay calls `shield-pool::withdraw(proof, pub_inputs, outputs)`
  - Program verifies proof, checks `root`, marks `nf`, recomputes `outputs_hash`, pays recipients, sends fee to treasury