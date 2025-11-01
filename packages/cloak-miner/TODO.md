# Cloak Miner Implementation TODO

**Priority System:**
- 🔴 **P0 - CRITICAL**: Core privacy infrastructure (do first!)
- 🟡 **P1 - HIGH**: Essential features
- 🟢 **P2 - MEDIUM**: Nice-to-have improvements
- 🔵 **P3 - LOW**: Future enhancements

**Status Legend:**
- ❌ Not Started
- 🚧 In Progress
- ✅ Complete
- 🔄 Blocked/Waiting
- 🧪 Testing

---

## PHASE 1: Privacy Infrastructure (P0 - CRITICAL)

### 🔴 1.1 Decoy Transaction Core

**Goal**: Enable miners to generate basic decoy withdraw transactions.

#### 1.1.1 Setup ZK Proof Infrastructure
- [ ] ❌ Research SP1 prover integration for cloak-miner
  - **File**: Research document or spike branch
  - **Dependencies**: SP1 SDK, shield-pool circuit specs
  - **Estimated Time**: 2-3 days
  - **Blocker**: Need access to withdraw circuit WASM or ELF
  - **Reference**: `programs/shield-pool/README.md:159-168`

- [ ] ❌ Create `src/decoy/` module structure
  ```bash
  mkdir -p packages/cloak-miner/src/decoy
  touch packages/cloak-miner/src/decoy/mod.rs
  touch packages/cloak-miner/src/decoy/proof_generator.rs
  touch packages/cloak-miner/src/decoy/transaction_builder.rs
  ```

- [ ] ❌ Implement `DecoyProofGenerator` struct
  - **File**: `src/decoy/proof_generator.rs`
  - **Functions**:
    - `new()` - Initialize with prover and commitment
    - `generate_decoy_proof()` - Generate valid ZK proof
    - `compute_nullifier()` - Derive fresh nullifier
    - `compute_outputs_hash()` - BLAKE3 hash of outputs
  - **Testing**: Unit tests with mock commitments
  - **Reference**: Shield pool proof format (260 bytes + 104 bytes public inputs)

- [ ] ❌ Implement `DecoyTransactionBuilder`
  - **File**: `src/decoy/transaction_builder.rs`
  - **Functions**:
    - `new()` - Initialize with keypair and RPC
    - `build_decoy_tx()` - Build complete transaction
    - `derive_accounts()` - Derive all PDAs
    - `estimate_cost()` - Calculate transaction cost
  - **Integration**: Use existing `transaction_builder.rs` helpers from relay
  - **Reference**: `services/relay/src/solana/transaction_builder.rs:135-150`

#### 1.1.2 Deposit & Commitment Management
- [ ] ❌ Implement deposit functionality
  - **File**: `src/decoy/deposit_manager.rs`
  - **Functions**:
    - `deposit_to_pool()` - Submit deposit transaction
    - `generate_commitment()` - Create note commitment
    - `store_note()` - Encrypt and save note locally
    - `track_deposits()` - Monitor deposit confirmations
  - **Storage**: Use local SQLite or JSON file for notes
  - **Security**: Encrypt notes with miner keypair

- [ ] ❌ Build Merkle tree tracker
  - **File**: `src/decoy/merkle_tracker.rs`
  - **Functions**:
    - `sync_from_indexer()` - Fetch tree state from indexer
    - `update_tree()` - Add new deposits
    - `generate_proof()` - Create Merkle inclusion proof
    - `get_current_root()` - Latest root hash
  - **Integration**: Connect to indexer service
  - **Reference**: Indexer API at `services/indexer/`

#### 1.1.3 Decoy CLI Command
- [ ] ❌ Add `decoy` subcommand to CLI
  - **File**: `src/main.rs`
  - **Args**:
    - `--rate <NUM>` - Decoys per hour
    - `--max-budget <SOL>` - Daily spend limit
    - `--sync-with-relay` - Enable smart scheduling
  - **Example**:
    ```bash
    cloak-miner --keypair ./miner.json decoy \
      --rate 20 \
      --max-budget 2.0 \
      --sync-with-relay true
    ```

- [ ] ❌ Implement decoy generation loop
  - **File**: `src/decoy/generator.rs`
  - **Functions**:
    - `run_decoy_loop()` - Main generation loop
    - `should_generate_decoy()` - Decision logic
    - `submit_decoy_tx()` - Send transaction
    - `record_metrics()` - Track stats
  - **Rate Limiting**: Configurable interval between decoys
  - **Budget Tracking**: Stop when daily budget exhausted

### 🔴 1.2 Basic Privacy Metrics

#### 1.2.1 On-Chain Activity Monitoring
- [ ] ❌ Implement withdrawal observer
  - **File**: `src/privacy/observer.rs`
  - **Functions**:
    - `subscribe_to_withdrawals()` - WebSocket subscription
    - `parse_withdraw_event()` - Parse on-chain events
    - `track_withdrawal()` - Add to recent activity buffer
  - **Data Source**: Shield Pool program logs
  - **Reference**: `programs/shield-pool/README.md:104` (withdraw_event)

- [ ] ❌ Calculate anonymity set size
  - **File**: `src/privacy/metrics.rs`
  - **Functions**:
    - `estimate_anonymity_set()` - Count recent withdrawals
    - `calculate_required_decoys()` - Gap to target
    - `get_privacy_score()` - Overall privacy rating
  - **Time Windows**: 10 min, 1 hour, 24 hour views

#### 1.2.2 Cost Tracking
- [ ] ❌ Implement decoy cost tracker
  - **File**: `src/privacy/cost_tracker.rs`
  - **Functions**:
    - `record_decoy_cost()` - Log TX + pool fees
    - `get_daily_spend()` - Total spent today
    - `get_average_cost_per_decoy()` - Historical avg
    - `estimate_monthly_cost()` - Project costs
  - **Storage**: Append-only log file or SQLite

---

## PHASE 2: Intelligent Decoy Scheduling (P1 - HIGH)

### 🟡 2.1 Relay Integration

#### 2.1.1 Backlog Synchronization
- [ ] ❌ Implement relay client for decoy system
  - **File**: `src/decoy/relay_client.rs`
  - **Functions**:
    - `get_pending_withdrawals()` - Fetch backlog
    - `detect_real_activity()` - Identify user withdrawals
    - `calculate_decoy_multiplier()` - Determine response
  - **API**: `GET http://localhost:3002/backlog`
  - **Reference**: `services/relay/src/api/backlog.rs`

- [ ] ❌ Smart decoy trigger logic
  - **File**: `src/decoy/smart_scheduler.rs`
  - **Logic**:
    - If `pending_count > 0`: Generate 10x decoys IMMEDIATELY
    - If `anonymity_set < target`: Generate maintenance decoys
    - Else: Generate occasional background noise
  - **Timing**: Random jitter ±30 minutes around real withdrawals

#### 2.1.2 Adaptive Strategy
- [ ] ❌ Implement `AdaptiveDecoyStrategy`
  - **File**: `src/privacy/adaptive_strategy.rs`
  - **Functions**:
    - `should_generate_decoy_now()` - Real-time decision
    - `calculate_timing_jitter()` - Randomize schedule
    - `adjust_strategy()` - Learn from patterns
  - **Decision Types**:
    - `DecoyDecision::Urgent` - Hide real users NOW
    - `DecoyDecision::Maintenance` - Baseline privacy
    - `DecoyDecision::Background` - Occasional noise
    - `DecoyDecision::Skip` - Budget exhausted

### 🟡 2.2 Privacy Dashboard

#### 2.2.1 Real-Time Monitoring
- [ ] ❌ Create privacy metrics display
  - **File**: `src/privacy/dashboard.rs`
  - **Display**:
    ```
    ╔════════════════════════════════════════╗
    ║     PRIVACY INFRASTRUCTURE STATUS      ║
    ╠════════════════════════════════════════╣
    ║ Anonymity Set (1h):       147 txs      ║
    ║ Real Withdrawals:           5 txs      ║
    ║ Decoy Contribution:       142 txs      ║
    ║ Privacy Score:              95%        ║
    ║                                        ║
    ║ Today's Decoys:            320 txs     ║
    ║ Cost:                   0.78 SOL       ║
    ║ Budget Remaining:       1.22 SOL       ║
    ║                                        ║
    ║ Status: 🟢 PROTECTING USERS            ║
    ╚════════════════════════════════════════╝
    ```
  - **Update Frequency**: Every 10 seconds

- [ ] ❌ Add to main stats display
  - **File**: `src/main.rs`
  - **Integration**: Print alongside mining stats
  - **Hotkey**: Press 'P' to toggle privacy view

---

## PHASE 3: Coordination & Optimization (P2 - MEDIUM)

### 🟢 3.1 Multi-Miner Coordination

#### 3.1.1 On-Chain Coordination
- [ ] ❌ Implement activity observer for coordination
  - **File**: `src/coordination/onchain_observer.rs`
  - **Functions**:
    - `observe_pool_activity()` - Monitor shield pool
    - `detect_decoy_spikes()` - Identify synchronized decoys
    - `calculate_participation_probability()` - Self-coordinate
  - **Algorithm**: Randomized probability based on recent activity

#### 3.1.2 Gossip Protocol (Advanced)
- [ ] ❌ Research libp2p integration
  - **Decision**: Determine if gossip is necessary
  - **Alternative**: On-chain coordination may be sufficient
  - **Dependencies**: libp2p-gossipsub, peer discovery

- [ ] ❌ Implement miner gossip network (if needed)
  - **File**: `src/coordination/gossip.rs`
  - **Functions**:
    - `announce_decoy_intent()` - Broadcast plans
    - `coordinate_timing()` - Find gaps
    - `join_network()` - Peer discovery
  - **Privacy**: Don't reveal miner identity in gossip

### 🟢 3.2 Advanced Privacy Features

#### 3.2.1 Variable Amount Decoys
- [ ] ❌ Sample real user withdrawal distribution
  - **File**: `src/privacy/amount_sampler.rs`
  - **Functions**:
    - `fetch_historical_withdrawals()` - Get real data
    - `build_distribution()` - Create statistical model
    - `sample_realistic_amount()` - Generate decoy amount
  - **Source**: On-chain shield pool logs (last 30 days)

- [ ] ❌ Implement realistic amount decoys
  - **Integration**: Use sampler in `DecoyProofGenerator`
  - **Goal**: Decoy amounts indistinguishable from real

#### 3.2.2 Multi-Address Mining
- [ ] ❌ Implement address rotation
  - **File**: `src/privacy/multi_address.rs`
  - **Functions**:
    - `generate_address_pool()` - Create N keypairs
    - `rotate_address()` - Switch to next address
    - `track_addresses()` - Monitor all addresses
  - **Security**: Derive from single seed for backup
  - **Storage**: Encrypted address pool file

---

## PHASE 4: Testing & Validation (P1 - HIGH)

### 🟡 4.1 Unit Tests

- [ ] ❌ Test ZK proof generation
  - **File**: `tests/decoy_proof_test.rs`
  - **Coverage**:
    - Valid proof generation
    - Nullifier uniqueness
    - Outputs hash computation
    - Error handling (invalid commitment)

- [ ] ❌ Test transaction building
  - **File**: `tests/decoy_tx_test.rs`
  - **Coverage**:
    - Correct account ordering
    - Valid instruction data
    - Signature verification
    - Fee calculation

- [ ] ❌ Test privacy metrics
  - **File**: `tests/privacy_metrics_test.rs`
  - **Coverage**:
    - Anonymity set calculation
    - Cost tracking accuracy
    - Decision logic branches

### 🟡 4.2 Integration Tests

- [ ] ❌ Localnet end-to-end test
  - **File**: `tests/integration_decoy_localnet.rs`
  - **Setup**:
    1. Deploy shield-pool + scramble-registry
    2. Initialize miner with deposit
    3. Generate and submit decoy
    4. Verify transaction success
    5. Check privacy metrics
  - **Run**: `cargo test --test integration_decoy_localnet -- --ignored`

- [ ] ❌ Devnet stress test
  - **File**: `tests/integration_decoy_stress.rs`
  - **Test**: Generate 100 decoys over 1 hour
  - **Validate**:
    - All transactions succeed
    - Cost tracking accurate
    - No duplicate nullifiers
    - Privacy score increases

### 🟡 4.3 Privacy Audits

- [ ] ❌ Statistical analysis of decoys
  - **Tool**: Python notebook or Rust analysis script
  - **Metrics**:
    - Timing distribution (should be random)
    - Amount distribution (should match real users)
    - Nullifier entropy (should be uniform)
  - **Goal**: Prove decoys are indistinguishable

- [ ] ❌ Third-party privacy audit
  - **Scope**: Hire security firm to analyze
  - **Questions**:
    - Can observer distinguish decoys from real withdrawals?
    - What's the deanonymization success rate?
    - Any pattern leaks or fingerprinting vectors?

---

## PHASE 5: Documentation & Deployment (P2 - MEDIUM)

### 🟢 5.1 User Documentation

- [ ] ❌ Update README.md
  - **File**: `packages/cloak-miner/README.md`
  - **Add Sections**:
    - Privacy Mining Overview
    - Decoy Generation Guide
    - Cost-Benefit Analysis
    - Best Practices
  - **Examples**: Include command examples

- [ ] ❌ Create privacy tutorial
  - **File**: `packages/cloak-miner/PRIVACY_TUTORIAL.md`
  - **Content**:
    - Why decoys matter
    - How to run privacy mining
    - Understanding privacy metrics
    - Troubleshooting

- [ ] ❌ Video walkthrough
  - **Platform**: YouTube or Loom
  - **Content**:
    - Setup cloak-miner
    - Run decoy generation
    - Monitor privacy dashboard
    - Interpret metrics

### 🟢 5.2 Deployment Tooling

- [ ] ❌ Create deployment scripts
  - **File**: `scripts/deploy_privacy_miner.sh`
  - **Steps**:
    1. Install dependencies
    2. Generate miner keypair
    3. Fund with SOL
    4. Initialize deposit
    5. Start decoy generation
  - **Platform**: Linux, macOS, Docker

- [ ] ❌ Docker container for privacy mining
  - **File**: `Dockerfile.privacy-miner`
  - **Features**:
    - Lightweight Alpine base
    - Auto-restart on failure
    - Environment variable config
    - Logging to stdout
  - **Example**:
    ```bash
    docker run -d \
      -e MINER_KEYPAIR=$(cat miner.json) \
      -e DECOY_RATE=20 \
      -e MAX_BUDGET=2.0 \
      cloak/privacy-miner:latest
    ```

---

## PHASE 6: Monitoring & Operations (P3 - LOW)

### 🔵 6.1 Operational Metrics

- [ ] ❌ Prometheus metrics export
  - **File**: `src/monitoring/prometheus.rs`
  - **Metrics**:
    - `cloak_decoys_generated_total` (counter)
    - `cloak_decoy_cost_sol` (gauge)
    - `cloak_anonymity_set_size` (gauge)
    - `cloak_privacy_score` (gauge)
  - **Endpoint**: `http://localhost:9090/metrics`

- [ ] ❌ Grafana dashboard
  - **File**: `monitoring/grafana/privacy_dashboard.json`
  - **Panels**:
    - Anonymity set over time
    - Decoy generation rate
    - Cost tracking
    - Privacy score heatmap

### 🔵 6.2 Alerting

- [ ] ❌ Setup alerts for privacy degradation
  - **Tool**: Prometheus Alertmanager
  - **Alerts**:
    - `PrivacyScoreLow` - Anonymity set < 50 (1 hour)
    - `DecoyGenerationStopped` - No decoys in 10 minutes
    - `BudgetExhausted` - Daily budget reached
  - **Notification**: Slack, PagerDuty, or email

---

## DEPENDENCIES & BLOCKERS

### External Dependencies

- [ ] 🔄 **SP1 Prover Access**
  - **Needed For**: Decoy ZK proof generation
  - **Action**: Get withdraw circuit ELF or WASM
  - **Owner**: Shield pool team
  - **Status**: BLOCKED

- [ ] 🔄 **Indexer API Access**
  - **Needed For**: Merkle tree sync
  - **Action**: Deploy indexer or use existing
  - **Owner**: Indexer service
  - **Status**: Available (check `services/indexer/`)

- [ ] ❌ **Shield Pool Devnet Deployment**
  - **Needed For**: Integration testing
  - **Action**: Deploy programs to devnet
  - **Owner**: Deployment team

### Internal Dependencies

- [ ] ❌ **Relay Backlog API Finalized**
  - **Needed For**: Smart scheduling
  - **Action**: Verify API contract
  - **File**: `services/relay/src/api/backlog.rs`
  - **Status**: Looks complete

- [ ] ❌ **Scramble Registry Fee Share Config**
  - **Needed For**: Cost calculations
  - **Action**: Confirm `fee_share_bps` value
  - **File**: `programs/scramble-registry/README.md:88`
  - **Status**: Should be in registry config

---

## ESTIMATED TIMELINE

### Sprint 1 (Weeks 1-2): Core Decoy Infrastructure
- Set up ZK proof generation (3 days)
- Implement deposit & commitment management (3 days)
- Build basic decoy transaction builder (3 days)
- Add CLI command (1 day)
- Unit tests (2 days)

**Deliverable**: Miners can generate manual decoy transactions

### Sprint 2 (Weeks 3-4): Intelligent Scheduling
- Relay integration (2 days)
- Adaptive strategy implementation (3 days)
- Privacy metrics & dashboard (3 days)
- Integration tests (2 days)

**Deliverable**: Miners automatically generate decoys based on demand

### Sprint 3 (Weeks 5-6): Coordination & Polish
- Multi-miner coordination (3 days)
- Variable amount decoys (2 days)
- Documentation (3 days)
- Deployment tooling (2 days)

**Deliverable**: Production-ready privacy mining system

### Sprint 4 (Week 7+): Monitoring & Optimization
- Prometheus metrics (2 days)
- Grafana dashboards (1 day)
- Privacy audits (ongoing)
- Performance optimization (ongoing)

**Deliverable**: Observable, optimized privacy infrastructure

---

## IMMEDIATE NEXT STEPS

### This Week (Priority Order):

1. **🔴 UNBLOCK SP1 Prover Integration**
   - Contact shield pool team for circuit access
   - Document proof generation requirements
   - Create spike branch for proof integration

2. **🔴 Create Decoy Module Structure**
   - Set up `src/decoy/` directory
   - Create basic module files
   - Add to `src/lib.rs`

3. **🔴 Implement Deposit Functionality**
   - Build deposit transaction builder
   - Test on localnet
   - Store commitment locally

4. **🟡 Write First Integration Test**
   - Set up localnet test environment
   - Deploy required programs
   - Test deposit → generate decoy flow

---

## NOTES & DECISIONS

### Design Decisions

- **Decision 1**: Use SP1 for ZK proofs (not custom circuit)
  - **Rationale**: Compatibility with shield pool
  - **Date**: 2025-10-30

- **Decision 2**: On-chain coordination first, gossip later
  - **Rationale**: Simpler, sufficient for MVP
  - **Date**: 2025-10-30

- **Decision 3**: Local note storage (not centralized)
  - **Rationale**: Privacy and decentralization
  - **Date**: 2025-10-30

### Open Questions

- **Q1**: What's the target anonymity set size?
  - **Answer**: 100 withdrawals/hour (aim for 10:1 decoy:real ratio)

- **Q2**: How do we prevent Sybil attacks on decoy generation?
  - **Answer**: Require PoW claims + actual deposits (economic cost)

- **Q3**: Should miners coordinate via gossip or on-chain observation?
  - **Answer**: Start with on-chain, add gossip if needed

---

## CHANGELOG

| Date | Change | Author |
|------|--------|--------|
| 2025-10-30 | Initial TODO created | Claude |

---

**Remember**: Every checkbox checked is a step toward building unbreakable privacy on Solana. This is critical infrastructure, not just a nice-to-have feature. 🔐
