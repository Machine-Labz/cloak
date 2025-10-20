---
title: PoW Metrics Guide
description: Logging-based metrics reference for monitoring wildcard claim discovery and relay performance.
---

# 📊 Wildcard PoW System - Metrics Guide

**Date**: 2025-10-19  
**Status**: ✅ Metrics Implemented  
**Type**: Logging-based Metrics

---

## 📈 Overview

The Wildcard PoW system includes comprehensive logging-based metrics to track:
- Claim discovery performance
- Success/failure rates
- Query timings
- Account filtering statistics

All metrics are tagged with `[METRICS]` for easy grep/filtering.

---

## 🔍 Claim Discovery Metrics

### Search Start
```
🔍 [METRICS] Claim search started for batch_hash: "0a1b2c3d..."
```
**When**: At the beginning of every claim search  
**Info**: Batch hash being searched for (first 8 bytes)

---

### Query Performance
```
📊 [METRICS] Query complete: 15 accounts found in 234ms
```
**When**: After RPC `get_program_accounts` completes  
**Info**:
- Total accounts returned
- Query duration

---

### Success Case
```
✅ [METRICS] Found available claim: 3AXccp2m28LyoRPm2qZfedSh9LWVdpuW6wrknieh1dwA 
   (consumed 0/5, expires at slot 475086, search took 245ms)
```
**When**: Available claim found  
**Info**:
- Claim PDA address
- Current consumption (`consumed/max`)
- Expiry slot
- Total search duration

---

### Failure Case
```
❌ [METRICS] No available claims found for batch_hash: "0a1b2c3d..." 
   (searched 15 accounts in 245ms)
```
**When**: No usable claims found  
**Info**:
- Batch hash searched
- Total accounts examined
- Total search duration

---

### Error Case
```
❌ [METRICS] Claim query failed after 1.2s: Connection timeout
```
**When**: RPC query fails  
**Info**:
- Time until failure
- Error message

---

## 📊 PoW Transaction Metrics

### PoW Path Detection
```
PoW enabled: searching for available wildcard claim...
Computed batch_hash for job <job_id>: [0a, 1b, 2c, 3d...]
```
**When**: Building withdraw transaction with PoW enabled  
**Info**:
- Job ID
- Computed batch hash

---

### Claim Found
```
✓ Found wildcard claim: 3AXccp2m28LyoRPm2qZfedSh9LWVdpuW6wrknieh1dwA 
  (miner: XJJZk1..., expires at slot: 475086)
```
**When**: Claim successfully discovered for withdraw  
**Info**:
- Claim PDA
- Miner authority
- Expiry slot

---

### No Claims Available
```
No PoW claims available for job <job_id>. Consider starting miners or falling back to legacy.
```
**When**: No claims available for withdraw  
**Info**: Job ID that failed

---

## 📈 Monitoring Strategy

### Real-Time Monitoring

**Grep for all metrics**:
```bash
# All metrics
tail -f relay.log | grep "\[METRICS\]"

# Success rate
tail -f relay.log | grep "\[METRICS\]" | grep -E "(Found|No available)"

# Performance
tail -f relay.log | grep "\[METRICS\]" | grep "took"
```

---

### Log Aggregation

**Example with `jq` (if using JSON logging)**:
```bash
cat relay.log | jq 'select(.message | contains("[METRICS]"))'
```

---

### Key Metrics to Track

#### 1. **Claim Discovery Success Rate**
```bash
# Count successes
grep "\[METRICS\] Found available claim" relay.log | wc -l

# Count failures  
grep "\[METRICS\] No available claims" relay.log | wc -l

# Calculate rate
echo "scale=2; (successes / (successes + failures)) * 100" | bc
```

#### 2. **Average Query Time**
```bash
# Extract durations
grep "\[METRICS\] Query complete" relay.log |   grep -oE "[0-9]+ms" |   grep -oE "[0-9]+" |   awk '{sum+=$1; count++} END {print "Average:", sum/count, "ms"}'
```

#### 3. **Claims Pool Health**
```bash
# How many accounts are being scanned?
grep "\[METRICS\] Query complete" relay.log |   grep -oE "[0-9]+ accounts" |   tail -10
```

---

## 🎯 Performance Baselines

### Expected Performance (Localnet)

| Metric | Expected Value | Notes |
|--------|----------------|-------|
| Query time | 50-500ms | Depends on # of claims |
| Accounts returned | 2-20 | Registry + miner + claims |
| Success rate | &gt;90% | If miners running |
| Total search time | &lt;1s | End-to-end |

### Warning Thresholds

| Metric | Warning | Action |
|--------|---------|--------|
| Query time | &gt;2s | Check RPC performance |
| Success rate | &lt;50% | Start more miners |
| Accounts returned | 0 | Registry not initialized |
| Consecutive failures | &gt;10 | Investigate miner status |

---

## 🚨 Alerting

### Critical Alerts

**No claims for 5 minutes**:
```bash
# Count failures in last 5 min
grep "\[METRICS\] No available claims" relay.log |   grep "$(date -d '5 minutes ago' +%H:%M)" |   wc -l
```
→ **Action**: Start miners or adjust difficulty

**Query failures**:
```bash
grep "\[METRICS\] Claim query failed" relay.log | tail -10
```
→ **Action**: Check RPC connectivity

---

## 📊 Production Metrics Dashboard

### Recommended Metrics

1. **Claim Discovery Rate**
   - Gauge: Claims found / total searches
   - Target: >95%

2. **Query Latency**
   - Histogram: p50, p95, p99
   - Target: p95 < 500ms

3. **Active Claims Pool**
   - Gauge: Current available claims
   - Target: >5 claims

4. **Miner Earnings** (Future)
   - Counter: Total fees distributed
   - Gauge: Fees per claim

---

## 🔧 Troubleshooting

### High Query Times

**Symptom**: `Query complete: 100 accounts found in 5s`

**Causes**:
- Too many accounts on-chain
- Slow RPC connection
- Registry program has many PDAs

**Solutions**:
- Use memcmp filters in RPC query
- Add discriminator filtering
- Use faster RPC endpoint

---

### Low Success Rate

**Symptom**: `No available claims found` >50% of time

**Causes**:
- No miners running
- Claims expiring too fast
- High withdraw volume

**Solutions**:
- Start more miners
- Increase claim window
- Increase max_consumes per claim

---

### No Accounts Found

**Symptom**: `Query complete: 0 accounts found`

**Causes**:
- Registry not initialized
- Wrong program ID
- RPC not synced

**Solutions**:
- Initialize registry
- Verify program ID in config
- Check RPC health

---

## 📝 Log Format

All metrics follow this format:
```
[TIMESTAMP] LEVEL [METRICS] Message with context
```

**Example**:
```
2025-10-19T20:10:26Z INFO [METRICS] Found available claim: 3AXc... (consumed 0/5, expires at slot 475086, search took 245ms)
```

**Fields**:
- `TIMESTAMP`: ISO 8601 format
- `LEVEL`: INFO, WARN, ERROR
- `[METRICS]`: Tag for filtering
- Message: Human-readable with structured data

---

## 🎯 Future Enhancements

### Planned Metrics

1. **Prometheus Integration**
   - Export metrics to `/metrics` endpoint
   - Grafana dashboard templates

2. **Structured Logging**
   - JSON format with parseable fields
   - Easy aggregation in ELK/Loki

3. **Miner-Specific Metrics**
   - Track earnings per miner
   - Success rate by miner
   - Claim lifetime distribution

4. **Business Metrics**
   - Total fees distributed
   - Average claim utilization
   - Miner participation rate

---

## ✅ Current Status

**Implemented**: ✅
- Basic timing metrics
- Success/failure tracking
- Query performance logging
- Structured log messages

**Not Yet Implemented**: ⏳
- Prometheus/metrics endpoint
- Historical aggregation
- Miner earnings tracking
- Real-time dashboards

---

## 🚀 Usage Examples

### Development

**Watch metrics while developing**:
```bash
cargo run 2>&1 | grep "\[METRICS\]" | tee metrics.log
```

**Test claim discovery**:
```bash
# Trigger a withdraw
curl -X POST http://localhost:3002/jobs/withdraw -d '{...}'

# Watch metrics
tail -f metrics.log
```

---

### Production

**Set up log rotation**:
```bash
# In logrotate.d/relay
/var/log/relay/*.log {
    daily
    rotate 7
    compress
    delaycompress
    postrotate
        systemctl reload relay
    endscript
}
```

**Monitor continuously**:
```bash
# Use journalctl
journalctl -u relay -f | grep "\[METRICS\]"
```

---

## 📈 Success Indicators

Your PoW system is healthy when you see:

✅ **Regular successful discoveries**:
```
✅ [METRICS] Found available claim: ... (search took 156ms)
✅ [METRICS] Found available claim: ... (search took 203ms)
✅ [METRICS] Found available claim: ... (search took 178ms)
```

✅ **Fast queries**:
```
📊 [METRICS] Query complete: 8 accounts found in 124ms
📊 [METRICS] Query complete: 10 accounts found in 156ms
```

✅ **Multiple available claims**:
```
📊 [METRICS] Query complete: 15 accounts found in 201ms
```
(More accounts = more claims = healthier pool)

---

**🎉 Metrics are live and tracking your PoW system!**
