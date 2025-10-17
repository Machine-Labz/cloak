# WASM Prover Porting Blueprint

## Goal
Execute the `zk-guest-sp1` prover entirely inside the browser by compiling the SP1 CPU prover stack to WebAssembly (initially targeting `wasm32-wasip1` for Node/WASI iteration and graduating to `wasm32-unknown-unknown + atomics` for WebWorkers).

## High-Level Phases
1. **Dependency Hygiene** – Vendor the SP1 crates we rely on (`sp1-sdk`, `sp1-core-machine`, `sp1-core-executor`, `p3-maybe-rayon`) and introduce `#[cfg(target_arch = "wasm32")]` fast paths that remove POSIX requirements (threads, `sysinfo`, `tempfile`, filesystem dumps, env probing). Start with:
   - Gate `setup_memory_usage_monitoring` behind `cfg(not(target_arch = "wasm32"))`.
   - Replace `std::thread::scope`/`sync_channel` loops with single-threaded equivalents when the `wasm` feature is active (use sequential iterators + `VecDeque`).
   - Re-export a `wasm` feature flag from each crate to avoid pulling `rayon`, `tokio`, and `network` dependencies.
2. **Build Toolchain** – Install `wasm32-wasip1` target and use `wasm-bindgen-cli` 0.2.95+ with `--target web --reference-types --weak-refs`. Configure `RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"` when compiling `wasm32-unknown-unknown` to enable thread support via WebWorkers.
3. **Runtime Wrapper** – Create a new crate (`packages/zk-guest-sp1/wasm-runner`) that exposes `#[wasm_bindgen]` bindings for `generate_proof` and `verify_proof`. Internally it should:
   - Deserialize JSON inputs into the existing host structs.
   - Invoke the CPU prover builder (`ProverClient::builder().cpu().build()`) using the wasm-safe SP1 crates from Phase 1.
   - Return proofs/public inputs as hex to match the API shape already expected by `services/web`.
4. **Browser Integration** – Load the resulting `.wasm` through a dedicated WebWorker. Ensure Next.js enables `SharedArrayBuffer` (COOP/COEP headers already configured in `next.config.mjs`). Update `useSP1WasmProver` to post messages to the worker and stream progress events.
5. **Performance Profiling** – Use Chrome’s Performance panel to watch memory usage. Expect the first cut (single-threaded) to be slow; add task queue instrumentation so we can later reintroduce Rayon-style parallelism via `wasm-bindgen-rayon`.

## Recommended Next Steps
- Fork SP1 upstream or vendor it under `third-party/sp1/*` with `[patch.crates-io]` overrides in the root `Cargo.toml`.
- Introduce a `wasm` cargo feature that disables `parallel`/`network` features across the patched crates.
- Create a smoke test that runs `cargo test -p zk-guest-sp1-host --target wasm32-wasip1` to ensure the patched crates compile (even before Web integration).
- Once compilation succeeds, wire the new wasm module into `services/web/wasm-prover` and retire the simulated proof path.

### Immediate Action Items
- Install the `wasm32-wasip1` target and `wasm-bindgen-cli` on a machine without sandbox restrictions, then rerun `services/web/scripts/build-wasm.sh --release --skip-deploy` to confirm the current workspace builds end-to-end.
- Wire the new `zk-guest-sp1-wasm-runner` crate into the Next.js loader once the SP1 fork compiles for WASM, replacing the simulated proof hooks in `useSP1WasmProver`.
- Incrementally gate thread-heavy sections inside the forked SP1 crates under a shared `wasm` feature so the CPU prover executes single-threaded when compiled for the browser.

## Open Questions
- **Parallel Proofs**: browser threads demand COOP/COEP + `SharedArrayBuffer`; decide whether to ship a single-thread MVP first.
- **Artifacts Storage**: large verification keys currently touch the filesystem—replace with in-memory buffers or JS-managed IndexedDB.
- **Deterministic RNG**: `rand` defaults to `getrandom`; confirm the wasm build uses the `js` feature (already enabled in `services/web/wasm-prover/Cargo.toml`).
