# SPL Token Support Implementation Status

## What's Been Implemented ✅

### 1. Pool State (COMPLETE)
- Added `Pool` struct in `state/mod.rs`
- Stores mint address (32 bytes)
- `Pubkey::default()` = native SOL
- Non-default = SPL token mint
- Methods: `mint()`, `set_mint()`, `is_native()`

### 2. Initialize Instruction (COMPLETE)
- Updated to accept optional mint parameter in instruction data
- Creates pool with `Pool::SIZE` (32 bytes) instead of 0
- Initializes pool state with mint
- Backward compatible: empty data = native SOL

### 3. Deposit Instruction (COMPLETE)
- Detects mode based on account count:
  - 4 accounts = SOL mode
  - 7 accounts = SPL mode
- `process_native_deposit()`: Uses `SystemTransfer` for SOL
- `process_token_deposit()`: Uses `TokenTransfer` for SPL tokens
- Verifies pool mint matches deposit type

## What Still Needs Implementation 🚧

### 4. Withdraw Instruction (IN PROGRESS)
Need to update withdraw to support both SOL and SPL:

**Account layouts:**
- SOL (legacy): 6 accounts
- SOL + POW: 13 accounts  
- SPL (new): 9 accounts (add token_program, pool_token_account, recipient_token_account)
- SPL + POW: 16 accounts (add same 3 + miner_token_account, treasury_token_account)

**Changes needed:**
1. Import pinocchio_token
2. Detect token mode by checking pool state
3. Use token transfers instead of lamport manipulation for SPL
4. Handle POW fee distribution in tokens

### 5. Tests (PENDING)
Need to update:
- `deposit.rs` test: Add SPL token test case
- `withdraw.rs` test: Add SPL token test case
- `prove_test.rs`: Add SPL mode testing

## Testing Strategy

### Unit Tests (Mollusk)
```bash
cd programs/shield-pool
cargo test-sbf
```

### Integration Test (Testnet)
```bash
# 1. Create SPL token on testnet
spl-token create-token --decimals 9

# 2. Initialize pool with token mint
# (update tooling to support mint parameter)

# 3. Run prove_test.rs with token mode
SP1_TOKEN_MINT=<mint_address> cargo run --bin prove_test
```

## Key Design Decisions

1. **Backward Compatibility**: SOL deposits/withdraws still work with 4/6 accounts
2. **Pool-Level Config**: Each pool supports ONE asset type (SOL or specific token)
3. **Account Count Detection**: Simple, no need for instruction discriminator changes
4. **No Mixed Pools**: Can't deposit SOL and withdraw tokens from same pool

## Next Steps

1. ✅ Complete withdraw instruction update
2. ✅ Add comprehensive tests
3. ✅ Update client tooling (prove_test.rs)
4. Test on testnet
5. Document usage in README

## Files Modified

- ✅ `programs/shield-pool/src/state/mod.rs`
- ✅ `programs/shield-pool/src/lib.rs`
- ✅ `programs/shield-pool/src/instructions/initialize.rs`
- ✅ `programs/shield-pool/src/instructions/deposit.rs`
- 🚧 `programs/shield-pool/src/instructions/withdraw.rs`
- ⏳ `programs/shield-pool/src/tests/deposit.rs`
- ⏳ `programs/shield-pool/src/tests/withdraw.rs`
- ⏳ `tooling/test/src/prove_test.rs`

