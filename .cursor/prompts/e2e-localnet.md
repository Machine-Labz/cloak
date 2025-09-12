# Task: E2E Localnet

Goal: Script to run deposit → index → prove → withdraw on localnet.

Deliver:
- tooling script that:
  1) Starts local validator
  2) Deploys `shield-pool`
  3) Sends a deposit (stub ingest into indexer)
  4) Proves withdraw via SP1 guest
  5) Calls relay to submit
  6) Asserts balances and nullifier stored
