---
title: Deposit Workflow
description: Step-by-step guide to creating Cloak deposits, from note generation to Merkle ingestion.
---

# Deposit Workflow

The deposit workflow lets users fund the shared shield pool while producing all data needed for future withdrawals. The sequence below maps the responsibility split between clients, the indexer, and on-chain programs.

## 1. Prepare the Note

| Actor | Action |
| --- | --- |
| Client | Generate note secret `sk_spend` (32 bytes) and randomness `r` (32 bytes). |
| Client | Derive `pk_spend = BLAKE3(sk_spend)` (32 bytes). |
| Client | Compute commitment `C = BLAKE3(amount ∥ r ∥ pk_spend)` using little-endian encoding for `amount`. |

Store `(sk_spend, r, leaf_hint)` securely—the withdrawal circuit will require them.

## 2. Encrypt the Output Payload

- Serialize the recipient note details (recipient address, amount, metadata).
- Encrypt using the client-side scheme (currently symmetric key per recipient or test-only JSON payload).
- Result is a base64 payload `encryptedOutput` served back to wallets.

## 3. Submit to `shield-pool`

Send a transaction containing:

1. System transfer of `amount + fee` lamports to the pool PDA.
2. `deposit` instruction with fields:
   - `leaf_commit` – 32-byte commitment `C`.
   - `enc_output` – encrypted payload bytes (length-prefixed).

The on-chain program emits `deposit_commit:{hex}` in logs. No state mutation occurs besides lamport transfer—the Merkle tree is maintained off-chain.

## 4. Indexer Ingestion

The Rust indexer (`services/indexer`):

1. Subscribes to program logs via WebSocket or polling.
2. Parses `deposit_commit` events, stores them in PostgreSQL, and appends to the Merkle tree.
3. Exposes the updated root and next index through `GET /api/v1/merkle/root`.
4. Persists the encrypted output for client retrieval through `GET /api/v1/notes/range`.

## 5. Client Confirmation

- Confirm transaction finality via Solana RPC.
- Query `GET /api/v1/merkle/root` and `GET /api/v1/notes/range` to confirm the note is indexed (compare commitment hash).
- Store the assigned `leaf_index` (indexer returns `nextIndex - 1`).

## Failure & Retry Considerations

- **Transaction failure:** inspect Solana logs and retry; deposit instruction is idempotent if the commitment is unique.
- **Indexer lag:** if the commitment does not appear, check indexer logs and ensure WebSocket streams are healthy.
- **Client storage:** if secrets are lost, the note becomes unspendable—integrate secure backups early.

With deposits confirmed, proceed to the [Withdraw Workflow](./withdraw.md) to spend notes privately.
