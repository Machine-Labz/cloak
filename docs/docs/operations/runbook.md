---
title: Validator Runbook
description: Operational handbook for running Cloak relay/scrambler infrastructure in production.
---

# Cloak Validators — Runbook

This runbook describes how to provision, operate, monitor, and troubleshoot a Cloak Validator/Scrambler node (Relay + optional Prover). It assumes you run the Rust services in this repo: Indexer (optional), Relay (API + worker + queue), and an SP1 prover pipeline (local or remote).

## 1) Provisioning & Sizing

Profiles below assume devnet/mainnet priorities and the default SLOs. Start with the Scrambler (no in-house proving) profile and scale up.

### A. Scrambler-Only (no in-house proving)
- CPU: 2–4 vCPU
- RAM: 8–16 GB
- Disk: 50–100 GB (logs + Postgres + OS)
- Postgres: 1–2 vCPU, 2–4 GB RAM, 20+ GB SSD
  - Pool size: 10–20 connections; WAL on SSD
- Redis: 1 vCPU, 1 GB RAM (queue only)
- Outbound bandwidth: ≥50 Mbps sustained, low jitter
- RPC access: 1–2 dedicated RPC endpoints (confirmed/finalized), rate limit ≥ 50 rps burst
- Jito (optional): 1–2 bundle relays; ensure low latency path

### B. Prover + Scrambler (bundled)
- CPU: 8–16 vCPU (dedicated prover threads)
- RAM: 32–64 GB (SP1 memory during proving)
- Disk: 100–200 GB (proof artifacts + logs)
- Postgres: 2–4 vCPU, 8 GB RAM, 50+ GB SSD
  - Enable auto-vacuum; tune shared_buffers (25% RAM)
- Redis: 2 vCPU, 2–4 GB RAM
- Outbound bandwidth: ≥100 Mbps; ensure consistent egress for Jito
- GPU: not required for current SP1 pipeline (CPU proving). If moved to GPU/cloud prover, right-size accordingly.

### C. OS & System
- OS: Linux x86_64 LTS (Ubuntu 22.04+, Debian 12+, or equivalent)
- Time sync: Chrony/ntpd
- Filesystems: ext4/xfs on SSD; noatime recommended
- Limits: increase file descriptors (e.g., `nofile` 262144)

### D. Dependencies
- Postgres 14+
- Redis 6+
- Solana RPC(s) (confirmed and finalized)
- Optional Jito bundle relay access
- Indexer URL (if using a shared indexer) and artifacts route

## 2) SLOs (Targets)
- Proof generation p95: ≤ 120 seconds (single-withdraw) 
- Broadcast success rate: ≥ 95% (over rolling 1h window)
- Confirmation time: ≤ 2 blocks (p95) with appropriate priority fees
- Queue latency (queued→processing): ≤ 5 seconds (p95)

## 3) Configuration Hints
- Priority fees: start with 500–3_000 micro-lamports/CU; auto-bump on failures
- Compute budget: 1,000,000 CU per withdraw (builder sets this)
- Jitter: set `RELAY_JITTER_BLOCK_MS` (default 400 ms) to spread submissions
- Database:
  - Postgres pool: 10–20 connections, statement timeout 10–30s
  - Redis: persistence off for queue-only, notify-keyspace-events disabled

## 4) Alerting (Symptoms, Signals, Thresholds)

- Proof queue backlog (critical for Prover role)
  - Signal: `jobs.queued > 100` for 5 minutes or `queued/throughput > 10x baseline`
  - Action: add prover workers; scale CPU/RAM; check SP1 artifacts cache; verify RPC rate limits
- RPC send/confirm errors
  - Signal: `rpc.send.error_rate > 5%` or `confirm.timeout.count > 0` per 5 minutes
  - Action: failover RPC endpoint; increase priority fee; re-simulate a sample; inspect recent Solana health
- Jito rejection rate (if enabled)
  - Signal: `jito.bundle.reject_rate > 10%`
  - Action: raise tip; try alternate relay; ensure bundle format; check clock drift
- Nullifier collisions (should be zero)
  - Signal: `nullifier.duplicate.count > 0`
  - Action: immediately halt related job source; verify Merkle proofs & note selection; inspect on-chain shard
- InvalidOutputsHash
  - Signal: `outputs_hash.mismatch.count > 0`
  - Action: verify endianness (u64 LE); outputs order; recipient pubkey bytes

## 5) Dashboards (KPIs)

- Job pipeline
  - Queued / Running / Done / Failed (time series + current counts)
  - Queue latency (queued→processing), processing duration
- Prover
  - Proof p50/p95 latency; total SP1 cycles; success/failure counts
  - Artifact sizes (proof 260B fragment confirm; public inputs 104B)
- Chain & Submit
  - CU per tx; priority fee; tips (Jito) spent per tx
  - Broadcast success rate; confirmation blocks; simulate units_consumed
- Economics
  - Fees collected; revenue share per role (treasury/prover/scrambler/LP)
  - Net margin after tips & RPC costs

## 6) Operations Playbooks

### A. Root Drift
- Symptoms: on-chain `Root not found in RootsRing`; relay retries; proof mismatch.
- Immediate actions:
  1) Refresh latest root from Indexer (`GET /api/v1/merkle/root`)
  2) Ensure admin pushed root to program (AdminPushRoot path)
  3) Regenerate proof with accepted root from ring window
- Preventative: reduce root window lag; automate AdminPushRoot on indexer root updates

### B. Insufficient Pool Lamports
- Symptoms: on-chain `InsufficientLamports`; repeated failed withdrawals
- Immediate actions:
  1) Query pool balance; compare against planned amount
  2) Backoff and requeue; prefer smaller notes/amount buckets
  3) Replenish pool PDA (LP deposit) or throttle jobs
- Preventative: alert when `pool_balance < P95(amount)` within last 5 minutes

### C. Repeated InvalidOutputsHash
- Symptoms: `InvalidOutputsHash` from on-chain; simulation matches but runtime fails
- Immediate actions:
  1) Verify outputs hashing: BLAKE3(address:32 || amount:u64 LE) exactly
  2) Ensure recipient address bytes are 32 raw pubkey bytes (not base58 string)
  3) Confirm `public.inputs[64..96] == blake3(output)` and amount LE at `[96..104]`
- Preventative: preflight check in Relay before submission; job-reject on mismatch

## 7) Runbooks & Procedures

### Routine maintenance
- Rotate RPC endpoints (weekly) and validate health/latency
- Vacuum/analyze Postgres weekly; ensure sufficient disk
- Trim logs, archive prover artifacts if stored

### Incident triage checklist
- Is the backlog rising? (queued, running)
- Are RPC errors spiking? (send/confirm)
- Are Jito rejects rising? (if enabled)
- Are nullifier duplicates observed? (must be zero)
- Is the pool balance healthy relative to current workload?

## 8) References
- ZK contract & encoding: docs/zk/encoding.md, docs/zk/circuit-withdraw.md
- On-chain program: programs/shield-pool
- Prover: packages/zk-guest-sp1
- Relay: services/relay

---

Appendix: Quick Checks
- Confirm 104B public inputs ordering on a job before submit.
- Simulate a built tx and log `units_consumed`.
- Verify fee math: `fee = 2_500_000 + (amount*5)/1000` and `recipient = amount - fee`.
