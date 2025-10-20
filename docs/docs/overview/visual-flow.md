---
title: Visual Flow
description: Sequence diagrams and ASCII representations for Cloak deposits, withdrawals, and PoW mining.
---

# Visual Flow

The repository ships a detailed ASCII diagram in [`docs/VISUAL_FLOW.md`](../VISUAL_FLOW.md). This page highlights the key sequences and links to the full diagram for reference.

## Deposit Path

1. **Wallet Preparation:** Generate the private randomness `r`, note secret key `sk_spend`, and derive `pk_spend` (BLAKE3).
2. **Commitment Creation:** Compute `C = H(amount ∥ r ∥ pk_spend)` and encrypt the output payload for the recipient.
3. **On-Chain Submit:** Send the deposit instruction plus a native SOL transfer to the `shield-pool` program.
4. **Indexer Ingestion:** The indexer listens for `deposit_commit` logs, appends the commitment to the Merkle tree, and stores encrypted outputs for note discovery.

## Withdraw Path

1. **Note Discovery:** Clients poll the indexer for encrypted notes, decrypt locally, and select a spendable commitment.
2. **Proof Generation:** Use `zk-guest-sp1-host` to produce a Groth16 proof with public inputs `(root, nullifier, outputs_hash, amount, fee_bps)`.
3. **Relay Submission:** Send the signed withdraw request to the relay API. The relay validates inputs, checks nullifiers, and enqueues a job.
4. **PoW Claim Selection:** Workers request a wildcard claim from the scramble registry via `ClaimFinder` when PoW is enabled.
5. **Transaction Execution:** The relay assembles the transaction, simulates via Solana RPC, handles optional Jito submission, and confirms the result.
6. **On-Chain Verification:** `shield-pool` verifies the Groth16 proof through `sp1-solana`, checks Merkle root membership, enforces fee policy, and transfers lamports to recipients.

## Wildcard PoW Loop

1. `cloak-miner` fetches the current difficulty and recent slot hash.
2. A mining engine iterates nonces until `BLAKE3(preimage) < target` with the wildcard batch hash (`[0u8; 32]`).
3. Successful mines submit `mine_claim` and, after slot drift, `reveal_claim` transactions to the scramble registry.
4. Relay workers consume revealed claims through the `consume_claim` CPI when forming withdraw transactions.

## Full Diagram

For a complete ASCII diagram that combines the above steps, see [`docs/VISUAL_FLOW.md`](../VISUAL_FLOW.md). It also includes an extended withdraw sequence chart and cross-service interactions.
