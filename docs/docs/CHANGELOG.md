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

- add docusaurus documentation site ([1ed3972](https://github.com/Machine-Labz/cloak/commit/1ed39723e845333c35aa9fefe09c0d61dd85e59a))

### 🐛 Bug Fixes

- fixes ([6c0933f](https://github.com/Machine-Labz/cloak/commit/6c0933fae35a8f5c44d0d4b0d59c614fae90119b))
- docker divergencies ([157b26b](https://github.com/Machine-Labz/cloak/commit/157b26b18792346a64c3b6de80072aea8487ae08))
- workspace ([48fc7b1](https://github.com/Machine-Labz/cloak/commit/48fc7b1317d0efaeb678b1bc6527f705a8d524fd))

### 📚 Documentation

- update documentation structure and remove obsolete files ([23dfa5f](https://github.com/Machine-Labz/cloak/commit/23dfa5f34322facbdf798856756b0f5786bb379a))
- restructure Docusaurus documentation site and update .gitignore ([cac3670](https://github.com/Machine-Labz/cloak/commit/cac367056911b527416b020546da2e4d4c405d82))

### 🔧 Maintenance

- remove pnpm lock ([d788628](https://github.com/Machine-Labz/cloak/commit/d788628d8b8319474db50d0b56c72680b0a83ea2))
- remove obsolete files and clean up project structure ([660bb5b](https://github.com/Machine-Labz/cloak/commit/660bb5b9a233cc0dabfdb453d269414168011499))

---

## September 2025

### ✨ Features

- add rust indexer ([1549328](https://github.com/Machine-Labz/cloak/commit/154932892fc64a74ecbe6464433a35b574a71a98))
- updating todo list ([ad31307](https://github.com/Machine-Labz/cloak/commit/ad31307975417542ea1c20899a7e074235cf7445))
- feat(indexer): add commitment uniqueness check and admin cleanup endpoint ([1c9fe3f](https://github.com/Machine-Labz/cloak/commit/1c9fe3f96bb711c5f129e87d08ed587dc07ff1bc))
- Cloak privacy protocol implementation ([7db43e0](https://github.com/Machine-Labz/cloak/commit/7db43e0c3b72750a59348e46373cd036944beceb))
- relayer api layer ([7614e80](https://github.com/Machine-Labz/cloak/commit/7614e80275e51f96eb3448e545f2545592138717))
- project structure ([f409456](https://github.com/Machine-Labz/cloak/commit/f409456959429dbc030236b4f47d5aa69811110c))
- feat(indexer): add initial implementation of Cloak Indexer microservice ([c1bf17d](https://github.com/Machine-Labz/cloak/commit/c1bf17d39a885da006be55b0334eb93a63c498df))
- feat(shield-pool): enhance instruction handling and add new data structures ([283d1e0](https://github.com/Machine-Labz/cloak/commit/283d1e075797af3d728ea27d9ea309f730b83c09))
- implement shield pool program for SP1 withdrawals ([f2c19f1](https://github.com/Machine-Labz/cloak/commit/f2c19f15163ebe04318f26fa6945773643693f33))
- misc: add content folder with useful libraries for development ([134e674](https://github.com/Machine-Labz/cloak/commit/134e67430a41762f717128f73f744dd756819931))
- initialize project structure with Cargo.toml, LICENSE, and Makefile ([950af93](https://github.com/Machine-Labz/cloak/commit/950af93047213957e09f497c3a6e72d2c4d4feb7))
- add vkey-generator package for SP1 withdraw circuit ([baae538](https://github.com/Machine-Labz/cloak/commit/baae538a983836a3df854f85aa9d6a80c13f0930))
- implement SP1 withdraw guest MVP with ZK circuit ([0ae471c](https://github.com/Machine-Labz/cloak/commit/0ae471c2fb98be6d615739bf7c223dc245f3c8a9))

### 🐛 Bug Fixes

- relay as workspace member ([788bf1f](https://github.com/Machine-Labz/cloak/commit/788bf1f0be47a0d2f3f7e182b39b286b78aa4616))
- merge conflicts ([51b45eb](https://github.com/Machine-Labz/cloak/commit/51b45eb524404c8cc3cce87cf71d8d08573060f9))
- dependency conflict ([cc04567](https://github.com/Machine-Labz/cloak/commit/cc04567102783c85426ce2bd2a0ba62ed5ff3ddc))
- compilation errors ([403a1fd](https://github.com/Machine-Labz/cloak/commit/403a1fdbdcb379095967f3acd8b8bc5e7f98bfe1))
- fix(mollusk): Fix dependency version issue ([4c7c1a4](https://github.com/Machine-Labz/cloak/commit/4c7c1a4fe77936b020a23539c492bd4b668b225d))

### 📚 Documentation

- update documentation and project structure for improved clarity ([4d24452](https://github.com/Machine-Labz/cloak/commit/4d24452fed22260ad4b3d747499081888b4c3fd8))

### ♻️ Refactoring

- refactor(core): remove TS indexer and leave just the RS one ([60487fc](https://github.com/Machine-Labz/cloak/commit/60487fc7dd4bda8d6457ed65cc1c038398571439))
- feat(zk-guest-sp1): refactor fee calculation and public input structure ([edae219](https://github.com/Machine-Labz/cloak/commit/edae2197623016e1c22b4bc1eb4e7821d19767c4))
- refactor(relay): enhance withdraw handling and configuration management ([9e24758](https://github.com/Machine-Labz/cloak/commit/9e24758d4645925e619fe0f07c15ce0dbe8d4efe))
- feat(shield-pool): refactor instruction data handling and enhance hashing functionality ([d04f445](https://github.com/Machine-Labz/cloak/commit/d04f445e6df22ca6c2298951b275f68e5a97e077))
- refactor(indexer): update deposit handling to use route-based approach ([7d90cf3](https://github.com/Machine-Labz/cloak/commit/7d90cf3998e78ab3d0acfd3d50da211c4a241e36))

### 🔧 Maintenance

- remove obsolete Rust project files for test_complete_flow_rust ([de7364d](https://github.com/Machine-Labz/cloak/commit/de7364d450fbd41100866058187c7dfcadd6e13e))
- update dependencies and enhance shield pool program ([7177026](https://github.com/Machine-Labz/cloak/commit/7177026096a942f8ba73f9b93701f2d78af8fa35))

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
