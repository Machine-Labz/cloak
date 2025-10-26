# SPL Token Test Guide

This guide shows how to test the multi-token support with SPL tokens (like USDC) using the new `prove_test_spl.rs` test.

## Overview

The `prove_test_spl.rs` test demonstrates:
- ✅ SPL token deposit transactions
- ✅ SPL token withdraw transactions  
- ✅ Multi-token PDA derivation
- ✅ Token account creation and management
- ✅ Full privacy protocol with SPL tokens

## Prerequisites

1. **Deploy the updated program** with multi-token support
2. **Start the services** (indexer, relay, miner)
3. **Have USDC test tokens** on testnet (or create a test token)

## Running the SPL Token Test

### Step 1: Build the Test

```bash
cd tooling/test
cargo build --bin prove-test-spl
```

### Step 2: Set Up Environment

Create a `.env` file with:

```bash
# Solana Configuration
SOLANA_RPC_URL=https://api.testnet.solana.com
SOLANA_WS_URL=wss://api.testnet.solana.com

# Program Configuration  
SHIELD_POOL_PROGRAM_ID=c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp

# Keypair Paths
USER_KEYPAIR_PATH=user-keypair.json
RECIPIENT_KEYPAIR_PATH=recipient-keypair.json
MINER_KEYPAIR_PATH=miner.json

# Indexer Configuration
INDEXER_URL=http://localhost:3001

# Optional: TEE Configuration
SP1_TEE_ENABLED=false
```

### Step 3: Create Keypairs

```bash
# Create user keypair
solana-keygen new --outfile user-keypair.json --no-bip39-passphrase

# Create recipient keypair  
solana-keygen new --outfile recipient-keypair.json --no-bip39-passphrase

# Create miner keypair
solana-keygen new --outfile miner.json --no-bip39-passphrase
```

### Step 4: Fund Keypairs

```bash
# Fund user keypair (needs SOL for transaction fees)
solana airdrop 2 user-keypair.json --url testnet

# Fund recipient keypair
solana airdrop 2 recipient-keypair.json --url testnet

# Fund miner keypair
solana airdrop 2 miner.json --url testnet
```

### Step 5: Get USDC Test Tokens

For testnet, you can:
1. **Use existing USDC testnet token**: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
2. **Create your own test token** using Solana CLI
3. **Use a testnet faucet** for USDC

### Step 6: Start Services

```bash
# Terminal 1: Start Indexer
cd services/indexer
cargo run

# Terminal 2: Start Relay  
cd services/relay
cargo run

# Terminal 3: Start Miner
cd packages/cloak-miner
cargo run
```

### Step 7: Run the SPL Token Test

```bash
cd tooling/test
cargo run --bin prove-test-spl
```

## Expected Output

The test will:

1. **🔐 CLOAK PRIVACY PROTOCOL - SPL TOKEN FLOW TEST**
2. **💰 Check balances** for all keypairs
3. **🪙 Create SPL token accounts** (user, recipient, pool, treasury, miner)
4. **📥 Deposit SPL tokens** to the privacy pool
5. **🌳 Push merkle root** to the program
6. **🔐 Generate ZK proof** for withdrawal
7. **💸 Execute SPL token withdrawal** via relay
8. **⛏️ Verify miner rewards**

## Key Differences from SOL Test

### 1. **Token Account Creation**
- Creates separate token accounts for each participant
- Uses SPL Token Program for account management
- Handles token account ownership properly

### 2. **PDA Derivation**
- Uses mint address in PDA seeds: `["pool", mint]`
- Separate state accounts for each token type
- Multi-token support in same program

### 3. **Transaction Structure**
- SPL token deposit: 7 accounts (vs 4 for SOL)
- SPL token withdraw: 7 accounts (vs 4 for SOL)
- Includes token program and token accounts

### 4. **Configuration**
- Relay service needs `mint_address` in config
- Indexer service needs `mint_address` in config
- Test uses USDC mint: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

## Configuration Files

### Relay Service Config
```toml
[solana]
mint_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"  # USDC
# mint_address = ""  # Empty = native SOL
```

### Indexer Service Config  
```toml
[solana]
mint_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"  # USDC
# mint_address = ""  # Empty = native SOL
```

## Troubleshooting

### Common Issues

1. **"Insufficient funds"**
   - Ensure keypairs have enough SOL for transaction fees
   - Check USDC token balance for deposits

2. **"Account not found"**
   - Verify program is deployed with multi-token support
   - Check PDA derivation includes mint address

3. **"Invalid mint"**
   - Use correct USDC testnet mint address
   - Verify token program is available

4. **"Relay service error"**
   - Ensure relay service has `mint_address` configured
   - Check relay service is running and accessible

### Debug Commands

```bash
# Check program account
solana account c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp --url testnet

# Check token account
solana account <TOKEN_ACCOUNT_ADDRESS> --url testnet

# Check PDA derivation
solana address --seed "pool" --seed <MINT_ADDRESS> --program-id c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp
```

## Success Criteria

✅ **Test completes successfully**  
✅ **SPL token deposit transaction confirmed**  
✅ **SPL token withdraw transaction confirmed**  
✅ **Miner receives reward**  
✅ **Privacy protocol maintains anonymity**  
✅ **Multi-token support working**  

## Next Steps

After successful testing:

1. **Deploy to mainnet** with production tokens
2. **Add more token types** (USDT, DAI, etc.)
3. **Implement token-specific features** (different fees, limits)
4. **Add token metadata** and display names
5. **Create user-friendly interfaces** for token selection

This test validates that the multi-token support is working correctly and the privacy protocol can handle SPL tokens just like native SOL!
