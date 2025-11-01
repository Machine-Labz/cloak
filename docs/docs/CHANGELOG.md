---
title: Changelog
description: Recent updates and changes to the Cloak project
---

# Changelog

This changelog is automatically generated from Git commit history.

View the complete history on [GitHub](https://github.com/Machine-Labz/cloak/commits/master).

---

## November 2025

### ✨ Features

- feat(database): add beta interest registrations table for mainnet beta waitlist ([50b1384](https://github.com/Machine-Labz/cloak/commit/50b13840b21e1991143003b8e4d5e327f04348c2))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+18</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `deployment/init.sql` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>

### 📚 Documentation

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

### 🔧 Maintenance

- chore(web): update subproject commit reference to 5d56e8a ([ad1dfc9](https://github.com/Machine-Labz/cloak/commit/ad1dfc963570e40d76ad74e0338cd939fdcfce32))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

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
- feat(prover): implement rate limiting and deprecate server-side proof generation endpoint ([26f67a6](https://github.com/Machine-Labz/cloak/commit/26f67a648a40be73b04b0c311a546df0d51caec5))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+372</span> / <span className="text-red-500">-18</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+327</span> / <span className="text-red-500">-18</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+32</span> / <span className="text-red-500">-0</span> |

  </details>
- update documentation structure and add essential configuration files ([b7f58d0](https://github.com/Machine-Labz/cloak/commit/b7f58d0c2ecee43a950001859a6ec06ca7dbdc08))
  <details>
  <summary>📂 <strong>70 files changed</strong>: <span className="text-green-500">+17776</span> / <span className="text-red-500">-3710</span></summary>

  | File | Changes |
  |------|--------|
  | `docs-site/src/css/custom.css` | <span className="text-green-500">+0</span> / <span className="text-red-500">-195</span> |
  | `docs/.gitignore` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `docs/COMPLETE_FLOW_STATUS.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-12</span> |
  | `docs/DIAGRAMS_INDEX.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-236</span> |
  | `docs/POW_ARCHITECTURE_FIXED.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-184</span> |
  | `docs/POW_CORRECT_ARCHITECTURE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-425</span> |
  | `docs/POW_DOC_UPDATES_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-374</span> |
  | `docs/POW_INTEGRATION_COMPLETE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-317</span> |
  | `docs/POW_QUICK_REFERENCE.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-4</span> |
  | `docs/POW_REFACTOR_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-414</span> |
  | `docs/POW_WILDCARD_IMPLEMENTATION.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-438</span> |
  | `docs-site/README.md => docs/README-docusaurus.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-4</span> |
  | `docs/README.md` | <span className="text-green-500">+14</span> / <span className="text-red-500">-14</span> |
  | `docs/VERCEL_DEPLOYMENT.md` | <span className="text-green-500">+87</span> / <span className="text-red-500">-0</span> |
  | `docs/api/indexer.md` | <span className="text-green-500">+405</span> / <span className="text-red-500">-55</span> |
  | `docs/api/relay.md` | <span className="text-green-500">+458</span> / <span className="text-red-500">-47</span> |
  | `docs/api/validator-agent.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-5</span> |
  | `{docs-site => docs}/babel.config.js` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/docusaurus.config.ts` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/nonzk/frontend.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-14</span> |
  | `docs/nonzk/indexer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-17</span> |
  | `docs/nonzk/relayer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-13</span> |
  | `docs/offchain/indexer.md` | <span className="text-green-500">+391</span> / <span className="text-red-500">-41</span> |
  | `docs/offchain/overview.md` | <span className="text-green-500">+346</span> / <span className="text-red-500">-0</span> |
  | `docs/offchain/web-app.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `docs/onchain/program-integration.md` | <span className="text-green-500">+255</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/scramble-registry.md` | <span className="text-green-500">+378</span> / <span className="text-red-500">-35</span> |
  | `docs/onchain/shield-pool-upstream.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-62</span> |
  | `docs/onchain/shield-pool.md` | <span className="text-green-500">+216</span> / <span className="text-red-500">-60</span> |
  | `docs/operations/runbook.md` | <span className="text-green-500">+905</span> / <span className="text-red-500">-137</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+10</span> / <span className="text-red-500">-10</span> |
  | `docs/overview/quickstart.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span> |
  | `docs/overview/status.md` | <span className="text-green-500">+381</span> / <span className="text-red-500">-11</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+649</span> / <span className="text-red-500">-57</span> |
  | `docs/overview/tech-stack.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/overview/visual-flow.md` | <span className="text-green-500">+429</span> / <span className="text-red-500">-21</span> |
  | `{docs-site => docs}/package.json` | <span className="text-green-500">+4</span> / <span className="text-red-500">-2</span> |
  | `docs/packages/cloak-miner.md` | <span className="text-green-500">+652</span> / <span className="text-red-500">-36</span> |
  | `docs/packages/cloak-proof-extract.md` | <span className="text-green-500">+905</span> / <span className="text-red-500">-27</span> |
  | `docs/packages/overview.md` | <span className="text-green-500">+510</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/tooling-test.md` | <span className="text-green-500">+767</span> / <span className="text-red-500">-22</span> |
  | `docs/packages/vkey-generator.md` | <span className="text-green-500">+475</span> / <span className="text-red-500">-19</span> |
  | `docs/pow/overview.md` | <span className="text-green-500">+633</span> / <span className="text-red-500">-28</span> |
  | `docs/quickstart.md` | <span className="text-green-500">+96</span> / <span className="text-red-500">-0</span> |
  | `docs/roadmap.md` | <span className="text-green-500">+18</span> / <span className="text-red-500">-13</span> |
  | `docs/scripts/generate-changelog.js` | <span className="text-green-500">+370</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/sidebars.ts` | <span className="text-green-500">+3</span> / <span className="text-red-500">-12</span> |
  | `docs/src/css/custom.css` | <span className="text-green-500">+275</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/src/pages/index.mdx` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `{docs-site => docs}/static/img/cloaklogo.svg` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/static/img/favicon.svg` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/static/img/logo.svg` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/tsconfig.base.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/tsconfig.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/vercel.json` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `docs/workflows/deposit.md` | <span className="text-green-500">+720</span> / <span className="text-red-500">-34</span> |
  | `docs/workflows/pow-withdraw.md` | <span className="text-green-500">+994</span> / <span className="text-red-500">-37</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+971</span> / <span className="text-red-500">-71</span> |
  | `{docs-site => docs}/yarn.lock` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/README.md` | <span className="text-green-500">+401</span> / <span className="text-red-500">-22</span> |
  | `docs/zk/api-contracts.md` | <span className="text-green-500">+338</span> / <span className="text-red-500">-24</span> |
  | `docs/zk/circuit-withdraw.md` | <span className="text-green-500">+518</span> / <span className="text-red-500">-26</span> |
  | `docs/zk/design.md` | <span className="text-green-500">+412</span> / <span className="text-red-500">-13</span> |
  | `docs/zk/encoding.md` | <span className="text-green-500">+600</span> / <span className="text-red-500">-31</span> |
  | `docs/zk/merkle.md` | <span className="text-green-500">+585</span> / <span className="text-red-500">-19</span> |
  | `docs/zk/prover-sp1.md` | <span className="text-green-500">+697</span> / <span className="text-red-500">-22</span> |
  | `docs/zk/testing.md` | <span className="text-green-500">+899</span> / <span className="text-red-500">-17</span> |
  | `docs/zk/threat-model.md` | <span className="text-green-500">+499</span> / <span className="text-red-500">-16</span> |
  | `programs/scramble-registry/README.md` | <span className="text-green-500">+427</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/README.md` | <span className="text-green-500">+21</span> / <span className="text-red-500">-4</span> |

  </details>
- add .gitmodules file for web submodule integration ([6850e9f](https://github.com/Machine-Labz/cloak/commit/6850e9f67b90caa78ea9aee44efb7be404b3cda3))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitmodules` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |

  </details>
- enhance PoW mining workflow and remove HTTP server from miner ([60a0574](https://github.com/Machine-Labz/cloak/commit/60a05745df2302944926c6cd344efc9f0687112e))
  <details>
  <summary>📂 <strong>11 files changed</strong>: <span className="text-green-500">+633</span> / <span className="text-red-500">-198</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/cloak-miner/examples/init_registry.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `packages/cloak-miner/src/engine.rs` | <span className="text-green-500">+38</span> / <span className="text-red-500">-10</span> |
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+145</span> / <span className="text-red-500">-31</span> |
  | `packages/cloak-miner/src/manager.rs` | <span className="text-green-500">+68</span> / <span className="text-red-500">-33</span> |
  | `...mble-registry/src/instructions/consume_claim.rs` | <span className="text-green-500">+10</span> / <span className="text-red-500">-4</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-14</span> |
  | `services/relay/src/claim_manager.rs` | <span className="text-green-500">+254</span> / <span className="text-red-500">-80</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-23</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+67</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(logging): enhance CloudWatch logging integration and add connectivity verification ([4bca9f0](https://github.com/Machine-Labz/cloak/commit/4bca9f0dbc6e3e1e07d9d1af2ab395f4abab09cf))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+326</span> / <span className="text-red-500">-26</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/cloudwatch.rs` | <span className="text-green-500">+147</span> / <span className="text-red-500">-7</span> |
  | `services/relay/src/cloudwatch.rs` | <span className="text-green-500">+141</span> / <span className="text-red-500">-7</span> |
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-11</span> |

  </details>
- add diagrams and SP1 TEE integration documentation ([bae8fd4](https://github.com/Machine-Labz/cloak/commit/bae8fd4bb642113f88d0291ddb1c047e1ceb59d2))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+1096</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `DIAGRAMS.md` | <span className="text-green-500">+669</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/SP1_TEE_INTEGRATION.md` | <span className="text-green-500">+231</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/TEE_FIX_VERIFICATION.md` | <span className="text-green-500">+196</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(relay): add Solana client enhancements ([1fb16f8](https://github.com/Machine-Labz/cloak/commit/1fb16f886b175ccee6cccd6c51ecc3a9f50dc2c8))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+16</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(config): add environment variable validation ([146cd4f](https://github.com/Machine-Labz/cloak/commit/146cd4fae74f9c77c2380756f6cbb8fc4e079f24))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+126</span> / <span className="text-red-500">-32</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/config.rs` | <span className="text-green-500">+66</span> / <span className="text-red-500">-28</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-4</span> |

  </details>
- feat(logging): add CloudWatch logging modules ([60bcd31](https://github.com/Machine-Labz/cloak/commit/60bcd31f1d21c811049b3c785f388781ef16cbf4))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+334</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/cloudwatch.rs` | <span className="text-green-500">+171</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/cloudwatch.rs` | <span className="text-green-500">+163</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(logging): add AWS CloudWatch dependencies ([9454f14](https://github.com/Machine-Labz/cloak/commit/9454f14274ed698126f95f4d22fcd22c4b666d27))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+74</span> / <span className="text-red-500">-66</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+63</span> / <span className="text-red-500">-62</span> |
  | `services/indexer/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+6</span> / <span className="text-red-500">-4</span> |

  </details>
- tooling: add testnet harness binaries ([b5cc93e](https://github.com/Machine-Labz/cloak/commit/b5cc93e08f69f1063c10a496031783fd083dc22f))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+485</span> / <span className="text-red-500">-29</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+8</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/bin/call_initialize.rs` | <span className="text-green-500">+124</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/bin/check_claims.rs` | <span className="text-green-500">+165</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/bin/init_testnet.rs` | <span className="text-green-500">+110</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+67</span> / <span className="text-red-500">-22</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-7</span> |

  </details>
- shield-pool: replace upstream program and add initializer ([476537e](https://github.com/Machine-Labz/cloak/commit/476537e907ddd079a04f657d3ffe69f34eb0985e))
  <details>
  <summary>📂 <strong>29 files changed</strong>: <span className="text-green-500">+569</span> / <span className="text-red-500">-1301</span></summary>

  | File | Changes |
  |------|--------|
  | `...mble-registry/src/instructions/consume_claim.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-3</span> |
  | `programs/scramble-registry/src/state/mod.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool-upstream/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-54</span> |
  | `programs/shield-pool-upstream/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-111</span> |
  | `programs/shield-pool-upstream/src/constants.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-33</span> |
  | `programs/shield-pool-upstream/src/error.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-68</span> |
  | `programs/shield-pool-upstream/src/groth16/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-126</span> |
  | `...ol-upstream/src/instructions/admin_push_root.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-84</span> |
  | `...hield-pool-upstream/src/instructions/deposit.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-93</span> |
  | `...ms/shield-pool-upstream/src/instructions/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-38</span> |
  | `...ield-pool-upstream/src/instructions/withdraw.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-199</span> |
  | `programs/shield-pool-upstream/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-49</span> |
  | `programs/shield-pool-upstream/src/state/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-25</span> |
  | `...ield-pool-upstream/src/state/nullifier_shard.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-88</span> |
  | `...ms/shield-pool-upstream/src/state/roots_ring.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-82</span> |
  | `...ield-pool-upstream/src/tests/admin_push_root.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool-upstream/src/tests/deposit.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool-upstream/src/tests/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-9</span> |
  | `...rams/shield-pool-upstream/src/tests/withdraw.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+10</span> / <span className="text-red-500">-21</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+10</span> / <span className="text-red-500">-7</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-2</span> |
  | `...rams/shield-pool/src/instructions/initialize.rs` | <span className="text-green-500">+114</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+377</span> / <span className="text-red-500">-130</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+14</span> / <span className="text-red-500">-11</span> |
  | `programs/shield-pool/src/state/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-36</span> |

  </details>
- implement changelog generation and enhance documentation styling ([ff52fe9](https://github.com/Machine-Labz/cloak/commit/ff52fe95f3120a1c6f6b9ed7d2a8936e7946da93))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+118</span> / <span className="text-red-500">-6</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/scripts/generate-changelog.js` | <span className="text-green-500">+68</span> / <span className="text-red-500">-6</span> |
  | `docs/src/css/custom.css` | <span className="text-green-500">+50</span> / <span className="text-red-500">-0</span> |

  </details>
- enhance documentation with changelog generation and updates ([9adc544](https://github.com/Machine-Labz/cloak/commit/9adc5446bffa8da862922ed6a3163a6edec4f3fd))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+542</span> / <span className="text-red-500">-92</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+96</span> / <span className="text-red-500">-0</span> |
  | `docs/package.json` | <span className="text-green-500">+4</span> / <span className="text-red-500">-2</span> |
  | `docs/scripts/generate-changelog.js` | <span className="text-green-500">+308</span> / <span className="text-red-500">-0</span> |
  | `docs/sidebars.ts` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `docs/src/css/custom.css` | <span className="text-green-500">+119</span> / <span className="text-red-500">-89</span> |
  | `docs/src/pages/index.mdx` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/vercel.json` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |

  </details>
- add docusaurus documentation site ([1ed3972](https://github.com/Machine-Labz/cloak/commit/1ed39723e845333c35aa9fefe09c0d61dd85e59a))
  <details>
  <summary>📂 <strong>52 files changed</strong>: <span className="text-green-500">+5996</span> / <span className="text-red-500">-3</span></summary>

  | File | Changes |
  |------|--------|
  | `docs-site/README.md` | <span className="text-green-500">+55</span> / <span className="text-red-500">-0</span> |
  | `docs-site/babel.config.js` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `docs-site/docusaurus.config.ts` | <span className="text-green-500">+76</span> / <span className="text-red-500">-0</span> |
  | `docs-site/package.json` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `docs-site/sidebars.ts` | <span className="text-green-500">+121</span> / <span className="text-red-500">-0</span> |
  | `docs-site/src/css/custom.css` | <span className="text-green-500">+195</span> / <span className="text-red-500">-0</span> |
  | `docs-site/src/pages/index.mdx` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs-site/static/img/cloaklogo.svg` | <span className="text-green-500">+12</span> / <span className="text-red-500">-0</span> |
  | `docs-site/static/img/favicon.svg` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `docs-site/static/img/logo.svg` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `docs-site/tsconfig.base.json` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `docs-site/tsconfig.json` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `docs/CHANGELOG.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `docs/DIAGRAMS_INDEX.md` | <span className="text-green-500">+236</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_ARCHITECTURE_FIXED.md` | <span className="text-green-500">+184</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_CORRECT_ARCHITECTURE.md` | <span className="text-green-500">+425</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_DOC_UPDATES_SUMMARY.md` | <span className="text-green-500">+374</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_INTEGRATION_COMPLETE.md` | <span className="text-green-500">+317</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_INTEGRATION_GUIDE.md` | <span className="text-green-500">+453</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_QUICK_REFERENCE.md` | <span className="text-green-500">+333</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_REFACTOR_SUMMARY.md` | <span className="text-green-500">+414</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_WILDCARD_IMPLEMENTATION.md` | <span className="text-green-500">+438</span> / <span className="text-red-500">-0</span> |
  | `docs/api/indexer.md` | <span className="text-green-500">+88</span> / <span className="text-red-500">-0</span> |
  | `docs/api/relay.md` | <span className="text-green-500">+78</span> / <span className="text-red-500">-0</span> |
  | `docs/api/validator-agent.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `docs/glossary.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `docs/nonzk/frontend.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/indexer.md` | <span className="text-green-500">+96</span> / <span className="text-red-500">-0</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+102</span> / <span className="text-red-500">-0</span> |
  | `docs/offchain/web-app.md` | <span className="text-green-500">+73</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/scramble-registry.md` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/shield-pool-upstream.md` | <span className="text-green-500">+62</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/shield-pool.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `docs/operations/metrics-guide.md` | <span className="text-green-500">+419</span> / <span className="text-red-500">-0</span> |
  | `docs/operations/runbook.md` | <span className="text-green-500">+146</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+47</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/quickstart.md` | <span className="text-green-500">+118</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/status.md` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+76</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/tech-stack.md` | <span className="text-green-500">+40</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/visual-flow.md` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/cloak-miner.md` | <span className="text-green-500">+67</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/cloak-proof-extract.md` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/tooling-test.md` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/vkey-generator.md` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/zk-guest-sp1.md` | <span className="text-green-500">+72</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/zk-verifier-program.md` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `docs/pow/overview.md` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `docs/roadmap.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `docs/workflows/deposit.md` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `docs/workflows/pow-withdraw.md` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+109</span> / <span className="text-red-500">-0</span> |

  </details>
- add docusaurus documentation site ([78cbe05](https://github.com/Machine-Labz/cloak/commit/78cbe05ad6887df464067287c9f75cfbf89b61ab))
  <details>
  <summary>📂 <strong>53 files changed</strong>: <span className="text-green-500">+6261</span> / <span className="text-red-500">-35</span></summary>

  | File | Changes |
  |------|--------|
  | `docs-site/README.md` | <span className="text-green-500">+55</span> / <span className="text-red-500">-0</span> |
  | `docs-site/babel.config.js` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `docs-site/docusaurus.config.ts` | <span className="text-green-500">+76</span> / <span className="text-red-500">-0</span> |
  | `docs-site/package.json` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `docs-site/sidebars.ts` | <span className="text-green-500">+121</span> / <span className="text-red-500">-0</span> |
  | `docs-site/src/css/custom.css` | <span className="text-green-500">+195</span> / <span className="text-red-500">-0</span> |
  | `docs-site/src/pages/index.mdx` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs-site/static/img/cloaklogo.svg` | <span className="text-green-500">+12</span> / <span className="text-red-500">-0</span> |
  | `docs-site/static/img/favicon.svg` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `docs-site/static/img/logo.svg` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `docs-site/tsconfig.base.json` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `docs-site/tsconfig.json` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `docs/CHANGELOG.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `docs/DIAGRAMS_INDEX.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-3</span> |
  | `docs/POW_ARCHITECTURE_FIXED.md` | <span className="text-green-500">+184</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_CORRECT_ARCHITECTURE.md` | <span className="text-green-500">+425</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_DOC_UPDATES_SUMMARY.md` | <span className="text-green-500">+374</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_INTEGRATION_COMPLETE.md` | <span className="text-green-500">+317</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_INTEGRATION_GUIDE.md` | <span className="text-green-500">+453</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_QUICK_REFERENCE.md` | <span className="text-green-500">+333</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_REFACTOR_SUMMARY.md` | <span className="text-green-500">+414</span> / <span className="text-red-500">-0</span> |
  | `docs/POW_WILDCARD_IMPLEMENTATION.md` | <span className="text-green-500">+438</span> / <span className="text-red-500">-0</span> |
  | `docs/api/indexer.md` | <span className="text-green-500">+88</span> / <span className="text-red-500">-0</span> |
  | `docs/api/relay.md` | <span className="text-green-500">+78</span> / <span className="text-red-500">-0</span> |
  | `docs/api/validator-agent.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `docs/glossary.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `docs/nonzk/frontend.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/offchain/indexer.md` | <span className="text-green-500">+96</span> / <span className="text-red-500">-0</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+102</span> / <span className="text-red-500">-0</span> |
  | `docs/offchain/web-app.md` | <span className="text-green-500">+73</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/scramble-registry.md` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/shield-pool-upstream.md` | <span className="text-green-500">+62</span> / <span className="text-red-500">-0</span> |
  | `docs/onchain/shield-pool.md` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `docs/operations/metrics-guide.md` | <span className="text-green-500">+419</span> / <span className="text-red-500">-0</span> |
  | `docs/operations/runbook.md` | <span className="text-green-500">+146</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+47</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/quickstart.md` | <span className="text-green-500">+118</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/status.md` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+76</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/tech-stack.md` | <span className="text-green-500">+40</span> / <span className="text-red-500">-0</span> |
  | `docs/overview/visual-flow.md` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/cloak-miner.md` | <span className="text-green-500">+67</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/cloak-proof-extract.md` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/tooling-test.md` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/vkey-generator.md` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/zk-guest-sp1.md` | <span className="text-green-500">+72</span> / <span className="text-red-500">-0</span> |
  | `docs/packages/zk-verifier-program.md` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `docs/pow-scrambler-gate.md` | <span className="text-green-500">+499</span> / <span className="text-red-500">-29</span> |
  | `docs/pow/overview.md` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `docs/roadmap.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `docs/workflows/deposit.md` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `docs/workflows/pow-withdraw.md` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+109</span> / <span className="text-red-500">-0</span> |

  </details>
- complete remaining package updates and security improvements ([0d35f1f](https://github.com/Machine-Labz/cloak/commit/0d35f1f03bb93a8d16e80cb144df4fe719b60da7))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+405</span> / <span className="text-red-500">-269</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `Cargo.lock` | <span className="text-green-500">+354</span> / <span className="text-red-500">-255</span> |
  | `packages/cloak-proof-extract/src/lib.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-6</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-8</span> |

  </details>
- complete relay service implementation updates ([07104c0](https://github.com/Machine-Labz/cloak/commit/07104c0548745cc83cf659453c47cc72fb9391ec))
  <details>
  <summary>📂 <strong>22 files changed</strong>: <span className="text-green-500">+354</span> / <span className="text-red-500">-1905</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/examples/batch_commitment.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/examples/build_instructions.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-3</span> |
  | `services/relay/examples/fetch_mining_params.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-18</span> |
  | `services/relay/examples/test_mining.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-7</span> |
  | `services/relay/examples/verify_mining.rs` | <span className="text-green-500">+25</span> / <span className="text-red-500">-17</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/api/validator_agent.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-25</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-12</span> |
  | `services/relay/src/db/repository.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/miner/batch.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-163</span> |
  | `services/relay/src/miner/engine.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-380</span> |
  | `services/relay/src/miner/instructions.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-461</span> |
  | `services/relay/src/miner/manager.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-318</span> |
  | `services/relay/src/miner/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-20</span> |
  | `services/relay/src/miner/rpc.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-369</span> |
  | `services/relay/src/planner.rs` | <span className="text-green-500">+64</span> / <span className="text-red-500">-18</span> |
  | `services/relay/src/solana/submit.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-17</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-5</span> |
  | `services/relay/tests/miner_integration.rs` | <span className="text-green-500">+95</span> / <span className="text-red-500">-59</span> |

  </details>
- update scramble-registry Solana program ([6c00412](https://github.com/Machine-Labz/cloak/commit/6c004120186d62cdabaa7e73905ee9a34cb2cb92))
  <details>
  <summary>📂 <strong>21 files changed</strong>: <span className="text-green-500">+860</span> / <span className="text-red-500">-691</span></summary>

  | File | Changes |
  |------|--------|
  | `programs/scramble-registry/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/constants.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-4</span> |
  | `programs/scramble-registry/src/error.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `...mble-registry/src/instructions/consume_claim.rs` | <span className="text-green-500">+25</span> / <span className="text-red-500">-29</span> |
  | `...cramble-registry/src/instructions/initialize.rs` | <span className="text-green-500">+127</span> / <span className="text-red-500">-89</span> |
  | `...cramble-registry/src/instructions/mine_claim.rs` | <span className="text-green-500">+111</span> / <span className="text-red-500">-62</span> |
  | `programs/scramble-registry/src/instructions/mod.rs` | <span className="text-green-500">+25</span> / <span className="text-red-500">-4</span> |
  | `...amble-registry/src/instructions/reveal_claim.rs` | <span className="text-green-500">+14</span> / <span className="text-red-500">-22</span> |
  | `programs/scramble-registry/src/lib.rs` | <span className="text-green-500">+43</span> / <span className="text-red-500">-93</span> |
  | `programs/scramble-registry/src/state/claim.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-184</span> |
  | `programs/scramble-registry/src/state/miner.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-63</span> |
  | `programs/scramble-registry/src/state/mod.rs` | <span className="text-green-500">+443</span> / <span className="text-red-500">-6</span> |
  | `programs/scramble-registry/src/state/registry.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-116</span> |
  | `...ms/scramble-registry/src/tests/consume_claim.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/tests/initialize.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/tests/mine_claim.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/tests/mod.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `...ams/scramble-registry/src/tests/reveal_claim.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/utils/blake3.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `programs/scramble-registry/src/utils/difficulty.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-16</span> |
  | `programs/scramble-registry/src/utils/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- update cloak-miner package implementation ([51c05e3](https://github.com/Machine-Labz/cloak/commit/51c05e3441264292fe4182f4d60b54ab2f3ba416))
  <details>
  <summary>📂 <strong>11 files changed</strong>: <span className="text-green-500">+180</span> / <span className="text-red-500">-88</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/cloak-miner/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/examples/init_registry.rs` | <span className="text-green-500">+28</span> / <span className="text-red-500">-22</span> |
  | `packages/cloak-miner/examples/register_miner.rs` | <span className="text-green-500">+75</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/batch.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-4</span> |
  | `packages/cloak-miner/src/constants.rs` | <span className="text-green-500">+15</span> / <span className="text-red-500">-10</span> |
  | `packages/cloak-miner/src/engine.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-14</span> |
  | `packages/cloak-miner/src/instructions.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-9</span> |
  | `packages/cloak-miner/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+21</span> / <span className="text-red-500">-8</span> |
  | `packages/cloak-miner/src/manager.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-11</span> |
  | `packages/cloak-miner/src/rpc.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-8</span> |

  </details>
- add missing Cargo.toml features for relay package ([f12de9c](https://github.com/Machine-Labz/cloak/commit/f12de9ca024eeda1f265f8bb002f3fabcf9cc7b2))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |

  </details>
- test(tooling): add localnet init/proving helpers and update tests ([41d208b](https://github.com/Machine-Labz/cloak/commit/41d208bae3ebbde49a5ec968edd30d41fb196c48))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+330</span> / <span className="text-red-500">-10</span></summary>

  | File | Changes |
  |------|--------|
  | `justfile` | <span className="text-green-500">+19</span> / <span className="text-red-500">-5</span> |
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |
  | `tooling/test/src/bin/derive_pdas.rs` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/bin/init_localnet.rs` | <span className="text-green-500">+226</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/localnet_test.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-1</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-2</span> |
  | `tooling/test/src/testnet_test.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-1</span> |

  </details>
- feat(relay): add worker processor and queue; update schema and docs ([82ea262](https://github.com/Machine-Labz/cloak/commit/82ea2623a8d572ce047bed86528c23abeeba1d04))
  <details>
  <summary>📂 <strong>24 files changed</strong>: <span className="text-green-500">+1695</span> / <span className="text-red-500">-720</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/.cargo/config.toml` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `services/relay/ARCHITECTURE.md` | <span className="text-green-500">+313</span> / <span className="text-red-500">-0</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `services/relay/Dockerfile` | <span className="text-green-500">+34</span> / <span className="text-red-500">-41</span> |
  | `services/relay/FIXES_APPLIED.md` | <span className="text-green-500">+142</span> / <span className="text-red-500">-0</span> |
  | `services/relay/LOCAL_DEVELOPMENT.md` | <span className="text-green-500">+223</span> / <span className="text-red-500">-0</span> |
  | `services/relay/RUNNING_INSTRUCTIONS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-385</span> |
  | `services/relay/TODO.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-70</span> |
  | `services/relay/TROUBLESHOOTING.md` | <span className="text-green-500">+257</span> / <span className="text-red-500">-0</span> |
  | `services/relay/WORKER_IMPLEMENTATION.md` | <span className="text-green-500">+195</span> / <span className="text-red-500">-0</span> |
  | `services/relay/cleanup-redis.sh` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `services/relay/docker-compose.yml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-38</span> |
  | `services/relay/env.docker` | <span className="text-green-500">+23</span> / <span className="text-red-500">-0</span> |
  | `services/relay/env.example` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `services/relay/migrations/001_init.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/relay/prepare-sqlx.sh` | <span className="text-green-500">+38</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/db/models.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/db/repository.rs` | <span className="text-green-500">+81</span> / <span className="text-red-500">-147</span> |
  | `services/relay/src/error.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-6</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/queue/redis_queue.rs` | <span className="text-green-500">+27</span> / <span className="text-red-500">-27</span> |
  | `services/relay/src/worker/mod.rs` | <span className="text-green-500">+74</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/worker/processor.rs` | <span className="text-green-500">+194</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(indexer): add SP1 prover endpoint and config/migrations cleanup ([64f0ae6](https://github.com/Machine-Labz/cloak/commit/64f0ae6a48041f5f11f6f0be6e4d0b645e0cd4cc))
  <details>
  <summary>📂 <strong>14 files changed</strong>: <span className="text-green-500">+328</span> / <span className="text-red-500">-308</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/Dockerfile` | <span className="text-green-500">+80</span> / <span className="text-red-500">-47</span> |
  | `services/indexer/env.docker` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/env.example` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/init.sql` | <span className="text-green-500">+14</span> / <span className="text-red-500">-8</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+15</span> / <span className="text-red-500">-10</span> |
  | `services/indexer/src/database/connection.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-3</span> |
  | `services/indexer/src/database/migrations.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/logging.rs` | <span className="text-green-500">+14</span> / <span className="text-red-500">-5</span> |
  | `...s/indexer/src/migrations/001_initial_schema.sql` | <span className="text-green-500">+33</span> / <span className="text-red-500">-78</span> |
  | `...xer/src/migrations/001_initial_schema_fixed.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-107</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-19</span> |
  | `services/indexer/src/server/rate_limiter.rs` | <span className="text-green-500">+40</span> / <span className="text-red-500">-26</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-3</span> |

  </details>
- chore(infra): add root compose and dockerignore; remove legacy docker-compose.yml ([b1708d4](https://github.com/Machine-Labz/cloak/commit/b1708d47cdcf3fbcea97ab69b2a1e565d7d7aeed))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+146</span> / <span className="text-red-500">-116</span></summary>

  | File | Changes |
  |------|--------|
  | `.dockerignore` | <span className="text-green-500">+45</span> / <span className="text-red-500">-0</span> |
  | `compose.yml` | <span className="text-green-500">+101</span> / <span className="text-red-500">-0</span> |
  | `docker-compose.yml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-116</span> |

  </details>
- enhance SP1 proof generation and update configurations ([4af5dbb](https://github.com/Machine-Labz/cloak/commit/4af5dbb841c498d64a511a450a89f9f9378c9f0f))
  <details>
  <summary>📂 <strong>31 files changed</strong>: <span className="text-green-500">+876</span> / <span className="text-red-500">-83</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+3</span> / <span className="text-red-500">-1</span> |
  | `docker-compose.yml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `packages/sp1-wasm-prover/Cargo.toml` | <span className="text-green-500">+30</span> / <span className="text-red-500">-0</span> |
  | `packages/sp1-wasm-prover/build.sh` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |
  | `packages/sp1-wasm-prover/examples/web/index.html` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...s/sp1-wasm-prover/examples/web/proof-example.js` | <span className="text-green-500">+182</span> / <span className="text-red-500">-0</span> |
  | `packages/sp1-wasm-prover/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-7</span> |
  | `packages/zk-guest-sp1/host/src/lib.rs` | <span className="text-green-500">+86</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/src/main.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-5</span> |
  | `packages/zk-guest-sp1/out/public.json` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-5</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-9</span> |
  | `programs/shield-pool/src/tests/admin_push_root.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-3</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-8</span> |
  | `services/indexer/.env.example` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/Cargo.toml` | <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/config.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/src/database/connection.rs` | <span className="text-green-500">+46</span> / <span className="text-red-500">-12</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+49</span> / <span className="text-red-500">-18</span> |
  | `services/indexer/src/server/mod.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+145</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/rate_limiter.rs` | <span className="text-green-500">+144</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/start.sh` | <span className="text-green-500">+51</span> / <span className="text-red-500">-0</span> |
  | `services/relay/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/config.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span> |

  </details>

### 🐛 Bug Fixes

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
- resolve relay package compilation errors ([42c0e65](https://github.com/Machine-Labz/cloak/commit/42c0e6525a2a25612d90565b74665b49fdbf3339))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+201</span> / <span className="text-red-500">-180</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/examples/end_to_end_mining.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-14</span> |
  | `services/relay/src/api/prove_local.rs` | <span className="text-green-500">+13</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-19</span> |
  | `services/relay/src/planner/orchestrator.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-16</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+78</span> / <span className="text-red-500">-28</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+52</span> / <span className="text-red-500">-93</span> |

  </details>
- resolve zk-guest-sp1 golden test failures ([e4b5d37](https://github.com/Machine-Labz/cloak/commit/e4b5d376f5c8eacff157dc68eefbf2b3fa8205af))
  <details>
  <summary>📂 <strong>57 files changed</strong>: <span className="text-green-500">+8311</span> / <span className="text-red-500">-192</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `docs/pow-architecture.md` | <span className="text-green-500">+505</span> / <span className="text-red-500">-0</span> |
  | `docs/pow-scrambler-gate.md` | <span className="text-green-500">+678</span> / <span className="text-red-500">-0</span> |
  | `miner.json` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/Cargo.toml` | <span className="text-green-500">+53</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/QUICKSTART.md` | <span className="text-green-500">+85</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/README.md` | <span className="text-green-500">+310</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/examples/init_registry.rs` | <span className="text-green-500">+134</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/batch.rs` | <span className="text-green-500">+163</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/constants.rs` | <span className="text-green-500">+118</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/engine.rs` | <span className="text-green-500">+380</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/instructions.rs` | <span className="text-green-500">+461</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/lib.rs` | <span className="text-green-500">+25</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/main.rs` | <span className="text-green-500">+298</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/manager.rs` | <span className="text-green-500">+318</span> / <span className="text-red-500">-0</span> |
  | `packages/cloak-miner/src/rpc.rs` | <span className="text-green-500">+369</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/src/lib.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-3</span> |
  | `packages/zk-guest-sp1/tests/golden.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-8</span> |
  | `programs/scramble-registry/Cargo.toml` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/init-localnet.sh` | <span className="text-green-500">+30</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/constants.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/error.rs` | <span className="text-green-500">+62</span> / <span className="text-red-500">-0</span> |
  | `...mble-registry/src/instructions/consume_claim.rs` | <span className="text-green-500">+95</span> / <span className="text-red-500">-0</span> |
  | `...cramble-registry/src/instructions/initialize.rs` | <span className="text-green-500">+167</span> / <span className="text-red-500">-0</span> |
  | `...cramble-registry/src/instructions/mine_claim.rs` | <span className="text-green-500">+229</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/instructions/mod.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...amble-registry/src/instructions/reveal_claim.rs` | <span className="text-green-500">+65</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/lib.rs` | <span className="text-green-500">+112</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/state/claim.rs` | <span className="text-green-500">+184</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/state/miner.rs` | <span className="text-green-500">+63</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/state/mod.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/state/registry.rs` | <span className="text-green-500">+116</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/utils/blake3.rs` | <span className="text-green-500">+209</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/utils/difficulty.rs` | <span className="text-green-500">+78</span> / <span className="text-red-500">-0</span> |
  | `programs/scramble-registry/src/utils/mod.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+41</span> / <span className="text-red-500">-4</span> |
  | `programs/shield-pool/src/state/nullifier_shard.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-88</span> |
  | `programs/shield-pool/src/state/roots_ring.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-82</span> |
  | `services/relay/examples/batch_commitment.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/build_instructions.rs` | <span className="text-green-500">+85</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/end_to_end_mining.rs` | <span className="text-green-500">+112</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/fetch_mining_params.rs` | <span className="text-green-500">+71</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/test_mining.rs` | <span className="text-green-500">+86</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/verify_mining.rs` | <span className="text-green-500">+54</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/lib.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/miner/batch.rs` | <span className="text-green-500">+163</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/miner/engine.rs` | <span className="text-green-500">+380</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/miner/instructions.rs` | <span className="text-green-500">+461</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/miner/manager.rs` | <span className="text-green-500">+318</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/miner/mod.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/miner/rpc.rs` | <span className="text-green-500">+369</span> / <span className="text-red-500">-0</span> |
  | `services/relay/tests/miner_integration.rs` | <span className="text-green-500">+643</span> / <span className="text-red-500">-0</span> |
  | `services/web` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |

  </details>
- Merge branch 'feat/hotfix-proof-gen' of https://github.com/Cloak-Labz/cloak into feat/hotfix-proof-gen ([9d0c041](https://github.com/Machine-Labz/cloak/commit/9d0c041f59284b22ab5323f6274bf486596c9cd9))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/out/public.json` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

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
- remove DIAGRAMS.md file containing architectural and transaction flow documentation ([04f5be7](https://github.com/Machine-Labz/cloak/commit/04f5be7594ebb6ec3d67b978c849deedc05a5d92))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-669</span></summary>

  | File | Changes |
  |------|--------|
  | `DIAGRAMS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-669</span> |

  </details>
- refactor(tests): streamline test suite by removing unused binaries and enhancing documentation ([965827e](https://github.com/Machine-Labz/cloak/commit/965827ea5f266ffae4c49f2312fa9ce9bed4f6f9))
  <details>
  <summary>📂 <strong>9 files changed</strong>: <span className="text-green-500">+112</span> / <span className="text-red-500">-1024</span></summary>

  | File | Changes |
  |------|--------|
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-16</span> |
  | `tooling/test/src/bin/call_initialize.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-124</span> |
  | `tooling/test/src/bin/check_claims.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-165</span> |
  | `tooling/test/src/bin/derive_pdas.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-24</span> |
  | `tooling/test/src/bin/init_localnet.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-226</span> |
  | `tooling/test/src/bin/init_testnet.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-110</span> |
  | `tooling/test/src/lib.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/prove_test.rs` | <span className="text-green-500">+43</span> / <span className="text-red-500">-173</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+67</span> / <span className="text-red-500">-186</span> |

  </details>
- update documentation to reflect transition from Redis to database for job queuing and service configuration ([2a439b3](https://github.com/Machine-Labz/cloak/commit/2a439b356c99f8cc6d771b4d884f988325d936fe))
  <details>
  <summary>📂 <strong>9 files changed</strong>: <span className="text-green-500">+105</span> / <span className="text-red-500">-134</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+28</span> / <span className="text-red-500">-0</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+24</span> / <span className="text-red-500">-50</span> |
  | `docs/operations/runbook.md` | <span className="text-green-500">+11</span> / <span className="text-red-500">-50</span> |
  | `docs/overview/quickstart.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+30</span> / <span className="text-red-500">-22</span> |
  | `docs/overview/tech-stack.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/overview/visual-flow.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/zk/testing.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |

  </details>
- restructure runbook and update service configuration details ([7b17f90](https://github.com/Machine-Labz/cloak/commit/7b17f90193b45f8fb01619a6ca616a0d23af3be8))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+27</span> / <span className="text-red-500">-15</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/operations/runbook.md` | <span className="text-green-500">+24</span> / <span className="text-red-500">-12</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/overview/quickstart.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- update documentation structure and improve relay service details ([1050aaf](https://github.com/Machine-Labz/cloak/commit/1050aaf81246f92efed6f224e49414de6e088314))
  <details>
  <summary>📂 <strong>8 files changed</strong>: <span className="text-green-500">+125</span> / <span className="text-red-500">-122</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/api/relay.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-6</span> |
  | `docs/docusaurus.config.ts` | <span className="text-green-500">+18</span> / <span className="text-red-500">-2</span> |
  | `docs/offchain/overview.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-17</span> |
  | `docs/offchain/relay.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-9</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+35</span> / <span className="text-red-500">-35</span> |
  | `docs/packages/zk-guest-sp1.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/src/pages/index.js` | <span className="text-green-500">+55</span> / <span className="text-red-500">-0</span> |
  | `docs/src/pages/index.mdx` | <span className="text-green-500">+0</span> / <span className="text-red-500">-52</span> |

  </details>
- update README files ([59faa58](https://github.com/Machine-Labz/cloak/commit/59faa5882e0909bd87cf711b767fd5224d1cb0da))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+11</span> / <span className="text-red-500">-31</span></summary>

  | File | Changes |
  |------|--------|
  | `README.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/README.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-31</span> |

  </details>
- update documentation across all sections ([15a2d14](https://github.com/Machine-Labz/cloak/commit/15a2d147fae50d9e0aca310db5b0460f56e6d5f8))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+21</span> / <span className="text-red-500">-29</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/offchain/relay.md` | <span className="text-green-500">+8</span> / <span className="text-red-500">-11</span> |
  | `docs/operations/runbook.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-4</span> |
  | `docs/overview/introduction.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/overview/quickstart.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span> |
  | `docs/overview/system-architecture.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-3</span> |
  | `docs/overview/tech-stack.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/workflows/withdraw.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |

  </details>
- remove outdated documentation files ([13be501](https://github.com/Machine-Labz/cloak/commit/13be5017bf39e98438200482c1e6d414250f2807))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-308</span></summary>

  | File | Changes |
  |------|--------|
  | `RUNBOOK.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-142</span> |
  | `VALIDATOR-WORK-NOTES.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-166</span> |

  </details>
- chore(docker): remove Redis service from compose ([6df48a6](https://github.com/Machine-Labz/cloak/commit/6df48a62a776456c235c70439732f58f5d4ce126))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-25</span></summary>

  | File | Changes |
  |------|--------|
  | `compose.yml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-25</span> |

  </details>
- sync architecture references from master ([9ade3df](https://github.com/Machine-Labz/cloak/commit/9ade3df85fed2905bf426bb8489c4876cfd97f90))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-2250</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/QUICK_REFERENCE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-325</span> |
  | `docs/TECH_STACK.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-741</span> |
  | `docs/VISUAL_FLOW.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-385</span> |
  | `docs/api/validator-agent.yaml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-294</span> |
  | `docs/pow-architecture.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-505</span> |

  </details>
- update documentation structure and remove obsolete files ([23dfa5f](https://github.com/Machine-Labz/cloak/commit/23dfa5f34322facbdf798856756b0f5786bb379a))
  <details>
  <summary>📂 <strong>27 files changed</strong>: <span className="text-green-500">+74</span> / <span className="text-red-500">-2704</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/docs/CHANGELOG.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-126</span> |
  | `docs/docs/COMPLETE_FLOW_STATUS.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-12</span> |
  | `docs/docs/DIAGRAMS_INDEX.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-236</span> |
  | `docs/docs/POW_ARCHITECTURE_FIXED.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-184</span> |
  | `docs/docs/POW_CORRECT_ARCHITECTURE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-425</span> |
  | `docs/docs/POW_DOC_UPDATES_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-374</span> |
  | `docs/docs/POW_INTEGRATION_COMPLETE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-317</span> |
  | `docs/docs/POW_QUICK_REFERENCE.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-4</span> |
  | `docs/docs/POW_REFACTOR_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-414</span> |
  | `docs/docs/POW_WILDCARD_IMPLEMENTATION.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-438</span> |
  | `docs/docs/README.md` | <span className="text-green-500">+14</span> / <span className="text-red-500">-14</span> |
  | `docs/docs/api/validator-agent.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/docs/nonzk/frontend.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-14</span> |
  | `docs/docs/nonzk/indexer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-17</span> |
  | `docs/docs/nonzk/relayer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-13</span> |
  | `docs/docs/offchain/indexer.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `docs/docs/offchain/web-app.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `docs/docs/onchain/shield-pool-upstream.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-62</span> |
  | `docs/docs/overview/introduction.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-9</span> |
  | `docs/docs/overview/tech-stack.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/docs/overview/visual-flow.md` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `docs/docs/packages/cloak-miner.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/docs/pow/overview.md` | <span className="text-green-500">+11</span> / <span className="text-red-500">-10</span> |
  | `docs/docs/roadmap.md` | <span className="text-green-500">+18</span> / <span className="text-red-500">-13</span> |
  | `docs/docs/workflows/pow-withdraw.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/docs/zk/encoding.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/sidebars.ts` | <span className="text-green-500">+2</span> / <span className="text-red-500">-11</span> |

  </details>
- restructure Docusaurus documentation site and update .gitignore ([cac3670](https://github.com/Machine-Labz/cloak/commit/cac367056911b527416b020546da2e4d4c405d82))
  <details>
  <summary>📂 <strong>73 files changed</strong>: <span className="text-green-500">+228</span> / <span className="text-red-500">-16</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `DOCUSAURUS_RESTRUCTURE.md` | <span className="text-green-500">+105</span> / <span className="text-red-500">-0</span> |
  | `README.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-9</span> |
  | `docs/.gitignore` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `docs-site/README.md => docs/README-docusaurus.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-4</span> |
  | `docs/VERCEL_DEPLOYMENT.md` | <span className="text-green-500">+87</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/babel.config.js` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/CHANGELOG.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/COMPLETE_FLOW_STATUS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/DIAGRAMS_INDEX.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_ARCHITECTURE_FIXED.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_CORRECT_ARCHITECTURE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_DOC_UPDATES_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_INTEGRATION_COMPLETE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_INTEGRATION_GUIDE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_QUICK_REFERENCE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_REFACTOR_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/POW_WILDCARD_IMPLEMENTATION.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/api/indexer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/api/relay.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/api/validator-agent.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/glossary.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/nonzk/frontend.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/nonzk/indexer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/nonzk/relayer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/offchain/indexer.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/offchain/relay.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/offchain/web-app.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/onchain/scramble-registry.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/onchain/shield-pool-upstream.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/onchain/shield-pool.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/operations/metrics-guide.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/operations/runbook.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/overview/introduction.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/overview/quickstart.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/overview/status.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/overview/system-architecture.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/overview/tech-stack.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/overview/visual-flow.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/packages/cloak-miner.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/packages/cloak-proof-extract.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/packages/tooling-test.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/packages/vkey-generator.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/packages/zk-guest-sp1.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/packages/zk-verifier-program.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/pow-scrambler-gate.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/pow/overview.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/roadmap.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/workflows/deposit.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/workflows/pow-withdraw.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `docs/{ => docs}/workflows/withdraw.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
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
  | `{docs-site => docs}/docusaurus.config.ts` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `{docs-site => docs}/package.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/sidebars.ts` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `{docs-site => docs}/src/css/custom.css` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/src/pages/index.mdx` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/static/img/cloaklogo.svg` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/static/img/favicon.svg` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/static/img/logo.svg` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/tsconfig.base.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/tsconfig.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `{docs-site => docs}/yarn.lock` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |

  </details>
- remove sp1-wasm-prover references; document zk-guest-sp1/backend proving ([b83810d](https://github.com/Machine-Labz/cloak/commit/b83810dac6e63ddf94931566992562bb29a03652))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+52</span> / <span className="text-red-500">-44</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/ARCHITECTURE_DIAGRAM.md` | <span className="text-green-500">+47</span> / <span className="text-red-500">-1</span> |
  | `docs/TECH_STACK.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-10</span> |
  | `docs/zk/wasm-port.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-33</span> |

  </details>
- update README and architecture docs ([6516a33](https://github.com/Machine-Labz/cloak/commit/6516a33b5feee069c487f29857e086c3d11414fa))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+2708</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `README.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `docs/ARCHITECTURE_DIAGRAM.md` | <span className="text-green-500">+973</span> / <span className="text-red-500">-0</span> |
  | `docs/DIAGRAMS_INDEX.md` | <span className="text-green-500">+237</span> / <span className="text-red-500">-0</span> |
  | `docs/QUICK_REFERENCE.md` | <span className="text-green-500">+325</span> / <span className="text-red-500">-0</span> |
  | `docs/TECH_STACK.md` | <span className="text-green-500">+746</span> / <span className="text-red-500">-0</span> |
  | `docs/VISUAL_FLOW.md` | <span className="text-green-500">+385</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/wasm-port.md` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |

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
- refactor(prover): clean up code formatting and improve logging in proof generation endpoint ([8b0c4d1](https://github.com/Machine-Labz/cloak/commit/8b0c4d14370428b9310260bca077310efa49717f))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+54</span> / <span className="text-red-500">-36</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+31</span> / <span className="text-red-500">-24</span> |
  | `services/relay/src/claim_manager.rs` | <span className="text-green-500">+23</span> / <span className="text-red-500">-12</span> |

  </details>
- refactor(indexer): deprecate server-side proof generation endpoint and remove rate limiting ([e33ba2f](https://github.com/Machine-Labz/cloak/commit/e33ba2f1b66706ec3b1ea0fbcea432a57f8256e4))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+20</span> / <span className="text-red-500">-331</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-295</span> |
  | `services/indexer/src/server/routes.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-32</span> |

  </details>
- refactor(indexer): update prover handler and SP1 TEE client ([834c6e9](https://github.com/Machine-Labz/cloak/commit/834c6e99fe36f34d6deb5b320c64f8cb66853115))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+4</span> / <span className="text-red-500">-13</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/src/server/prover_handler.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-8</span> |
  | `services/indexer/src/sp1_tee_client.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-5</span> |

  </details>
- remove unused TestCircuitInputs struct from golden.rs ([ffc2c50](https://github.com/Machine-Labz/cloak/commit/ffc2c509eb8ce11aabb0c02e693adb54e6a8b8e6))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-7</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/tests/golden.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-7</span> |

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
