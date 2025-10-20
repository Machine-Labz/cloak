---
title: Changelog
description: Recent updates and changes to the Cloak project
---

# Changelog

This changelog is automatically generated from Git commit history.

View the complete history on [GitHub](https://github.com/Machine-Labz/cloak/commits/master).

---

## October 2025

### ✨ Features

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

### 🐛 Bug Fixes

- fixes ([6c0933f](https://github.com/Machine-Labz/cloak/commit/6c0933fae35a8f5c44d0d4b0d59c614fae90119b))
  <details>
  <summary>📂 <strong>26 files changed</strong>: <span className="text-green-500">+440</span> / <span className="text-red-500">-296</span></summary>

  | File | Changes |
  |------|--------|
  | `docker-compose.yml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-4</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-9</span> |
  | `programs/shield-pool/src/tests/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+48</span> / <span className="text-red-500">-46</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+8</span> / <span className="text-red-500">-6</span> |
  | `services/indexer/src/artifacts.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-9</span> |
  | `services/indexer/src/bin/migrate.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-8</span> |
  | `services/indexer/src/database/connection.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-10</span> |
  | `services/indexer/src/database/migrations.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/src/database/storage.rs` | <span className="text-green-500">+42</span> / <span className="text-red-500">-31</span> |
  | `services/indexer/src/logging.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/merkle.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-3</span> |
  | `services/indexer/src/server/final_handlers.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-39</span> |
  | `services/indexer/src/server/middleware.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/src/server/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-32</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-8</span> |
  | `services/relay/src/db/models.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/db/repository.rs` | <span className="text-green-500">+28</span> / <span className="text-red-500">-6</span> |
  | `services/relay/src/error.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-7</span> |
  | `services/relay/src/queue/mod.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/queue/redis_queue.rs` | <span className="text-green-500">+98</span> / <span className="text-red-500">-50</span> |

  </details>
- docker divergencies ([157b26b](https://github.com/Machine-Labz/cloak/commit/157b26b18792346a64c3b6de80072aea8487ae08))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+16</span> / <span className="text-red-500">-16</span></summary>

  | File | Changes |
  |------|--------|
  | `docker-compose.yml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `justfile` | <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+5</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/justfile` | <span className="text-green-500">+5</span> / <span className="text-red-500">-5</span> |

  </details>
- workspace ([48fc7b1](https://github.com/Machine-Labz/cloak/commit/48fc7b1317d0efaeb678b1bc6527f705a8d524fd))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+386</span> / <span className="text-red-500">-74</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+375</span> / <span className="text-red-500">-66</span> |
  | `services/indexer/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+8</span> / <span className="text-red-500">-5</span> |

  </details>

### 📚 Documentation

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

### 🔧 Maintenance

- remove pnpm lock ([d788628](https://github.com/Machine-Labz/cloak/commit/d788628d8b8319474db50d0b56c72680b0a83ea2))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/pnpm-workspace.yaml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span> |

  </details>
- remove obsolete files and clean up project structure ([660bb5b](https://github.com/Machine-Labz/cloak/commit/660bb5b9a233cc0dabfdb453d269414168011499))
  <details>
  <summary>📂 <strong>143 files changed</strong>: <span className="text-green-500">+1594</span> / <span className="text-red-500">-9558</span></summary>

  | File | Changes |
  |------|--------|
  | `.context/pinocchio-programs-reference/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-4</span> |
  | `.context/pinocchio-programs-reference/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-20</span> |
  | `.context/pinocchio-programs-reference/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-146</span> |
  | `.../pinocchio-programs-reference/escrow/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-22</span> |
  | `...chio-programs-reference/escrow/src/constants.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `...nocchio-programs-reference/escrow/src/errors.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-11</span> |
  | `...grams-reference/escrow/src/instructions/make.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-27</span> |
  | `...ograms-reference/escrow/src/instructions/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-25</span> |
  | `...ams-reference/escrow/src/instructions/refund.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span> |
  | `...grams-reference/escrow/src/instructions/take.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-113</span> |
  | `.../pinocchio-programs-reference/escrow/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-40</span> |
  | `...grams-reference/escrow/src/state/contributor.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-24</span> |
  | `...o-programs-reference/escrow/src/state/escrow.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-47</span> |
  | `...ograms-reference/escrow/src/state/fundraiser.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-47</span> |
  | `...chio-programs-reference/escrow/src/state/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `...rograms-reference/escrow/src/tests/make_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-73</span> |
  | `...chio-programs-reference/escrow/src/tests/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-54</span> |
  | `...rograms-reference/escrow/src/tests/take_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-222</span> |
  | `...occhio-programs-reference/fundraiser/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-22</span> |
  | `...-programs-reference/fundraiser/src/constants.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `...hio-programs-reference/fundraiser/src/errors.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-11</span> |
  | `...-reference/fundraiser/src/instructions/check.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-28</span> |
  | `...rence/fundraiser/src/instructions/contribute.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-27</span> |
  | `...rence/fundraiser/src/instructions/initialize.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-31</span> |
  | `...ms-reference/fundraiser/src/instructions/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-28</span> |
  | `...reference/fundraiser/src/instructions/refund.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-32</span> |
  | `...occhio-programs-reference/fundraiser/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-50</span> |
  | `...s-reference/fundraiser/src/state/contributor.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-24</span> |
  | `...ms-reference/fundraiser/src/state/fundraiser.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-47</span> |
  | `...-programs-reference/fundraiser/src/state/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-4</span> |
  | `...ms-reference/fundraiser/src/tests/check_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-199</span> |
  | `...ference/fundraiser/src/tests/contribute_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-195</span> |
  | `...eference/fundraiser/src/tests/initalize_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-114</span> |
  | `...-programs-reference/fundraiser/src/tests/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-60</span> |
  | `...s-reference/fundraiser/src/tests/refund_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-238</span> |
  | `...occhio-programs-reference/sbpf_close/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-10</span> |
  | `...occhio-programs-reference/sbpf_close/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-13</span> |
  | `...nocchio-programs-reference/sbpf_close/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-3</span> |
  | `...occhio-programs-reference/sbpf_close/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-49</span> |
  | `...eference/sbpf_close/src/sbpf_close/sbpf_close.s` | <span className="text-green-500">+0</span> / <span className="text-red-500">-45</span> |
  | `...t/pinocchio-programs-reference/vault/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-2</span> |
  | `...t/pinocchio-programs-reference/vault/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-23</span> |
  | `...pinocchio-programs-reference/vault/src/based.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-84</span> |
  | `...t/pinocchio-programs-reference/vault/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-19</span> |
  | `...inocchio-programs-reference/vault/src/native.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-43</span> |
  | `...cchio-programs-reference/vault/src/optimized.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-39</span> |
  | `...pinocchio-programs-reference/vault/src/tests.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-48</span> |
  | `.context/sp1-solana/.github/workflows/pr.yml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-44</span> |
  | `.context/sp1-solana/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-12</span> |
  | `.context/sp1-solana/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-52</span> |
  | `.context/sp1-solana/LICENSE-MIT` | <span className="text-green-500">+0</span> / <span className="text-red-500">-21</span> |
  | `.context/sp1-solana/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-163</span> |
  | `...sp1-solana/example-sp1-wasm-verifier/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-14</span> |
  | `...sp1-solana/example-sp1-wasm-verifier/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-9</span> |
  | `...p1-solana/example-sp1-wasm-verifier/LICENSE-MIT` | <span className="text-green-500">+0</span> / <span className="text-red-500">-19</span> |
  | `.../sp1-solana/example-sp1-wasm-verifier/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-113</span> |
  | `...m-verifier/example/fibonacci-program/Cargo.lock` | <span className="text-green-500">+0</span> / <span className="text-red-500">-468</span> |
  | `...m-verifier/example/fibonacci-program/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-8</span> |
  | `...-verifier/example/fibonacci-program/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-37</span> |
  | `...ifier/example/json/fibonacci_groth16_proof.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `...erifier/example/json/fibonacci_plonk_proof.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `...ple-sp1-wasm-verifier/example/script/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-15</span> |
  | `...ample-sp1-wasm-verifier/example/script/build.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-4</span> |
  | `...le-sp1-wasm-verifier/example/script/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-86</span> |
  | `...-sp1-wasm-verifier/example/wasm_example/main.js` | <span className="text-green-500">+0</span> / <span className="text-red-500">-56</span> |
  | `...wasm-verifier/example/wasm_example/package.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-13</span> |
  | `...sm-verifier/example/wasm_example/pnpm-lock.yaml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-9</span> |
  | `...a/example-sp1-wasm-verifier/verifier/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-20</span> |
  | `...a/example-sp1-wasm-verifier/verifier/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-20</span> |
  | `.context/sp1-solana/example/pinocchio/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-20</span> |
  | `.context/sp1-solana/example/pinocchio/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-120</span> |
  | `.context/sp1-solana/example/program/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-21</span> |
  | `.context/sp1-solana/example/program/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-58</span> |
  | `.context/sp1-solana/example/script/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `.context/sp1-solana/example/script/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-19</span> |
  | `.context/sp1-solana/example/script/build.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-11</span> |
  | `.context/sp1-solana/example/script/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-203</span> |
  | `.context/sp1-solana/example/sp1-program/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-8</span> |
  | `...ple/sp1-program/elf/riscv32im-succinct-zkvm-elf` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...text/sp1-solana/example/sp1-program/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-37</span> |
  | `...olana/solana-project/APPLICATION_FLOW_DESIGN.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-204</span> |
  | `...xt/sp1-solana/solana-project/CLEANUP_SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-66</span> |
  | `...solana-project/COMPLETE_APPLICATION_ANALYSIS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-251</span> |
  | `.context/sp1-solana/solana-project/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-33</span> |
  | `...text/sp1-solana/solana-project/DEMO_COMPLETE.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-171</span> |
  | `.context/sp1-solana/solana-project/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-238</span> |
  | `.context/sp1-solana/solana-project/SUMMARY.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-89</span> |
  | `...olana/solana-project/ZK_STUDY_SESSION_STATUS.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-216</span> |
  | `...a-project/pinocchio-withdrawal-proof/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-21</span> |
  | `...na-project/pinocchio-withdrawal-proof/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-126</span> |
  | `...a-project/pinocchio-withdrawal-proof/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-226</span> |
  | `...cchio-withdrawal-proof/test_withdrawal_proof.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-174</span> |
  | `...na-project/withdrawal-json-generator/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-19</span> |
  | `...oject/withdrawal-json-generator/src/bin/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-141</span> |
  | `...ithdrawal-json/withdrawal_compressed_proof.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-13</span> |
  | `...na/solana-project/withdrawal-proof/.env.example` | <span className="text-green-500">+0</span> / <span className="text-red-500">-9</span> |
  | `...drawal-proof/.github/workflows/foundry-test.yml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-43</span> |
  | `...ct/withdrawal-proof/.github/workflows/prove.yml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-46</span> |
  | `...lana/solana-project/withdrawal-proof/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-19</span> |
  | `...-project/withdrawal-proof/.vscode/settings.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-29</span> |
  | `...lana/solana-project/withdrawal-proof/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-9</span> |
  | `...ana/solana-project/withdrawal-proof/LICENSE-MIT` | <span className="text-green-500">+0</span> / <span className="text-red-500">-21</span> |
  | `...olana/solana-project/withdrawal-proof/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-89</span> |
  | `...olana-project/withdrawal-proof/get_vkey_hash.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-23</span> |
  | `...ana-project/withdrawal-proof/program/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-9</span> |
  | `...na-project/withdrawal-proof/program/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-193</span> |
  | `.../solana-project/withdrawal-proof/rust-toolchain` | <span className="text-green-500">+0</span> / <span className="text-red-500">-3</span> |
  | `...lana-project/withdrawal-proof/script/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-26</span> |
  | `...solana-project/withdrawal-proof/script/build.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span> |
  | `...project/withdrawal-proof/script/src/bin/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-205</span> |
  | `...ct/withdrawal-proof/script/src/bin/vkey_hash.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-29</span> |
  | `...project/withdrawal-proof/script/verify_proof.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-34</span> |
  | `...ana-project/withdrawal-wasm-verifier/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-17</span> |
  | `...ana-project/withdrawal-wasm-verifier/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-81</span> |
  | `...ana-project/withdrawal-web-interface/index.html` | <span className="text-green-500">+0</span> / <span className="text-red-500">-470</span> |
  | `...a-project/withdrawal-web-interface/package.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-20</span> |
  | `...lana-project/withdrawal-web-interface/server.js` | <span className="text-green-500">+0</span> / <span className="text-red-500">-105</span> |
  | `...lana-project/withdrawal-web-interface/server.py` | <span className="text-green-500">+0</span> / <span className="text-red-500">-126</span> |
  | `.context/sp1-solana/verifier/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `.context/sp1-solana/verifier/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-28</span> |
  | `.context/sp1-solana/verifier/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-107</span> |
  | `.context/sp1-solana/verifier/src/test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-77</span> |
  | `.context/sp1-solana/verifier/src/utils.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-279</span> |
  | `...hdrawal-proof-wasm-verifier/verifier/Cargo.lock` | <span className="text-green-500">+0</span> / <span className="text-red-500">-878</span> |
  | `.gitignore` | <span className="text-green-500">+1</span> / <span className="text-red-500">-10</span> |
  | `Cargo.lock` | <span className="text-green-500">+220</span> / <span className="text-red-500">-217</span> |
  | `Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `justfile` | <span className="text-green-500">+34</span> / <span className="text-red-500">-211</span> |
  | `packages/zk-guest-sp1/out/public.json` | <span className="text-green-500">+3</span> / <span className="text-red-500">-4</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-5</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-19</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-9</span> |
  | `programs/shield-pool/src/tests/mod.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-5</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+61</span> / <span className="text-red-500">-49</span> |
  | `tooling/test/Cargo.toml` | <span className="text-green-500">+30</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/lib.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/localnet_test.rs` | <span className="text-green-500">+796</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/shared.rs` | <span className="text-green-500">+323</span> / <span className="text-red-500">-0</span> |
  | `tooling/test/src/testnet_test.rs` | <span className="text-green-500">+62</span> / <span className="text-red-500">-0</span> |

  </details>

---

## September 2025

### ✨ Features

- add rust indexer ([1549328](https://github.com/Machine-Labz/cloak/commit/154932892fc64a74ecbe6464433a35b574a71a98))
  <details>
  <summary>📂 <strong>28 files changed</strong>: <span className="text-green-500">+3567</span> / <span className="text-red-500">-11</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-4</span> |
  | `docker-compose.yml` | <span className="text-green-500">+19</span> / <span className="text-red-500">-7</span> |
  | `services/indexer-rs/.dockerignore` | <span className="text-green-500">+53</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/.env.example` | <span className="text-green-500">+38</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/.gitignore` | <span className="text-green-500">+46</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/Cargo.toml` | <span className="text-green-500">+91</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/Dockerfile` | <span className="text-green-500">+72</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/README.md` | <span className="text-green-500">+301</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/artifacts/.gitkeep` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/init.sql` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/justfile` | <span className="text-green-500">+195</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/artifacts.rs` | <span className="text-green-500">+301</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/bin/migrate.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/config.rs` | <span className="text-green-500">+162</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/database/connection.rs` | <span className="text-green-500">+174</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/database/migrations.rs` | <span className="text-green-500">+124</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/database/mod.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/database/storage.rs` | <span className="text-green-500">+484</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/error.rs` | <span className="text-green-500">+232</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/lib.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/logging.rs` | <span className="text-green-500">+167</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/main.rs` | <span className="text-green-500">+31</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/merkle.rs` | <span className="text-green-500">+367</span> / <span className="text-red-500">-0</span> |
  | `...ndexer-rs/src/migrations/001_initial_schema.sql` | <span className="text-green-500">+151</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/server/final_handlers.rs` | <span className="text-green-500">+225</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/server/middleware.rs` | <span className="text-green-500">+102</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/server/mod.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `services/indexer-rs/src/server/routes.rs` | <span className="text-green-500">+161</span> / <span className="text-red-500">-0</span> |

  </details>
- updating todo list ([ad31307](https://github.com/Machine-Labz/cloak/commit/ad31307975417542ea1c20899a7e074235cf7445))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+47</span> / <span className="text-red-500">-47</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/TODO.md` | <span className="text-green-500">+47</span> / <span className="text-red-500">-47</span> |

  </details>
- feat(indexer): add commitment uniqueness check and admin cleanup endpoint ([1c9fe3f](https://github.com/Machine-Labz/cloak/commit/1c9fe3f96bb711c5f129e87d08ed587dc07ff1bc))
  <details>
  <summary>📂 <strong>3 files changed</strong>: <span className="text-green-500">+114</span> / <span className="text-red-500">-7</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer/pnpm-workspace.yaml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/api/routes.ts` | <span className="text-green-500">+41</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/src/db/storage.ts` | <span className="text-green-500">+68</span> / <span className="text-red-500">-6</span> |

  </details>
- Cloak privacy protocol implementation ([7db43e0](https://github.com/Machine-Labz/cloak/commit/7db43e0c3b72750a59348e46373cd036944beceb))
  <details>
  <summary>📂 <strong>28 files changed</strong>: <span className="text-green-500">+1763</span> / <span className="text-red-500">-2098</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+9</span> / <span className="text-red-500">-4</span> |
  | `Cargo.lock` | <span className="text-green-500">+81</span> / <span className="text-red-500">-122</span> |
  | `LICENSE` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `Makefile` | <span className="text-green-500">+0</span> / <span className="text-red-500">-26</span> |
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-4</span> |
  | `packages/zk-guest-sp1/host/src/encoding.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-2</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-19</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-21</span> |
  | `...rams/shield-pool/src/instructions/initialize.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-88</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+135</span> / <span className="text-red-500">-151</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+7</span> / <span className="text-red-500">-12</span> |
  | `programs/shield-pool/src/state/mod.rs` | <span className="text-green-500">+23</span> / <span className="text-red-500">-34</span> |
  | `programs/shield-pool/src/tests/admin_push_root.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-119</span> |
  | `...s/shield-pool/src/tests/admin_push_root_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-90</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+85</span> / <span className="text-red-500">-59</span> |
  | `...shield-pool/src/tests/end_to_end_integration.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-404</span> |
  | `programs/shield-pool/src/tests/indexer_api_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-301</span> |
  | `...ield-pool/src/tests/integration_with_indexer.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-333</span> |
  | `programs/shield-pool/src/tests/mod.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-8</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+178</span> / <span className="text-red-500">-201</span> |
  | `services/indexer/src/db/storage.ts` | <span className="text-green-500">+26</span> / <span className="text-red-500">-5</span> |
  | `services/indexer/src/lib/merkle.ts` | <span className="text-green-500">+28</span> / <span className="text-red-500">-3</span> |
  | `test_complete_flow_rust/Cargo.toml` | <span className="text-green-500">+10</span> / <span className="text-red-500">-0</span> |
  | `test_complete_flow_rust/src/main.rs` | <span className="text-green-500">+138</span> / <span className="text-red-500">-78</span> |
  | `test_complete_flow_rust/src/testnet_test.rs` | <span className="text-green-500">+1000</span> / <span className="text-red-500">-0</span> |
  | `tooling/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- relayer api layer ([7614e80](https://github.com/Machine-Labz/cloak/commit/7614e80275e51f96eb3448e545f2545592138717))
  <details>
  <summary>📂 <strong>5 files changed</strong>: <span className="text-green-500">+26</span> / <span className="text-red-500">-4</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/config.toml` | <span className="text-green-500">+22</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/metrics.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |

  </details>
- project structure ([f409456](https://github.com/Machine-Labz/cloak/commit/f409456959429dbc030236b4f47d5aa69811110c))
  <details>
  <summary>📂 <strong>12 files changed</strong>: <span className="text-green-500">+1177</span> / <span className="text-red-500">-5</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+57</span> / <span className="text-red-500">-0</span> |
  | `services/relay/README.md` | <span className="text-green-500">+244</span> / <span className="text-red-500">-4</span> |
  | `services/relay/TODO.md` | <span className="text-green-500">+70</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+105</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+173</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+93</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/error.rs` | <span className="text-green-500">+124</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+78</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/metrics.rs` | <span className="text-green-500">+138</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(indexer): add initial implementation of Cloak Indexer microservice ([c1bf17d](https://github.com/Machine-Labz/cloak/commit/c1bf17d39a885da006be55b0334eb93a63c498df))
  <details>
  <summary>📂 <strong>31 files changed</strong>: <span className="text-green-500">+4211</span> / <span className="text-red-500">-4</span></summary>

  | File | Changes |
  |------|--------|
  | `docker-compose.yml` | <span className="text-green-500">+113</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/.dockerignore` | <span className="text-green-500">+51</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/.env.example` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/.gitignore` | <span className="text-green-500">+59</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+83</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/README.md` | <span className="text-green-500">+144</span> / <span className="text-red-500">-4</span> |
  | `services/indexer/artifacts/.gitkeep` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/artifacts/README.md` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/bunfig.toml` | <span className="text-green-500">+11</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/jest.config.js` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/package.json` | <span className="text-green-500">+56</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/api/routes.ts` | <span className="text-green-500">+354</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/api/server.ts` | <span className="text-green-500">+221</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/db/connection.ts` | <span className="text-green-500">+164</span> / <span className="text-red-500">-0</span> |
  | `...ndexer/src/db/migrations/001_initial_schema.sql` | <span className="text-green-500">+151</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/db/storage.ts` | <span className="text-green-500">+337</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/index.ts` | <span className="text-green-500">+39</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/lib/artifacts.ts` | <span className="text-green-500">+266</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/lib/config.ts` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/lib/logger.ts` | <span className="text-green-500">+80</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/lib/merkle.ts` | <span className="text-green-500">+291</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/scripts/migrate.ts` | <span className="text-green-500">+167</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/scripts/test-deposit.ts` | <span className="text-green-500">+183</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/scripts/test-full-flow.ts` | <span className="text-green-500">+321</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/tests/api.test.ts` | <span className="text-green-500">+172</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/tests/artifacts.test.ts` | <span className="text-green-500">+208</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/tests/merkle.test.ts` | <span className="text-green-500">+205</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/tests/setup.ts` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/tests/storage.test.ts` | <span className="text-green-500">+264</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/types/index.ts` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/tsconfig.json` | <span className="text-green-500">+28</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(shield-pool): enhance instruction handling and add new data structures ([283d1e0](https://github.com/Machine-Labz/cloak/commit/283d1e075797af3d728ea27d9ea309f730b83c09))
  <details>
  <summary>📂 <strong>15 files changed</strong>: <span className="text-green-500">+1416</span> / <span className="text-red-500">-415</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `Cargo.lock` | <span className="text-green-500">+479</span> / <span className="text-red-500">-15</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-2</span> |
  | `programs/shield-pool/src/instruction_data.rs` | <span className="text-green-500">+189</span> / <span className="text-red-500">-0</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+27</span> / <span className="text-red-500">-9</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-5</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+32</span> / <span className="text-red-500">-219</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+17</span> / <span className="text-red-500">-15</span> |
  | `programs/shield-pool/src/tests/admin_push_root.rs` | <span className="text-green-500">+212</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+127</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/mod.rs` | <span className="text-green-500">+10</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+276</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/utils.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/tests/withdraw_e2e.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-132</span> |

  </details>
- implement shield pool program for SP1 withdrawals ([f2c19f1](https://github.com/Machine-Labz/cloak/commit/f2c19f15163ebe04318f26fa6945773643693f33))
  <details>
  <summary>📂 <strong>17 files changed</strong>: <span className="text-green-500">+1440</span> / <span className="text-red-500">-10</span></summary>

  | File | Changes |
  |------|--------|
  | `docs/development/CLEANUP_COMPLETE.md` | <span className="text-green-500">+70</span> / <span className="text-red-500">-0</span> |
  | `docs/development/README.md` | <span className="text-green-500">+44</span> / <span className="text-red-500">-0</span> |
  | `docs/development/VKEY_FILE_SYSTEM.md` | <span className="text-green-500">+146</span> / <span className="text-red-500">-0</span> |
  | `docs/development/WORKSPACE_SETUP_COMPLETE.md` | <span className="text-green-500">+87</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+27</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/README.md` | <span className="text-green-500">+228</span> / <span className="text-red-500">-10</span> |
  | `programs/shield-pool/build.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/error.rs` | <span className="text-green-500">+55</span> / <span className="text-red-500">-0</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+16</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+237</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/state.rs` | <span className="text-green-500">+166</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/mod.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/tests/withdraw_e2e.rs` | <span className="text-green-500">+132</span> / <span className="text-red-500">-0</span> |

  </details>
- misc: add content folder with useful libraries for development ([134e674](https://github.com/Machine-Labz/cloak/commit/134e67430a41762f717128f73f744dd756819931))
  <details>
  <summary>📂 <strong>124 files changed</strong>: <span className="text-green-500">+9024</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `.context/pinocchio-programs-reference/.gitignore` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `.context/pinocchio-programs-reference/Cargo.toml` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `.context/pinocchio-programs-reference/README.md` | <span className="text-green-500">+146</span> / <span className="text-red-500">-0</span> |
  | `.../pinocchio-programs-reference/escrow/Cargo.toml` | <span className="text-green-500">+22</span> / <span className="text-red-500">-0</span> |
  | `...chio-programs-reference/escrow/src/constants.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...nocchio-programs-reference/escrow/src/errors.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-0</span> |
  | `...grams-reference/escrow/src/instructions/make.rs` | <span className="text-green-500">+27</span> / <span className="text-red-500">-0</span> |
  | `...ograms-reference/escrow/src/instructions/mod.rs` | <span className="text-green-500">+25</span> / <span className="text-red-500">-0</span> |
  | `...ams-reference/escrow/src/instructions/refund.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `...grams-reference/escrow/src/instructions/take.rs` | <span className="text-green-500">+113</span> / <span className="text-red-500">-0</span> |
  | `.../pinocchio-programs-reference/escrow/src/lib.rs` | <span className="text-green-500">+40</span> / <span className="text-red-500">-0</span> |
  | `...grams-reference/escrow/src/state/contributor.rs` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `...o-programs-reference/escrow/src/state/escrow.rs` | <span className="text-green-500">+47</span> / <span className="text-red-500">-0</span> |
  | `...ograms-reference/escrow/src/state/fundraiser.rs` | <span className="text-green-500">+47</span> / <span className="text-red-500">-0</span> |
  | `...chio-programs-reference/escrow/src/state/mod.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `...rograms-reference/escrow/src/tests/make_test.rs` | <span className="text-green-500">+73</span> / <span className="text-red-500">-0</span> |
  | `...chio-programs-reference/escrow/src/tests/mod.rs` | <span className="text-green-500">+54</span> / <span className="text-red-500">-0</span> |
  | `...rograms-reference/escrow/src/tests/take_test.rs` | <span className="text-green-500">+222</span> / <span className="text-red-500">-0</span> |
  | `...occhio-programs-reference/fundraiser/Cargo.toml` | <span className="text-green-500">+22</span> / <span className="text-red-500">-0</span> |
  | `...-programs-reference/fundraiser/src/constants.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...hio-programs-reference/fundraiser/src/errors.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-0</span> |
  | `...-reference/fundraiser/src/instructions/check.rs` | <span className="text-green-500">+28</span> / <span className="text-red-500">-0</span> |
  | `...rence/fundraiser/src/instructions/contribute.rs` | <span className="text-green-500">+27</span> / <span className="text-red-500">-0</span> |
  | `...rence/fundraiser/src/instructions/initialize.rs` | <span className="text-green-500">+31</span> / <span className="text-red-500">-0</span> |
  | `...ms-reference/fundraiser/src/instructions/mod.rs` | <span className="text-green-500">+28</span> / <span className="text-red-500">-0</span> |
  | `...reference/fundraiser/src/instructions/refund.rs` | <span className="text-green-500">+32</span> / <span className="text-red-500">-0</span> |
  | `...occhio-programs-reference/fundraiser/src/lib.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-0</span> |
  | `...s-reference/fundraiser/src/state/contributor.rs` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `...ms-reference/fundraiser/src/state/fundraiser.rs` | <span className="text-green-500">+47</span> / <span className="text-red-500">-0</span> |
  | `...-programs-reference/fundraiser/src/state/mod.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `...ms-reference/fundraiser/src/tests/check_test.rs` | <span className="text-green-500">+199</span> / <span className="text-red-500">-0</span> |
  | `...ference/fundraiser/src/tests/contribute_test.rs` | <span className="text-green-500">+195</span> / <span className="text-red-500">-0</span> |
  | `...eference/fundraiser/src/tests/initalize_test.rs` | <span className="text-green-500">+114</span> / <span className="text-red-500">-0</span> |
  | `...-programs-reference/fundraiser/src/tests/mod.rs` | <span className="text-green-500">+60</span> / <span className="text-red-500">-0</span> |
  | `...s-reference/fundraiser/src/tests/refund_test.rs` | <span className="text-green-500">+238</span> / <span className="text-red-500">-0</span> |
  | `...occhio-programs-reference/sbpf_close/.gitignore` | <span className="text-green-500">+10</span> / <span className="text-red-500">-0</span> |
  | `...occhio-programs-reference/sbpf_close/Cargo.toml` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `...nocchio-programs-reference/sbpf_close/README.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `...occhio-programs-reference/sbpf_close/src/lib.rs` | <span className="text-green-500">+49</span> / <span className="text-red-500">-0</span> |
  | `...eference/sbpf_close/src/sbpf_close/sbpf_close.s` | <span className="text-green-500">+45</span> / <span className="text-red-500">-0</span> |
  | `...t/pinocchio-programs-reference/vault/.gitignore` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `...t/pinocchio-programs-reference/vault/Cargo.toml` | <span className="text-green-500">+23</span> / <span className="text-red-500">-0</span> |
  | `...pinocchio-programs-reference/vault/src/based.rs` | <span className="text-green-500">+84</span> / <span className="text-red-500">-0</span> |
  | `...t/pinocchio-programs-reference/vault/src/lib.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `...inocchio-programs-reference/vault/src/native.rs` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `...cchio-programs-reference/vault/src/optimized.rs` | <span className="text-green-500">+39</span> / <span className="text-red-500">-0</span> |
  | `...pinocchio-programs-reference/vault/src/tests.rs` | <span className="text-green-500">+48</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/.github/workflows/pr.yml` | <span className="text-green-500">+44</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/.gitignore` | <span className="text-green-500">+12</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/Cargo.toml` | <span className="text-green-500">+52</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/LICENSE-MIT` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/README.md` | <span className="text-green-500">+163</span> / <span className="text-red-500">-0</span> |
  | `...sp1-solana/example-sp1-wasm-verifier/.gitignore` | <span className="text-green-500">+14</span> / <span className="text-red-500">-0</span> |
  | `...sp1-solana/example-sp1-wasm-verifier/Cargo.toml` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...p1-solana/example-sp1-wasm-verifier/LICENSE-MIT` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `.../sp1-solana/example-sp1-wasm-verifier/README.md` | <span className="text-green-500">+113</span> / <span className="text-red-500">-0</span> |
  | `...m-verifier/example/fibonacci-program/Cargo.lock` | <span className="text-green-500">+468</span> / <span className="text-red-500">-0</span> |
  | `...m-verifier/example/fibonacci-program/Cargo.toml` | <span className="text-green-500">+8</span> / <span className="text-red-500">-0</span> |
  | `...-verifier/example/fibonacci-program/src/main.rs` | <span className="text-green-500">+37</span> / <span className="text-red-500">-0</span> |
  | `...ifier/example/json/fibonacci_groth16_proof.json` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...erifier/example/json/fibonacci_plonk_proof.json` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...ple-sp1-wasm-verifier/example/script/Cargo.toml` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `...ample-sp1-wasm-verifier/example/script/build.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-0</span> |
  | `...le-sp1-wasm-verifier/example/script/src/main.rs` | <span className="text-green-500">+86</span> / <span className="text-red-500">-0</span> |
  | `...-sp1-wasm-verifier/example/wasm_example/main.js` | <span className="text-green-500">+56</span> / <span className="text-red-500">-0</span> |
  | `...wasm-verifier/example/wasm_example/package.json` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `...sm-verifier/example/wasm_example/pnpm-lock.yaml` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...a/example-sp1-wasm-verifier/verifier/Cargo.toml` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `...a/example-sp1-wasm-verifier/verifier/src/lib.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/pinocchio/Cargo.toml` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/pinocchio/src/lib.rs` | <span className="text-green-500">+120</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/program/Cargo.toml` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/program/src/lib.rs` | <span className="text-green-500">+58</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/script/.gitignore` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/script/Cargo.toml` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/script/build.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/script/src/main.rs` | <span className="text-green-500">+203</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/example/sp1-program/Cargo.toml` | <span className="text-green-500">+8</span> / <span className="text-red-500">-0</span> |
  | `...ple/sp1-program/elf/riscv32im-succinct-zkvm-elf` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...text/sp1-solana/example/sp1-program/src/main.rs` | <span className="text-green-500">+37</span> / <span className="text-red-500">-0</span> |
  | `...olana/solana-project/APPLICATION_FLOW_DESIGN.md` | <span className="text-green-500">+204</span> / <span className="text-red-500">-0</span> |
  | `...xt/sp1-solana/solana-project/CLEANUP_SUMMARY.md` | <span className="text-green-500">+66</span> / <span className="text-red-500">-0</span> |
  | `...solana-project/COMPLETE_APPLICATION_ANALYSIS.md` | <span className="text-green-500">+251</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/solana-project/Cargo.toml` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |
  | `...text/sp1-solana/solana-project/DEMO_COMPLETE.md` | <span className="text-green-500">+171</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/solana-project/README.md` | <span className="text-green-500">+238</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/solana-project/SUMMARY.md` | <span className="text-green-500">+89</span> / <span className="text-red-500">-0</span> |
  | `...olana/solana-project/ZK_STUDY_SESSION_STATUS.md` | <span className="text-green-500">+216</span> / <span className="text-red-500">-0</span> |
  | `...a-project/pinocchio-withdrawal-proof/Cargo.toml` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `...na-project/pinocchio-withdrawal-proof/README.md` | <span className="text-green-500">+126</span> / <span className="text-red-500">-0</span> |
  | `...a-project/pinocchio-withdrawal-proof/src/lib.rs` | <span className="text-green-500">+226</span> / <span className="text-red-500">-0</span> |
  | `...cchio-withdrawal-proof/test_withdrawal_proof.rs` | <span className="text-green-500">+174</span> / <span className="text-red-500">-0</span> |
  | `...na-project/withdrawal-json-generator/Cargo.toml` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `...oject/withdrawal-json-generator/src/bin/main.rs` | <span className="text-green-500">+141</span> / <span className="text-red-500">-0</span> |
  | `...ithdrawal-json/withdrawal_compressed_proof.json` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `...na/solana-project/withdrawal-proof/.env.example` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...drawal-proof/.github/workflows/foundry-test.yml` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `...ct/withdrawal-proof/.github/workflows/prove.yml` | <span className="text-green-500">+46</span> / <span className="text-red-500">-0</span> |
  | `...lana/solana-project/withdrawal-proof/.gitignore` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `...-project/withdrawal-proof/.vscode/settings.json` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `...lana/solana-project/withdrawal-proof/Cargo.toml` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...ana/solana-project/withdrawal-proof/LICENSE-MIT` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `...olana/solana-project/withdrawal-proof/README.md` | <span className="text-green-500">+89</span> / <span className="text-red-500">-0</span> |
  | `...olana-project/withdrawal-proof/get_vkey_hash.rs` | <span className="text-green-500">+23</span> / <span className="text-red-500">-0</span> |
  | `...ana-project/withdrawal-proof/program/Cargo.toml` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `...na-project/withdrawal-proof/program/src/main.rs` | <span className="text-green-500">+193</span> / <span className="text-red-500">-0</span> |
  | `.../solana-project/withdrawal-proof/rust-toolchain` | <span className="text-green-500">+3</span> / <span className="text-red-500">-0</span> |
  | `...lana-project/withdrawal-proof/script/Cargo.toml` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |
  | `...solana-project/withdrawal-proof/script/build.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `...project/withdrawal-proof/script/src/bin/main.rs` | <span className="text-green-500">+205</span> / <span className="text-red-500">-0</span> |
  | `...ct/withdrawal-proof/script/src/bin/vkey_hash.rs` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `...project/withdrawal-proof/script/verify_proof.rs` | <span className="text-green-500">+34</span> / <span className="text-red-500">-0</span> |
  | `...ana-project/withdrawal-wasm-verifier/Cargo.toml` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `...ana-project/withdrawal-wasm-verifier/src/lib.rs` | <span className="text-green-500">+81</span> / <span className="text-red-500">-0</span> |
  | `...ana-project/withdrawal-web-interface/index.html` | <span className="text-green-500">+470</span> / <span className="text-red-500">-0</span> |
  | `...a-project/withdrawal-web-interface/package.json` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `...lana-project/withdrawal-web-interface/server.js` | <span className="text-green-500">+105</span> / <span className="text-red-500">-0</span> |
  | `...lana-project/withdrawal-web-interface/server.py` | <span className="text-green-500">+126</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/verifier/.gitignore` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/verifier/Cargo.toml` | <span className="text-green-500">+28</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/verifier/src/lib.rs` | <span className="text-green-500">+107</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/verifier/src/test.rs` | <span className="text-green-500">+77</span> / <span className="text-red-500">-0</span> |
  | `.context/sp1-solana/verifier/src/utils.rs` | <span className="text-green-500">+279</span> / <span className="text-red-500">-0</span> |
  | `...hdrawal-proof-wasm-verifier/verifier/Cargo.lock` | <span className="text-green-500">+878</span> / <span className="text-red-500">-0</span> |

  </details>
- initialize project structure with Cargo.toml, LICENSE, and Makefile ([950af93](https://github.com/Machine-Labz/cloak/commit/950af93047213957e09f497c3a6e72d2c4d4feb7))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+118</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `Cargo.toml` | <span className="text-green-500">+69</span> / <span className="text-red-500">-0</span> |
  | `LICENSE` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `Makefile` | <span className="text-green-500">+26</span> / <span className="text-red-500">-0</span> |

  </details>
- add vkey-generator package for SP1 withdraw circuit ([baae538](https://github.com/Machine-Labz/cloak/commit/baae538a983836a3df854f85aa9d6a80c13f0930))
  <details>
  <summary>📂 <strong>13 files changed</strong>: <span className="text-green-500">+646</span> / <span className="text-red-500">-352</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/vkey-generator/Cargo.toml` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `packages/vkey-generator/src/main.rs` | <span className="text-green-500">+43</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `packages/zk-guest-sp1/guest/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+43</span> / <span className="text-red-500">-35</span> |
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+22</span> / <span className="text-red-500">-21</span> |
  | `packages/zk-guest-sp1/host/Cargo.toml` | <span className="text-green-500">+6</span> / <span className="text-red-500">-2</span> |
  | `.../zk-guest-sp1/host/src/bin/generate_examples.rs` | <span className="text-green-500">+50</span> / <span className="text-red-500">-5</span> |
  | `...ages/zk-guest-sp1/host/src/bin/get_vkey_hash.rs` | <span className="text-green-500">+33</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/src/encoding.rs` | <span className="text-green-500">+58</span> / <span className="text-red-500">-92</span> |
  | `packages/zk-guest-sp1/host/src/main.rs` | <span className="text-green-500">+52</span> / <span className="text-red-500">-41</span> |
  | `packages/zk-guest-sp1/out/proof.bin` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/tests/golden.rs` | <span className="text-green-500">+323</span> / <span className="text-red-500">-153</span> |

  </details>
- implement SP1 withdraw guest MVP with ZK circuit ([0ae471c](https://github.com/Machine-Labz/cloak/commit/0ae471c2fb98be6d615739bf7c223dc245f3c8a9))
  <details>
  <summary>📂 <strong>57 files changed</strong>: <span className="text-green-500">+2358</span> / <span className="text-red-500">-0</span></summary>

  | File | Changes |
  |------|--------|
  | `.cursor/prompts/e2e-localnet.md` | <span className="text-green-500">+12</span> / <span className="text-red-500">-0</span> |
  | `.cursor/prompts/implement-indexer.md` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `.cursor/prompts/implement-onchain-withdraw.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `.cursor/prompts/implement-withdraw-guest-sp1.md` | <span className="text-green-500">+14</span> / <span className="text-red-500">-0</span> |
  | `.cursor/prompts/wire-relay-api.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/00-overview.md` | <span className="text-green-500">+23</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/10-zk-layer.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/20-encoding.md` | <span className="text-green-500">+29</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/30-merkle.md` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/40-circuit-withdraw.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/50-onchain.md` | <span className="text-green-500">+21</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/60-indexer.md` | <span className="text-green-500">+15</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/70-relay.md` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/80-frontend.md` | <span className="text-green-500">+18</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/90-roadmap.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `.cursor/rules/99-style-and-practices.md` | <span className="text-green-500">+9</span> / <span className="text-red-500">-0</span> |
  | `.gitignore` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `README.md` | <span className="text-green-500">+23</span> / <span className="text-red-500">-0</span> |
  | `apps/web/README.md` | <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span> |
  | `docs/README.md` | <span className="text-green-500">+8</span> / <span className="text-red-500">-0</span> |
  | `docs/glossary.md` | <span className="text-green-500">+12</span> / <span className="text-red-500">-0</span> |
  | `docs/nonzk/frontend.md` | <span className="text-green-500">+14</span> / <span className="text-red-500">-0</span> |
  | `docs/nonzk/indexer.md` | <span className="text-green-500">+17</span> / <span className="text-red-500">-0</span> |
  | `docs/nonzk/relayer.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `docs/roadmap.md` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/README.md` | <span className="text-green-500">+22</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/api-contracts.md` | <span className="text-green-500">+34</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/circuit-withdraw.md` | <span className="text-green-500">+25</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/design.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/encoding.md` | <span className="text-green-500">+51</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/merkle.md` | <span className="text-green-500">+34</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/onchain-verifier.md` | <span className="text-green-500">+24</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/prover-sp1.md` | <span className="text-green-500">+34</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/testing.md` | <span className="text-green-500">+22</span> / <span className="text-red-500">-0</span> |
  | `docs/zk/threat-model.md` | <span className="text-green-500">+16</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/Cargo.toml` | <span className="text-green-500">+37</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/README.md` | <span className="text-green-500">+176</span> / <span className="text-red-500">-0</span> |
  | `...ages/zk-guest-sp1/examples/outputs.example.json` | <span className="text-green-500">+10</span> / <span className="text-red-500">-0</span> |
  | `...ages/zk-guest-sp1/examples/private.example.json` | <span className="text-green-500">+14</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/examples/public.example.json` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/Cargo.toml` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+285</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+218</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/Cargo.toml` | <span className="text-green-500">+30</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/build.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `.../zk-guest-sp1/host/src/bin/generate_examples.rs` | <span className="text-green-500">+123</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/src/encoding.rs` | <span className="text-green-500">+194</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/src/lib.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/host/src/main.rs` | <span className="text-green-500">+253</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/out/proof.bin` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/out/public.json` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/tests/golden.rs` | <span className="text-green-500">+299</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-verifier-program/README.md` | <span className="text-green-500">+7</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/README.md` | <span className="text-green-500">+13</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/README.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/README.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `tooling/README.md` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |

  </details>

### 🐛 Bug Fixes

- relay as workspace member ([788bf1f](https://github.com/Machine-Labz/cloak/commit/788bf1f0be47a0d2f3f7e182b39b286b78aa4616))
  <details>
  <summary>📂 <strong>1 file changed</strong>: <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>
- merge conflicts ([51b45eb](https://github.com/Machine-Labz/cloak/commit/51b45eb524404c8cc3cce87cf71d8d08573060f9))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+371</span> / <span className="text-red-500">-189</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+370</span> / <span className="text-red-500">-186</span> |
  | `packages/zk-guest-sp1/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-3</span> |

  </details>
- dependency conflict ([cc04567](https://github.com/Machine-Labz/cloak/commit/cc04567102783c85426ce2bd2a0ba62ed5ff3ddc))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+61</span> / <span className="text-red-500">-69</span></summary>

  | File | Changes |
  |------|--------|
  | `services/relay/Cargo.toml` | <span className="text-green-500">+10</span> / <span className="text-red-500">-9</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-2</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+14</span> / <span className="text-red-500">-7</span> |
  | `services/relay/src/metrics.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-44</span> |

  </details>
- compilation errors ([403a1fd](https://github.com/Machine-Labz/cloak/commit/403a1fdbdcb379095967f3acd8b8bc5e7f98bfe1))
  <details>
  <summary>📂 <strong>6 files changed</strong>: <span className="text-green-500">+969</span> / <span className="text-red-500">-52</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+946</span> / <span className="text-red-500">-31</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+8</span> / <span className="text-red-500">-10</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-5</span> |
  | `services/relay/src/metrics.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-5</span> |

  </details>
- fix(mollusk): Fix dependency version issue ([4c7c1a4](https://github.com/Machine-Labz/cloak/commit/4c7c1a4fe77936b020a23539c492bd4b668b225d))
  <details>
  <summary>📂 <strong>8 files changed</strong>: <span className="text-green-500">+29</span> / <span className="text-red-500">-48</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `Cargo.toml` | <span className="text-green-500">+3</span> / <span className="text-red-500">-3</span> |
  | `docs/development/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/vkey-generator/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-24</span> |
  | `packages/zk-guest-sp1/guest/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `packages/zk-guest-sp1/host/Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `.../zk-guest-sp1/host/src/bin/generate_examples.rs` | <span className="text-green-500">+18</span> / <span className="text-red-500">-16</span> |

  </details>

### 📚 Documentation

- update documentation and project structure for improved clarity ([4d24452](https://github.com/Machine-Labz/cloak/commit/4d24452fed22260ad4b3d747499081888b4c3fd8))
  <details>
  <summary>📂 <strong>15 files changed</strong>: <span className="text-green-500">+297</span> / <span className="text-red-500">-42</span></summary>

  | File | Changes |
  |------|--------|
  | `.cursor/prompts/implement-onchain-withdraw.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `.cursor/rules/00-overview.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `.cursor/rules/50-onchain.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `.gitignore` | <span className="text-green-500">+10</span> / <span className="text-red-500">-1</span> |
  | `apps/web/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-6</span> |
  | `docs/CHANGELOG.md` | <span className="text-green-500">+121</span> / <span className="text-red-500">-0</span> |
  | `docs/COMPLETE_FLOW_STATUS.md` | <span className="text-green-500">+85</span> / <span className="text-red-500">-11</span> |
  | `docs/README.md` | <span className="text-green-500">+21</span> / <span className="text-red-500">-1</span> |
  | `docs/glossary.md` | <span className="text-green-500">+4</span> / <span className="text-red-500">-1</span> |
  | `docs/roadmap.md` | <span className="text-green-500">+39</span> / <span className="text-red-500">-13</span> |
  | `docs/zk/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/zk/circuit-withdraw.md` | <span className="text-green-500">+3</span> / <span className="text-red-500">-2</span> |
  | `docs/zk/encoding.md` | <span className="text-green-500">+8</span> / <span className="text-red-500">-1</span> |
  | `docs/zk/onchain-verifier.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/zk/testing.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>

### ♻️ Refactoring

- refactor(core): remove TS indexer and leave just the RS one ([60487fc](https://github.com/Machine-Labz/cloak/commit/60487fc7dd4bda8d6457ed65cc1c038398571439))
  <details>
  <summary>📂 <strong>55 files changed</strong>: <span className="text-green-500">+354</span> / <span className="text-red-500">-4499</span></summary>

  | File | Changes |
  |------|--------|
  | `services/indexer-rs/.dockerignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-53</span> |
  | `services/indexer-rs/.env.example` | <span className="text-green-500">+0</span> / <span className="text-red-500">-38</span> |
  | `services/indexer-rs/.gitignore` | <span className="text-green-500">+0</span> / <span className="text-red-500">-46</span> |
  | `services/indexer-rs/Dockerfile` | <span className="text-green-500">+0</span> / <span className="text-red-500">-72</span> |
  | `services/indexer-rs/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-301</span> |
  | `services/indexer-rs/artifacts/.gitkeep` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1</span> |
  | `services/indexer/.dockerignore` | <span className="text-green-500">+24</span> / <span className="text-red-500">-22</span> |
  | `services/indexer/.env.example` | <span className="text-green-500">+23</span> / <span className="text-red-500">-9</span> |
  | `services/indexer/.gitignore` | <span className="text-green-500">+17</span> / <span className="text-red-500">-30</span> |
  | `services/{indexer-rs => indexer}/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/Dockerfile` | <span className="text-green-500">+44</span> / <span className="text-red-500">-55</span> |
  | `services/indexer/README.md` | <span className="text-green-500">+246</span> / <span className="text-red-500">-90</span> |
  | `services/indexer/artifacts/README.md` | <span className="text-green-500">+0</span> / <span className="text-red-500">-26</span> |
  | `services/indexer/bunfig.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-11</span> |
  | `services/{indexer-rs => indexer}/init.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/jest.config.js` | <span className="text-green-500">+0</span> / <span className="text-red-500">-33</span> |
  | `services/{indexer-rs => indexer}/justfile` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/package.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-56</span> |
  | `services/indexer/src/api/routes.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-354</span> |
  | `services/indexer/src/api/server.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-221</span> |
  | `services/{indexer-rs => indexer}/src/artifacts.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...ices/{indexer-rs => indexer}/src/bin/migrate.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/{indexer-rs => indexer}/src/config.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...dexer-rs => indexer}/src/database/connection.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...dexer-rs => indexer}/src/database/migrations.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...ces/{indexer-rs => indexer}/src/database/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...{indexer-rs => indexer}/src/database/storage.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/db/connection.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-164</span> |
  | `...ndexer/src/db/migrations/001_initial_schema.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-151</span> |
  | `services/indexer/src/db/storage.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-358</span> |
  | `services/{indexer-rs => indexer}/src/error.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/index.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-39</span> |
  | `services/{indexer-rs => indexer}/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/lib/artifacts.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-266</span> |
  | `services/indexer/src/lib/config.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-52</span> |
  | `services/indexer/src/lib/logger.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-80</span> |
  | `services/indexer/src/lib/merkle.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-316</span> |
  | `services/{indexer-rs => indexer}/src/logging.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/{indexer-rs => indexer}/src/main.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/{indexer-rs => indexer}/src/merkle.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `... indexer}/src/migrations/001_initial_schema.sql` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/scripts/migrate.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-167</span> |
  | `services/indexer/src/scripts/test-deposit.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-183</span> |
  | `services/indexer/src/scripts/test-full-flow.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-321</span> |
  | `...xer-rs => indexer}/src/server/final_handlers.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...indexer-rs => indexer}/src/server/middleware.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/{indexer-rs => indexer}/src/server/mod.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...es/{indexer-rs => indexer}/src/server/routes.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `services/indexer/src/tests/api.test.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-172</span> |
  | `services/indexer/src/tests/artifacts.test.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-208</span> |
  | `services/indexer/src/tests/merkle.test.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-205</span> |
  | `services/indexer/src/tests/setup.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-17</span> |
  | `services/indexer/src/tests/storage.test.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-264</span> |
  | `services/indexer/src/types/index.ts` | <span className="text-green-500">+0</span> / <span className="text-red-500">-90</span> |
  | `services/indexer/tsconfig.json` | <span className="text-green-500">+0</span> / <span className="text-red-500">-28</span> |

  </details>
- feat(zk-guest-sp1): refactor fee calculation and public input structure ([edae219](https://github.com/Machine-Labz/cloak/commit/edae2197623016e1c22b4bc1eb4e7821d19767c4))
  <details>
  <summary>📂 <strong>7 files changed</strong>: <span className="text-green-500">+249</span> / <span className="text-red-500">-126</span></summary>

  | File | Changes |
  |------|--------|
  | `packages/zk-guest-sp1/guest/src/encoding.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-6</span> |
  | `packages/zk-guest-sp1/guest/src/main.rs` | <span className="text-green-500">+3</span> / <span className="text-red-500">-8</span> |
  | `packages/zk-guest-sp1/host/src/bin/cloak-zk.rs` | <span className="text-green-500">+12</span> / <span className="text-red-500">-9</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+6</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+65</span> / <span className="text-red-500">-36</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-14</span> |
  | `test_complete_flow_rust/src/main.rs` | <span className="text-green-500">+157</span> / <span className="text-red-500">-52</span> |

  </details>
- refactor(relay): enhance withdraw handling and configuration management ([9e24758](https://github.com/Machine-Labz/cloak/commit/9e24758d4645925e619fe0f07c15ce0dbe8d4efe))
  <details>
  <summary>📂 <strong>25 files changed</strong>: <span className="text-green-500">+3078</span> / <span className="text-red-500">-551</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.toml` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `services/relay/Cargo.toml` | <span className="text-green-500">+28</span> / <span className="text-red-500">-30</span> |
  | `services/relay/README.md` | <span className="text-green-500">+278</span> / <span className="text-red-500">-163</span> |
  | `services/relay/RUNNING_INSTRUCTIONS.md` | <span className="text-green-500">+272</span> / <span className="text-red-500">-0</span> |
  | `services/relay/config.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `services/relay/docker-compose.yml` | <span className="text-green-500">+38</span> / <span className="text-red-500">-0</span> |
  | `services/relay/examples/test_api.sh` | <span className="text-green-500">+81</span> / <span className="text-red-500">-0</span> |
  | `services/relay/migrations/001_init.sql` | <span className="text-green-500">+65</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/api/mod.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-20</span> |
  | `services/relay/src/api/status.rs` | <span className="text-green-500">+19</span> / <span className="text-red-500">-69</span> |
  | `services/relay/src/api/withdraw.rs` | <span className="text-green-500">+198</span> / <span className="text-red-500">-110</span> |
  | `services/relay/src/config.rs` | <span className="text-green-500">+20</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/db/mod.rs` | <span className="text-green-500">+81</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/db/models.rs` | <span className="text-green-500">+103</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/db/repository.rs` | <span className="text-green-500">+145</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/error.rs` | <span className="text-green-500">+32</span> / <span className="text-red-500">-102</span> |
  | `services/relay/src/main.rs` | <span className="text-green-500">+51</span> / <span className="text-red-500">-56</span> |
  | `services/relay/src/queue/mod.rs` | <span className="text-green-500">+86</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/queue/processor.rs` | <span className="text-green-500">+214</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/queue/redis_queue.rs` | <span className="text-green-500">+328</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/client.rs` | <span className="text-green-500">+158</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/mod.rs` | <span className="text-green-500">+233</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/solana/transaction_builder.rs` | <span className="text-green-500">+241</span> / <span className="text-red-500">-0</span> |
  | `services/relay/src/validation/mod.rs` | <span className="text-green-500">+360</span> / <span className="text-red-500">-0</span> |
  | `services/relay/test_compile.sh` | <span className="text-green-500">+31</span> / <span className="text-red-500">-0</span> |

  </details>
- feat(shield-pool): refactor instruction data handling and enhance hashing functionality ([d04f445](https://github.com/Machine-Labz/cloak/commit/d04f445e6df22ca6c2298951b275f68e5a97e077))
  <details>
  <summary>📂 <strong>17 files changed</strong>: <span className="text-green-500">+445</span> / <span className="text-red-500">-533</span></summary>

  | File | Changes |
  |------|--------|
  | `.gitignore` | <span className="text-green-500">+2</span> / <span className="text-red-500">-1</span> |
  | `Cargo.lock` | <span className="text-green-500">+133</span> / <span className="text-red-500">-162</span> |
  | `Cargo.toml` | <span className="text-green-500">+14</span> / <span className="text-red-500">-13</span> |
  | `docs/development/README.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/vkey-generator/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instruction_data.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-189</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-13</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+8</span> / <span className="text-red-500">-13</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+103</span> / <span className="text-red-500">-34</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+76</span> / <span className="text-red-500">-12</span> |
  | `programs/shield-pool/src/state/mod.rs` | <span className="text-green-500">+5</span> / <span className="text-red-500">-0</span> |
  | `...pool/src/{state.rs => state/nullifier_shard.rs}` | <span className="text-green-500">+1</span> / <span className="text-red-500">-79</span> |
  | `programs/shield-pool/src/state/roots_ring.rs` | <span className="text-green-500">+82</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/withdraw.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/utils.rs` | <span className="text-green-500">+4</span> / <span className="text-red-500">-17</span> |

  </details>
- refactor(indexer): update deposit handling to use route-based approach ([7d90cf3](https://github.com/Machine-Labz/cloak/commit/7d90cf3998e78ab3d0acfd3d50da211c4a241e36))
  <details>
  <summary>📂 <strong>4 files changed</strong>: <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span></summary>

  | File | Changes |
  |------|--------|
  | `.cursor/rules/60-indexer.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/nonzk/indexer.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/roadmap.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `docs/zk/onchain-verifier.md` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |

  </details>

### 🔧 Maintenance

- remove obsolete Rust project files for test_complete_flow_rust ([de7364d](https://github.com/Machine-Labz/cloak/commit/de7364d450fbd41100866058187c7dfcadd6e13e))
  <details>
  <summary>📂 <strong>2 files changed</strong>: <span className="text-green-500">+0</span> / <span className="text-red-500">-1029</span></summary>

  | File | Changes |
  |------|--------|
  | `test_complete_flow_rust/Cargo.toml` | <span className="text-green-500">+0</span> / <span className="text-red-500">-29</span> |
  | `test_complete_flow_rust/src/testnet_test.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-1000</span> |

  </details>
- update dependencies and enhance shield pool program ([7177026](https://github.com/Machine-Labz/cloak/commit/7177026096a942f8ba73f9b93701f2d78af8fa35))
  <details>
  <summary>📂 <strong>29 files changed</strong>: <span className="text-green-500">+2793</span> / <span className="text-red-500">-514</span></summary>

  | File | Changes |
  |------|--------|
  | `Cargo.lock` | <span className="text-green-500">+175</span> / <span className="text-red-500">-164</span> |
  | `Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-2</span> |
  | `justfile` | <span className="text-green-500">+237</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/Cargo.toml` | <span className="text-green-500">+1</span> / <span className="text-red-500">-0</span> |
  | `...-guest-sp1/examples/outputs.example.json.backup` | <span className="text-green-500">+6</span> / <span className="text-red-500">-0</span> |
  | `...ages/zk-guest-sp1/host/src/bin/get_vkey_hash.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `packages/zk-guest-sp1/host/src/encoding.rs` | <span className="text-green-500">+49</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/out/proof.bin` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `packages/zk-guest-sp1/out/public.json` | <span className="text-green-500">+4</span> / <span className="text-red-500">-4</span> |
  | `packages/zk-guest-sp1/tests/golden.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/Cargo.toml` | <span className="text-green-500">+5</span> / <span className="text-red-500">-1</span> |
  | `programs/shield-pool/build.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-50</span> |
  | `programs/shield-pool/src/constants.rs` | <span className="text-green-500">+2</span> / <span className="text-red-500">-2</span> |
  | `programs/shield-pool/src/instruction_data.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-189</span> |
  | `...shield-pool/src/instructions/admin_push_root.rs` | <span className="text-green-500">+11</span> / <span className="text-red-500">-5</span> |
  | `programs/shield-pool/src/instructions/deposit.rs` | <span className="text-green-500">+28</span> / <span className="text-red-500">-15</span> |
  | `...rams/shield-pool/src/instructions/initialize.rs` | <span className="text-green-500">+88</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/instructions/mod.rs` | <span className="text-green-500">+9</span> / <span className="text-red-500">-6</span> |
  | `programs/shield-pool/src/instructions/withdraw.rs` | <span className="text-green-500">+110</span> / <span className="text-red-500">-34</span> |
  | `programs/shield-pool/src/lib.rs` | <span className="text-green-500">+35</span> / <span className="text-red-500">-21</span> |
  | `...rams/shield-pool/src/{state.rs => state/mod.rs}` | <span className="text-green-500">+0</span> / <span className="text-red-500">-0</span> |
  | `...s/shield-pool/src/tests/admin_push_root_test.rs` | <span className="text-green-500">+90</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/deposit.rs` | <span className="text-green-500">+1</span> / <span className="text-red-500">-1</span> |
  | `...shield-pool/src/tests/end_to_end_integration.rs` | <span className="text-green-500">+404</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/tests/indexer_api_test.rs` | <span className="text-green-500">+301</span> / <span className="text-red-500">-0</span> |
  | `...ield-pool/src/tests/integration_with_indexer.rs` | <span className="text-green-500">+333</span> / <span className="text-red-500">-0</span> |
  | `programs/shield-pool/src/utils.rs` | <span className="text-green-500">+0</span> / <span className="text-red-500">-18</span> |
  | `test_complete_flow_rust/Cargo.toml` | <span className="text-green-500">+19</span> / <span className="text-red-500">-0</span> |
  | `test_complete_flow_rust/src/main.rs` | <span className="text-green-500">+878</span> / <span className="text-red-500">-0</span> |

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
