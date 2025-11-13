---
title: Scramble Registry Program
description: Proof-of-work registry managing miners, claims, and consumption for wildcard withdrawals.
---

# Scramble Registry Program

The scramble registry powers Cloak's wildcard PoW layer. It is implemented in Pinocchio and deployed alongside the shield pool program.

Source: [`programs/scramble-registry`](https://github.com/cloak-labz/cloak/tree/main/programs/scramble-registry)

## Responsibilities

- Register miners and store their authorities.
- Track network-wide difficulty and slot-based anti-precomputation salt.
- Manage claim accounts through `mine`, `reveal`, and `consume` lifecycles.
- Enforce claim expiration and consumption limits.
- Provide CPI surface (`consume_claim`) for the shield pool withdraw instruction.

## Instruction Set

| Instruction | Purpose | Highlights |
| --- | --- | --- |
| `InitializeRegistry` | Bootstrap registry PDA, set difficulty, admin authority, fee config. |
| `RegisterMiner` | Create miner PDA for signer; stores authority and metadata. |
| `MineClaim` | Miner commits to discovered hash; stores hash, batch hash, slot, expiry. |
| `RevealClaim` | Provide nonce/preimage after delay; transitions claim to revealed state. |
| `ConsumeClaim` | Called via CPI. Validates batch hash (unless wildcard), expiry, consumption budget; increments usage counter. |

The program ID is hardcoded (`EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6`), ensuring consistent CPI references.

## Wildcard Support

Wildcard mode marks claims whose `batch_hash == [0u8; 32]`. During consumption the program skips the batch hash equality check, letting the relay attach any job-specific hash post hoc. Implementation highlights:

- `Claim::is_wildcard()` helper in `state/mod.rs`.
- Conditional batch hash validation in `instructions/consume_claim.rs`.

## State Layout

- **Registry** – Difficulty target, admin pubkey, reveal interval.
- **Miner** – Authority pubkey, claim counters, status flags.
- **Claim** – Hash, batch hash, miner, expiry slot, consumption limit, consumed count.

All PDAs use deterministic seeds to simplify client derivations (`b"registry"`, `b"miner"`, `claim` seeds with miner pubkey + nonce).

## Integration Points

- `packages/cloak-miner` – Uses RPC helper modules in `manager.rs` and `engine` to interact with registry instructions.
- `services/relay` – `ClaimFinder` fetches claim accounts by scanning `get_program_accounts` responses and reusing state structs from the crate.

## Testing & Tooling

- Unit tests cover instruction encoding/decoding and claim state transitions.
- Localnet script `init-localnet.sh` initialises the registry with sample difficulty/reset path.
- Use `cloak-miner status` to inspect active claims and miner registration health.

Refer to the [PoW Overview](../pow/overview.md) and [`READY_FOR_YOU.md`](https://github.com/cloak-labz/cloak/blob/main/READY_FOR_YOU.md) for additional operator notes.
