---
title: Changelog
description: Recent updates and changes to the Cloak project
---

# Changelog

This changelog is automatically generated from Git commit history.

View the complete history on [GitHub](https://github.com/Machine-Labz/cloak/commits/master).

---

## December 2025

### ✨ Features

- feat(staking): implement private staking workflow, including new stake parameters and validation; add support for staking in relay service and SDK ([8183730](https://github.com/Machine-Labz/cloak/commit/8183730ca3940f018e33f99d0af53e98d7376144))
  <details>
  <summary>📂 <strong>30 files changed</strong>: <span className="text-green-500">+2790</span> / <span className="text-red-500">-36</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+7</span> / <span className="text-red-500">-2</span> |
  | `docs/workflows/stake.md` | <span className="text-green-500">+732</span> / <span className="text-red-500">-0</span> |
  | `...ages/zk-guest-sp1/.artifacts/zk-guest-sp1-guest` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `.../shield-pool/src/instructions/withdraw_stake.rs` | <span className="text-green-500">+324</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-3</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/sp1_tee_client.rs` | <span className="text-green-500">+105</span> / <span className="text-red-500">-19</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+63</span> / <span className="text-red-500">-8</span> |
  | `services/relay/src/lib.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+100</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+188</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/stake/mod.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/stake/types.rs` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `tooling/cloak-sdk/README.md` | <span className="text-green-500">+116</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/CloakSDK.ts` | <span className="text-green-500">+221</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/types.ts` | <span className="text-green-500">+48</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/index.ts` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/services/ProverService.ts` | <span className="text-green-500">+11</span> / <span className="text-red-500">-1</span> |
  | `tooling/cloak-sdk/src/services/RelayService.ts` | <span className="text-green-500">+104</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/crypto.ts` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test_stake.rs` | <span className="text-green-500">+605</span> / <span className="text-red-500">-0</span> |

  </details>

---

## November 2025

### ✨ Features

- feat(deposit): implement deposit preparation and confirmation endpoints; add transaction signature update functionality and enhance error handling ([d52475c](https://github.com/Machine-Labz/cloak/commit/d52475c87007b14de968811e0bd7402d7a68c5ac))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+345</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/database/storage.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+311</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-1</span> |

  </details>
- enhance(logging): improve JSON input handling for swap_params, adding validation and detailed logging to ensure correct serialization and facilitate debugging ([f885e5e](https://github.com/Machine-Labz/cloak/commit/f885e5e87e2b59997a98ac169f06485e22e280bc))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+62</span> / <span className="text-red-500">-25</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/sp1_tee_client.rs` | <span className="text-green-500">+57</span> / <span className="text-red-500">-24</span> |

  </details>
- enhance(logging): improve JSON input parsing and logging for swap_params, adding detailed error handling and debug information to facilitate troubleshooting ([a7d0a4d](https://github.com/Machine-Labz/cloak/commit/a7d0a4dca0459a190c8440d48c8e4f7ad2754490))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+53</span> / <span className="text-red-500">-5</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-2</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `...-pool/src/instructions/execute_swap_via_orca.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |

  </details>
- feat(instructions): add PrepareSwapSol instruction to facilitate SOL to wSOL conversion before executing swaps ([68b7b78](https://github.com/Machine-Labz/cloak/commit/68b7b781b3373d3a937af4fd94f9d240f21b3fc5))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+165</span> / <span className="text-red-500">-81</span></summary>

  | File | Changes |
  |------|--------|
  | `...-pool/src/instructions/execute_swap_via_orca.rs` | <span className="text-green-500">+72</span> / <span className="text-red-500">-80</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `...hield-pool/src/instructions/prepare_swap_sol.rs` | <span className="text-green-500">+86</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |

  </details>
- feat(instructions): add ExecuteSwapViaOrca instruction and corresponding processing function to support on-chain swaps ([1a940ce](https://github.com/Machine-Labz/cloak/commit/1a940ce9e7e1331ede8ca9212b9511c411cc48a7))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(dependencies): add new packages ethnum, orca_whirlpools_client, orca_whirlpools_core, orca_whirlpools_macros, and update spl-memo version in Cargo.lock ([d43a2ee](https://github.com/Machine-Labz/cloak/commit/d43a2eeb1fe2b7ed03709012daeda1b1b2b4f77c))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+65</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+65</span> / <span className="text-red-500">-1</span> |

  </details>
- feat(constants): update scramble program IDs for devnet, testnet, and localnet; add production API URL for relay service ([f272418](https://github.com/Machine-Labz/cloak/commit/f272418f66b57ccd8551542d73492c470dfba417))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+84</span> / <span className="text-red-500">-44</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/cloak-miner/src/constants.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-4</span> |
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-31</span> |
  | `packages/cloak-miner/src/manager.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-9</span> |

  </details>
- feat(swap): add prove-test-swap functionality and enhance withdrawal flow with relay integration ([99a1e88](https://github.com/Machine-Labz/cloak/commit/99a1e882f8f941369058dc7a12625f9ea7950cb3))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+1187</span> / <span className="text-red-500">-103</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `tooling/test/src/prove_test_multiple_outputs.rs` | <span className="text-green-500">+232</span> / <span className="text-red-500">-98</span> |
  | `tooling/test/src/prove_test_swap.rs` | <span className="text-green-500">+923</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+26</span> / <span className="text-red-500">-4</span> |

  </details>
- feat(swap): add execute_swap_via_orca instruction for atomic on-chain swaps and implement recover_swap_funds instruction for reclaiming SOL after timeout ([bd323f6](https://github.com/Machine-Labz/cloak/commit/bd323f638b287cf40d6fe1db460c951de4f3b86f))
  <details>
  <summary>📂 <strong>11 files changed</strong>: <span className="text-green-500">+449</span> / <span className="text-red-500">-72</span></summary>

  | File | Changes |
  |------|--------|
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...ms/shield-pool/src/instructions/execute_swap.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-7</span> |
  | `...-pool/src/instructions/execute_swap_via_orca.rs` | <span className="text-green-500">+262</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `...eld-pool/src/instructions/recover_swap_funds.rs` | <span className="text-green-500">+92</span> / <span className="text-red-500">-0</span> |
  | `...eld-pool/src/instructions/release_swap_funds.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-8</span> |
  | `...s/shield-pool/src/instructions/withdraw_swap.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-15</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/state/mod.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-6</span> |
  | `programs/shield-pool/src/tests/withdraw_swap.rs` | <span className="text-green-500">+54</span> / <span className="text-red-500">-33</span> |

  </details>
- feat(relay): enhance withdrawal flow with fee calculation adjustments and add minimum balance check for rent exemption ([c918e85](https://github.com/Machine-Labz/cloak/commit/c918e85f08ffaa264c18b9ca64a12987d62cf25d))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+173</span> / <span className="text-red-500">-22</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+15</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+48</span> / <span className="text-red-500">-14</span> |
  | `services/relay/src/solana/swap.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+101</span> / <span className="text-red-500">-5</span> |

  </details>
- feat(relay): implement swap withdrawal flow with off-chain DEX integration ([85685fa](https://github.com/Machine-Labz/cloak/commit/85685fa0b166bc0468518298de5e39783fe36849))
  <details>
  <summary>📂 <strong>42 files changed</strong>: <span className="text-green-500">+3514</span> / <span className="text-red-500">-180</span></summary>

  | File | Changes |
  |------|--------|
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-2</span> |
  | `...ms/shield-pool/src/instructions/execute_swap.rs` | <span className="text-green-500">+100</span> / <span className="text-red-500">-0</span> |
  | `...rams/shield-pool/src/instructions/initialize.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-6</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...eld-pool/src/instructions/release_swap_funds.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-5</span> |
  | `...s/shield-pool/src/instructions/withdraw_swap.rs` | <span className="text-green-500">+272</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/state/mod.rs` | <span className="text-green-500">+151</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/admin_push_root.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-3</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-3</span> |
  | `programs/shield-pool/src/tests/mod.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-3</span> |
  | `programs/shield-pool/src/tests/withdraw_swap.rs` | <span className="text-green-500">+219</span> / <span className="text-red-500">-0</span> |
  | `services/relay/.env.spl.example` | <span className="text-green-500">+14</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+31</span> / <span className="text-red-500">-1</span> |
  | `services/relay/examples/swap_withdraw_example.rs` | <span className="text-green-500">+134</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/test_jupiter_swap.rs` | <span className="text-green-500">+73</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/backlog.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+102</span> / <span className="text-red-500">-35</span> |
  | `services/relay/src/bin/check_swap_state.rs` | <span className="text-green-500">+107</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/bin/complete_swap.rs` | <span className="text-green-500">+167</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+103</span> / <span className="text-red-500">-29</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-6</span> |
  | `services/relay/src/error.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/lib.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-4</span> |
  | `services/relay/src/planner.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/planner/orchestrator.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-6</span> |
  | `services/relay/src/solana/jupiter.rs` | <span className="text-green-500">+114</span> / <span className="text-red-500">-21</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+266</span> / <span className="text-red-500">-25</span> |
  | `services/relay/src/solana/swap.rs` | <span className="text-green-500">+620</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+337</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/swap/jupiter.rs` | <span className="text-green-500">+230</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/swap/mod.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/swap/types.rs` | <span className="text-green-500">+97</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+116</span> / <span className="text-red-500">-10</span> |
  | `services/relay/src/worker/window_scheduler.rs` | <span className="text-green-500">+36</span> / <span className="text-red-500">-11</span> |

  </details>
- feat(swap): implement swap mode with new parameters and hash computation for outputs ([6e507c8](https://github.com/Machine-Labz/cloak/commit/6e507c890d5f56525233f57abbc98dccba90227f))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+153</span> / <span className="text-red-500">-27</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+82</span> / <span className="text-red-500">-2</span> |
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+36</span> / <span className="text-red-500">-11</span> |
  | `packages/zk-guest-sp1/host/build.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-14</span> |

  </details>
- Add SPL token support with Jupiter integration ([5e9b7db](https://github.com/Machine-Labz/cloak/commit/5e9b7dbcec580ffb7693ad0b97f85b4c851837d3))
  <details>
  <summary>📂 <strong>133 files changed</strong>: <span className="text-green-500">+11532</span> / <span className="text-red-500">-2008</span></summary>

  | File | Changes |
  |------|--------|
  | `.dockerignore` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+84</span> / <span className="text-red-500">-0</span> |
  | `.gitignore` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `Cargo.lock` | <span className="text-green-500">+202</span> / <span className="text-red-500">-32</span> |
  | `DOCUSAURUS_RESTRUCTURE.md` | <span className="text-green-500">+105</span> / <span className="text-red-500">-0</span> |
  | `README.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-9</span> |
  | `deployment/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-74</span> |
  | `deployment/compose.yml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `deployment/init.sql` | <span className="text-green-500">+267</span> / <span className="text-red-500">-227</span> |
  | `deployment/nginx/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-90</span> |
  | `deployment/nginx/TESTING.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-132</span> |
  | `deployment/nginx/nginx.conf` | <span className="text-green-500">+67</span> / <span className="text-red-500">-0</span> |
  | `docs/api/indexer.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+411</span> / <span className="text-red-500">-365</span> |
  | `docs/{ => docs}/COMPLETE_FLOW_STATUS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/POW_INTEGRATION_GUIDE.md` | <span className="text-green-500">+453</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/POW_QUICK_REFERENCE.md` | <span className="text-green-500">+332</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/SPL_TOKEN_SUPPORT.md` | <span className="text-green-500">+288</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/api/indexer.md` | <span className="text-green-500">+88</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/api/relay.md` | <span className="text-green-500">+78</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/api/validator-agent.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/glossary.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/offchain/indexer.md` | <span className="text-green-500">+95</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/offchain/relay.md` | <span className="text-green-500">+102</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/offchain/web-app.md` | <span className="text-green-500">+71</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/onchain/scramble-registry.md` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/onchain/shield-pool.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/operations/metrics-guide.md` | <span className="text-green-500">+419</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/operations/runbook.md` | <span className="text-green-500">+146</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/introduction.md` | <span className="text-green-500">+47</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/quickstart.md` | <span className="text-green-500">+118</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/status.md` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/system-architecture.md` | <span className="text-green-500">+76</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/tech-stack.md` | <span className="text-green-500">+40</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/view-spend-keys.md` | <span className="text-green-500">+286</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/overview/visual-flow.md` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/packages/cloak-miner.md` | <span className="text-green-500">+67</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/packages/cloak-proof-extract.md` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/packages/tooling-test.md` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/packages/vkey-generator.md` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/packages/zk-guest-sp1.md` | <span className="text-green-500">+72</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/packages/zk-verifier-program.md` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/pow/overview.md` | <span className="text-green-500">+44</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/roadmap.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/workflows/deposit.md` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/workflows/pow-withdraw.md` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `docs/docs/workflows/withdraw.md` | <span className="text-green-500">+109</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/api-contracts.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/circuit-withdraw.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/design.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/encoding.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/merkle.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/onchain-verifier.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/prover-sp1.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/testing.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/zk/threat-model.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/docusaurus.config.ts` | <span className="text-green-500">+3</span> / <span className="text-red-500">-6</span> |
  | `docs/offchain/indexer.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/overview.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/web-app.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `docs/onchain/shield-pool.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/cloak-miner.md` | <span className="text-green-500">+26</span> / <span className="text-red-500">-29</span> |
  | `docs/sidebars.ts` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |
  | `docs/src/pages/index.tsx` | <span className="text-green-500">+63</span> / <span className="text-red-500">-0</span> |
  | `docs/workflows/deposit.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/zk/anonymity-set-strategy.md` | <span className="text-green-500">+575</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/QUICKSTART.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-85</span> |
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-4</span> |
  | `...ages/zk-guest-sp1/.artifacts/zk-guest-sp1-guest` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |
  | `packages/zk-guest-sp1/host/build.rs` | <span className="text-green-500">+42</span> / <span className="text-red-500">-2</span> |
  | `services/indexer/.env.spl.example` | <span className="text-green-500">+126</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-3</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+30</span> / <span className="text-red-500">-89</span> |
  | `services/indexer/src/database/migrations.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/src/database/storage.rs` | <span className="text-green-500">+88</span> / <span className="text-red-500">-32</span> |
  | `services/indexer/src/merkle.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-38</span> |
  | `.../src/migrations/002_add_leaf_index_sequence.sql` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+101</span> / <span className="text-red-500">-188</span> |
  | `services/indexer/src/server/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+66</span> / <span className="text-red-500">-138</span> |
  | `services/indexer/src/server/rate_limiter.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-158</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-20</span> |
  | `services/relay/.env.spl.example` | <span className="text-green-500">+149</span> / <span className="text-red-500">-0</span> |
  | `services/relay/.gitleaksignore` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+30</span> / <span className="text-red-500">-70</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/api/prove_local.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-94</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+39</span> / <span className="text-red-500">-13</span> |
  | `services/relay/src/claim_manager.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/db/repository.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-13</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/solana/jupiter.rs` | <span className="text-green-500">+319</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+15</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/worker/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+71</span> / <span className="text-red-500">-3</span> |
  | `services/relay/src/worker/window_scheduler.rs` | <span className="text-green-500">+214</span> / <span className="text-red-500">-0</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `tooling/cloak-sdk/README.md` | <span className="text-green-500">+465</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/examples/basic-usage.ts` | <span className="text-green-500">+182</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/jest.config.cjs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/package.json` | <span className="text-green-500">+68</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/keys.ts` | <span className="text-green-500">+268</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/note-manager.ts` | <span className="text-green-500">+298</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/note.ts` | <span className="text-green-500">+233</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/storage.ts` | <span className="text-green-500">+205</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/core/types.ts` | <span className="text-green-500">+341</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/helpers/encrypted-output.ts` | <span className="text-green-500">+84</span> / <span className="text-red-500">-0</span> |
  | `...ing/cloak-sdk/src/helpers/wallet-integration.ts` | <span className="text-green-500">+122</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/index.ts` | <span className="text-green-500">+171</span> / <span className="text-red-500">-0</span> |
  | `...loak-sdk/src/services/DepositRecoveryService.ts` | <span className="text-green-500">+337</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/services/IndexerService.ts` | <span className="text-green-500">+256</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/services/ProverService.ts` | <span className="text-green-500">+253</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/services/RelayService.ts` | <span className="text-green-500">+233</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/solana/instructions.ts` | <span className="text-green-500">+132</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/crypto.ts` | <span className="text-green-500">+196</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/errors.ts` | <span className="text-green-500">+208</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/fees.ts` | <span className="text-green-500">+128</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/network.ts` | <span className="text-green-500">+101</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/pda.ts` | <span className="text-green-500">+67</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/src/utils/validation.ts` | <span className="text-green-500">+192</span> / <span className="text-red-500">-0</span> |
  | `tooling/cloak-sdk/tsconfig.json` | <span className="text-green-500">+34</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span> |
  | `tooling/test/src/prove_test_multiple_outputs.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-55</span> |

  </details>
- feat(relay): implement window scheduler for batched job processing based on Solana slot patterns ([8c1ddb4](https://github.com/Machine-Labz/cloak/commit/8c1ddb474996d5ce3d9f3e10c4bd072676dbea0c))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+245</span> / <span className="text-red-500">-8</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/src/main.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-8</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/worker/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/worker/window_scheduler.rs` | <span className="text-green-500">+214</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(indexer): implement atomic leaf index allocation and enhance Docker build process ([304cfa3](https://github.com/Machine-Labz/cloak/commit/304cfa3548b6c0196bc49a084986ea73ad6afe76))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+99</span> / <span className="text-red-500">-37</span></summary>

  | File | Changes |
  |------|--------|
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+17</span> / <span className="text-red-500">-10</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+9</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/src/database/migrations.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/src/database/storage.rs` | <span className="text-green-500">+26</span> / <span className="text-red-500">-14</span> |
  | `.../src/migrations/002_add_leaf_index_sequence.sql` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+9</span> / <span className="text-red-500">-4</span> |

  </details>
- feat(build): implement pre-built ELF support and optional guest program build ([6e0d49f](https://github.com/Machine-Labz/cloak/commit/6e0d49ff273ca8dddbd8c9e1b285b94ed16eb6c5))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+47</span> / <span className="text-red-500">-3</span></summary>

  | File | Changes |
  |------|--------|
  | `...ages/zk-guest-sp1/.artifacts/zk-guest-sp1-guest` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |
  | `packages/zk-guest-sp1/host/build.rs` | <span className="text-green-500">+42</span> / <span className="text-red-500">-2</span> |

  </details>
- feat(docs): add comprehensive guide on view/spend key architecture ([cb5b95d](https://github.com/Machine-Labz/cloak/commit/cb5b95d0060acf34dbbe0dff6e0766245378aebd))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+882</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/overview/view-spend-keys.md` | <span className="text-green-500">+286</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/anonymity-set-strategy.md` | <span className="text-green-500">+575</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(database): add beta interest registrations table for mainnet beta waitlist ([50b1384](https://github.com/Machine-Labz/cloak/commit/50b13840b21e1991143003b8e4d5e327f04348c2))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+18</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `deployment/init.sql` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>

### 🐛 Bug Fixes

- fix(tests): update comments for clarity and correct transaction link in swap withdrawal output ([5db55ee](https://github.com/Machine-Labz/cloak/commit/5db55eef1c569f52df84a500c9e2c6a223441218))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/src/prove_test_swap.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |

  </details>
- fix(processor): refetch job status from database to ensure accurate processing; improve logging for job retrieval errors ([5262ae6](https://github.com/Machine-Labz/cloak/commit/5262ae6f31c0a699e12fafae0e415b99ace09497))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+22</span> / <span className="text-red-500">-5</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+22</span> / <span className="text-red-500">-5</span> |

  </details>
- refactor(swap): update fee calculation to include fixed fee in Merkle proof and withdrawal processes, enhance debug logging for better traceability ([6787eb6](https://github.com/Machine-Labz/cloak/commit/6787eb695f10f1eac3fa8ab7e818aca6a3d688fe))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+46</span> / <span className="text-red-500">-18</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/src/prove_test_swap.rs` | <span className="text-green-500">+46</span> / <span className="text-red-500">-18</span> |

  </details>
- refactor(fee): update fee calculation logic to include fixed fee for SOL withdrawals and clarify fee structure in comments ([c9a9df4](https://github.com/Machine-Labz/cloak/commit/c9a9df4fca08fbf1095307730dd1455e55a8e000))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+10</span> / <span className="text-red-500">-3</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+10</span> / <span className="text-red-500">-3</span> |

  </details>
- fix(Cargo): specify branch for sp1-solana dependency in Cargo.toml ([35bb6fa](https://github.com/Machine-Labz/cloak/commit/35bb6fa4c583c067299dfc38857666adc26b47b0))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- fix(miner): improve error handling and logging in relay demand check ([cc23d34](https://github.com/Machine-Labz/cloak/commit/cc23d34a20d3fed1747f5cd26a812612bcd45eee))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+16</span> / <span className="text-red-500">-4</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-4</span> |

  </details>

### 📚 Documentation

- docs(cloak-miner): update network references from devnet to testnet in usage examples and environment variable configurations ([f46bda8](https://github.com/Machine-Labz/cloak/commit/f46bda8d4331e65a9b3de52b7e88d23445a7f25c))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+26</span> / <span className="text-red-500">-29</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/packages/cloak-miner.md` | <span className="text-green-500">+26</span> / <span className="text-red-500">-29</span> |

  </details>
- delete(docs): remove outdated design documents and quick start guide ([b966137](https://github.com/Machine-Labz/cloak/commit/b9661377f00e5eb93ade55b75b829b2427a1f46c))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-639</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/cloak-miner/QUICKSTART.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-85</span> |
  | `packages/cloak-miner/TODO.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-554</span> |

  </details>
- chore(docker): enhance Docker build workflow with caching and optimization ([58f0500](https://github.com/Machine-Labz/cloak/commit/58f05007f2354ea8d66e9e2b1727f7aed7b8574e))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+98</span> / <span className="text-red-500">-122</span></summary>

  | File | Changes |
  |------|--------|
  | `.dockerignore` | <span className="text-green-500">+31</span> / <span className="text-red-500">-0</span> |
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+16</span> / <span className="text-red-500">-2</span> |
  | `deployment/compose.yml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+25</span> / <span className="text-red-500">-65</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+25</span> / <span className="text-red-500">-55</span> |

  </details>
- chore(docker): optimize Dockerfiles for indexer and relay services ([899c64c](https://github.com/Machine-Labz/cloak/commit/899c64c9393d826e28f45c9112f47b1d7466dc65))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+36</span> / <span className="text-red-500">-75</span></summary>

  | File | Changes |
  |------|--------|
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+19</span> / <span className="text-red-500">-43</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+16</span> / <span className="text-red-500">-31</span> |

  </details>
- refactor(docs): restructure homepage from MD to TSX and update links ([3de0a23](https://github.com/Machine-Labz/cloak/commit/3de0a235eff6b49613f1d076c00c6d207711e327))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+481</span> / <span className="text-red-500">-424</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+411</span> / <span className="text-red-500">-365</span> |
  | `docs/docusaurus.config.ts` | <span className="text-green-500">+3</span> / <span className="text-red-500">-6</span> |
  | `docs/sidebars.ts` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |
  | `docs/src/pages/index.mdx` | <span className="text-green-500">+0</span> / <span className="text-red-500">-52</span> |
  | `docs/src/pages/index.tsx` | <span className="text-green-500">+63</span> / <span className="text-red-500">-0</span> |

  </details>
- chore(docs): update documentation links and environment variable names ([5dfde7c](https://github.com/Machine-Labz/cloak/commit/5dfde7c01d7439f544ddd84e59fe534c271f7c03))
  <details>
  <summary>📂 <strong>8 files changed</strong>: <span className="text-green-500">+11</span> / <span className="text-red-500">-12</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/api/indexer.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/offchain/indexer.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/overview.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/web-app.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `docs/onchain/shield-pool.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/workflows/deposit.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>

### ♻️ Refactoring

- refactor(build): update ELF path resolution in build.rs and enhance guest ELF search logic in get_vkey_hash.rs for improved flexibility and correctness ([5ce3235](https://github.com/Machine-Labz/cloak/commit/5ce3235d36d948e70424d7c42c6136d8eeac8dad))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+21</span> / <span className="text-red-500">-8</span></summary>

  | File | Changes |
  |------|--------|
  | `...ages/zk-guest-sp1/.artifacts/zk-guest-sp1-guest` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/build.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-1</span> |
  | `...ages/zk-guest-sp1/host/src/bin/get_vkey_hash.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-7</span> |

  </details>
- refactor(logging): streamline JSON input handling and enhance swap parameter validation; improve error handling and logging for better debugging ([56c99ae](https://github.com/Machine-Labz/cloak/commit/56c99ae8634b80b50433e15d650016e9b9822b4c))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+131</span> / <span className="text-red-500">-58</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/README.md` | <span className="text-green-500">+80</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+30</span> / <span className="text-red-500">-53</span> |
  | `packages/zk-guest-sp1/host/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...ages/zk-guest-sp1/host/src/bin/get_vkey_hash.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-5</span> |

  </details>
- refactor(swap): update swap execution flow to use ExecuteSwapViaOrca for atomic on-chain swaps, enhancing efficiency and simplifying the process; adjust logging and improve dependency management in Cargo.toml ([678ca6d](https://github.com/Machine-Labz/cloak/commit/678ca6d4c6bd710def2f48f3c464b242531e60bf))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+310</span> / <span className="text-red-500">-135</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `services/relay/src/bin/complete_swap.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+211</span> / <span className="text-red-500">-96</span> |
  | `services/relay/src/solana/swap.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+72</span> / <span className="text-red-500">-32</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-1</span> |

  </details>
- refactor(logging): improve debug logging format and consistency across various modules, enhancing readability and traceability ([a0b208c](https://github.com/Machine-Labz/cloak/commit/a0b208cae39025d0c0a09b4ffaaac22881b3354d))
  <details>
  <summary>📂 <strong>10 files changed</strong>: <span className="text-green-500">+86</span> / <span className="text-red-500">-49</span></summary>

  | File | Changes |
  |------|--------|
  | `...cramble-registry/src/instructions/initialize.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `...cramble-registry/src/instructions/mine_claim.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `...-pool/src/instructions/execute_swap_via_orca.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/sp1_tee_client.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-4</span> |
  | `services/relay/src/planner.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `tooling/test/src/prove_test_spl.rs` | <span className="text-green-500">+51</span> / <span className="text-red-500">-22</span> |
  | `tooling/test/src/prove_test_swap.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-9</span> |

  </details>
- clean up imports and improve code formatting across multiple files for better readability ([311ed9d](https://github.com/Machine-Labz/cloak/commit/311ed9d7eed928631ad1c1a9758d48ae1a36276f))
  <details>
  <summary>📂 <strong>10 files changed</strong>: <span className="text-green-500">+444</span> / <span className="text-red-500">-287</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/src/solana.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/bin/check_swap_state.rs` | <span className="text-green-500">+43</span> / <span className="text-red-500">-27</span> |
  | `services/relay/src/bin/complete_swap.rs` | <span className="text-green-500">+70</span> / <span className="text-red-500">-48</span> |
  | `services/relay/src/solana/jupiter.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-8</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+61</span> / <span className="text-red-500">-25</span> |
  | `services/relay/src/solana/swap.rs` | <span className="text-green-500">+188</span> / <span className="text-red-500">-127</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-31</span> |
  | `services/relay/src/swap/jupiter.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-11</span> |

  </details>
- refactor(indexer): change root push to synchronous handling to prevent race conditions during deposit processing ([33102fe](https://github.com/Machine-Labz/cloak/commit/33102fefb5b3ab1de81e2663df0b510e9ac70812))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+71</span> / <span className="text-red-500">-97</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-12</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+10</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+47</span> / <span className="text-red-500">-80</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-4</span> |

  </details>
- refactor(indexer): streamline left sibling handling in Merkle tree insertion logic ([05a5f69](https://github.com/Machine-Labz/cloak/commit/05a5f6982827c7fb6a93770c9719a3ed102c96a8))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+8</span> / <span className="text-red-500">-36</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/merkle.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-36</span> |

  </details>
- refactor(prove_test): update DepositRequest structure and change API endpoints ([bff07c9](https://github.com/Machine-Labz/cloak/commit/bff07c93f32144bb519fb2e53ad02bf3f38d425d))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span> |

  </details>
- refactor(relay): remove local proving endpoint and associated module ([83002d1](https://github.com/Machine-Labz/cloak/commit/83002d192c6f5180142b7615ba77c5b1a7a3e69c))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+123</span> / <span className="text-red-500">-106</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/api/prove_local.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-94</span> |
  | `services/relay/src/db/repository.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+33</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+14</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+71</span> / <span className="text-red-500">-3</span> |

  </details>

---

## October 2025

### ✨ Features

- refactor(indexer): remove rate limiter implementation and related references ([2d4110a](https://github.com/Machine-Labz/cloak/commit/2d4110af49326ce8f749af5be8988528363614f3))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-172</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `services/indexer/src/server/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/server/rate_limiter.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-158</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-11</span> |

  </details>
- feat(nginx): enhance Nginx configuration to support additional CORS origin and improve header management ([e861a11](https://github.com/Machine-Labz/cloak/commit/e861a11b18d2e8dca37bcd4d5d4a3fc75018fd42))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+22</span> / <span className="text-red-500">-31</span></summary>

  | File | Changes |
  |------|--------|
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-28</span> |
  | `deployment/compose.yml` | <span className="text-green-500">+4</span> / <span className="text-red-500">-3</span> |
  | `deployment/nginx/nginx.conf` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(client): add mint_address field to configuration for Solana client tests ([e4da1d7](https://github.com/Machine-Labz/cloak/commit/e4da1d72de222580f82355dc581ae39a6f02d825))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(ci): add GitHub Actions workflow for building, pushing, and deploying Docker images ([9119120](https://github.com/Machine-Labz/cloak/commit/91191200421b565660edf198f4b986e193e6d219))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+71</span> / <span className="text-red-500">-3</span></summary>

  | File | Changes |
  |------|--------|
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+70</span> / <span className="text-red-500">-0</span> |
  | `deployment/compose.yml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-3</span> |

  </details>
- feat(deployment): add Docker Compose setup and database initialization script ([8a88b7b](https://github.com/Machine-Labz/cloak/commit/8a88b7b6d41d74d25d91399321f7829f4ca61a95))
  <details>
  <summary>📂 <strong>8 files changed</strong>: <span className="text-green-500">+751</span> / <span className="text-red-500">-239</span></summary>

  | File | Changes |
  |------|--------|
  | `deployment/README.md` | <span className="text-green-500">+74</span> / <span className="text-red-500">-0</span> |
  | `compose.yml => deployment/compose.yml` | <span className="text-green-500">+23</span> / <span className="text-red-500">-2</span> |
  | `deployment/init.sql` | <span className="text-green-500">+240</span> / <span className="text-red-500">-0</span> |
  | `deployment/nginx/Dockerfile` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `deployment/nginx/README.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `deployment/nginx/TESTING.md` | <span className="text-green-500">+132</span> / <span className="text-red-500">-0</span> |
  | `deployment/nginx/nginx.conf` | <span className="text-green-500">+171</span> / <span className="text-red-500">-0</span> |
  | `init.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-237</span> |

  </details>
- feat(tests): add comprehensive test for complete flow with multiple outputs ([2894260](https://github.com/Machine-Labz/cloak/commit/2894260200cdec35a66f35ecd8c0e67f96531a11))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+100</span> / <span className="text-red-500">-39</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-10</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+84</span> / <span className="text-red-500">-26</span> |

  </details>
- feat(docker): update Docker Compose configuration and add database initialization script ([9219c52](https://github.com/Machine-Labz/cloak/commit/9219c527293bfcd51af1506b857b00aa71d8343d))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+269</span> / <span className="text-red-500">-28</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+5</span> / <span className="text-red-500">-5</span> |
  | `compose.yml` | <span className="text-green-500">+27</span> / <span className="text-red-500">-23</span> |
  | `init.sql` | <span className="text-green-500">+237</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(spl): add associated token account derivation for SPL tokens in shield pool, enhancing support for multi-token transactions and improving account management ([7dd7e95](https://github.com/Machine-Labz/cloak/commit/7dd7e95b9aee2a242b2bc2bdaf08afae737801ff))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+211</span> / <span className="text-red-500">-10</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+74</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+135</span> / <span className="text-red-500">-10</span> |

  </details>
- remove obsolete files related to previous PDA and multi-token implementations, streamlining the codebase for improved maintainability ([d2be2ab](https://github.com/Machine-Labz/cloak/commit/d2be2ab7a36caae8250c08af0801f13b8a0b8063))
  <details>
  <summary>📂 <strong>11 files changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-1763</span></summary>

  | File | Changes |
  |------|--------|
  | `MULTI_TOKEN_SUPPORT_COMPLETE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-128</span> |
  | `PDA_FIX_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-93</span> |
  | `README_SPL_SUPPORT.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-250</span> |
  | `SPL_IMPLEMENTATION_COMPLETE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-290</span> |
  | `SPL_IMPLEMENTATION_GUIDE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-258</span> |
  | `SPL_IMPLEMENTATION_STATUS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-94</span> |
  | `SPL_TESTING_GUIDE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-330</span> |
  | `SPL_TOKEN_TEST_GUIDE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-212</span> |
  | `TEST_PROGRESS_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-69</span> |
  | `debug_pda.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-21</span> |
  | `test_pda.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-18</span> |

  </details>
- feat(backlog): add backlog status API and enhance database migrations ([d021b64](https://github.com/Machine-Labz/cloak/commit/d021b64f12fd86c83ea5c97511dd79e9a82219d0))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+102</span> / <span className="text-red-500">-12</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/src/api/backlog.rs` | <span className="text-green-500">+31</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/claim_manager.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+66</span> / <span className="text-red-500">-10</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(miner): implement demand-gated mining and enhance claim management ([44000ba](https://github.com/Machine-Labz/cloak/commit/44000bac023352a290692e889dc55c5fee2c4e07))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+184</span> / <span className="text-red-500">-39</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/README.md` | <span className="text-green-500">+31</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/examples/init_registry.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+78</span> / <span className="text-red-500">-7</span> |
  | `packages/cloak-miner/src/manager.rs` | <span className="text-green-500">+69</span> / <span className="text-red-500">-30</span> |

  </details>
- feat(pda): implement updated PDA derivation logic to support multi-token functionality, including mint address in derivation and account management improvements ([693f1ea](https://github.com/Machine-Labz/cloak/commit/693f1ea821f4a630425a7806e9f8003a8b0398c9))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+308</span> / <span className="text-red-500">-35</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+4</span> / <span className="text-red-500">-2</span> |
  | `PDA_FIX_SUMMARY.md` | <span className="text-green-500">+93</span> / <span className="text-red-500">-0</span> |
  | `TEST_PROGRESS_SUMMARY.md` | <span className="text-green-500">+69</span> / <span className="text-red-500">-0</span> |
  | `debug_pda.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `test_pda.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+87</span> / <span className="text-red-500">-33</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(multi-token): implement multi-token support in shield pool, updating PDA derivations to include mint addresses for enhanced isolation and backward compatibility with existing SOL pools ([0b208f0](https://github.com/Machine-Labz/cloak/commit/0b208f0c774ab691880ee1ae3013180e2dc6ac10))
  <details>
  <summary>📂 <strong>11 files changed</strong>: <span className="text-green-500">+414</span> / <span className="text-red-500">-252</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+16</span> / <span className="text-red-500">-0</span> |
  | `MULTI_TOKEN_SUPPORT_COMPLETE.md` | <span className="text-green-500">+93</span> / <span className="text-red-500">-221</span> |
  | `SPL_TOKEN_TEST_GUIDE.md` | <span className="text-green-500">+212</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/indexer/src/solana.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-4</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+30</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-8</span> |
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-7</span> |

  </details>
- feat(spl): implement complete SPL token support in shield pool, enabling deposits and withdrawals for both SOL and SPL tokens with full privacy guarantees ([eec57dc](https://github.com/Machine-Labz/cloak/commit/eec57dc7028785d555e9e1e72a702f5ce4aa3102))
  <details>
  <summary>📂 <strong>15 files changed</strong>: <span className="text-green-500">+2150</span> / <span className="text-red-500">-94</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+4</span> / <span className="text-red-500">-2</span> |
  | `MULTI_TOKEN_SUPPORT_COMPLETE.md` | <span className="text-green-500">+256</span> / <span className="text-red-500">-0</span> |
  | `README_SPL_SUPPORT.md` | <span className="text-green-500">+250</span> / <span className="text-red-500">-0</span> |
  | `SPL_IMPLEMENTATION_COMPLETE.md` | <span className="text-green-500">+290</span> / <span className="text-red-500">-0</span> |
  | `SPL_IMPLEMENTATION_GUIDE.md` | <span className="text-green-500">+258</span> / <span className="text-red-500">-0</span> |
  | `SPL_IMPLEMENTATION_STATUS.md` | <span className="text-green-500">+94</span> / <span className="text-red-500">-0</span> |
  | `SPL_TESTING_GUIDE.md` | <span className="text-green-500">+330</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+111</span> / <span className="text-red-500">-24</span> |
  | `...rams/shield-pool/src/instructions/initialize.rs` | <span className="text-green-500">+27</span> / <span className="text-red-500">-6</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+415</span> / <span className="text-red-500">-37</span> |
  | `programs/shield-pool/src/state/mod.rs` | <span className="text-green-500">+48</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/tests/admin_push_root.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-4</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-9</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-6</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+41</span> / <span className="text-red-500">-5</span> |

  </details>
- feat(dependencies): add Solana SDK and system interface to enhance program account management ([c9b9f98](https://github.com/Machine-Labz/cloak/commit/c9b9f98d3c8e6cd77d3d89d07254250db02ddfda))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+23</span> / <span className="text-red-500">-48</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-48</span> |

  </details>
- feat(indexer): add Solana client integration for on-chain root management and enhance configuration options ([ba34749](https://github.com/Machine-Labz/cloak/commit/ba347490bc608e8d18b3e36643a11e94c84c8988))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+141</span> / <span className="text-red-500">-2</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/Cargo.toml` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+28</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/lib.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/main.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/solana.rs` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |

  </details>

### 🐛 Bug Fixes

- fix ([79e69c1](https://github.com/Machine-Labz/cloak/commit/79e69c163bb253a7e32923fcbd1e089579e97d0c))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+11</span> / <span className="text-red-500">-8</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+2</span> / <span className="text-red-500">-4</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-4</span> |

  </details>
- migration script ([451cc99](https://github.com/Machine-Labz/cloak/commit/451cc9964903f89da3094aef99a8cc3d6ff0f6b9))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/migrations/001_init.sql` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |

  </details>
- fix(init_registry): update fee share basis points from 10% to 50% for improved revenue distribution ([54b738f](https://github.com/Machine-Labz/cloak/commit/54b738fd2a9f1d26a9ff4160b8180f034b7728fa))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/cloak-miner/examples/init_registry.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |

  </details>

### 📚 Documentation

- Remove obsolete README and testing documentation for Nginx and deployment configurations. Update Nginx configuration to include CORS support and enhance proxy settings for the prove endpoint, allowing for extended timeouts and streaming responses. ([8a381c5](https://github.com/Machine-Labz/cloak/commit/8a381c5647620ce4f79e17bfdf28c1da61531a98))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+54</span> / <span className="text-red-500">-296</span></summary>

  | File | Changes |
  |------|--------|
  | `deployment/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-74</span> |
  | `deployment/nginx/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-90</span> |
  | `deployment/nginx/TESTING.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-132</span> |
  | `deployment/nginx/nginx.conf` | <span className="text-green-500">+54</span> / <span className="text-red-500">-0</span> |

  </details>
- refactor(docker): simplify Dockerfile build commands for indexer and relay services ([581dcc0](https://github.com/Machine-Labz/cloak/commit/581dcc00942c0f97a48b30c41357865c07e8069c))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+21</span> / <span className="text-red-500">-4</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/Dockerfile` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/claim_manager.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-2</span> |

  </details>
- refactor(indexer): optimize Dockerfile and update configuration handling ([a1904e6](https://github.com/Machine-Labz/cloak/commit/a1904e609e42d891abbec380e108fa287520fbc2))
  <details>
  <summary>📂 <strong>14 files changed</strong>: <span className="text-green-500">+234</span> / <span className="text-red-500">-249</span></summary>

  | File | Changes |
  |------|--------|
  | `compose.yml` | <span className="text-green-500">+7</span> / <span className="text-red-500">-19</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-54</span> |
  | `services/indexer/src/database/connection.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-8</span> |
  | `services/indexer/src/logging.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-3</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+2</span> / <span className="text-red-500">-4</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/claim_manager.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-50</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+189</span> / <span className="text-red-500">-49</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-6</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-27</span> |
  | `services/relay/src/validation/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-23</span> |

  </details>
- feat(relay): enhance Dockerfile and refactor withdraw logic for multiple outputs ([eb1b4c5](https://github.com/Machine-Labz/cloak/commit/eb1b4c5d2b91676ffaa1b854108584ec4ca8a328))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+175</span> / <span className="text-red-500">-153</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/Dockerfile` | <span className="text-green-500">+27</span> / <span className="text-red-500">-2</span> |
  | `services/relay/migrations/001_init.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-55</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-3</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+38</span> / <span className="text-red-500">-11</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+105</span> / <span className="text-red-500">-81</span> |

  </details>
- refactor(indexer): optimize Dockerfile build process and update logging configuration ([cb3b65e](https://github.com/Machine-Labz/cloak/commit/cb3b65ed289b6ff2c64140e17b6624c202798ae7))
  <details>
  <summary>📂 <strong>8 files changed</strong>: <span className="text-green-500">+71</span> / <span className="text-red-500">-65</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/Dockerfile` | <span className="text-green-500">+1</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/init.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-21</span> |
  | `services/indexer/src/cloudwatch.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+36</span> / <span className="text-red-500">-11</span> |
  | `services/indexer/src/logging.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `...s/indexer/src/migrations/001_initial_schema.sql` | <span className="text-green-500">+22</span> / <span className="text-red-500">-19</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |

  </details>
- refactor(docs): update homepage configuration and enhance documentation structure ([5d8dfcc](https://github.com/Machine-Labz/cloak/commit/5d8dfcc887e9c982a4db5bac77231be087e5de7c))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+133</span> / <span className="text-red-500">-231</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+131</span> / <span className="text-red-500">-229</span> |
  | `docs/docusaurus.config.ts` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(docs): update documentation structure and content for clarity and consistency ([309595b](https://github.com/Machine-Labz/cloak/commit/309595b92137941327a9cfc7adb4f4b1527074cd))
  <details>
  <summary>📂 <strong>8 files changed</strong>: <span className="text-green-500">+34</span> / <span className="text-red-500">-83</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docusaurus.config.ts` | <span className="text-green-500">+11</span> / <span className="text-red-500">-5</span> |
  | `docs/offchain/indexer.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/offchain/overview.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/packages/zk-guest-sp1.md` | <span className="text-green-500">+10</span> / <span className="text-red-500">-10</span> |
  | `docs/src/pages/index.js` | <span className="text-green-500">+0</span> / <span className="text-red-500">-55</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-5</span> |
  | `docs/zk/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- update quickstart guide with detailed installation steps for required tools and troubleshooting tips ([71cf2ee](https://github.com/Machine-Labz/cloak/commit/71cf2eeab45f791e059ac7ac3a8692e968361c40))
  <details>
  <summary>📂 <strong>11 files changed</strong>: <span className="text-green-500">+437</span> / <span className="text-red-500">-640</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+85</span> / <span className="text-red-500">-283</span> |
  | `docs/offchain/overview.md` | <span className="text-green-500">+33</span> / <span className="text-red-500">-35</span> |
  | `docs/operations/runbook.md` | <span className="text-green-500">+59</span> / <span className="text-red-500">-18</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+77</span> / <span className="text-red-500">-22</span> |
  | `docs/overview/visual-flow.md` | <span className="text-green-500">+39</span> / <span className="text-red-500">-92</span> |
  | `docs/pow/overview.md` | <span className="text-green-500">+36</span> / <span className="text-red-500">-45</span> |
  | `docs/workflows/deposit.md` | <span className="text-green-500">+15</span> / <span className="text-red-500">-29</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+25</span> / <span className="text-red-500">-40</span> |
  | `docs/zk/README.md` | <span className="text-green-500">+31</span> / <span className="text-red-500">-24</span> |
  | `docs/zk/design.md` | <span className="text-green-500">+22</span> / <span className="text-red-500">-24</span> |
  | `docs/zk/testing.md` | <span className="text-green-500">+15</span> / <span className="text-red-500">-28</span> |

  </details>
- update quickstart guide with detailed installation steps for required tools and troubleshooting tips ([c3f3dcf](https://github.com/Machine-Labz/cloak/commit/c3f3dcf6b037d2cfa69f5d677aacb10a0ec8e8ef))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+57</span> / <span className="text-red-500">-8</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/overview/quickstart.md` | <span className="text-green-500">+57</span> / <span className="text-red-500">-8</span> |

  </details>
- enhance README and package documentation with detailed installation instructions for SP1 toolchain and troubleshooting steps ([d33bf38](https://github.com/Machine-Labz/cloak/commit/d33bf3841100d370734099d72225f11e5dfdc55b))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+166</span> / <span className="text-red-500">-11</span></summary>

  | File | Changes |
  |------|--------|
  | `README.md` | <span className="text-green-500">+65</span> / <span className="text-red-500">-6</span> |
  | `docs/packages/zk-guest-sp1.md` | <span className="text-green-500">+34</span> / <span className="text-red-500">-3</span> |
  | `packages/zk-guest-sp1/README.md` | <span className="text-green-500">+66</span> / <span className="text-red-500">-1</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>

### ♻️ Refactoring

- refactor(test): clean up account creation logic and improve code readability ([4b223b3](https://github.com/Machine-Labz/cloak/commit/4b223b3dccafef50a81d228661fee487a234ca70))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+10</span> / <span className="text-red-500">-64</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/src/prove_test_multiple_outputs.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-55</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-9</span> |

  </details>
- refactor(solana): derive roots_ring PDA and update instruction creation ([abf887f](https://github.com/Machine-Labz/cloak/commit/abf887fa76ade60d8ee9751fdfe2dfd8e7fbf8d2))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+16</span> / <span className="text-red-500">-5</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/solana.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-5</span> |

  </details>
- feat(instructions): refactor withdraw logic to support multiple recipients and improve data parsing ([315d971](https://github.com/Machine-Labz/cloak/commit/315d9714c593f3ed603029f7a269d17dcbda012f))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+314</span> / <span className="text-red-500">-258</span></summary>

  | File | Changes |
  |------|--------|
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+314</span> / <span className="text-red-500">-258</span> |

  </details>
- refactor(instructions): clean up deposit and withdraw instructions by removing unused code ([3bfe576](https://github.com/Machine-Labz/cloak/commit/3bfe576c908231d2ae7c842714b6ae9ad7ac7dc1))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+1</span> / <span className="text-red-500">-11</span></summary>

  | File | Changes |
  |------|--------|
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- refactor(instructions): remove unnecessary logging messages and improve error handling in consume, initialize, mine, and reveal claim instructions ([34e6cfd](https://github.com/Machine-Labz/cloak/commit/34e6cfd11c99bedb39423ae45f14f20b9fcaf10f))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+59</span> / <span className="text-red-500">-116</span></summary>

  | File | Changes |
  |------|--------|
  | `...mble-registry/src/instructions/consume_claim.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-11</span> |
  | `...cramble-registry/src/instructions/initialize.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-17</span> |
  | `...cramble-registry/src/instructions/mine_claim.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-18</span> |
  | `...amble-registry/src/instructions/reveal_claim.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...rams/shield-pool/src/instructions/initialize.rs` | <span className="text-green-500">+44</span> / <span className="text-red-500">-50</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-15</span> |

  </details>

### 🔧 Maintenance

- chore(web): update subproject commit reference to e8b33d2-dirty ([122a171](https://github.com/Machine-Labz/cloak/commit/122a17134efa7eaffe3c9aaadcac1567ed16d410))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- feat(ci): update GitHub Actions workflow for building and deploying services ([ec5e0fd](https://github.com/Machine-Labz/cloak/commit/ec5e0fd3f01a75e1fa88b208ccd79a6e0aa93b6a))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+320</span> / <span className="text-red-500">-271</span></summary>

  | File | Changes |
  |------|--------|
  | `.github/workflows/build-and-push-images.yml` | <span className="text-green-500">+36</span> / <span className="text-red-500">-19</span> |
  | `deployment/init.sql` | <span className="text-green-500">+243</span> / <span className="text-red-500">-237</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+39</span> / <span className="text-red-500">-13</span> |

  </details>

---


## Contributing

To keep this changelog useful:

1. Write clear, descriptive commit messages
2. Use conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `refactor:` for code refactoring
   - `chore:` for maintenance tasks

The changelog is automatically regenerated before each documentation build.
