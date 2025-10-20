---
title: Wildcard Mining System
description: Economic incentive layer for prioritized transaction processing through proof-of-work claims.
---

# Wildcard Mining System

The Wildcard Mining system provides economic incentives for maintaining transaction throughput in the Cloak privacy protocol. Miners generate BLAKE3 proof-of-work claims that enable prioritized withdraw processing.

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

## Additional Resources

- [Quick Reference Guide](../POW_QUICK_REFERENCE.md) – Commands and API reference
- [Integration Guide](../POW_INTEGRATION_GUIDE.md) – Integration with relay and shield pool
- [Operations Guide](../operations/metrics-guide.md) – Metrics and monitoring

## Economic Model

Miners invest computational resources to generate claims, which are then consumed by users during withdrawals. This creates a marketplace for priority transaction processing while maintaining the security properties of the privacy protocol.
