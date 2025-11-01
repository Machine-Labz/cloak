# Scramble Registry - Cloak PoW Program

A Pinocchio-based Solana program that manages proof-of-work (PoW) claims for the Cloak protocol's scrambler gate system. This program enables miners to earn fees by providing wildcard claims that can be consumed during withdrawals.

## Overview

The Scramble Registry implements a decentralized mining system inspired by [Ore](https://ore.supply/), where:
- **Miners** continuously mine PoW solutions using BLAKE3
- **Claims** go through a lifecycle: Mine → Reveal → Consume
- **Wildcard claims** can be used for any withdrawal batch
- **Difficulty** adjusts automatically based on mining activity
- **Fees** are distributed to miners when their claims are consumed

## Program ID

```
EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6
```

## Instructions

### 1. Initialize Registry (`0x00`)

**Purpose**: Initialize the scramble registry with mining parameters.

**Accounts**: `[Registry (writable), Admin]`

**Data Layout**:
```
[tag: u8 = 0x00]
[initial_difficulty: 32 bytes]
[min_difficulty: 32 bytes]
[max_difficulty: 32 bytes]
[target_interval_slots: 8 bytes LE]
[fee_share_bps: 2 bytes LE]
[reveal_window: 8 bytes LE]
[claim_window: 8 bytes LE]
[max_k: 2 bytes LE]
```

**Effects**:
- Initializes registry with mining parameters
- Sets admin authority
- Logs `registry_initialized` event

### 2. Register Miner (`0x01`)

**Purpose**: Register a new miner to participate in PoW mining.

**Accounts**: `[Miner (writable), Registry, MinerAuthority, ClockSysvar]`

**Data Layout**:
```
[tag: u8 = 0x01]
```

**Effects**:
- Creates miner account with authority
- Records registration slot
- Logs `miner_registered` event

### 3. Mine Claim (`0x02`)

**Purpose**: Submit a mined PoW solution to create a claim.

**Accounts**: `[Claim (writable), Miner (writable), Registry (writable), MinerAuthority, SlotHashesSysvar, ClockSysvar, System]`

**Data Layout**:
```
[tag: u8 = 0x02]
[slot: 8 bytes LE]
[slot_hash: 32 bytes]
[batch_hash: 32 bytes]
[nonce: 16 bytes LE]
[proof_hash: 32 bytes]
[max_consumes: 2 bytes LE]
```

**Effects**:
1. **PoW Verification**: Validates BLAKE3 proof against difficulty
2. **Slot Validation**: Ensures slot_hash matches SlotHashes sysvar
3. **Difficulty Check**: Verifies solution meets current difficulty
4. **Account Creation**: Creates claim account with mined status
5. **Miner Update**: Increments miner's total_mined counter
6. **Registry Update**: Records solution and updates difficulty
7. **Event**: Logs `claim_mined` with claim details

### 4. Reveal Claim (`0x03`)

**Purpose**: Reveal a mined claim to make it available for consumption.

**Accounts**: `[Claim (writable), Registry, MinerAuthority, ClockSysvar]`

**Data Layout**:
```
[tag: u8 = 0x03]
```

**Effects**:
1. **Status Check**: Verifies claim is in Mined status
2. **Window Check**: Ensures reveal window hasn't expired
3. **Authority Check**: Verifies miner authority
4. **Status Update**: Changes status to Revealed
5. **Expiry Set**: Sets expiration slot based on claim_window
6. **Event**: Logs `claim_revealed` event

### 5. Consume Claim (`0x04`)

**Purpose**: Consume a revealed claim (called by shield-pool program).

**Accounts**: `[Claim (writable), Miner (writable), Registry (writable), ShieldPoolProgram, ClockSysvar]`

**Data Layout**:
```
[tag: u8 = 0x04]
[expected_miner_authority: 32 bytes]
[expected_batch_hash: 32 bytes]
```

**Effects**:
1. **CPI Check**: Verifies caller is shield-pool program
2. **Status Check**: Ensures claim is revealed and not expired
3. **Consumption Check**: Verifies claim hasn't reached max_consumes
4. **Batch Validation**: Validates batch_hash (or wildcard)
5. **Consumption**: Increments consumed_count
6. **Miner Update**: Increments miner's total_consumed counter
7. **Status Update**: Sets to Consumed if fully consumed
8. **Event**: Logs `claim_consumed` event

## Account Layouts

### ScrambleRegistry (188 bytes)

```
Offset | Size | Field
-------|------|-------
0      | 32   | admin: Pubkey
32     | 32   | current_difficulty: [u8; 32]
64     | 8    | last_retarget_slot: u64
72     | 8    | solutions_observed: u64
80     | 8    | target_interval_slots: u64
88     | 2    | fee_share_bps: u16
90     | 8    | reveal_window: u64
98     | 8    | claim_window: u64
106    | 2    | max_k: u16
108    | 32   | min_difficulty: [u8; 32]
140    | 32   | max_difficulty: [u8; 32]
172    | 8    | total_claims: u64
180    | 8    | active_claims: u64
```

**Registry Management**:
- **Difficulty Adjustment**: Automatic retargeting based on solution frequency
- **Parameter Control**: Admin can update mining parameters
- **Statistics**: Tracks total and active claims

### Miner (56 bytes)

```
Offset | Size | Field
-------|------|-------
0      | 32   | authority: Pubkey
32     | 8    | total_mined: u64
40     | 8    | total_consumed: u64
48     | 8    | registered_at_slot: u64
```

**Miner Tracking**:
- **Authority**: Pubkey that can mine and reveal claims
- **Statistics**: Tracks mining and consumption activity
- **Registration**: Records when miner joined

### Claim (256 bytes)

```
Offset | Size | Field
-------|------|-------
0      | 32   | miner_authority: Pubkey
32     | 32   | batch_hash: [u8; 32]
64     | 8    | slot: u64
72     | 32   | slot_hash: [u8; 32]
104    | 16   | nonce: u128
120    | 32   | proof_hash: [u8; 32]
152    | 8    | mined_at_slot: u64
160    | 8    | revealed_at_slot: u64
168    | 2    | consumed_count: u16
170    | 2    | max_consumes: u16
172    | 8    | expires_at_slot: u64
180    | 1    | status: ClaimStatus
181    | 75   | _reserved: [u8; 75]
```

**Claim Lifecycle**:
- **Mined**: PoW solution submitted, waiting for reveal
- **Revealed**: Available for consumption, has expiration
- **Active**: Being consumed (consumed_count < max_consumes)
- **Consumed**: Fully consumed (consumed_count == max_consumes)
- **Expired**: Past expiration slot

## PoW Algorithm

### Mining Process

1. **Fetch Parameters**: Get current difficulty and recent slot hash
2. **Build Preimage**: `DOMAIN(17) + slot(8) + slot_hash(32) + miner(32) + batch_hash(32) + nonce(16)`
3. **Hash**: `BLAKE3(preimage)`
4. **Verify**: `hash < difficulty_target`
5. **Submit**: Send mine_claim instruction with solution

### Difficulty Adjustment

- **Target Interval**: Aim for 1 solution per `target_interval_slots`
- **Retarget Frequency**: Adjust every `DEFAULT_RETARGET_INTERVAL` slots
- **Clamp**: ±20% maximum change per adjustment
- **Bounds**: Between `min_difficulty` and `max_difficulty`

### Wildcard Claims

- **Batch Hash**: `[0; 32]` indicates wildcard
- **Usage**: Can be consumed for any withdrawal batch
- **Value**: Higher utility, potentially higher fees

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 0x00 | InvalidProofHash | PoW verification failed |
| 0x01 | DifficultyNotMet | Solution doesn't meet difficulty |
| 0x02 | SlotHashMismatch | Slot hash doesn't match sysvar |
| 0x03 | SlotHashNotFound | Slot hash too old/not found |
| 0x04 | RevealWindowExpired | Too late to reveal claim |
| 0x05 | ClaimWindowExpired | Claim has expired |
| 0x06 | AlreadyRevealed | Claim already revealed |
| 0x07 | NotRevealed | Claim not revealed yet |
| 0x08 | FullyConsumed | Claim fully consumed |
| 0x09 | BatchSizeTooLarge | Batch exceeds max_k |
| 0x0A | FeeShareTooHigh | Fee share exceeds maximum |
| 0x0B | InvalidMinerAuthority | Invalid miner authority |
| 0x0C | MinerNotRegistered | Miner not registered |
| 0x0D | InvalidAdminAuthority | Invalid admin authority |
| 0x0E | ArithmeticOverflow | Integer overflow |
| 0x0F | InvalidDifficulty | Invalid difficulty bounds |
| 0x10 | InvalidSlotHashesSysvar | Invalid SlotHashes sysvar |
| 0x11 | UnauthorizedMiner | Miner authority mismatch |
| 0x12 | SlotTooOld | Slot outside valid range |
| 0x13 | BatchSizeExceedsMaxK | Batch size too large |
| 0x14 | InvalidBatchSize | Batch size must be > 0 |
| 0x15 | SlotNotFound | Slot not in SlotHashes |
| 0x16 | InvalidClaimStatus | Wrong claim status |
| 0x17 | ClaimExpired | Claim has expired |
| 0x18 | BatchHashMismatch | Batch hash mismatch |
| 0x19 | InvalidTag | Unknown instruction tag |

## Constants

```rust
// Domain tags for BLAKE3 hashing
DOMAIN: "CLOAK:SCRAMBLE:v1"
JOB_DOMAIN: "CLOAK:JOB:v1"
BATCH_DOMAIN: "CLOAK:BATCH:v1"

// Mining parameters
MAX_FEE_SHARE_BPS: 5000  // 50%
MAX_BATCH_SIZE: 20
DEFAULT_RETARGET_INTERVAL: 1000 slots
DEFAULT_TARGET_INTERVAL: 100 slots

// Difficulty adjustment
DIFFICULTY_CLAMP_MIN: 0.8  // -20%
DIFFICULTY_CLAMP_MAX: 1.2  // +20%
```

## Integration with Shield Pool

The scramble registry integrates with the shield-pool program through CPI calls:

1. **Withdraw Request**: User requests withdrawal with batch_hash
2. **Claim Search**: Shield-pool searches for available claims
3. **CPI Call**: Shield-pool calls `consume_claim` instruction
4. **Validation**: Registry validates and consumes claim
5. **Fee Distribution**: Miner earns fees from withdrawal

## Build & Test

### Build Program
```bash
cargo build-sbf
```

### Run Tests
```bash
cargo test
```

### Unit Tests
- `test_instruction_parsing`: Instruction data parsing
- `test_pow_verification`: BLAKE3 PoW validation
- `test_claim_lifecycle`: Mine → Reveal → Consume flow
- `test_difficulty_adjustment`: Automatic difficulty retargeting
- `test_wildcard_claims`: Wildcard claim functionality

### Integration Tests
- `test_miner_registration`: End-to-end miner setup
- `test_mining_workflow`: Complete mining process
- `test_claim_consumption`: Shield-pool integration

## Usage Example

```rust
// 1. Initialize registry
let init_ix = Instruction {
    program_id: SCRAMBLE_REGISTRY_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(registry_pubkey, false),
        AccountMeta::new_readonly(admin_pubkey, true),
    ],
    data: [
        &[0x00], // tag
        initial_difficulty.as_ref(),
        min_difficulty.as_ref(),
        max_difficulty.as_ref(),
        &target_interval.to_le_bytes(),
        &fee_share_bps.to_le_bytes(),
        &reveal_window.to_le_bytes(),
        &claim_window.to_le_bytes(),
        &max_k.to_le_bytes(),
    ].concat(),
};

// 2. Register miner
let register_ix = Instruction {
    program_id: SCRAMBLE_REGISTRY_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(miner_pubkey, false),
        AccountMeta::new_readonly(registry_pubkey, false),
        AccountMeta::new_readonly(miner_authority, true),
        AccountMeta::new_readonly(clock_sysvar, false),
    ],
    data: vec![0x01], // tag
};

// 3. Mine claim
let mine_data = [
    &[0x02], // tag
    &slot.to_le_bytes(),
    slot_hash.as_ref(),
    batch_hash.as_ref(),
    &nonce.to_le_bytes(),
    proof_hash.as_ref(),
    &max_consumes.to_le_bytes(),
].concat();

let mine_ix = Instruction {
    program_id: SCRAMBLE_REGISTRY_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(claim_pubkey, false),
        AccountMeta::new(miner_pubkey, false),
        AccountMeta::new(registry_pubkey, false),
        AccountMeta::new_readonly(miner_authority, true),
        AccountMeta::new_readonly(slot_hashes_sysvar, false),
        AccountMeta::new_readonly(clock_sysvar, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ],
    data: mine_data,
};

// 4. Reveal claim
let reveal_ix = Instruction {
    program_id: SCRAMBLE_REGISTRY_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(claim_pubkey, false),
        AccountMeta::new_readonly(registry_pubkey, false),
        AccountMeta::new_readonly(miner_authority, true),
        AccountMeta::new_readonly(clock_sysvar, false),
    ],
    data: vec![0x03], // tag
};

// 5. Consume claim (from shield-pool)
let consume_data = [
    &[0x04], // tag
    expected_miner_authority.as_ref(),
    expected_batch_hash.as_ref(),
].concat();

let consume_ix = Instruction {
    program_id: SCRAMBLE_REGISTRY_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(claim_pubkey, false),
        AccountMeta::new(miner_pubkey, false),
        AccountMeta::new(registry_pubkey, false),
        AccountMeta::new_readonly(shield_pool_program_id, false),
        AccountMeta::new_readonly(clock_sysvar, false),
    ],
    data: consume_data,
};
```

## Security Notes

- **PoW Verification**: BLAKE3 hash must meet difficulty target
- **Slot Validation**: Slot hash must match SlotHashes sysvar
- **Authority Checks**: Only registered miners can mine/reveal
- **CPI Protection**: Only shield-pool can consume claims
- **Expiration**: Claims expire after claim_window slots
- **Double-Spend**: Claims can only be consumed up to max_consumes
- **Difficulty Bounds**: Difficulty clamped to prevent manipulation

## Dependencies

- **pinocchio**: Efficient Solana program framework
- **blake3**: Hash function for PoW verification
- **five8_const**: Constant pubkey encoding
- **pinocchio-system**: System program integration
- **pinocchio-token**: Token program integration

## Deployment

The program compiles to BPF bytecode for Solana deployment. The registry must be initialized before miners can register and start mining. All accounts are created dynamically as needed during the mining process.

## Mining Economics

- **Mining Cost**: CPU cycles for BLAKE3 hashing
- **Revenue**: Fees from consumed claims
- **Competition**: Difficulty adjusts to maintain target interval
- **Efficiency**: Miners optimize for hash rate vs. electricity cost
- **Sustainability**: Economic incentives align with protocol needs
