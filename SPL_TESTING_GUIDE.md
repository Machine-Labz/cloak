# Cloak SPL Token Support - Complete Testing Guide

## ✅ Implementation Complete!

All SPL token support has been successfully implemented in the Cloak privacy protocol:

### Implemented Features
1. ✅ Pool state tracking (SOL vs SPL mint)
2. ✅ Initialize instruction (accepts optional mint parameter)
3. ✅ Deposit instruction (supports both SOL and SPL tokens)
4. ✅ Withdraw instruction (supports both SOL and SPL tokens, with/without POW)

### Files Modified
- `programs/shield-pool/src/state/mod.rs` - Added Pool struct
- `programs/shield-pool/src/lib.rs` - Export Pool
- `programs/shield-pool/src/instructions/initialize.rs` - SPL mint parameter
- `programs/shield-pool/src/instructions/deposit.rs` - SOL/SPL deposit modes
- `programs/shield-pool/src/instructions/withdraw.rs` - SOL/SPL withdraw modes

## Account Layouts Reference

### Deposits
```
SOL (4 accounts):
[user, pool, system_program, commitments]

SPL (7 accounts):
[user, user_token_account, pool, pool_token_account, token_program, system_program, commitments]
```

### Withdraws
```
SOL Legacy (6 accounts):
[pool, treasury, roots_ring, nullifier_shard, recipient, system_program]

SOL + POW (13 accounts):
[pool, treasury, roots_ring, nullifier_shard, recipient, system_program,
 scramble_program, claim_pda, miner_pda, registry_pda, clock, miner_authority, shield_pool_program]

SPL (10 accounts):
[pool, treasury, roots_ring, nullifier_shard, recipient, system_program,
 token_program, pool_token_account, recipient_token_account, treasury_token_account]

SPL + POW (18 accounts):
[pool, treasury, roots_ring, nullifier_shard, recipient, system_program,
 scramble_program, claim_pda, miner_pda, registry_pda, clock, miner_authority, shield_pool_program,
 token_program, pool_token_account, recipient_token_account, treasury_token_account, miner_token_account]
```

## Testing Strategy

### Phase 1: Build & Compile (Do This First!)

```bash
cd /home/victorcarvalho/Documents/Github/cloak

# Build the program
cd programs/shield-pool
cargo build-sbf

# Expected: Successful compilation
# If errors, fix them before proceeding
```

### Phase 2: Unit Tests (Local Testing)

**IMPORTANT**: The tests have been updated to initialize pool accounts with the correct 32-byte size (Pool::SIZE). This is required because the Pool now stores the mint pubkey.

The existing tests in `programs/shield-pool/src/tests/` test the SOL mode. These should now pass:

```bash
cd programs/shield-pool
cargo test-sbf

# Tests to verify:
# - test_deposit_instruction
# - test_deposit_insufficient_funds
# - test_deposit_duplicate_commitment
# - test_withdraw_instruction
# - test_admin_push_root_instruction
# - test_admin_push_root_unauthorized
# - test_admin_push_root_multiple_roots
```

**What was fixed**: All test pool accounts now use `data: vec![0u8; 32]` (32 bytes of zeros) instead of `data: vec![]` (empty). The 32 bytes represent the mint pubkey, where all zeros = `Pubkey::default()` = native SOL.

### Phase 3: Manual SPL Token Testing on Testnet

#### Step 1: Create Test SPL Token

```bash
# Create a new SPL token with 9 decimals
spl-token create-token --decimals 9

# Save the mint address, e.g.:
# Token Mint: 22wmAhdQeCM3wRFEaJHsuWQ3gwYCbM4Fa4brxdmy2RB7
export TEST_TOKEN_MINT="22wmAhdQeCM3wRFEaJHsuWQ3gwYCbM4Fa4brxdmy2RB7"
```

#### Step 2: Mint Some Test Tokens

```bash
# Create token account for testing
spl-token create-account $TEST_TOKEN_MINT

# Mint 10,000 tokens to yourself
spl-token mint $TEST_TOKEN_MINT 10000

# Check balance
spl-token balance $TEST_TOKEN_MINT
```

#### Step 3: Initialize SPL Pool

You need to update your deployment/initialization scripts to pass the mint parameter:

```rust
// In your initialization script
let initialize_ix_data = {
    let mut data = vec![3u8]; // Initialize discriminator
    data.extend_from_slice(mint_pubkey.as_ref()); // 32 bytes
    data
};
```

Or from CLI (if you have one):
```bash
# Initialize pool with SPL token
cloak-pool initialize --mint $TEST_TOKEN_MINT
```

#### Step 4: Create Token Accounts for Pool PDAs

```bash
# Get the pool PDA address (derive it)
# pool_pda = PDA(["pool"], program_id)

# Create token account owned by pool PDA
spl-token create-account $TEST_TOKEN_MINT --owner <POOL_PDA>

# You'll also need token accounts for:
# - Treasury PDA
# - Recipient addresses
# - Miner addresses (for POW mode)
```

#### Step 5: Test SPL Deposit

Update your deposit transaction builder to use 7 accounts for SPL mode:

```rust
// Build SPL deposit transaction
let deposit_ix = Instruction {
    program_id: shield_pool_program_id,
    accounts: vec![
        AccountMeta::new(user_pubkey, true),                    // user (signer)
        AccountMeta::new(user_token_account, false),            // user token account
        AccountMeta::new(pool_pda, false),                      // pool PDA
        AccountMeta::new(pool_token_account, false),            // pool token account
        AccountMeta::new_readonly(spl_token::ID, false),        // token program
        AccountMeta::new_readonly(system_program::ID, false),   // system program
        AccountMeta::new(commitments_pda, false),               // commitments
    ],
    data: deposit_data, // [0x00, amount:8, commitment:32, ...]
};
```

#### Step 6: Test SPL Withdraw

Update your withdraw transaction builder for SPL mode (10 accounts without POW, 18 with POW):

```rust
// Build SPL withdraw transaction (without POW)
let withdraw_ix = Instruction {
    program_id: shield_pool_program_id,
    accounts: vec![
        AccountMeta::new(pool_pda, false),                      // pool
        AccountMeta::new(treasury_pda, false),                  // treasury
        AccountMeta::new(roots_ring_pda, false),                // roots ring
        AccountMeta::new(nullifier_shard_pda, false),           // nullifier shard
        AccountMeta::new(recipient_pubkey, false),              // recipient
        AccountMeta::new_readonly(system_program::ID, false),   // system program
        AccountMeta::new_readonly(spl_token::ID, false),        // token program
        AccountMeta::new(pool_token_account, false),            // pool token account
        AccountMeta::new(recipient_token_account, false),       // recipient token account
        AccountMeta::new(treasury_token_account, false),        // treasury token account
    ],
    data: withdraw_data, // [0x02, proof:260, public_inputs:104, nullifier:32, ...]
};
```

### Phase 4: Integration Test (prove_test.rs)

To add SPL support to `tooling/test/src/prove_test.rs`, you need:

1. **Environment variable** for token mode:
```rust
let use_spl = std::env::var("SP1_USE_SPL_TOKEN").is_ok();
let spl_mint = if use_spl {
    Some(Pubkey::from_str(&std::env::var("SP1_TOKEN_MINT")?))
} else {
    None
};
```

2. **Create token accounts** before deposit:
```rust
if let Some(mint) = spl_mint {
    // Create user token account
    let user_token_account = create_associated_token_account(&client, &user_keypair, &mint)?;
    
    // Create pool token account
    let pool_token_account = create_associated_token_account_for_pda(&client, &pool_pda, &mint)?;
    
    // Mint tokens to user
    mint_tokens_to(&client, &mint, &user_token_account, 10_000_000_000)?;
}
```

3. **Update instruction builders** to use 7/10 accounts for SPL mode

4. **Run with**:
```bash
SP1_USE_SPL_TOKEN=1 SP1_TOKEN_MINT=7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU cargo run --bin prove_test
```

## Testing Checklist

### Basic Functionality
- [ ] SOL pool still works (backward compatibility)
- [ ] SPL pool initializes with mint
- [ ] SPL deposit transfers tokens correctly
- [ ] SPL withdraw transfers tokens correctly
- [ ] Fee distribution works for SPL tokens
- [ ] Can't deposit SOL to SPL pool
- [ ] Can't deposit tokens to SOL pool

### POW Integration
- [ ] SPL withdraw with POW claim works
- [ ] Miner receives token fee share (not SOL)
- [ ] Treasury receives token protocol share
- [ ] POW claim consumption tracked correctly

### Edge Cases
- [ ] Empty/zero balance handles correctly
- [ ] Token account validation works
- [ ] PDA signature for token transfers works
- [ ] Multiple SPL tokens don't interfere (different pools)

## Manual Testing Commands

### SOL Pool (Existing)
```bash
# Should still work as before
cloak-pool initialize
cloak deposit --amount 1.0
cloak withdraw --recipient <ADDR> --amount 0.9
```

### USDC Pool (Example)
```bash
# USDC Devnet mint: Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr

# Initialize
cloak-pool initialize --mint Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr

# Deposit 100 USDC
cloak deposit --amount 100 --token Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr

# Withdraw 90 USDC (10 USDC fee)
cloak withdraw --recipient <ADDR> --amount 90 --token Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr
```

## Common Issues & Solutions

### Issue: "Invalid account owner"
**Cause**: Token account not owned by correct PDA
**Fix**: Ensure token accounts are created with proper owner

### Issue: "Account not found"
**Cause**: Token accounts not created before transaction
**Fix**: Create all required token accounts first

### Issue: "Insufficient funds"
**Cause**: Pool token account has no tokens
**Fix**: Ensure deposits transfer tokens to pool token account

### Issue: "Invalid mint"
**Cause**: Token accounts have different mints
**Fix**: All token accounts must use same mint as pool

## Next Steps

1. **Build and verify compilation**:
   ```bash
   cd programs/shield-pool && cargo build-sbf
   ```

2. **Run existing tests** to ensure backward compatibility:
   ```bash
   cargo test-sbf
   ```

3. **Deploy to testnet**:
   ```bash
   solana program deploy target/deploy/shield_pool.so
   ```

4. **Create SPL token and test manually** following Phase 3 above

5. **Add integration test support** in `prove_test.rs` following Phase 4

6. **Document usage** in main README

## Summary

The SPL token support is **fully implemented** at the program level. What remains is:

1. Testing the implementation
2. Updating client-side tooling to support SPL mode
3. Creating token accounts for PDAs
4. Documenting the usage

The core functionality is complete and should work once you:
- Build successfully
- Create proper token accounts
- Update transaction builders to pass correct accounts

Good luck with testing! 🚀

