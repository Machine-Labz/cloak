---
title: Quickstart Guide
description: Get started with Cloak in under 5 minutes.
---

# Quickstart Guide

This guide will get you up and running with Cloak in under 5 minutes. You'll learn how to create privacy-preserving deposits and withdrawals using Cloak's zero-knowledge proof system.

## Prerequisites

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) installed
- [Rust](https://rustup.rs/) installed
- [Node.js](https://nodejs.org/) installed
- [Docker](https://www.docker.com/) installed

## Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/cloak.git
   cd cloak
   ```

2. **Install dependencies:**
   ```bash
   cargo build
   npm install
   ```

3. **Start local services:**
   ```bash
   docker-compose up -d
   ```

## Basic Usage

### 1. Create a Deposit

```typescript
import { CloakClient } from '@cloak/client';

const client = new CloakClient({
  rpcUrl: 'http://localhost:8899',
  indexerUrl: 'http://localhost:3001',
  relayUrl: 'http://localhost:3002',
});

// Create a deposit
const deposit = await client.deposit({
  amount: 1000000, // 0.001 SOL
  recipient: '11111111111111111111111111111112',
});

console.log('Deposit created:', deposit.txid);
```

### 2. Create a Withdraw

```typescript
// Create a withdraw
const withdraw = await client.withdraw({
  outputs: [
    { address: '11111111111111111111111111111112', amount: 500000 },
    { address: '11111111111111111111111111111113', amount: 300000 },
  ],
  policy: { feeBps: 60 },
});

console.log('Withdraw created:', withdraw.requestId);
```

## Next Steps

- Read the [Architecture Overview](overview/system-architecture.md)
- Explore the [API Reference](api/indexer.md)
- Check out the [Workflows](workflows/deposit.md)

## Troubleshooting

### Common Issues

**Issue:** `Connection refused`
**Solution:** Ensure all services are running with `docker-compose ps`

**Issue:** `Insufficient funds`
**Solution:** Check your account balance with `solana balance`

**Issue:** `Program not deployed`
**Solution:** Deploy programs with `cargo run --bin deploy`

### Getting Help

- Check the [FAQ](faq.md)
- Join our [Discord](https://discord.gg/cloak)
- Open an [issue](https://github.com/your-org/cloak/issues)
