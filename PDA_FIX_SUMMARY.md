# PDA Derivation Fix Summary

## Problem
The test was failing with error `0x1051` (PoolOwnerNotProgramId) because:
1. **Old PDA derivation**: `["pool"]` (without mint)
2. **New PDA derivation**: `["pool", mint]` (with mint)
3. **Pool account doesn't exist** at the new PDA

## Root Cause
The multi-token support requires PDAs to include the mint address as a seed:
- **Native SOL**: `["pool", Pubkey::default()]` (all zeros)
- **SPL Token**: `["pool", mint_pubkey]`

## Solution
Updated the test to:
1. **Use new PDA derivation** with mint
2. **Check if pool account exists** at new PDA
3. **Create accounts if needed** with correct size (32 bytes for mint)
4. **Initialize pool** with mint data

## Code Changes

### 1. Updated PDA Derivation
```rust
// Before (old)
let (pool, commitments, roots_ring, nullifier_shard, treasury) =
    get_pda_addresses_sol(&program_id);

// After (new)
let mint = solana_sdk::pubkey::Pubkey::default(); // Native SOL
let (pool, commitments, roots_ring, nullifier_shard, treasury) =
    get_pda_addresses(&program_id, &mint);
```

### 2. Added Account Existence Check
```rust
// Check if pool account exists
match client.get_account(&pool) {
    Ok(account) => {
        if account.owner != program_id {
            // Create new accounts
        } else {
            // Use existing accounts
        }
    }
    Err(_) => {
        // Create new accounts
    }
}
```

### 3. Updated Pool Account Creation
```rust
// Pool now stores mint (32 bytes)
let pool_rent_exempt = client.get_minimum_balance_for_rent_exemption(32)?;
let create_pool_ix = system_instruction::create_account(
    &admin_keypair.pubkey(),
    &pool_pda,
    pool_rent_exempt,
    32, // Pool now stores mint
    &program_id,
);
```

### 4. Added Pool Initialization
```rust
// Initialize pool with mint data
let initialize_pool_ix = Instruction {
    program_id: *program_id,
    accounts: vec![...],
    data: {
        let mut data = vec![0u8]; // Initialize discriminator
        data.extend_from_slice(&mint.to_bytes()); // Mint pubkey (32 bytes)
        data
    },
};
```

## Expected Behavior
- **First run**: Creates accounts at new PDA with correct size
- **Subsequent runs**: Uses existing accounts if they exist and are owned by program
- **Multi-token support**: Different PDAs for different tokens

## Files Modified
- `tooling/test/src/prove_test.rs` - Updated PDA derivation and account creation
- `tooling/test/src/shared.rs` - Fixed keypair loading (already done)

## Testing
The test should now:
1. ✅ Derive correct PDAs with mint
2. ✅ Create pool account with 32-byte size
3. ✅ Initialize pool with mint data
4. ✅ Pass deposit transaction validation
