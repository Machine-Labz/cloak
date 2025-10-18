# Cloak Validator — Development Notes and Worklog

This document captures the work completed across six prompts plus a PoW gate design, with file locations, how to use them, and concrete next steps. Use it to quickly pick up where we left off.

## 0) Contents
- 1) Validator Agent API (OpenAPI + TS types + client)
- 2) Rust crate: `cloak-proof-extract`
- 3) Transaction Builder (exact 437‑byte layout)
- 4) Planner (note selection, hashing, jitter)
- 5) Submit trait (RPC + Jito feature)
- 6) RUNBOOK (operations)
- 7) PoW Gate (minimal spec)
- 8) Quick file map and next steps

---

## 1) Validator Agent API (OpenAPI + TS types + client)

- Status: Spec, zod schemas, and client are authored (in prior response) but not yet committed as files.
- Purpose: Job lifecycle for withdraws, strict canonical encodings, and a submission endpoint.

Canonical constraints
- Withdraw data length (after 1‑byte discriminant): `437` bytes.
- SP1 public inputs: `104` bytes in order: `root(32)`, `nf(32)`, `outputs_hash(32)`, `amount_le(8)`.
- Proof fragment: `260`‑byte Groth16 extracted from SP1 bundle.

Endpoints
- POST `/jobs/withdraw` → body `{ public_bin_hex(208 hex), outputs[], deadline_iso, payer_hints?, fee_caps? }` → `{ job_id, status }`.
- GET `/jobs/{job_id}` → `{ job_id, status, artifacts?{ proof_hex_260, public_bin_hex_104, tx_bytes_base64? }, error? }`.
- POST `/submit` → body `{ tx_bytes_base64 }` → `{ signature, slot? }`.

Security/limits
- 429 + `Retry-After`; 413 for payloads (≤ ~64 KiB suggested).

Suggested repo locations to commit
- OpenAPI: `docs/api/validator-agent.yaml`
- Zod types: `services/web/lib/schemas.ts`
- Client: `services/web/lib/validatorAgent.ts`

Next steps
- Commit the above artifacts and (optionally) add minimal server handlers in Relay matching the spec.

---

## 2) Rust crate: `cloak-proof-extract`

- Path: `packages/cloak-proof-extract/` (added to workspace)
- APIs:
  - `extract_groth16_260(sp1_proof_bundle: &[u8]) -> Result<[u8;260], Error>`
    - Heuristic 1: stable offset `0x2b0` (observed in current SP1 artifacts).
    - Heuristic 2: scan for bincode‑like `u64` length prefix `== 260`.
  - `parse_public_inputs_104(bytes: &[u8]) -> Result<PublicInputs, Error>` where `PublicInputs { root, nf, outputs_hash, amount }`.
- Features: `no_std` optional via `alloc`, and `hex` for serde hex field encoding.
- Tests: read `packages/zk-guest-sp1/out/{proof.bin, public.bin}`; assert extraction/parse correctness.

Usage
```rust
let frag = cloak_proof_extract::extract_groth16_260(&bundle)?;
let pi = cloak_proof_extract::parse_public_inputs_104(&public_bytes)?;
assert_eq!(public_bytes.len(), 104);
```

Next steps
- Use in Relay worker to normalize proof extraction and public input parsing (legacy wrapper does this already).

---

## 3) Transaction Builder (exact 437‑byte layout)

- Path: `services/relay/src/solana/transaction_builder.rs` (rewritten)
- Core:
  - `build_withdraw_ix_body(&groth16_260, &public_104, &recipient_addr_32, recipient_amount)` → `Vec<u8>`
    - Layout (must equal 437): `[0..260]=proof(260) | [260..364]=public(104) | [364..396]=nf-dup(public[32..64]) | [396]=1 | [397..429]=recipient(32) | [429..437]=amount_le(8)`
  - `build_withdraw_instruction(program_id, body_437, pool, treasury, roots_ring, nullifier_shard, recipient)` → `Instruction` with discriminant `2` and required accounts: `[pool, treasury, roots_ring, nullifier_shard, recipient, system]`.
  - `build_withdraw_transaction(.., fee_payer, recent_blockhash, priority_micro_lamports)` → `Transaction` with compute budget ix (1,000,000 CU) + priority fee.
  - `#[cfg(feature=\"jito\")] build_withdraw_versioned(..)` → `VersionedTransaction` for bundle submission.
  - `simulate(rpc_client, &tx)` → logs `units_consumed` and returns CU.
  - Back‑compat: `build_withdraw_instruction_legacy(..)` extracts 260‑byte proof with `cloak-proof-extract`, validates 104‑byte public inputs, and builds a minimal tx (placeholder PDAs).
- Tests: assert offsets and little‑endian amount; length = 437.

Next steps
- Replace placeholder PDAs with real derivations in production path; wire relay to use the new builders end‑to‑end.

---

## 4) Planner (note selection, hashing, jitter)

- Path: `services/relay/src/planner.rs`
- Types: `RootMeta`, `NoteMeta`, `Output`, `Selected`.
- Functions:
  - `select_note(target_amount, roots_window, notes)` → pick a note in the largest same‑amount bucket; tie‑break by most recent root slot; enforce feasibility: `note.amount >= target + fee(note.amount)`.
  - `compute_outputs_single(recipient_addr, recipient_amount)` → returns single `Output` and `outputs_hash = BLAKE3(address:32 || amount:u64_le)`.
  - `jitter_delay(now)` → 0–3 blocks delay; `RELAY_JITTER_BLOCK_MS` (default `400ms`).
- Fee conservation: `fee = 2_500_000 + (amount * 5) / 1_000` (MVP).
- Tests: conservation; outputs_hash matches guest/on‑chain; selection rule; jitter bounds.

Next steps
- Integrate into worker: choose note → compute outputs → enqueue prove/build/submit cycles with jitter.

---

## 5) Submit trait (RPC + Jito feature)

- Path: `services/relay/src/solana/submit.rs`
- Trait: `trait Submit { fn send(&self, tx: VersionedTransaction) -> Result<Signature, Error>; }`
- RpcSubmit: `send_transaction_with_config` + exponential jitter backoff; assumes CU price ix already in tx.
- JitoSubmit (feature `jito`): placeholder fallback to RPC with backoff; ready to integrate `jito-solana` bundle submit + tip.
- Helper: `confirm(&RpcClient, &Signature, min_slot: Option<u64>, timeout: Duration)` to wait for confirmation/slot target.

Next steps
- Add config in Relay to select Rpc vs Jito; implement Jito bundling with tips once dependency is added.

---

## 6) RUNBOOK (operations)

- Path: `RUNBOOK.md`
- Contents: provisioning profiles (Scrambler‑only vs Prover+Scrambler), DB/Redis sizing, SLOs (p95 proof ≤ 120s, broadcast ≥ 95%, confirmation ≤ 2 blocks), alerts (proof backlog, RPC errors, Jito rejection, nullifier collisions), dashboards (jobs, CU/tx, tips, revenue), and playbooks (root drift, insufficient pool lamports, InvalidOutputsHash).

Next steps
- Hook up to your monitoring stack and set alert thresholds.

---

## 7) PoW Gate (minimal spec)

- Status: Spec drafted (in prior response), not yet coded.
- Purpose: Sybil resistance for scrambler batch rights. Mine over batch descriptor; on‑chain registry issues claim windows; scrambler earns fee share per withdraw; withholding prevention via reveal windows and re‑auction.
- Key elements:
  - Hashing: BLAKE3‑256; domain `"CLOAK:SCRAMBLE:v1"`; `H = BLAKE3(tag || slot || miner_pubkey || batch_hash || nonce)`; `H < T(slot)`.
  - On‑chain Registry: `Miner`, `Claim`, CPI from withdraw to `consume_claim`, pay scrambler share out of fee; slashing/penalty rules.
  - Off‑chain: build batch descriptor (`roots_window`, `jobs_root`, `k`, policy, expiry), mine and reveal within R slots; use within W slots; re‑auction on timeout.

Next steps
- Draft a minimal Pinocchio program for the registry (accounts/state) and integrate withdraw CPI.

---

## 8) Quick File Map + Next Actions

Files added/updated
- `packages/cloak-proof-extract/` (new crate; tests pass)
- `services/relay/src/solana/transaction_builder.rs` (rewritten; tests added)
- `services/relay/src/planner.rs` (new; tests added)
- `services/relay/src/solana/submit.rs` (new)
- `RUNBOOK.md` (new)

Suggested commits to add
- OpenAPI spec → `docs/api/validator-agent.yaml`
- Zod schemas → `services/web/lib/schemas.ts`
- Client SDK → `services/web/lib/validatorAgent.ts`
- PoW gate spec → `docs/pow-gate.md` (optional)

Immediate next steps
- Replace placeholder PDAs/treasury in builder and wire real derivations.
- Preflight outputs_hash and conservation before submit; reject bad jobs.
- Add config to select Rpc vs Jito submit; call `confirm()` with 2‑block target.
- Expose `/jobs/*` API if you plan to run the Validator Agent as a service.

---

## Reference: Canonical Encodings
- Public inputs (104B): `root(32) || nf(32) || outputs_hash(32) || amount_le(8)`
- outputs_hash: `BLAKE3(address:32 || amount:u64_le)` for MVP single output.
- Withdraw body (437B after discriminant): `proof(260) || public(104) || nf_dup(32) || 1 || recipient(32) || amount_le(8)`

