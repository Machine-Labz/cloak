# SPL Token Support

Cloak protocol now supports SPL tokens in addition to native SOL! This guide covers how to configure and use SPL tokens with the Cloak privacy protocol.

## Overview

The SPL token support allows users to deposit and withdraw SPL tokens (like USDC, USDT, or any custom SPL token) privately through the Cloak protocol. The implementation includes:

- **Multi-mint support**: Each pool can be configured for a specific token mint
- **Associated Token Accounts (ATA)**: Automatic handling of SPL token accounts
- **Jupiter integration**: Optional DEX aggregation for token swaps (devnet/testnet)
- **Backward compatibility**: Native SOL support is maintained

## Architecture

### On-Chain Components

#### Pool State
Each shield pool is now mint-aware:
```rust
pub struct Pool {
    mint: Pubkey, // Token mint (Pubkey::default() = native SOL)
}
```

#### PDA Derivation
PDAs now include the mint address in their seeds:
- Pool PDA: `["pool", mint.as_ref()]`
- Treasury PDA: `["treasury", mint.as_ref()]`
- Roots Ring PDA: `["roots_ring", mint.as_ref()]`
- Nullifier Shard PDA: `["nullifier_shard", mint.as_ref()]`
- Commitments PDA: `["commitments", mint.as_ref()]`

This allows multiple pools to coexist for different tokens.

### Off-Chain Components

#### Relay Service
The relay service automatically handles:
- Mint detection from configuration
- ATA derivation for recipients
- SPL token program accounts in transactions
- Jupiter swap integration (optional)

#### Indexer Service
The indexer tracks deposits and commitments per mint address.

## Configuration

### Environment Variables

#### Relay Service (`services/relay/.env`)

```bash
# Token mint address (empty or not set = native SOL)
MINT_ADDRESS=

# For SPL tokens, set to the mint address, e.g.:
# USDC devnet: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
# MINT_ADDRESS=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Jupiter swap integration (optional, for devnet/testnet)
JUPITER_ENABLED=false
JUPITER_API_URL=https://quote-api.jup.ag/v6
JUPITER_SLIPPAGE_BPS=50  # 0.5% slippage tolerance
```

#### Indexer Service (`services/indexer/.env`)

```bash
# Token mint address (must match relay configuration)
MINT_ADDRESS=

# For SPL tokens:
# MINT_ADDRESS=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

## Usage

### Native SOL (Default)

Leave `MINT_ADDRESS` empty or unset to use native SOL:

```bash
# .env
MINT_ADDRESS=
```

### SPL Tokens

Set `MINT_ADDRESS` to the token mint pubkey:

```bash
# Example: USDC on devnet
MINT_ADDRESS=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

### Initialize Pool for SPL Token

When deploying the shield-pool program, initialize it with the desired mint:

```bash
# Native SOL
solana program deploy shield_pool.so

# SPL Token
# The pool will be initialized with the mint from the initialization instruction
```

### Deposits

Users deposit SPL tokens the same way as SOL:

1. Approve token transfer to pool
2. Call deposit instruction with token amount
3. Pool transfers tokens to treasury's ATA
4. Commitment is recorded

### Withdrawals

The relay automatically:
1. Derives recipient ATAs
2. Includes SPL token program in transaction
3. Transfers from treasury ATA to recipient ATAs

## Jupiter Integration

Jupiter aggregation allows for efficient token swaps on devnet/testnet.

### Configuration

```bash
# Enable Jupiter
JUPITER_ENABLED=true

# API endpoint (default: mainnet)
JUPITER_API_URL=https://quote-api.jup.ag/v6

# Slippage tolerance in basis points (50 = 0.5%)
JUPITER_SLIPPAGE_BPS=50
```

### Usage

The Jupiter service can be used programmatically:

```rust
use relay::solana::jupiter::{JupiterService, JupiterConfig};

// Initialize
let config = JupiterConfig {
    enabled: true,
    api_url: "https://quote-api.jup.ag/v6".to_string(),
    slippage_bps: 50,
};
let jupiter = JupiterService::new(config)?;

// Get quote
let quote = jupiter.get_quote(
    &input_mint,
    &output_mint,
    amount
).await?;

// Execute swap
let tx = jupiter.swap(&quote, &user_pubkey, Some(10_000)).await?;
```

### Limitations on Devnet

Jupiter's devnet support has limitations:
- Pool liquidity may not match mainnet
- Not all token pairs are available
- Higher slippage may occur

## Testing

### Running Tests

```bash
# Build shield-pool program for SPL
cargo build-sbf

# Run relay tests
cargo test --package relay

# Run integration tests with SPL
cargo run --package test --example prove_test_spl
```

### Devnet Testing

1. Get devnet SOL from faucet:
   ```bash
   solana airdrop 2
   ```

2. Get devnet test tokens (USDC):
   ```bash
   spl-token create-account EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
   # Request tokens from Solana devnet faucet or test token dispenser
   ```

3. Configure services for devnet USDC:
   ```bash
   MINT_ADDRESS=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
   SOLANA_RPC_URL=https://api.devnet.solana.com
   ```

4. Test deposit and withdrawal

## Multi-Pool Setup

To run multiple pools for different tokens:

1. Deploy separate shield-pool instances for each mint
2. Configure each relay/indexer pair with its mint:

```bash
# Pool 1: Native SOL
MINT_ADDRESS=
CLOAK_PROGRAM_ID=<pool1_program_id>

# Pool 2: USDC
MINT_ADDRESS=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
CLOAK_PROGRAM_ID=<pool2_program_id>
```

3. Run separate instances of relay and indexer for each pool

## Security Considerations

### Token Approval
Users must approve the pool to transfer their SPL tokens. Ensure:
- Approval is only for the specific amount needed
- Approval is revoked after deposit (or set to exact amount)

### ATA Rent
Associated Token Accounts require rent. The protocol:
- Derives recipient ATAs automatically
- Expects ATAs to exist or creates them (future enhancement)

### Price Impact
When using Jupiter for swaps:
- Monitor `price_impact_pct` in quotes
- Set appropriate `slippage_bps` based on market conditions
- Be aware of sandwich attack risks on DEX swaps

## Troubleshooting

### "Invalid mint address" Error
- Check `MINT_ADDRESS` is a valid Solana pubkey
- Ensure mint address matches between relay and indexer
- Verify token mint exists on the network you're using

### "Token account not found" Error
- Recipient's ATA may not exist
- Create ATA first: `spl-token create-account <MINT>`
- Or implement automatic ATA creation in withdraw instruction

### Jupiter API Errors
- Check `JUPITER_ENABLED=true` in config
- Verify network connectivity to Jupiter API
- Ensure sufficient liquidity exists for the token pair
- Try increasing `JUPITER_SLIPPAGE_BPS` if routes fail

### Transaction Too Large
- SPL withdrawals add extra accounts (token program, ATAs)
- May exceed 1232 byte transaction limit with many recipients
- Reduce number of outputs if transaction fails

## Roadmap

Future enhancements planned:
- [ ] Automatic ATA creation in withdraw instruction
- [ ] Support for multiple recipients with different mints
- [ ] Cross-mint swaps (deposit USDC, withdraw SOL)
- [ ] Token2022 program support
- [ ] Mainnet Jupiter integration
- [ ] Raydium integration for additional DEX options
- [ ] Liquidity pool analytics and routing optimization

## References

- [SPL Token Program Documentation](https://spl.solana.com/token)
- [Jupiter Aggregator](https://jup.ag/)
- [Associated Token Account Program](https://spl.solana.com/associated-token-account)
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)
