# Dual-Mining Design: Cloak + Ore Profitability Optimization

**Status:** Design Phase
**Created:** 2025-10-30
**Last Updated:** 2025-10-30
**Authors:** Cloak Team

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Goals & Requirements](#goals--requirements)
4. [System Architecture](#system-architecture)
5. [Phased Implementation Plan](#phased-implementation-plan)
6. [Technical Specifications](#technical-specifications)
7. [Challenges & Solutions](#challenges--solutions)
8. [Testing Strategy](#testing-strategy)
9. [Success Criteria](#success-criteria)
10. [References](#references)

---

## Executive Summary

### Vision

Transform `cloak-miner` from a single-purpose Cloak PoW miner into an **intelligent dual-mining system** that can simultaneously track and optimize between Cloak claim mining and Ore token mining based on real-time profitability analysis.

### Key Principle

**⚠️ CRITICAL: Maintain 100% backward compatibility with existing Cloak mining functionality.**

All new features will be opt-in additions that do not modify or break current behavior.

### Value Proposition

- **For Miners**: Maximize revenue by automatically mining the most profitable protocol
- **For Cloak Protocol**: Ensure claim availability through economic incentives
- **For Users**: Faster withdrawals when mining is economically attractive

### Inspiration

Similar to established mining tools:
- **NiceHash**: Auto-switches between cryptocurrencies based on profitability
- **Awesome Miner**: Multi-coin mining management
- **Mining Pool Hub**: Multi-algo profit switching

---

## Current State Analysis

### Existing Architecture

```
┌─────────────────────────────────────────────────────┐
│ cloak-miner v0.1.0 (Current)                        │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────────────────────┐              │
│  │ Mining Engine (BLAKE3)           │              │
│  │ - Single-threaded brute force    │              │
│  │ - ~5M H/s on single core         │              │
│  │ - Timeout-based mining           │              │
│  └──────────────────────────────────┘              │
│                                                     │
│  ┌──────────────────────────────────┐              │
│  │ Claim Manager                    │              │
│  │ - Tracks active claims           │              │
│  │ - Submits mine + reveal txs      │              │
│  │ - Manages claim lifecycle        │              │
│  └──────────────────────────────────┘              │
│                                                     │
│  ┌──────────────────────────────────┐              │
│  │ Demand-Gated Mining              │              │
│  │ - Polls relay /backlog endpoint  │              │
│  │ - Mines when pending_count > 0   │              │
│  │ - Maintains min buffer (2 claims)│              │
│  │ - Idles when no demand           │              │
│  └──────────────────────────────────┘              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Current Behavior (Must Preserve)

#### Mining Triggers
1. **Active Demand**: `pending_count > 0` from relay backlog
2. **Buffer Maintenance**: `active_claims < min_buffer` (default: 2)
3. **Idle Mode**: `no demand && claims >= buffer` → wait

#### Performance Characteristics
- **Algorithm**: BLAKE3 (fast, CPU-efficient)
- **Hash Rate**: ~5M H/s per core (single-threaded)
- **Success Rate**: Depends on difficulty (~1/256 for easy targets)
- **Cost**: ~0.00001 SOL per claim (mine + reveal txs)

#### Revenue Model
- **Source**: Fee share from claim consumption
- **Distribution**: `fee_share_bps` from registry (e.g., 20% of withdrawal fee)
- **Tracking**: On-chain via `miner_pda.total_consumed` counter
- **Payout**: Lamports transferred to `miner_authority` during `consume_claim` CPI

### What Works Well (Keep)
✅ Demand-gated mining prevents wasteful claim production
✅ Relay backlog integration provides reactive mining
✅ Claim lifecycle management is robust
✅ Statistics tracking (hash rate, success rate, claims/hour)
✅ Graceful shutdown with Ctrl-C

### What's Missing (Add)
❌ No profitability tracking or earnings visibility
❌ No comparison with alternative mining opportunities (e.g., Ore)
❌ No multi-threaded mining (underutilizes CPU)
❌ No automatic optimization between protocols

---

## Goals & Requirements

### Primary Goals

#### Goal 1: Earnings Visibility
**What**: Track and display real-time profitability from Cloak mining
**Why**: Miners need to see ROI to justify running the software
**How**: Monitor SOL balance changes, claim consumption, and fee earnings

#### Goal 2: Dual-Mining Support
**What**: Enable miners to run both Cloak and Ore mining
**Why**: Diversify revenue streams and maximize hardware utilization
**How**: Integrate Ore CLI's mining logic alongside Cloak engine

#### Goal 3: Intelligent Optimization
**What**: Automatically switch between Cloak and Ore based on profitability
**Why**: Maximize miner revenue without manual intervention
**How**: Compare earnings rates and adjust mining strategy dynamically

### Non-Goals (Out of Scope)

❌ Mining other protocols beyond Cloak and Ore
❌ GPU acceleration (CPU-only for now)
❌ Pool mining for Cloak (solo mining only)
❌ Cross-chain mining (Solana-only)
❌ Modifying on-chain programs (client-side changes only)

### Requirements

#### Functional Requirements

**FR-1: Backward Compatibility**
- ✅ MUST preserve all existing CLI flags and behavior
- ✅ MUST maintain current mining logic as default
- ✅ MUST keep demand-gated mining unchanged
- ✅ New features MUST be opt-in via flags

**FR-2: Earnings Tracking**
- ✅ Track SOL balance at start vs. current
- ✅ Track claim consumption events
- ✅ Calculate earnings rate (SOL/hour)
- ✅ Display profitability reports

**FR-3: Ore Mining Integration**
- ✅ Import Ore CLI mining logic
- ✅ Support manual mode switching (`--mode cloak|ore|auto`)
- ✅ Share hardware resources efficiently
- ✅ Track Ore token earnings separately

**FR-4: Profit Optimization**
- ✅ Compare Cloak vs. Ore profitability
- ✅ Auto-switch when profit difference > threshold (e.g., 20%)
- ✅ Add hysteresis to prevent excessive switching
- ✅ Use demand as tiebreaker when earnings are close

#### Non-Functional Requirements

**NFR-1: Performance**
- Mining performance MUST NOT degrade vs. current implementation
- Balance checks MUST NOT block mining operations
- Memory usage MUST remain under 100MB per miner

**NFR-2: Reliability**
- Miner MUST recover from RPC failures gracefully
- Earnings data MUST persist across restarts (future: use local DB)
- Transaction failures MUST NOT corrupt state

**NFR-3: Usability**
- CLI output MUST be clear and actionable
- Earnings reports MUST update every N rounds (configurable)
- Error messages MUST be helpful for debugging

**NFR-4: Security**
- Private keys MUST remain secure (no logging/transmission)
- RPC calls MUST use HTTPS (no plaintext secrets)
- No external dependencies beyond Solana SDK and Ore CLI

---

## System Architecture

### Target Architecture (After All Phases)

```
┌───────────────────────────────────────────────────────────────┐
│  cloak-miner (Enhanced Multi-Mining)                          │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────┐         ┌─────────────────────┐    │
│  │ Cloak Mining Engine │         │  Ore Mining Engine  │    │
│  │                     │         │                     │    │
│  │ • BLAKE3 PoW        │         │ • Equix PoW         │    │
│  │ • Multi-threaded    │         │ • From ore-cli      │    │
│  │ • Claim lifecycle   │         │ • Token rewards     │    │
│  │ • Demand-gated      │         │ • Bus selection     │    │
│  └──────────┬──────────┘         └──────────┬──────────┘    │
│             │                               │               │
│             └───────────┬───────────────────┘               │
│                         │                                   │
│              ┌──────────▼──────────┐                        │
│              │ Earnings Tracker    │                        │
│              │                     │                        │
│              │ • SOL balance Δ     │                        │
│              │ • ORE balance Δ     │                        │
│              │ • Claim fees earned │                        │
│              │ • ORE rewards earned│                        │
│              │ • Earnings/hour     │                        │
│              │ • ROI %             │                        │
│              │ • TX cost tracking  │                        │
│              └──────────┬──────────┘                        │
│                         │                                   │
│              ┌──────────▼──────────┐                        │
│              │ Profitability Oracle│                        │
│              │                     │                        │
│              │ • ORE/SOL price     │                        │
│              │ • USD conversion    │                        │
│              │ • Electricity cost  │                        │
│              │ • Difficulty metrics│                        │
│              └──────────┬──────────┘                        │
│                         │                                   │
│              ┌──────────▼──────────┐                        │
│              │  Strategy Selector  │                        │
│              │                     │                        │
│              │  Mode: Auto         │                        │
│              │                     │                        │
│              │  if cloak_$/hr >    │                        │
│              │     ore_$/hr * 1.2: │                        │
│              │      mine_cloak()   │                        │
│              │  else:               │                        │
│              │      mine_ore()     │                        │
│              │                     │                        │
│              │  Hysteresis: 20%    │                        │
│              │  Tiebreak: demand   │                        │
│              └─────────────────────┘                        │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### Component Interaction Flow

```
┌────────┐     ┌──────────┐     ┌──────────────┐     ┌────────────┐
│ Relay  │────>│ Strategy │────>│ Mining Engine│────>│ On-Chain   │
│Backlog │     │ Selector │     │   (Active)   │     │ Programs   │
└────────┘     └──────────┘     └──────────────┘     └────────────┘
    │               │                   │                   │
    │               │                   │                   │
    │               v                   v                   │
    │          ┌──────────┐       ┌──────────┐             │
    │          │Earnings  │<──────│ Balance  │<────────────┘
    │          │ Tracker  │       │  Monitor │
    └──────────>          │       └──────────┘
               └──────────┘
```

---

## Phased Implementation Plan

### Overview

We'll implement this in **three phases**, with each phase adding value independently while building toward the full vision.

**Timeline Estimate:**
- Phase 1: 1-2 weeks
- Phase 2: 2-3 weeks
- Phase 3: 2-3 weeks
- **Total: 5-8 weeks**

---

### Phase 1: Earnings Tracking & Visibility

**Goal**: Add profitability monitoring without changing mining behavior.

**Status**: 🎯 **START HERE**

#### What We're Building

A new `EarningsTracker` component that:
- Records starting SOL balance when miner starts
- Periodically queries current SOL balance
- Tracks claim consumption events
- Calculates earnings rate (SOL/hour)
- Displays comprehensive profitability reports

#### Why This Matters

Miners currently have **zero visibility** into whether they're making money. This phase answers:
- "How much am I earning?"
- "What's my ROI?"
- "Is this profitable given my electricity costs?"

#### Implementation Details

##### New File: `src/earnings.rs`

```rust
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Tracks earnings from Cloak claim mining
#[derive(Debug)]
pub struct EarningsTracker {
    // Cloak mining metrics
    start_sol_balance: u64,
    current_sol_balance: AtomicU64,
    claims_consumed: AtomicU64,
    total_claim_fees_earned: AtomicU64, // lamports

    // Transaction costs
    total_tx_costs: AtomicU64, // lamports spent on TX fees
    total_txs_sent: AtomicU64,

    // Timing
    start_time: Instant,
    last_balance_update: std::sync::Mutex<Instant>,

    // RPC client for balance checks
    rpc_client: RpcClient,
    miner_pubkey: Pubkey,
}

impl EarningsTracker {
    /// Create new earnings tracker with initial balance snapshot
    pub async fn new(
        rpc_url: String,
        miner_pubkey: Pubkey
    ) -> Result<Self> {
        let rpc_client = RpcClient::new(rpc_url);
        let start_sol_balance = rpc_client.get_balance(&miner_pubkey)?;

        tracing::info!(
            "📊 Earnings tracker initialized with starting balance: {:.6} SOL",
            start_sol_balance as f64 / 1_000_000_000.0
        );

        Ok(Self {
            start_sol_balance,
            current_sol_balance: AtomicU64::new(start_sol_balance),
            claims_consumed: AtomicU64::new(0),
            total_claim_fees_earned: AtomicU64::new(0),
            total_tx_costs: AtomicU64::new(0),
            total_txs_sent: AtomicU64::new(0),
            start_time: Instant::now(),
            last_balance_update: std::sync::Mutex::new(Instant::now()),
            rpc_client,
            miner_pubkey,
        })
    }

    /// Update SOL balance from on-chain (call periodically)
    ///
    /// This should be called every 5-10 minutes to track balance changes
    /// without spamming RPC nodes.
    pub async fn update_balance(&self) -> Result<()> {
        let current_sol = self.rpc_client.get_balance(&self.miner_pubkey)?;
        let previous = self.current_sol_balance.swap(current_sol, Ordering::Relaxed);

        *self.last_balance_update.lock().unwrap() = Instant::now();

        tracing::debug!(
            "Balance updated: {:.6} SOL (Δ{:+.6} SOL)",
            current_sol as f64 / 1_000_000_000.0,
            (current_sol as i64 - previous as i64) as f64 / 1_000_000_000.0
        );

        Ok(())
    }

    /// Record a transaction submission
    pub fn record_tx_sent(&self, fee_lamports: u64) {
        self.total_txs_sent.fetch_add(1, Ordering::Relaxed);
        self.total_tx_costs.fetch_add(fee_lamports, Ordering::Relaxed);
    }

    /// Record a claim consumption event
    ///
    /// Called when we detect that one of our claims was consumed.
    /// Fee amount is calculated from registry.fee_share_bps and withdrawal amount.
    pub fn record_claim_consumed(&self, fee_earned_lamports: u64) {
        self.claims_consumed.fetch_add(1, Ordering::Relaxed);
        self.total_claim_fees_earned.fetch_add(fee_earned_lamports, Ordering::Relaxed);

        tracing::info!(
            "💰 Claim consumed! Earned {:.6} SOL (total: {} claims, {:.6} SOL)",
            fee_earned_lamports as f64 / 1_000_000_000.0,
            self.claims_consumed.load(Ordering::Relaxed),
            self.total_claim_fees_earned.load(Ordering::Relaxed) as f64 / 1_000_000_000.0
        );
    }

    /// Calculate gross earnings rate (SOL per hour, before costs)
    pub fn gross_earnings_per_hour(&self) -> f64 {
        let elapsed_hours = self.start_time.elapsed().as_secs_f64() / 3600.0;
        if elapsed_hours == 0.0 {
            return 0.0;
        }

        let total_fees = self.total_claim_fees_earned.load(Ordering::Relaxed);
        (total_fees as f64 / 1_000_000_000.0) / elapsed_hours
    }

    /// Calculate net earnings rate (SOL per hour, after TX costs)
    pub fn net_earnings_per_hour(&self) -> f64 {
        let elapsed_hours = self.start_time.elapsed().as_secs_f64() / 3600.0;
        if elapsed_hours == 0.0 {
            return 0.0;
        }

        let total_fees = self.total_claim_fees_earned.load(Ordering::Relaxed);
        let total_costs = self.total_tx_costs.load(Ordering::Relaxed);
        let net = total_fees.saturating_sub(total_costs);

        (net as f64 / 1_000_000_000.0) / elapsed_hours
    }

    /// Calculate net balance change (including all TX costs)
    pub fn net_balance_change(&self) -> f64 {
        let current = self.current_sol_balance.load(Ordering::Relaxed);
        let start = self.start_sol_balance;

        (current as i64 - start as i64) as f64 / 1_000_000_000.0
    }

    /// Calculate ROI percentage
    pub fn roi_percentage(&self) -> f64 {
        let net_change = self.net_balance_change();
        let start_sol = self.start_sol_balance as f64 / 1_000_000_000.0;

        if start_sol == 0.0 {
            return 0.0;
        }

        (net_change / start_sol) * 100.0
    }

    /// Calculate average cost per transaction
    pub fn avg_tx_cost(&self) -> f64 {
        let total_txs = self.total_txs_sent.load(Ordering::Relaxed);
        if total_txs == 0 {
            return 0.0;
        }

        let total_costs = self.total_tx_costs.load(Ordering::Relaxed);
        (total_costs as f64 / total_txs as f64) / 1_000_000_000.0
    }

    /// Calculate breakeven claims needed (to cover TX costs)
    pub fn breakeven_claims(&self, avg_fee_per_claim: u64) -> u64 {
        if avg_fee_per_claim == 0 {
            return 0;
        }

        let total_costs = self.total_tx_costs.load(Ordering::Relaxed);
        total_costs / avg_fee_per_claim
    }

    /// Print comprehensive earnings report
    pub fn print_earnings_report(&self) {
        let uptime_hours = self.start_time.elapsed().as_secs_f64() / 3600.0;
        let uptime_days = uptime_hours / 24.0;

        let claims_consumed = self.claims_consumed.load(Ordering::Relaxed);
        let total_fees = self.total_claim_fees_earned.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
        let total_costs = self.total_tx_costs.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
        let net_fees = total_fees - total_costs;

        let gross_per_hour = self.gross_earnings_per_hour();
        let net_per_hour = self.net_earnings_per_hour();
        let net_change = self.net_balance_change();
        let roi = self.roi_percentage();

        let total_txs = self.total_txs_sent.load(Ordering::Relaxed);
        let avg_tx = self.avg_tx_cost();

        let start_balance = self.start_sol_balance as f64 / 1_000_000_000.0;
        let current_balance = self.current_sol_balance.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;

        println!("\n╔═══════════════════════════════════════════════════╗");
        println!("║       CLOAK MINING EARNINGS REPORT                ║");
        println!("╠═══════════════════════════════════════════════════╣");
        println!("║ UPTIME                                            ║");
        println!("║   Runtime:           {:<8.2} hours ({:.2} days)   ", uptime_hours, uptime_days);
        println!("║                                                   ║");
        println!("║ BALANCE                                           ║");
        println!("║   Starting:          {:<12.6} SOL              ", start_balance);
        println!("║   Current:           {:<12.6} SOL              ", current_balance);
        println!("║   Net change:        {:<12.6} SOL ({:+.2}%)    ", net_change, roi);
        println!("║                                                   ║");
        println!("║ CLAIMS                                            ║");
        println!("║   Consumed:          {:<8}                       ", claims_consumed);
        println!("║   Gross fees:        {:<12.6} SOL              ", total_fees);
        println!("║   TX costs:          {:<12.6} SOL              ", total_costs);
        println!("║   Net fees:          {:<12.6} SOL              ", net_fees);
        println!("║                                                   ║");
        println!("║ EARNINGS RATE                                     ║");
        println!("║   Gross:             {:<12.6} SOL/hour         ", gross_per_hour);
        println!("║   Net:               {:<12.6} SOL/hour         ", net_per_hour);
        println!("║   Per claim:         {:<12.6} SOL              ", if claims_consumed > 0 { total_fees / claims_consumed as f64 } else { 0.0 });
        println!("║                                                   ║");
        println!("║ TRANSACTION COSTS                                 ║");
        println!("║   Total TXs:         {:<8}                       ", total_txs);
        println!("║   Total cost:        {:<12.6} SOL              ", total_costs);
        println!("║   Avg per TX:        {:<12.6} SOL              ", avg_tx);

        // Only show projections if we have enough data
        if uptime_hours > 0.5 && claims_consumed > 0 {
            println!("║                                                   ║");
            println!("║ PROJECTIONS (at current rate)                    ║");
            println!("║   Next 24h:          {:<12.6} SOL              ", net_per_hour * 24.0);
            println!("║   Next 7d:           {:<12.6} SOL              ", net_per_hour * 24.0 * 7.0);
            println!("║   Next 30d:          {:<12.6} SOL              ", net_per_hour * 24.0 * 30.0);

            // Profitability indicator
            let profitability_status = if net_per_hour > 0.001 {
                "🟢 PROFITABLE"
            } else if net_per_hour > 0.0 {
                "🟡 MARGINAL"
            } else {
                "🔴 UNPROFITABLE"
            };

            println!("║                                                   ║");
            println!("║ STATUS: {:<41} ║", profitability_status);
        } else {
            println!("║                                                   ║");
            println!("║ ⏳ Collecting data... (need more runtime/claims) ║");
        }

        println!("╚═══════════════════════════════════════════════════╝\n");
    }

    /// Get JSON representation of earnings data (for logging/export)
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "uptime_seconds": self.start_time.elapsed().as_secs(),
            "claims_consumed": self.claims_consumed.load(Ordering::Relaxed),
            "gross_fees_lamports": self.total_claim_fees_earned.load(Ordering::Relaxed),
            "tx_costs_lamports": self.total_tx_costs.load(Ordering::Relaxed),
            "net_balance_change_lamports": self.net_balance_change() * 1_000_000_000.0,
            "gross_earnings_per_hour_sol": self.gross_earnings_per_hour(),
            "net_earnings_per_hour_sol": self.net_earnings_per_hour(),
            "roi_percentage": self.roi_percentage(),
            "total_txs": self.total_txs_sent.load(Ordering::Relaxed),
        })
    }
}
```

##### Integration into `src/main.rs`

```rust
// Add to imports
use crate::earnings::EarningsTracker;

// In mine_continuously() function, after initializing MinerStats:

// Initialize earnings tracker
let earnings = Arc::new(
    EarningsTracker::new(rpc_url.to_string(), miner_pubkey)
        .await
        .context("Failed to initialize earnings tracker")?
);

info!("💰 Earnings tracking enabled");

// Spawn background task to update balances every 5 minutes
let earnings_clone = earnings.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    loop {
        interval.tick().await;
        if let Err(e) = earnings_clone.update_balance().await {
            warn!("Failed to update balance: {}", e);
        }
    }
});

// Print earnings report every 10 mining rounds (alongside stats)
if mining_round % 10 == 1 {
    stats.print_summary();
    earnings.print_earnings_report();  // NEW!
}

// TODO Phase 1: Track claim consumption
// This requires either:
// 1. Polling miner_pda.total_consumed every N minutes, OR
// 2. Subscribing to on-chain logs for claim_consumed events
// For now, we'll implement polling approach
```

##### Tracking Claim Consumption

We need to detect when our claims are consumed. Two approaches:

**Approach A: Polling Miner PDA** (simpler, implement first)

```rust
// In background task (spawn alongside balance updater)
let earnings_clone2 = earnings.clone();
let program_id_clone = program_id.clone();
let miner_pubkey_clone = miner_pubkey.clone();

tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60)); // 1 minute
    let mut last_consumed = 0u64;

    loop {
        interval.tick().await;

        // Derive miner PDA
        let (miner_pda, _) = derive_miner_pda(&program_id_clone, &miner_pubkey_clone);

        // Fetch miner account
        match client.get_account(&miner_pda) {
            Ok(account) => {
                if account.data.len() >= 48 {
                    // Parse total_consumed from offset 40 (u64 LE)
                    let total_consumed = u64::from_le_bytes(
                        account.data[40..48].try_into().unwrap()
                    );

                    // Check for new consumptions
                    if total_consumed > last_consumed {
                        let new_consumptions = total_consumed - last_consumed;

                        // Estimate fee per claim (will be inaccurate, but better than nothing)
                        // In Phase 2, we'll track this more precisely
                        let estimated_fee_per_claim = 1_000_000; // 0.001 SOL estimate

                        for _ in 0..new_consumptions {
                            earnings_clone2.record_claim_consumed(estimated_fee_per_claim);
                        }

                        last_consumed = total_consumed;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch miner account: {}", e);
            }
        }
    }
});
```

**Approach B: WebSocket Log Subscription** (future enhancement)

```rust
// Subscribe to logs mentioning our miner authority
// Parse claim_consumed events from logs
// More real-time but requires WebSocket support
```

##### CLI Additions (Backward Compatible)

```bash
# New optional flags (all existing flags work as before)
cloak-miner mine \
  --keypair ./miner.json \
  --earnings-report-interval 10  # Print report every N rounds (default: 10)
  --earnings-json-export ./earnings.json  # Export JSON every hour (optional)
```

#### Testing Plan

1. **Unit Tests**
   - Test balance change calculations
   - Test earnings rate formulas
   - Test ROI calculations
   - Test JSON export format

2. **Integration Tests**
   - Run miner for 10 minutes on devnet
   - Verify balance tracking works
   - Verify earnings report displays correctly
   - Test with actual claim consumptions

3. **Manual Testing**
   - Run on localnet with mock withdrawals
   - Verify reports match expected earnings
   - Test with multiple claim consumptions

#### Success Criteria

✅ Earnings tracker initializes without errors
✅ Balance updates every 5 minutes
✅ Claim consumption detection works (polling approach)
✅ Earnings report displays correctly every 10 rounds
✅ All existing functionality remains unchanged
✅ No performance degradation in mining
✅ Documentation updated with examples

#### Deliverables

- [ ] `src/earnings.rs` implementation
- [ ] Integration into `main.rs`
- [ ] Unit tests for earnings calculations
- [ ] Integration test on devnet
- [ ] Documentation in `README.md`
- [ ] Example earnings report screenshot

---

### Phase 2: Ore Mining Integration

**Goal**: Add Ore mining capability alongside Cloak, with manual mode switching.

**Status**: 🔜 **NEXT**

**Prerequisites**: Phase 1 complete

#### What We're Building

Integration of Ore CLI's mining engine into cloak-miner:
- Import Ore's Equix mining logic
- Add `--mode` flag: `cloak`, `ore`, or `auto`
- Track Ore token earnings separately
- Display side-by-side profitability comparison

#### Why This Matters

Miners want to **diversify revenue** and avoid putting all eggs in one basket. By supporting both protocols, we:
- Maximize hardware utilization (CPU cores)
- Provide fallback when one protocol is unprofitable
- Enable future profit-switching automation

#### Implementation Details

##### Step 1: Add Ore Dependencies

**Update `Cargo.toml`:**

```toml
[dependencies]
# ... existing dependencies ...

# Ore mining
ore-api = "3.6.0"
ore-boost-api = "4.0.0"
drillx = "2.1.0"
core_affinity = "0.8.1"
num_cpus = "1.16.0"
```

##### Step 2: Create Ore Mining Module

**New file: `src/ore_engine.rs`**

```rust
//! Ore mining engine integration
//!
//! This module wraps ore-cli's mining logic for use within cloak-miner.
//! We import the proven Equix algorithm from ore-cli rather than reimplementing.

use anyhow::Result;
use drillx::{equix, Hash, Solution};
use ore_api::{
    consts::{BUS_ADDRESSES, BUS_COUNT},
    state::{proof_pda, Bus, Config},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Ore mining result
#[derive(Debug)]
pub struct OreMiningResult {
    pub solution: Solution,
    pub difficulty: u32,
    pub hash_attempts: u64,
    pub mining_time: Duration,
}

/// Ore mining engine
pub struct OreEngine {
    rpc_client: RpcClient,
    miner_keypair: Keypair,
    cores: usize,
}

impl OreEngine {
    pub fn new(rpc_url: String, miner_keypair: Keypair, cores: usize) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url),
            miner_keypair,
            cores,
        }
    }

    /// Mine Ore using Equix algorithm (from ore-cli)
    ///
    /// This uses the same multi-threaded approach as ore-cli for maximum
    /// performance and compatibility.
    pub async fn mine(&self, timeout: Duration) -> Result<OreMiningResult> {
        info!("⛏️  Starting Ore mining (Equix, {} cores)", self.cores);

        // Fetch proof account
        let proof_address = proof_pda(self.miner_keypair.pubkey()).0;
        let proof = self.rpc_client.get_account(&proof_address)?;

        // TODO: Parse proof data and get challenge
        // This is simplified - actual implementation needs to match ore-cli

        let start_time = Instant::now();
        let solution = self.find_hash_parallel(
            challenge,
            timeout,
            min_difficulty,
        ).await?;

        let mining_time = start_time.elapsed();

        Ok(OreMiningResult {
            solution,
            difficulty,
            hash_attempts,
            mining_time,
        })
    }

    /// Parallel hash finding (from ore-cli's implementation)
    async fn find_hash_parallel(
        &self,
        challenge: [u8; 32],
        timeout: Duration,
        min_difficulty: u32,
    ) -> Result<Solution> {
        // Import ore-cli's parallel mining implementation
        // See: ore-cli/src/command/mine.rs:275-399

        // This would use:
        // - core_affinity for pinning threads to cores
        // - drillx::hashes_with_memory for Equix hashing
        // - Arc<RwLock<>> for shared best solution
        // - Timeout mechanism

        todo!("Implement parallel Equix mining from ore-cli")
    }

    /// Submit ore mining transaction
    pub async fn submit_solution(&self, solution: Solution) -> Result<String> {
        // Build and submit ore mining transaction
        // See: ore-cli mining submission logic

        todo!("Implement Ore transaction submission")
    }
}
```

**Note**: Instead of reimplementing, we should consider using ore-cli as a library dependency and calling its functions directly.

##### Step 3: Unified Mining Interface

**New file: `src/mining_mode.rs`**

```rust
//! Mining mode selection and strategy

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiningMode {
    /// Only mine Cloak claims
    Cloak,

    /// Only mine Ore tokens
    Ore,

    /// Automatically switch based on profitability
    Auto,
}

impl MiningMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "cloak" => Ok(Self::Cloak),
            "ore" => Ok(Self::Ore),
            "auto" => Ok(Self::Auto),
            _ => Err(anyhow::anyhow!(
                "Invalid mode: {}. Must be 'cloak', 'ore', or 'auto'",
                s
            )),
        }
    }
}

impl fmt::Display for MiningMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cloak => write!(f, "cloak"),
            Self::Ore => write!(f, "ore"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

/// Mining strategy coordinator
pub struct MiningCoordinator {
    mode: MiningMode,
    cloak_engine: crate::engine::MiningEngine,
    ore_engine: crate::ore_engine::OreEngine,
    earnings_tracker: Arc<crate::earnings::EarningsTracker>,
}

impl MiningCoordinator {
    pub fn new(
        mode: MiningMode,
        cloak_engine: crate::engine::MiningEngine,
        ore_engine: crate::ore_engine::OreEngine,
        earnings_tracker: Arc<crate::earnings::EarningsTracker>,
    ) -> Self {
        Self {
            mode,
            cloak_engine,
            ore_engine,
            earnings_tracker,
        }
    }

    /// Execute mining based on current mode
    pub async fn mine(&mut self) -> Result<()> {
        match self.mode {
            MiningMode::Cloak => self.mine_cloak().await,
            MiningMode::Ore => self.mine_ore().await,
            MiningMode::Auto => self.mine_optimal().await,
        }
    }

    async fn mine_cloak(&mut self) -> Result<()> {
        // Use existing Cloak mining logic
        todo!("Implement Cloak mining")
    }

    async fn mine_ore(&mut self) -> Result<()> {
        // Use Ore engine
        todo!("Implement Ore mining")
    }

    async fn mine_optimal(&mut self) -> Result<()> {
        // Profit-switching logic (Phase 3)
        todo!("Implement auto-switching")
    }
}
```

##### Step 4: CLI Updates

**Add to `main.rs` CLI args:**

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    Mine {
        // ... existing mine args ...

        /// Mining mode: cloak, ore, or auto (profit-switching)
        #[arg(long, default_value = "cloak", env = "MINING_MODE")]
        mode: String,

        /// Number of CPU cores to use (for Ore mining)
        #[arg(long, default_value = "ALL")]
        cores: String,
    },
}
```

##### Step 5: Enhanced Earnings Tracking

**Update `EarningsTracker` to support Ore:**

```rust
// Add to EarningsTracker struct
pub struct EarningsTracker {
    // ... existing fields ...

    // Ore mining metrics
    start_ore_balance: u64,
    current_ore_balance: AtomicU64,
    total_ore_mined: AtomicU64,
    ore_mining_successes: AtomicU64,
}

// Add methods
impl EarningsTracker {
    pub async fn update_ore_balance(&self) -> Result<()> {
        // Fetch ORE token balance from associated token account
        todo!("Implement ORE balance tracking")
    }

    pub fn record_ore_mined(&self, amount: u64) {
        self.total_ore_mined.fetch_add(amount, Ordering::Relaxed);
        self.ore_mining_successes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn ore_earnings_per_hour(&self) -> f64 {
        let elapsed_hours = self.start_time.elapsed().as_secs_f64() / 3600.0;
        if elapsed_hours == 0.0 {
            return 0.0;
        }

        let total_ore = self.total_ore_mined.load(Ordering::Relaxed);
        (total_ore as f64 / 1e11) / elapsed_hours // Adjust for ORE decimals
    }

    /// Print comparative earnings report
    pub fn print_dual_earnings_report(&self, ore_price_sol: f64) {
        let cloak_sol_per_hour = self.net_earnings_per_hour();
        let ore_tokens_per_hour = self.ore_earnings_per_hour();
        let ore_sol_per_hour = ore_tokens_per_hour * ore_price_sol;

        println!("\n╔═══════════════════════════════════════════════════╗");
        println!("║       DUAL MINING EARNINGS COMPARISON            ║");
        println!("╠═══════════════════════════════════════════════════╣");
        println!("║ CLOAK MINING                                      ║");
        println!("║   Claims consumed:   {:<8}                       ", self.claims_consumed.load(Ordering::Relaxed));
        println!("║   Earnings rate:     {:.6} SOL/hour             ", cloak_sol_per_hour);
        println!("║                                                   ║");
        println!("║ ORE MINING                                        ║");
        println!("║   ORE mined:         {:<8.2}                     ", ore_tokens_per_hour);
        println!("║   ORE price:         {:.6} SOL                  ", ore_price_sol);
        println!("║   Earnings rate:     {:.6} SOL/hour             ", ore_sol_per_hour);
        println!("║                                                   ║");
        println!("║ COMPARISON                                        ║");

        let better = if cloak_sol_per_hour > ore_sol_per_hour {
            "CLOAK"
        } else {
            "ORE"
        };

        let diff_pct = ((cloak_sol_per_hour - ore_sol_per_hour).abs()
            / cloak_sol_per_hour.max(ore_sol_per_hour)) * 100.0;

        println!("║   More profitable:   {} ({:+.1}%)              ", better, diff_pct);
        println!("╚═══════════════════════════════════════════════════╝\n");
    }
}
```

#### Testing Plan

1. **Unit Tests**
   - Test mode parsing
   - Test Ore mining engine (with mocked RPC)
   - Test dual earnings tracking

2. **Integration Tests**
   - Mine Cloak for 5 minutes, verify stats
   - Mine Ore for 5 minutes, verify stats
   - Switch between modes manually

3. **Compatibility Tests**
   - Verify Cloak-only mode matches Phase 1 behavior
   - Verify no regression in Cloak mining performance

#### Success Criteria

✅ Ore mining works with manual mode selection
✅ Earnings tracked separately for both protocols
✅ Dual earnings report displays correctly
✅ Manual mode switching works (`--mode cloak|ore`)
✅ No performance degradation vs. Phase 1
✅ Cloak-only mode still works as default

#### Deliverables

- [ ] `src/ore_engine.rs` implementation
- [ ] `src/mining_mode.rs` coordinator
- [ ] Enhanced earnings tracking for Ore
- [ ] Updated CLI with `--mode` flag
- [ ] Dual earnings report
- [ ] Integration tests on devnet
- [ ] Documentation updates

---

### Phase 3: Profit-Switching Automation

**Goal**: Automatically optimize mining strategy based on real-time profitability.

**Status**: 🔮 **FUTURE**

**Prerequisites**: Phase 1 & 2 complete

#### What We're Building

Intelligent strategy selector that:
- Compares Cloak vs. Ore profitability in real-time
- Auto-switches when profit delta exceeds threshold (e.g., 20%)
- Uses demand signals as tiebreaker
- Fetches ORE/SOL price from oracles
- Considers electricity costs (optional)

#### Why This Matters

Manual mode switching requires constant monitoring. Automation:
- Maximizes miner revenue 24/7
- Responds to market changes instantly
- Reduces operational overhead

#### Implementation Details

##### Step 1: Price Oracle

**New file: `src/price_oracle.rs`**

```rust
//! Price oracle for fetching ORE/SOL exchange rate

use anyhow::Result;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct PriceData {
    pub ore_per_sol: f64,
    pub timestamp: Instant,
}

pub struct PriceOracle {
    cached_price: std::sync::Mutex<Option<PriceData>>,
    cache_duration: Duration,
}

impl PriceOracle {
    pub fn new() -> Self {
        Self {
            cached_price: std::sync::Mutex::new(None),
            cache_duration: Duration::from_secs(60), // 1 minute cache
        }
    }

    /// Fetch ORE/SOL price with caching
    pub async fn get_ore_price_sol(&self) -> Result<f64> {
        // Check cache
        {
            let cached = self.cached_price.lock().unwrap();
            if let Some(price_data) = cached.as_ref() {
                if price_data.timestamp.elapsed() < self.cache_duration {
                    return Ok(price_data.ore_per_sol);
                }
            }
        }

        // Fetch fresh price
        let price = self.fetch_price_from_jupiter().await
            .or_else(|_| self.fetch_price_from_birdeye().await)
            .or_else(|_| self.fetch_price_from_raydium().await)?;

        // Update cache
        {
            let mut cached = self.cached_price.lock().unwrap();
            *cached = Some(PriceData {
                ore_per_sol: price,
                timestamp: Instant::now(),
            });
        }

        Ok(price)
    }

    async fn fetch_price_from_jupiter(&self) -> Result<f64> {
        // Jupiter Price API v2
        let url = "https://price.jup.ag/v6/price?ids=ORE";

        #[derive(Deserialize)]
        struct JupiterResponse {
            data: serde_json::Value,
        }

        let response: JupiterResponse = reqwest::get(url)
            .await?
            .json()
            .await?;

        let price = response.data["ORE"]["price"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse price"))?;

        info!("📊 Fetched ORE price from Jupiter: {} SOL", price);
        Ok(price)
    }

    async fn fetch_price_from_birdeye(&self) -> Result<f64> {
        // Birdeye API (requires API key)
        // Alternative price source
        todo!("Implement Birdeye price fetching")
    }

    async fn fetch_price_from_raydium(&self) -> Result<f64> {
        // On-chain TWAP from Raydium pool
        // Most reliable but slowest
        todo!("Implement Raydium on-chain price fetching")
    }
}
```

##### Step 2: Profitability Comparator

**New file: `src/profitability.rs`**

```rust
//! Profitability analysis and comparison

use crate::{earnings::EarningsTracker, price_oracle::PriceOracle};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum ProfitabilityRecommendation {
    /// Continue mining Cloak
    Cloak { advantage_pct: f64 },

    /// Switch to mining Ore
    Ore { advantage_pct: f64 },

    /// Both are roughly equal
    Neutral,
}

pub struct ProfitabilityAnalyzer {
    earnings: Arc<EarningsTracker>,
    price_oracle: PriceOracle,

    /// Minimum profit advantage to trigger switch (e.g., 20%)
    switch_threshold: f64,
}

impl ProfitabilityAnalyzer {
    pub fn new(
        earnings: Arc<EarningsTracker>,
        switch_threshold: f64,
    ) -> Self {
        Self {
            earnings,
            price_oracle: PriceOracle::new(),
            switch_threshold,
        }
    }

    /// Analyze and recommend mining strategy
    pub async fn analyze(&self) -> anyhow::Result<ProfitabilityRecommendation> {
        // Get Cloak earnings rate (SOL/hour)
        let cloak_sol_per_hour = self.earnings.net_earnings_per_hour();

        // Get Ore earnings rate (tokens/hour)
        let ore_tokens_per_hour = self.earnings.ore_earnings_per_hour();

        // Convert Ore to SOL
        let ore_price_sol = self.price_oracle.get_ore_price_sol().await?;
        let ore_sol_per_hour = ore_tokens_per_hour * ore_price_sol;

        // Compare with hysteresis
        if cloak_sol_per_hour > ore_sol_per_hour * (1.0 + self.switch_threshold) {
            let advantage = ((cloak_sol_per_hour - ore_sol_per_hour) / ore_sol_per_hour) * 100.0;
            Ok(ProfitabilityRecommendation::Cloak { advantage_pct: advantage })
        } else if ore_sol_per_hour > cloak_sol_per_hour * (1.0 + self.switch_threshold) {
            let advantage = ((ore_sol_per_hour - cloak_sol_per_hour) / cloak_sol_per_hour) * 100.0;
            Ok(ProfitabilityRecommendation::Ore { advantage_pct: advantage })
        } else {
            Ok(ProfitabilityRecommendation::Neutral)
        }
    }
}
```

##### Step 3: Auto-Switching Logic

**Update `mining_mode.rs`:**

```rust
impl MiningCoordinator {
    async fn mine_optimal(&mut self) -> Result<()> {
        info!("🤖 Auto mode: analyzing profitability...");

        let analyzer = ProfitabilityAnalyzer::new(
            self.earnings_tracker.clone(),
            0.20, // 20% threshold
        );

        let recommendation = analyzer.analyze().await?;

        match recommendation {
            ProfitabilityRecommendation::Cloak { advantage_pct } => {
                info!(
                    "💰 Cloak is more profitable ({:+.1}% advantage), mining claims...",
                    advantage_pct
                );

                // Check for demand as additional signal
                if self.has_cloak_demand().await {
                    info!("📦 Demand detected, prioritizing Cloak");
                }

                self.mine_cloak().await
            }

            ProfitabilityRecommendation::Ore { advantage_pct } => {
                info!(
                    "💰 Ore is more profitable ({:+.1}% advantage), mining ORE...",
                    advantage_pct
                );
                self.mine_ore().await
            }

            ProfitabilityRecommendation::Neutral => {
                info!("⚖️  Profitability is neutral, using demand as tiebreaker");

                // Use Cloak demand as tiebreaker
                if self.has_cloak_demand().await {
                    info!("📦 Cloak demand exists, mining claims");
                    self.mine_cloak().await
                } else {
                    info!("🪙 No Cloak demand, mining ORE");
                    self.mine_ore().await
                }
            }
        }
    }

    async fn has_cloak_demand(&self) -> bool {
        // Check relay backlog
        match check_relay_demand(&self.relay_url).await {
            Ok((has_demand, _)) => has_demand,
            Err(_) => false,
        }
    }
}
```

##### Step 4: Advanced Features (Optional)

**Electricity Cost Consideration:**

```rust
pub struct CostAnalyzer {
    electricity_cost_per_kwh: f64, // USD
    miner_power_watts: f64,
    sol_price_usd: f64,
}

impl CostAnalyzer {
    pub fn net_profit_sol_per_hour(
        &self,
        gross_earnings_sol_per_hour: f64,
    ) -> f64 {
        let power_kwh = self.miner_power_watts / 1000.0;
        let electricity_cost_usd_per_hour = power_kwh * self.electricity_cost_per_kwh;
        let electricity_cost_sol_per_hour = electricity_cost_usd_per_hour / self.sol_price_usd;

        gross_earnings_sol_per_hour - electricity_cost_sol_per_hour
    }
}
```

**Difficulty-Aware Mining:**

```rust
// Prefer mining when difficulty is low
let cloak_difficulty = fetch_registry_difficulty().await?;
let ore_difficulty = fetch_ore_difficulty().await?;

// Adjust profitability by difficulty
let cloak_score = cloak_sol_per_hour / cloak_difficulty_factor;
let ore_score = ore_sol_per_hour / ore_difficulty_factor;
```

#### Testing Plan

1. **Unit Tests**
   - Test price oracle caching
   - Test profitability calculations
   - Test switching logic with various scenarios

2. **Integration Tests**
   - Run in auto mode for 1 hour on devnet
   - Verify switches occur correctly
   - Test with simulated price changes

3. **Stress Tests**
   - Rapid price fluctuations
   - Relay downtime
   - RPC failures

#### Success Criteria

✅ Auto-switching works reliably
✅ Price fetching is resilient (multiple sources)
✅ Hysteresis prevents excessive switching
✅ Demand tiebreaker works as expected
✅ Performance remains stable
✅ Earnings are correctly attributed

#### Deliverables

- [ ] `src/price_oracle.rs` implementation
- [ ] `src/profitability.rs` analyzer
- [ ] Auto-switching logic in coordinator
- [ ] Optional electricity cost tracking
- [ ] Comprehensive integration tests
- [ ] Performance benchmarks
- [ ] User guide for auto mode

---

## Technical Specifications

### Data Structures

#### EarningsTracker State

```rust
struct EarningsTracker {
    // Balances
    start_sol_balance: u64,
    current_sol_balance: AtomicU64,
    start_ore_balance: u64,
    current_ore_balance: AtomicU64,

    // Cloak metrics
    claims_consumed: AtomicU64,
    total_claim_fees_earned: AtomicU64,

    // Ore metrics
    total_ore_mined: AtomicU64,
    ore_mining_successes: AtomicU64,

    // Transaction costs
    total_tx_costs: AtomicU64,
    total_txs_sent: AtomicU64,

    // Timing
    start_time: Instant,
    last_balance_update: Mutex<Instant>,
}
```

#### Mining Mode State Machine

```
        ┌─────────┐
        │  Start  │
        └────┬────┘
             │
             v
    ┌────────────────┐
    │  Parse CLI     │
    │  --mode flag   │
    └────┬───────────┘
         │
         v
    Is mode "auto"?
         │
    ┌────┴────┐
    │         │
   YES       NO
    │         │
    v         v
┌───────┐  ┌──────────┐
│ Auto  │  │ Manual   │
│ Mode  │  │ Mode     │
└───┬───┘  └────┬─────┘
    │           │
    v           v
Analyze    Use fixed
profit     strategy
    │           │
    v           v
┌───────┐  ┌──────────┐
│ Cloak │  │   Ore    │
│ Engine│  │  Engine  │
└───────┘  └──────────┘
```

### API Integrations

#### Relay Backlog API

```
GET http://localhost:3002/backlog

Response:
{
  "pending_count": 5,
  "queued_jobs": [
    "uuid-1",
    "uuid-2",
    ...
  ]
}
```

#### Jupiter Price API

```
GET https://price.jup.ag/v6/price?ids=ORE

Response:
{
  "data": {
    "ORE": {
      "id": "ORE",
      "price": 0.0012345,  // in SOL
      "type": "derivedPrice"
    }
  }
}
```

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cloak hash rate (single-threaded) | ≥5M H/s | `engine::mine()` |
| Cloak hash rate (8 cores) | ≥40M H/s | `engine::mine_parallel()` |
| Ore hash rate (8 cores) | As per ore-cli | `ore_engine::mine()` |
| Balance check frequency | 5 min | Background task |
| Claim consumption check | 1 min | Background task |
| Price oracle cache | 1 min | `PriceOracle` |
| Memory usage | <100MB | Runtime monitoring |
| Mode switch delay | <5 sec | Auto mode transition |

### Error Handling

#### Recoverable Errors
- RPC connection failures → retry with backoff
- Balance check failures → log warning, continue
- Price fetch failures → use cached/fallback price
- Mining timeout → try next round

#### Fatal Errors
- Keypair load failure → exit with message
- Invalid program ID → exit with message
- Insufficient SOL for TXs → warn and exit
- Corrupted state → exit (future: recovery)

---

## Challenges & Solutions

### Challenge 1: Tracking Claim Consumption

**Problem**: Miner doesn't know when their claims are consumed by the relay.

**Solution A (Phase 1)**: Poll `miner_pda.total_consumed` every minute
- ✅ Simple to implement
- ✅ Works with existing RPC
- ⚠️ Delayed feedback (~1 min lag)
- ⚠️ Doesn't know fee amount per claim

**Solution B (Future)**: WebSocket log subscription
- ✅ Real-time notifications
- ✅ Can parse fee from event
- ⚠️ Requires WebSocket support
- ⚠️ More complex error handling

**Implementation**:
```rust
// Polling approach (Phase 1)
tokio::spawn(async move {
    let mut last_consumed = 0u64;
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        let account = rpc_client.get_account(&miner_pda).await?;
        let total_consumed = parse_total_consumed(&account.data);

        if total_consumed > last_consumed {
            let new_consumptions = total_consumed - last_consumed;
            for _ in 0..new_consumptions {
                earnings.record_claim_consumed(estimated_fee);
            }
            last_consumed = total_consumed;
        }
    }
});
```

### Challenge 2: Fee Share Calculation

**Problem**: How much SOL did each claim earn?

**Context**:
- Relay charges users a withdrawal fee (e.g., 1% = 100 bps)
- Registry config has `fee_share_bps` (e.g., 20% = 2000 bps)
- Miner earns: `withdrawal_fee * (fee_share_bps / 10000)`

**Solution**: Parse registry config and estimate fee per claim
```rust
async fn estimate_fee_per_claim(
    registry_pda: &Pubkey,
    avg_withdrawal_amount: u64,
) -> Result<u64> {
    // Fetch registry
    let registry = fetch_registry(&rpc_client, registry_pda).await?;
    let fee_share_bps = registry.fee_share_bps;

    // Assume relay charges 1% (100 bps)
    let relay_fee_bps = 100;
    let relay_fee = (avg_withdrawal_amount * relay_fee_bps) / 10_000;

    // Miner gets fee_share_bps of relay fee
    let miner_fee = (relay_fee * fee_share_bps) / 10_000;

    Ok(miner_fee)
}
```

### Challenge 3: Algorithm Performance Differences

**Problem**: Cloak uses BLAKE3, Ore uses Equix. Different CPUs perform differently.

**Solution**: Benchmark both algorithms on startup
```rust
async fn benchmark_mining_performance() -> BenchmarkResults {
    println!("🔬 Running mining benchmarks...");

    // Benchmark Cloak (BLAKE3)
    let cloak_start = Instant::now();
    let cloak_attempts = benchmark_cloak_hashing(Duration::from_secs(10));
    let cloak_hashrate = cloak_attempts as f64 / cloak_start.elapsed().as_secs_f64();

    // Benchmark Ore (Equix)
    let ore_start = Instant::now();
    let ore_attempts = benchmark_ore_hashing(Duration::from_secs(10));
    let ore_hashrate = ore_attempts as f64 / ore_start.elapsed().as_secs_f64();

    println!("Benchmark Results:");
    println!("  Cloak (BLAKE3): {:.0} H/s", cloak_hashrate);
    println!("  Ore (Equix):    {:.0} H/s", ore_hashrate);

    BenchmarkResults {
        cloak_hashrate,
        ore_hashrate,
        cloak_advantage: cloak_hashrate / ore_hashrate,
    }
}
```

### Challenge 4: Ore CLI Integration

**Problem**: Ore CLI is a binary, not a library. How do we reuse its code?

**Options**:

**Option A**: Import ore-cli as dependency and use its modules
- ✅ Reuse proven code
- ✅ Get updates automatically
- ⚠️ May have tight coupling
- ⚠️ Dependency maintenance

**Option B**: Copy/adapt relevant mining code
- ✅ Full control
- ✅ Can optimize for our use case
- ⚠️ Need to maintain fork
- ⚠️ Miss upstream improvements

**Recommendation**: Start with Option A (dependency), consider Option B if needed.

```toml
# Cargo.toml
[dependencies]
# Import ore-cli as library dependency
ore-cli = { git = "https://github.com/regolith-labs/ore-cli", branch = "main" }
```

### Challenge 5: Backward Compatibility

**Problem**: Existing Cloak users must not experience any breaking changes.

**Solution**: Default to Cloak-only mode, make all new features opt-in
```rust
// CLI defaults preserve existing behavior
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "cloak")]  // ← Default to Cloak
    mode: String,

    #[arg(long, default_value = "true")]   // ← Earnings tracking on by default
    earnings_tracking: bool,

    // ... other args
}

// Ensure Cloak mining logic is unchanged
if mode == MiningMode::Cloak {
    // Use EXACT same code path as before Phase 1
    mine_cloak_legacy().await?;
}
```

### Challenge 6: Price Oracle Reliability

**Problem**: Price APIs can fail or return stale data.

**Solution**: Multi-source fallback with caching
```rust
impl PriceOracle {
    async fn get_price_with_fallback(&self) -> Result<f64> {
        // Try sources in order of preference
        self.fetch_from_jupiter().await
            .or_else(|_| {
                warn!("Jupiter failed, trying Birdeye...");
                self.fetch_from_birdeye().await
            })
            .or_else(|_| {
                warn!("Birdeye failed, trying on-chain...");
                self.fetch_from_raydium_onchain().await
            })
            .or_else(|_| {
                warn!("All price sources failed, using cached price");
                self.get_cached_price()
            })
    }
}
```

---

## Testing Strategy

### Unit Tests

#### Phase 1 Tests
- [ ] `test_earnings_balance_change()`
- [ ] `test_earnings_rate_calculation()`
- [ ] `test_roi_percentage()`
- [ ] `test_claim_consumption_tracking()`
- [ ] `test_tx_cost_tracking()`

#### Phase 2 Tests
- [ ] `test_mining_mode_parsing()`
- [ ] `test_ore_engine_initialization()`
- [ ] `test_dual_earnings_tracking()`
- [ ] `test_coordinator_mode_switching()`

#### Phase 3 Tests
- [ ] `test_price_oracle_caching()`
- [ ] `test_profitability_comparison()`
- [ ] `test_switching_hysteresis()`
- [ ] `test_demand_tiebreaker()`

### Integration Tests

#### Localnet Tests
```bash
# Setup
./scripts/setup-localnet.sh

# Test Cloak mining
cargo test --test integration_cloak_mining -- --ignored

# Test Ore mining
cargo test --test integration_ore_mining -- --ignored

# Test auto-switching
cargo test --test integration_auto_mode -- --ignored
```

#### Devnet Tests
```bash
# Run miner for 10 minutes
RUST_LOG=info cargo run --release -- \
  --network devnet \
  --keypair ./test-miner.json \
  mine --mode auto --timeout 30

# Verify earnings tracking
# Verify mode switching occurs
# Check logs for errors
```

### Performance Benchmarks

```rust
#[bench]
fn bench_cloak_mining_single_thread(b: &mut Bencher) {
    b.iter(|| {
        let engine = MiningEngine::new(/* ... */);
        engine.mine_with_timeout(Duration::from_secs(1))
    });
}

#[bench]
fn bench_cloak_mining_multi_thread(b: &mut Bencher) {
    b.iter(|| {
        let engine = MiningEngine::new(/* ... */);
        engine.mine_parallel(8, Duration::from_secs(1))
    });
}

#[bench]
fn bench_earnings_update(b: &mut Bencher) {
    let earnings = EarningsTracker::new(/* ... */);
    b.iter(|| {
        earnings.update_balance()
    });
}
```

### Manual Testing Checklist

#### Phase 1
- [ ] Run for 1 hour, verify balance tracking
- [ ] Submit test withdrawals, verify claim consumption
- [ ] Check earnings report formatting
- [ ] Test with low balance (near minimum)
- [ ] Test Ctrl-C graceful shutdown

#### Phase 2
- [ ] Mine Cloak for 30 min, check stats
- [ ] Mine Ore for 30 min, check stats
- [ ] Switch between modes manually
- [ ] Verify dual earnings report
- [ ] Test invalid mode handling

#### Phase 3
- [ ] Run auto mode for 2 hours
- [ ] Simulate price changes (mock)
- [ ] Verify switching logic
- [ ] Test with relay downtime
- [ ] Check profitability reports

---

## Success Criteria

### Phase 1 Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| Backward compatibility | Cloak mining behavior | 100% identical |
| Earnings tracking | Balance check success rate | >99% |
| Claim detection | Detection latency | <2 minutes |
| Performance | Mining hash rate | No degradation |
| Memory usage | RSS | <50MB increase |
| Report quality | User satisfaction | Positive feedback |

### Phase 2 Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| Ore integration | Mining success rate | >95% |
| Mode switching | Manual switch latency | <5 seconds |
| Dual tracking | Accuracy vs. on-chain | >99% |
| Compatibility | Cloak-only mode | Still works |
| Documentation | Coverage | All features |

### Phase 3 Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| Auto-switching | Uptime without intervention | >24 hours |
| Price accuracy | Oracle vs. market | <1% deviation |
| Profitability | Earnings vs. manual mode | ≥10% improvement |
| Reliability | Error recovery rate | >95% |
| User adoption | Auto mode usage | >50% of miners |

---

## References

### External Resources

- **Ore Protocol**: https://ore.supply/
- **Ore CLI GitHub**: https://github.com/regolith-labs/ore-cli
- **NiceHash Profit Switching**: https://www.nicehash.com/profitability-calculator
- **Jupiter Price API**: https://station.jup.ag/docs/apis/price-api
- **Solana RPC Methods**: https://docs.solana.com/api/http

### Internal Documentation

- **Cloak Architecture**: `docs/offchain/overview.md`
- **Scramble Registry**: `programs/scramble-registry/README.md`
- **Shield Pool**: `programs/shield-pool/README.md`
- **Relay Service**: `docs/offchain/relay.md`
- **Current Miner**: `packages/cloak-miner/README.md`

### Related Discussions

- Initial dual-mining concept discussion (2025-10-30)
- Ore mining integration feasibility analysis
- Profitability tracking requirements

---

## Appendix: Command Reference

### Phase 1 Commands

```bash
# Standard Cloak mining (backward compatible)
cloak-miner --keypair ./miner.json mine

# With earnings tracking (default)
cloak-miner --keypair ./miner.json mine \
  --earnings-report-interval 10

# Export earnings to JSON
cloak-miner --keypair ./miner.json mine \
  --earnings-json-export ./earnings.json
```

### Phase 2 Commands

```bash
# Cloak-only mode (explicit)
cloak-miner --keypair ./miner.json mine --mode cloak

# Ore-only mode
cloak-miner --keypair ./miner.json mine --mode ore --cores 8

# View dual earnings report
cloak-miner --keypair ./miner.json status --dual
```

### Phase 3 Commands

```bash
# Auto profit-switching
cloak-miner --keypair ./miner.json mine --mode auto

# With custom switch threshold
cloak-miner --keypair ./miner.json mine \
  --mode auto \
  --switch-threshold 0.15  # 15% profit advantage

# With electricity cost consideration
cloak-miner --keypair ./miner.json mine \
  --mode auto \
  --electricity-kwh 0.12 \  # USD per kWh
  --power-watts 150
```

---

## Change Log

| Date | Version | Changes |
|------|---------|---------|
| 2025-10-30 | 0.1.0 | Initial design document |

---

**Document Status**: ✅ Ready for Implementation
**Next Steps**: Begin Phase 1 implementation with `src/earnings.rs`
