# Private Staking Workflow

This document describes the complete private staking flow in Cloak, including both **staking** (shield pool → stake account) and **unstaking** (stake account → shield pool).

## Overview

Private staking allows users to delegate SOL to validators without revealing the source of funds, and later return those funds to the shield pool privately.

```
┌─────────────────────────────────────────────────────────────┐
│                    PRIVATE STAKING FLOW                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. DEPOSIT (optional if already have shielded funds)       │
│     Wallet ───────────────────────────► Shield Pool         │
│     (public)        Deposit             (commitment added)  │
│                                                             │
│  2. STAKE (private)                                         │
│     Shield Pool ──────────────────────► Stake Account       │
│     (ZK proof)       WithdrawStake      (delegated)         │
│                                                             │
│  3. EARN REWARDS                                            │
│     Stake Account ────────────────────► Stake Account       │
│     (validator)      (rewards compound automatically)       │
│                                                             │
│  4. DEACTIVATE (public, but doesn't reveal destination)     │
│     Stake Account ────────────────────► Deactivating        │
│     (stake program)  Deactivate                             │
│                                                             │
│  5. UNSTAKE (private)                                       │
│     Stake Account ────────────────────► Shield Pool         │
│     (ZK proof)     UnstakeToPool       (new commitment)     │
│                                                             │
│  6. WITHDRAW (private)                                      │
│     Shield Pool ──────────────────────► Any Wallet          │
│     (ZK proof)        Withdraw          (unlinkable)        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Privacy Guarantees

### What is Private
- **Source of staked funds**: Cannot be linked to any specific deposit
- **Destination after unstaking**: Returns to shield pool, can be withdrawn to any address
- **Staking rewards**: Compound privately within the stake account

### What is Public
- **Stake account existence**: The stake account itself is visible on-chain
- **Delegation**: Which validator the stake is delegated to
- **Stake amount**: The amount staked (required by Solana staking)
- **Deactivation**: When the stake is deactivated (cooldown period)

## Step-by-Step Instructions

### 1. Private Stake

**Prerequisites:**
- Shielded SOL in the pool (via deposit)
- Valid Cloak note (generated during deposit)

**Steps:**
1. Navigate to `/stake` in the web app
2. Enter the amount of SOL to stake
3. Select a validator from the dropdown
4. Click "Stake SOL Privately"
5. Sign the deposit transaction (if needed)
6. Wait for ZK proof generation (~30-60 seconds)
7. The relay submits the `WithdrawStake` transaction

**What happens on-chain:**
- ZK proof verifies you own a valid commitment in the pool
- SOL is transferred from the shield pool to a stake account
- Stake account is automatically delegated to your chosen validator

### 2. Deactivate Stake

**Prerequisites:**
- Active stake account that you control (as withdrawer)

**Steps:**
1. Navigate to `/unstake` in the web app
2. Enter your stake account address
3. Verify the stake is "Active"
4. Click "Deactivate Stake"
5. Sign the transaction

**What happens on-chain:**
- Standard Solana stake deactivation
- Stake enters cooldown period (~1 epoch ≈ 2-3 days)

### 3. Private Unstake

**Prerequisites:**
- Deactivated stake account (cooldown complete)
- Must be the withdrawer of the stake account

**Steps:**
1. Navigate to `/unstake` in the web app
2. Enter your stake account address
3. Verify the stake is "Deactivated"
4. Click "Unstake to Pool Privately"
5. Wait for ZK proof generation
6. The relay submits the `UnstakeToPool` transaction

**What happens on-chain:**
- ZK proof proves you own the stake account and creates a valid commitment
- SOL is withdrawn from stake account to the shield pool
- New commitment is added to the Merkle tree
- You receive a new Cloak note for your shielded balance

## Fee Structure

| Operation | Fee |
|-----------|-----|
| Stake (WithdrawStake) | 0.0025 SOL + 0.5% |
| Deactivate | Network fee only (~0.000005 SOL) |
| Unstake (UnstakeToPool) | 0.5% |

## Technical Details

### Stake Account Generation

Stake accounts are generated as Program Derived Addresses (PDAs) using:
- **Seeds**: `["stake", stake_authority, validator_vote_account]`
- **Program**: Shield Pool Program

This ensures:
- Deterministic address generation
- No linkage to user's main wallet
- Automatic ownership by the user's derived authority

### ZK Circuit for Staking

The staking proof verifies:
1. Valid nullifier (prevents double-spending)
2. Commitment exists in Merkle tree
3. Amount matches the stake request
4. `outputs_hash = H(stake_account || amount)`

### ZK Circuit for Unstaking

The unstaking proof verifies:
1. Valid commitment formation: `C = H(amount || r || pk_spend)`
2. User knows `sk_spend` such that `pk_spend = H(sk_spend)`
3. Stake account hash matches the account being unstaked

### On-Chain Instructions

#### WithdrawStake (discriminant = 9)
```rust
// Accounts:
// [0] pool - Shield pool PDA (writable)
// [1] treasury - Treasury PDA (writable)
// [2] roots_ring - Roots ring PDA
// [3] nullifier_shard - Nullifier shard PDA (writable)
// [4] stake_account - Stake account (writable)
// [5] system_program

// Data:
// [proof (260)][public_inputs (104)][duplicate_nullifier (32)][stake_account (32)]
```

#### UnstakeToPool (discriminant = 10)
```rust
// Accounts:
// [0] pool - Shield pool PDA (writable)
// [1] roots_ring - Roots ring PDA (writable)
// [2] stake_account - Stake account (writable)
// [3] stake_authority - Stake authority (signer)
// [4] clock_sysvar
// [5] stake_history
// [6] stake_program

// Data:
// [proof (260)][public_inputs (104)][stake_account (32)]
```

## API Endpoints

### Relay API

#### POST /api/v1/withdraw (with stake config)
```json
{
  "proof": "base64_encoded_proof",
  "public_inputs": {
    "root": "hex",
    "nullifier": "hex",
    "outputs_hash": "hex",
    "amount": 1000000000
  },
  "stake": {
    "stake_account": "pubkey",
    "stake_authority": "pubkey",
    "validator_vote_account": "pubkey"
  }
}
```

#### POST /api/v1/unstake
```json
{
  "proof": "base64_encoded_proof",
  "public_inputs": {
    "commitment": "hex",
    "stake_account_hash": "hex",
    "outputs_hash": "hex",
    "amount": 1000000000
  },
  "unstake": {
    "stake_account": "pubkey",
    "stake_authority": "pubkey",
    "commitment": "hex",
    "amount": 1000000000
  }
}
```

## Security Considerations

1. **Stake Authority**: The stake authority is derived from the user's wallet, ensuring only they can deactivate and unstake.

2. **Cooldown Period**: Solana's native ~1 epoch cooldown prevents rapid stake/unstake cycles.

3. **Proof Verification**: All proofs are verified on-chain using SP1 Groth16, ensuring mathematical guarantees.

4. **Commitment Binding**: The unstake commitment is bound to the stake account hash, preventing commitment forgery.

## Troubleshooting

### "Stake account not found"
- Verify the address is correct
- Ensure the stake account has been created and funded

### "You are not the withdrawer"
- Only the withdrawer authority can deactivate and unstake
- Use a stake account created through Cloak's private staking

### "Stake is deactivating"
- Wait for the cooldown period to complete (~1 epoch)
- Check current epoch on Solana Explorer

### "Insufficient balance"
- Ensure you have enough SOL in the shield pool to cover stake + fees
- Minimum stake is typically 1 SOL

## Future Enhancements

- [ ] Partial unstaking (unstake portion of stake account)
- [ ] Automatic reward compounding optimization
- [ ] Multi-validator staking from single commitment
- [ ] Liquid staking token support (mSOL, stSOL, etc.)

