---
title: Proof-of-Work Overview
description: Orientation for the Wildcard PoW subsystem, including registry accounts, miner flow, and relay wiring.
---

# Proof-of-Work Overview

The Wildcard PoW subsystem ensures withdraw throughput by requiring miners to pre-compute expendable claims. This page summarises the moving pieces and points to the detailed documents in this section.

## Components

- **Scramble Registry (`programs/scramble-registry`)** – Pinocchio program managing miner PDAs, claim accounts, difficulty, and claim consumption.
- **Cloak Miner (`packages/cloak-miner`)** – Standalone CLI that mines wildcard claims, submits transactions, and monitors registry health.
- **Relay ClaimFinder (`services/relay/src/claim_manager.rs`)** – RPC helper that retrieves revealed claims and selects one per job.
- **Shield Pool (`programs/shield-pool`)** – Consumes claims through CPI in the withdraw instruction.

## Account Model

- **Registry PDA** – Stores difficulty, mining parameters, and admin configuration.
- **Miner PDA** – Created by `register` instruction; stores miner authority and configuration.
- **Claim PDA** – Holds mined hash, reveal nonce, expiry slot, consumption counter, and wildcard flag.

## Claim States

1. `Pending` – After `mine_claim`, before reveal window.
2. `Revealed` – After `reveal_claim`; eligible for consumption.
3. `Consumed` – Each use increments the counter until reaching the cap.
4. `Expired` – After expiry slot; no longer usable.

## Difficulty Adjustment

- Registry stores a 256-bit target.
- Miners fetch the target via RPC; lowering the value increases difficulty.
- Operators can adjust via admin instruction (see `instructions/initialize.rs`).

## Key Resources

- [`pow-architecture.md`](../pow-architecture.md) – Baseline architecture design.
- [`POW_WILDCARD_IMPLEMENTATION.md`](../POW_WILDCARD_IMPLEMENTATION.md) – Wildcard extension and integration steps.
- [`POW_QUICK_REFERENCE.md`](../POW_QUICK_REFERENCE.md) – Commands and API references for operators.
- [`READY_FOR_YOU.md`](https://github.com/cloak-labz/cloak/blob/main/READY_FOR_YOU.md) – Implementation hand-off instructions.

For operational guidance and metrics, continue to the [PoW Metrics Guide](../operations/metrics-guide.md).
