# Task: Implement SP1 Withdraw Guest

Goal: SP1 guest (Rust) that enforces constraints in `docs/zk/circuit-withdraw.md`.

Deliver:
- `packages/zk-guest-sp1/` with Cargo workspace
- Guest function taking private + public inputs
- BLAKE3-256 hashing, fixed encodings per docs
- CLI: `prove_withdraw private.json public.json > proof.bin`
- Golden test vectors

Read:
- docs/zk/circuit-withdraw.md
- docs/zk/encoding.md
