---
title: Anonymity Set Strategy
description: Comprehensive analysis of Cloak's anonymity set bootstrapping, growth strategy, and privacy guarantees at different pool sizes.
---

# Anonymity Set Strategy

**⚠️ Critical Privacy Consideration:** An empty or small anonymity set provides NO privacy. This document explains how Cloak addresses the cold-start problem and maintains meaningful privacy guarantees.

## Understanding the Anonymity Set Problem

### What is an Anonymity Set?

The **anonymity set** is the group of users/deposits among which your transaction is indistinguishable. For privacy protocols:

- **Anonymity set of 1** = Zero privacy (you know exactly who withdrew)
- **Anonymity set of 10** = Weak privacy (10 possible sources)
- **Anonymity set of 1,000** = Strong privacy (1,000 possible sources)
- **Anonymity set of 100,000+** = Robust privacy

### The Cold-Start Problem

**Every privacy protocol faces this challenge:**

1. **Day 0:** Pool is empty → No privacy possible
2. **Day 1-7:** First 10 users → Extremely weak privacy (10-anonymity)
3. **Week 1-4:** 100-1,000 users → Weak privacy
4. **Month 1-3:** 1,000-10,000 users → Moderate privacy
5. **Month 6+:** 10,000+ users → Strong privacy

**The Question:** How do we bootstrap from 0 to meaningful privacy?

## Cloak's Bootstrap Strategy

### Phase 1: Testnet Launch (Current)

**Approach: Transparency About Limitations**

```
┌─────────────────────────────────────────────┐
│ TESTNET WARNING                             │
│                                             │
│ Current anonymity set: ~10-50 deposits     │
│ Privacy level: EXPERIMENTAL                 │
│                                             │
│ DO NOT USE FOR PRODUCTION PRIVACY          │
└─────────────────────────────────────────────┘
```

**Actions:**
- ✅ Explicit warnings in UI about current set size
- ✅ Public dashboard showing real-time anonymity metrics
- ✅ Education about what "weak privacy" means
- ✅ Testnet-only operations until threshold reached

**Why This Matters:**
Most "privacy" projects hide this information or claim privacy with 10 users. We don't.

### Phase 2: Liquidity Mining (Pre-Launch)

**Approach: Incentivized Bootstrapping**

**Liquidity Mining Program:**
```rust
pub struct BootstrapIncentives {
    // Early depositors earn rewards for providing privacy
    pub base_reward_per_epoch: u64,
    
    // Rewards scale inversely with anonymity set size
    // Higher rewards when privacy is weak (need more users)
    pub reward_multiplier: fn(anonymity_set_size: u64) -> f64,
    
    // Minimum lock period to prevent quick entry/exit
    pub min_lock_period: u64, // 30 days recommended
    
    // Total bootstrap fund allocation
    pub total_allocation: u64, // e.g., 10M tokens
}

// Reward formula
pub fn calculate_bootstrap_reward(
    deposit_amount: u64,
    anonymity_set_size: u64,
    lock_duration: u64,
) -> u64 {
    let base_reward = deposit_amount * BASE_APR / 100;
    
    // Multiplier decreases as anonymity set grows
    let multiplier = match anonymity_set_size {
        0..=100 => 10.0,      // 10x rewards for first 100
        101..=500 => 5.0,     // 5x for next 400
        501..=2000 => 2.0,    // 2x for next 1500
        2001..=10000 => 1.5,  // 1.5x for next 8000
        _ => 1.0,             // Normal rewards after 10k
    };
    
    // Lock duration bonus
    let lock_bonus = (lock_duration / 30) as f64 * 0.1; // +10% per 30 days
    
    (base_reward as f64 * multiplier * (1.0 + lock_bonus)) as u64
}
```

**Key Design Principles:**
1. **Higher rewards when privacy is weak** - Incentivizes early bootstrapping
2. **Lock periods prevent quick exit** - Maintains anonymity set stability
3. **Decreasing rewards as set grows** - Self-regulating mechanism
4. **Transparent metrics** - Users see current anonymity before participating

### Phase 3: Coordinated Launch

**Approach: Batch Launch with Minimum Threshold**

**Launch Criteria:**
```rust
pub struct LaunchReadiness {
    // MINIMUM criteria for mainnet launch
    pub min_committed_deposits: u64,      // Target: 1,000+ deposits
    pub min_unique_depositors: u64,       // Target: 500+ unique users
    pub min_total_value_locked: u64,      // Target: 100,000 SOL
    pub min_time_diversity: u64,          // Target: 30+ days of activity
    
    // Requires ALL criteria to be met
    pub launch_approved: bool,
}
```

**Coordinated Launch Process:**

1. **Pre-commitment Phase (60 days)**
   - Users commit deposits but funds aren't deployed yet
   - Build up critical mass before going live
   - No privacy risk because pool isn't active

2. **Threshold Check**
   - Must reach minimum 1,000 pre-committed deposits
   - If threshold not met, extend pre-commitment period
   - Transparent progress tracking

3. **Synchronized Launch**
   - All pre-committed deposits go live simultaneously
   - Instant anonymity set of 1,000+ from day one
   - Clear "mainnet privacy guarantees active" announcement

4. **Post-Launch Growth**
   - Continue liquidity mining for 6-12 months
   - Target: 10,000+ deposits within first 6 months
   - Progressive fee reductions as set grows

### Phase 4: Sustainable Growth

**Approach: Network Effects and Utility**

**Growth Mechanisms:**

1. **DeFi Integration**
   - Partner protocols integrate Cloak deposits
   - Natural demand from DeFi privacy needs
   - Composability creates sticky deposits

2. **Fixed Denomination Pools**
   ```rust
   pub enum FixedDenomination {
       Tier1 = 100_000_000,      // 0.1 SOL
       Tier2 = 500_000_000,      // 0.5 SOL  
       Tier3 = 1_000_000_000,    // 1.0 SOL
       Tier4 = 5_000_000_000,    // 5.0 SOL
       Tier5 = 10_000_000_000,   // 10.0 SOL
   }
   
   // Each tier maintains separate anonymity set
   // Prevents amount-based linking
   ```

3. **Privacy Metrics Dashboard**
   ```typescript
   interface PublicPrivacyMetrics {
     // Real-time anonymity set sizes
     anonymitySetByDenomination: Map<Denomination, number>;
     
     // Time-based metrics
     avgTimeBetweenDepositWithdraw: number; // Days
     medianHoldTime: number; // Days
     
     // Diversity metrics  
     uniqueDepositorsLast30Days: number;
     geographicDistribution: number; // Estimate via relay diversity
     
     // Velocity metrics (IMPORTANT)
     depositWithdrawRatio: number; // Should be close to 1.0
     quickExitRate: number; // % of funds exiting within 24hrs
     
     // Privacy strength estimate
     estimatedPrivacyBits: number; // log2(anonymity_set)
   }
   ```

## Understanding Privacy vs. TVL/Volume

### ⚠️ WARNING: High Volume ≠ Strong Privacy

**Common Misconception:** "We have $100M volume in 24 hours!"

**Reality:** High volume with high turnover DESTROYS privacy.

### The Volume Trap

```
Example: Bad "Privacy" Protocol
┌────────────────────────────────────────┐
│ TVL: $50M                              │
│ 24hr Volume: $100M                     │
│ Unique Users: 1,000                    │
│                                        │
│ Reality Check:                         │
│ - Average user exits within 2 hours    │
│ - Timing correlation is EASY           │
│ - Anonymity set is WEAK (10-50)       │
│                                        │
│ Privacy Rating: ⭐ POOR                 │
└────────────────────────────────────────┘
```

### What Actually Matters

**Cloak's Target Metrics (Mainnet Launch):**

```
┌────────────────────────────────────────┐
│ Target: Meaningful Privacy             │
├────────────────────────────────────────┤
│ ✓ Active deposits: 10,000+             │
│ ✓ Median hold time: 7+ days            │
│ ✓ Quick exit rate: <10%                │
│ ✓ Deposit/withdraw ratio: 0.9-1.1      │
│ ✓ Time diversity: Uniform distribution │
│                                        │
│ NOT focusing on:                       │
│ ✗ High 24hr volume                     │
│ ✗ Quick turnover                       │
│ ✗ "Mixer" speed metrics                │
│                                        │
│ Privacy Rating: ⭐⭐⭐⭐ STRONG          │
└────────────────────────────────────────┘
```

### Velocity vs. Privacy

**The Trade-off:**

| Metric | Mixer Model | Privacy Pool Model (Cloak) |
|--------|-------------|----------------------------|
| **Average Hold Time** | Minutes to hours | Days to weeks |
| **Quick Exit Rate** | 80-90% | <10% target |
| **Anonymity Set Size** | 10-100 | 1,000-100,000 |
| **Timing Correlation** | Easy | Hard |
| **Amount Correlation** | Easy | Moderate (mitigated by tiers) |
| **Network Analysis Resistance** | Weak | Strong |

**Why This Matters:**

```rust
// Timing correlation attack example

// BAD: Mixer model
pub struct UserFlow {
    deposit_time: i64,     // 2024-11-02 14:23:45
    withdraw_time: i64,    // 2024-11-02 14:29:12
    time_diff: i64,        // 5 minutes 27 seconds
}
// Anonymity set of users who deposited around same time: ~5-10
// Easy to correlate

// GOOD: Privacy pool model (Cloak target)
pub struct UserFlow {
    deposit_time: i64,     // 2024-11-02 14:23:45
    withdraw_time: i64,    // 2024-11-15 09:15:33
    time_diff: i64,        // 12 days 18 hours
}
// Anonymity set of users who deposited in that window: 1,000+
// Hard to correlate
```

## Privacy Guarantees by Phase

### Honest Assessment of Current State

```rust
pub enum PrivacyLevel {
    None {
        description: "No privacy - pool too small",
        min_anonymity_set: 0,
        max_anonymity_set: 10,
        recommendation: "DO NOT USE for real privacy needs",
    },
    Weak {
        description: "Experimental privacy - vulnerable to timing correlation",
        min_anonymity_set: 10,
        max_anonymity_set: 100,
        recommendation: "Testnet only - Educational purposes",
    },
    Moderate {
        description: "Limited privacy - resistant to casual analysis",
        min_anonymity_set: 100,
        max_anonymity_set: 1_000,
        recommendation: "Early mainnet - Use with caution",
    },
    Strong {
        description: "Robust privacy - resistant to sophisticated analysis",
        min_anonymity_set: 1_000,
        max_anonymity_set: 10_000,
        recommendation: "Production-ready for most use cases",
    },
    VeryStrong {
        description: "High privacy - resistant to advanced correlation attacks",
        min_anonymity_set: 10_000,
        max_anonymity_set: u64::MAX,
        recommendation: "Production-ready for sensitive applications",
    },
}

// Current status function
pub fn current_privacy_level(
    anonymity_set_size: u64,
    median_hold_time_hours: u64,
    quick_exit_rate: f64,
) -> PrivacyLevel {
    match (anonymity_set_size, median_hold_time_hours, quick_exit_rate) {
        (0..=10, _, _) => PrivacyLevel::None,
        (11..=100, _, _) => PrivacyLevel::Weak,
        (101..=1000, h, q) if h < 24 || q > 0.3 => PrivacyLevel::Weak,
        (101..=1000, _, _) => PrivacyLevel::Moderate,
        (1001..=10000, h, q) if h < 48 || q > 0.2 => PrivacyLevel::Moderate,
        (1001..=10000, _, _) => PrivacyLevel::Strong,
        (10001.., h, q) if h < 72 || q > 0.15 => PrivacyLevel::Strong,
        (10001.., _, _) => PrivacyLevel::VeryStrong,
    }
}
```

### Transparent Privacy Dashboard

**Required UI Elements for Mainnet:**

```typescript
// Public dashboard showing REAL privacy metrics
interface PrivacyDashboard {
  // Core metrics
  currentAnonymitySet: number;
  privacyLevel: "None" | "Weak" | "Moderate" | "Strong" | "VeryStrong";
  estimatedPrivacyBits: number; // log2(anonymity_set)
  
  // Time-based security
  medianHoldTime: number; // Days
  avgTimeBetweenOps: number; // Hours
  quickExitRate: number; // Percentage
  
  // User warnings
  warnings: {
    lowAnonymitySet: boolean;
    highQuickExitRate: boolean;
    recentDeposit: boolean; // Warn if user deposited <24hrs ago
    unusualAmount: boolean; // Warn if amount is unique
  };
  
  // Recommendations
  recommendations: {
    suggestedWaitTime: number; // Hours to wait before withdraw
    suggestedAmounts: number[]; // Common denominations
    privacyScore: number; // 0-100
  };
}
```

## Attack Resistance Analysis

### Timing Correlation Attacks

**Attack:** Link deposits to withdrawals based on timing patterns.

**Cloak Defenses:**

1. **Long Hold Times**
   - Target median: 7+ days
   - Large time windows blur correlations
   - Random delays in relay processing

2. **Batch Processing**
   - Relay can batch multiple withdrawals
   - Processes at irregular intervals
   - Not first-in-first-out

3. **User Education**
   ```typescript
   // UI warning system
   if (timeSinceDeposit < 24 * 60 * 60 * 1000) {
     showWarning({
       severity: "high",
       message: "Withdrawing within 24 hours of deposit significantly reduces privacy",
       recommendation: "Wait at least 7 days for strong privacy guarantees"
     });
   }
   ```

### Amount Correlation Attacks

**Attack:** Link deposits to withdrawals based on unique amounts.

**Cloak Defenses:**

1. **Fixed Denomination Pools** (Roadmap)
   - Separate pools for 0.1, 0.5, 1, 5, 10 SOL
   - All outputs within pool are same amount
   - Impossible to correlate by amount

2. **Amount Rounding Suggestions**
   ```typescript
   // Current implementation helper
   function suggestPrivacyAmount(desiredAmount: number): number[] {
     const commonAmounts = [
       100_000_000,   // 0.1 SOL
       500_000_000,   // 0.5 SOL
       1_000_000_000, // 1.0 SOL
       5_000_000_000, // 5.0 SOL
     ];
     
     return commonAmounts.filter(amt => 
       Math.abs(amt - desiredAmount) / desiredAmount < 0.1
     );
   }
   ```

3. **Split Deposits**
   - Break large amounts into multiple common denominations
   - Example: 7.3 SOL → 5 SOL + 2 SOL + 0.3 SOL (3 separate deposits)

### Network Analysis Attacks

**Attack:** Analyze IP addresses, relay patterns, blockchain metadata.

**Cloak Defenses:**

1. **Multiple Relays**
   - No single relay sees all traffic
   - Geographic distribution
   - Tor/VPN friendly

2. **Metadata Minimization**
   - Encrypted communication
   - Minimal logging
   - No PII collection

3. **Decoy Traffic** (Roadmap)
   - Dummy withdrawals
   - Fake API calls
   - Traffic padding

## Comparison to Existing Protocols

### Tornado Cash (Ethereum)

**What They Did Well:**
- Fixed denomination pools
- Large anonymity sets (10,000+ at peak)
- Long operational history

**What Cloak Improves:**
- Solana-native (faster, cheaper)
- Programmable circuits (more flexible)
- PoW integration (decentralized priority)
- Transparent privacy metrics

### Zcash

**What They Did Well:**
- True amount privacy (shielded values)
- Mature cryptography
- Long track record

**What Cloak Differs:**
- Account model (UTXO-like notes on Solana)
- Different trade-offs (speed vs. features)
- More accessible (no trusted setup ceremony)

### Recent "Privacy" Projects on Solana

**Red Flags We Avoid:**

❌ **They:** Launch with no anonymity set plan
✅ **We:** Explicit bootstrapping strategy document (this file)

❌ **They:** Boast about TVL/volume without understanding turnover issue
✅ **We:** Focus on hold time, exit rate, stability metrics

❌ **They:** AI-generated documentation
✅ **We:** Technical documentation by cryptography engineers

❌ **They:** Hide privacy limitations
✅ **We:** Transparent about current weaknesses

❌ **They:** No source code or vague "coming soon"
✅ **We:** Open source everything, working code

## Launch Checklist

### Pre-Mainnet Requirements

- [ ] **Bootstrap fund allocated** - Liquidity mining rewards ready
- [ ] **Minimum commitments** - 1,000+ pre-committed deposits
- [ ] **Unique depositors** - 500+ unique users committed
- [ ] **Value threshold** - 100,000+ SOL committed
- [ ] **Time diversity** - 30+ days of testnet activity
- [ ] **Privacy dashboard** - Real-time metrics UI complete
- [ ] **User education** - Privacy guides and warnings
- [ ] **Audit complete** - Security audit by reputable firm
- [ ] **Attack simulations** - Correlation attacks tested
- [ ] **Relay diversity** - 3+ independent relays operational

### Post-Launch Monitoring

**Daily Metrics:**
- Anonymity set size per denomination
- Median hold time
- Quick exit rate (< 24hr withdrawals)
- Deposit/withdraw ratio

**Weekly Reviews:**
- Privacy attack simulations
- User behavior analysis
- Relay performance
- Bootstrap program effectiveness

**Monthly Assessments:**
- Overall privacy level evaluation
- Compare against target metrics
- Adjust incentives if needed
- Publish transparency report

## Conclusion

**Our Commitment:**

1. **Honesty First** - We will not claim privacy we don't provide
2. **Metrics Transparency** - Real-time public anonymity metrics
3. **User Education** - Clear warnings about privacy limitations
4. **Responsible Launch** - No mainnet until meaningful privacy threshold
5. **Continuous Improvement** - Regular privacy audits and updates

**Current Status (Testnet):**

```
Privacy Level: EXPERIMENTAL
Anonymity Set: 10-50 deposits
Recommendation: DO NOT use for production privacy needs
Timeline to Mainnet: TBD based on bootstrap milestones
```

**Next Steps:**

1. Complete this documentation review
2. Implement privacy metrics dashboard
3. Launch liquidity mining pre-commitment program
4. Build to 1,000+ committed deposits
5. Coordinated mainnet launch when ready

---

**Questions or Concerns?**

This is a living document. If you have questions about our anonymity set strategy or see gaps we haven't addressed, please open an issue or reach out to the team.

**We're building real privacy, not privacy theater.**



