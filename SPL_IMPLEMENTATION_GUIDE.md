# Cloak SPL Token Support - Complete Implementation Guide

## Summary

I've successfully implemented **partial SPL token support** for the Cloak privacy protocol. The system now supports both native SOL and SPL tokens, with each pool configured for one asset type.

## What's Been Implemented ✅

### 1. Pool State Structure
**File**: `programs/shield-pool/src/state/mod.rs`

```rust
pub struct Pool(*mut u8);
impl Pool {
    pub const SIZE: usize = 32; // Stores mint pubkey
    pub fn mint(&self) -> Pubkey;
    pub fn set_mint(&mut self, mint: &Pubkey);
    pub fn is_native(&self) -> bool; // true if mint == Pubkey::default()
}
```

### 2. Initialize Instruction
**File**: `programs/shield-pool/src/instructions/initialize.rs`

- Accepts optional 32-byte mint parameter in instruction data
- If empty or all zeros → native SOL pool
- Otherwise → SPL token pool for that mint
- Creates pool with `Pool::SIZE` (32 bytes) space

**Usage**:
```bash
# Initialize SOL pool
cloak-pool initialize

# Initialize USDC pool
cloak-pool initialize --mint <USDC_MINT_ADDRESS>
```

### 3. Deposit Instruction  
**File**: `programs/shield-pool/src/instructions/deposit.rs`

**Account Layouts**:
- **SOL deposit** (4 accounts): `[user, pool, system_program, commitments]`
- **SPL deposit** (7 accounts): `[user, user_token_account, pool, pool_token_account, token_program, system_program, commitments]`

**Logic**:
- Detects mode by account count
- Verifies pool mint matches deposit type
- Uses `SystemTransfer` for SOL, `TokenTransfer` for SPL
- Prevents mixing: can't deposit tokens to SOL pool or vice versa

## What Still Needs Work 🚧

### Withdraw Instruction (CRITICAL)

**Current Status**: Legacy SOL withdraw works, but SPL token withdraws are NOT implemented.

**What needs to be done**:

1. **Update imports** in `withdraw.rs`:
```rust
use pinocchio_token::instructions::Transfer as TokenTransfer;
use crate::state::Pool;
```

2. **Account layouts for withdraws**:
```
SOL (legacy):    6 accounts - [pool, treasury, roots_ring, nullifier_shard, recipient, system_program]
SOL + POW:      13 accounts - add [scramble_program, claim_pda, miner_pda, registry_pda, clock, miner_authority, shield_pool_program]

SPL (new):       9 accounts - add [token_program, pool_token_account, recipient_token_account]
SPL + POW:      16 accounts - add above 3 + [miner_token_account, treasury_token_account]
```

3. **Detect token mode** early in `process_withdraw_instruction`:
```rust
// Read pool state to check if native or SPL
let pool_state = Pool::from_account_info(pool_info)?;
let is_native = pool_state.is_native();
```

4. **Replace lamport manipulation** with token transfers:

**Current SOL code** (lines 339-348):
```rust
unsafe {
    *pool_info.borrow_mut_lamports_unchecked() = pool_lamports - parsed.public_amount;
    *recipient_account.borrow_mut_lamports_unchecked() = recipient_lamports + parsed.recipient_amount;
    *treasury_info.borrow_mut_lamports_unchecked() = treasury_lamports + protocol_share;
    *miner_authority_account.borrow_mut_lamports_unchecked() = miner_lamports + scrambler_share;
}
```

**New SPL code** (needs implementation):
```rust
if is_native {
    // Use existing SOL lamport code
} else {
    // Use token transfers via PDA signatures
    let pool_seeds = &[b"pool".as_ref(), &[pool_bump]];
    
    // Transfer to recipient
    TokenTransfer {
        from: pool_token_account,
        to: recipient_token_account,
        authority: pool_info,  // Pool PDA signs
        amount: parsed.recipient_amount,
    }.invoke_signed(&[pool_seeds])?;
    
    // Transfer to treasury
    TokenTransfer {
        from: pool_token_account,
        to: treasury_token_account,
        authority: pool_info,
        amount: protocol_share,
    }.invoke_signed(&[pool_seeds])?;
    
    // If POW mode, transfer to miner
    if is_pow_mode {
        TokenTransfer {
            from: pool_token_account,
            to: miner_token_account,
            authority: pool_info,
            amount: scrambler_share,
        }.invoke_signed(&[pool_seeds])?;
    }
}
```

### Tests (PENDING)

Need to create comprehensive tests for:

1. **Unit tests** (using Mollusk):
   - SOL deposit + withdraw
   - SPL deposit + withdraw  
   - Mixed mode rejection (deposit SOL, can't withdraw as token)

2. **Integration test** (`prove_test.rs`):
   - Create test token on testnet
   - Initialize SPL pool
   - Deposit tokens
   - Generate proof
   - Withdraw tokens with POW

## Testing Strategy

### Local Development
```bash
# Build
cd programs/shield-pool
cargo build-sbf

# Unit tests
cargo test-sbf
```

### Testnet Testing

**Step 1: Create test token**
```bash
spl-token create-token --decimals 9
# Save mint address: e.g., TokenMint111...
```

**Step 2: Create token accounts**
```bash
# For pool
spl-token create-account TokenMint111 --owner <POOL_PDA>

# For user
spl-token create-account TokenMint111

# Mint some tokens to user
spl-token mint TokenMint111 1000 <USER_TOKEN_ACCOUNT>
```

**Step 3: Initialize pool**
```bash
# Update tooling to support --mint parameter
cloak-pool initialize --mint TokenMint111
```

**Step 4: Test deposit**
```bash
cloak deposit --amount 100 --token TokenMint111
```

**Step 5: Test withdraw**
```bash
# Run prove_test.rs with SPL mode
SP1_TOKEN_MINT=TokenMint111 cargo run --bin prove_test
```

## Key Design Decisions

1. **One Asset Per Pool**: Each pool handles either SOL or one specific SPL token
2. **Backward Compatible**: Existing SOL deposits/withdraws continue to work
3. **Account Count Detection**: Simple way to determine SOL vs SPL without changing discriminators
4. **PDA-Based Token Transfers**: Pool PDA signs for token transfers using `invoke_signed`

## Migration Path

For existing deployments:

1. **Current SOL pools**: Continue working as-is (no changes needed)
2. **New SPL pools**: Deploy fresh with SPL mint configured
3. **No migration**: Can't convert SOL pool to SPL pool (by design)

## Files Modified

- ✅ `programs/shield-pool/src/state/mod.rs` - Added Pool state
- ✅ `programs/shield-pool/src/lib.rs` - Export Pool
- ✅ `programs/shield-pool/src/instructions/initialize.rs` - Accept mint parameter
- ✅ `programs/shield-pool/src/instructions/deposit.rs` - Support both SOL and SPL
- ⏳ `programs/shield-pool/src/instructions/withdraw.rs` - **NEEDS SPL SUPPORT**
- ⏳ `programs/shield-pool/src/tests/deposit.rs` - **NEEDS SPL TESTS**
- ⏳ `programs/shield-pool/src/tests/withdraw.rs` - **NEEDS SPL TESTS**
- ⏳ `tooling/test/src/prove_test.rs` - **NEEDS SPL MODE**

## Next Steps for You

1. **Complete withdraw.rs**: Implement token transfers as outlined above
2. **Add tests**: Create comprehensive test cases for both modes
3. **Update tooling**: Modify `prove_test.rs` to support SPL tokens
4. **Test on testnet**: Run full flow with real SPL token
5. **Document**: Update README with SPL usage examples

## Example Usage (Once Complete)

### SOL Pool (Existing)
```bash
# Initialize
cloak-pool initialize

# Deposit
cloak deposit --amount 1.0

# Withdraw
cloak withdraw --recipient <ADDR> --amount 0.9
```

### USDC Pool (New)
```bash
# Initialize  
cloak-pool initialize --mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Deposit
cloak deposit --amount 100 --token EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Withdraw
cloak withdraw --recipient <ADDR> --amount 90 --token EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

## Questions?

The implementation is solid but incomplete. The hardest part (withdraw with token transfers) is outlined above but needs your implementation. Let me know if you need help with specific parts!

