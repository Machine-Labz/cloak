use shield_pool::instructions::ShieldPoolInstruction;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    system_program,
    transaction::Transaction,
};
#[cfg(feature = "jito")]
use solana_sdk::{message::VersionedMessage, transaction::VersionedTransaction};
use spl_token;

use crate::{error::Error, planner::Output};

// Manual implementation of associated token account derivation
// This avoids dependency conflicts with spl-associated-token-account
fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    // Associated Token Account Program ID
    const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
        solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

    // Token Program ID
    const TOKEN_PROGRAM_ID: Pubkey =
        solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    // Find the associated token account address
    let (ata, _) = Pubkey::find_program_address(
        &[wallet.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    ata
}

const PUBLIC_INPUTS_LEN: usize = 104;
const DUPLICATE_NULLIFIER_LEN: usize = 32;
const NUM_OUTPUTS_LEN: usize = 1;
const RECIPIENT_ADDR_LEN: usize = 32;
const RECIPIENT_AMOUNT_LEN: usize = 8;
const POW_BATCH_HASH_LEN: usize = 32;

// Swap-mode specific constants
const OUTPUT_MINT_LEN: usize = 32;
const RECIPIENT_ATA_LEN: usize = 32;
const MIN_OUTPUT_AMOUNT_LEN: usize = 8;

/// Build the withdraw instruction body (supports 1-N outputs)
/// Layout: [proof][public:104][nf-dup:32][num_outputs:1][(recipient:32, amount:8)...]
pub fn build_withdraw_ix_body(
    proof: &[u8],
    public_104: &[u8; PUBLIC_INPUTS_LEN],
    outputs: &[Output], // Vec of (address: [u8;32], amount: u64)
) -> Result<Vec<u8>, Error> {
    if proof.is_empty() {
        return Err(Error::ValidationError("proof must be non-empty".into()));
    }

    let num_outputs = outputs.len();
    if num_outputs == 0 || num_outputs > 10 {
        return Err(Error::ValidationError(
            "Number of outputs must be between 1 and 10".into(),
        ));
    }

    let per_output_len = RECIPIENT_ADDR_LEN + RECIPIENT_AMOUNT_LEN;
    let expected_len = proof.len()
        + PUBLIC_INPUTS_LEN
        + DUPLICATE_NULLIFIER_LEN
        + NUM_OUTPUTS_LEN
        + (per_output_len * num_outputs);

    let mut data = Vec::with_capacity(expected_len);
    data.extend_from_slice(proof);
    data.extend_from_slice(public_104);
    data.extend_from_slice(&public_104[32..64]); // duplicate nullifier
    data.push(num_outputs as u8); // number of outputs
    for output in outputs {
        data.extend_from_slice(&output.address);
        data.extend_from_slice(&output.amount.to_le_bytes());
    }

    debug_assert_eq!(data.len(), expected_len);
    Ok(data)
}

/// Build the withdraw instruction body with PoW batch hash appended
/// Layout: [proof][public:104][nf-dup:32][num_outputs:1][(recipient:32, amount:8)...][batch_hash:32]
pub fn build_withdraw_ix_body_with_pow(
    proof: &[u8],
    public_104: &[u8; PUBLIC_INPUTS_LEN],
    outputs: &[Output], // Vec of (address: [u8;32], amount: u64)
    batch_hash: &[u8; POW_BATCH_HASH_LEN],
) -> Result<Vec<u8>, Error> {
    if proof.is_empty() {
        return Err(Error::ValidationError("proof must be non-empty".into()));
    }

    let num_outputs = outputs.len();
    if num_outputs == 0 || num_outputs > 10 {
        return Err(Error::ValidationError(
            "Number of outputs must be between 1 and 10".into(),
        ));
    }

    let per_output_len = RECIPIENT_ADDR_LEN + RECIPIENT_AMOUNT_LEN;
    let expected_len = proof.len()
        + PUBLIC_INPUTS_LEN
        + DUPLICATE_NULLIFIER_LEN
        + NUM_OUTPUTS_LEN
        + (per_output_len * num_outputs)
        + POW_BATCH_HASH_LEN;

    let mut data = Vec::with_capacity(expected_len);
    data.extend_from_slice(proof);
    data.extend_from_slice(public_104);
    data.extend_from_slice(&public_104[32..64]);
    data.push(num_outputs as u8); // number of outputs
    for output in outputs {
        data.extend_from_slice(&output.address);
        data.extend_from_slice(&output.amount.to_le_bytes());
    }
    data.extend_from_slice(batch_hash);

    debug_assert_eq!(data.len(), expected_len);
    Ok(data)
}

/// Build the withdraw-swap instruction body (swap mode)
/// Layout: [proof][public:104][nf-dup:32][output_mint:32][recipient_ata:32][min_output_amount:8]
pub fn build_withdraw_swap_ix_body(
    proof: &[u8],
    public_104: &[u8; PUBLIC_INPUTS_LEN],
    output_mint: &Pubkey,
    recipient_ata: &Pubkey,
    min_output_amount: u64,
) -> Result<Vec<u8>, Error> {
    if proof.is_empty() {
        return Err(Error::ValidationError("proof must be non-empty".into()));
    }

    let expected_len = proof.len()
        + PUBLIC_INPUTS_LEN
        + DUPLICATE_NULLIFIER_LEN
        + OUTPUT_MINT_LEN
        + RECIPIENT_ATA_LEN
        + MIN_OUTPUT_AMOUNT_LEN;

    let mut data = Vec::with_capacity(expected_len);
    data.extend_from_slice(proof);
    data.extend_from_slice(public_104);
    // duplicate nullifier = public[32..64]
    data.extend_from_slice(&public_104[32..64]);
    data.extend_from_slice(output_mint.as_ref());
    data.extend_from_slice(recipient_ata.as_ref());
    data.extend_from_slice(&min_output_amount.to_le_bytes());

    debug_assert_eq!(data.len(), expected_len);

    tracing::info!(
        "WithdrawSwap instruction data: proof_len={}, total_len={}, expected=468",
        proof.len(),
        data.len()
    );

    Ok(data)
}

/// Build an Instruction for shield-pool::Withdraw with discriminant = 2
///
/// Accounts layout for native SOL:
/// 0. pool_pda (writable)
/// 1. treasury (writable)
/// 2. roots_ring_pda (readonly)
/// 3. nullifier_shard_pda (writable)
/// 4. recipient (writable)
/// 5. system_program (readonly)
///
/// Accounts layout for SPL tokens:
/// 0. pool_pda (writable)
/// 1. treasury (writable)
/// 2. roots_ring_pda (readonly)
/// 3. nullifier_shard_pda (writable)
/// 4. recipient (writable)
/// 5. system_program (readonly)
/// 6. token_program (readonly)
/// 7. pool_token_account (writable)
/// 8. recipient_token_account (writable)
pub fn build_withdraw_instruction(
    program_id: Pubkey,
    body: &[u8],
    pool_pda: Pubkey,
    treasury: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    recipients: &[Pubkey], // 1-N recipient accounts
    mint: Option<Pubkey>,
    pool_token_account: Option<Pubkey>,
    recipient_token_accounts: Option<&[Pubkey]>,
    treasury_token_account: Option<Pubkey>,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + body.len());
    data.push(ShieldPoolInstruction::Withdraw as u8);
    data.extend_from_slice(body);

    let mut accounts = Vec::with_capacity(5 + recipients.len());
    accounts.push(AccountMeta::new(pool_pda, false)); // pool (writable)
    accounts.push(AccountMeta::new(treasury, false)); // treasury (writable)
    accounts.push(AccountMeta::new_readonly(roots_ring_pda, false)); // roots ring (readonly)
    accounts.push(AccountMeta::new(nullifier_shard_pda, false)); // nullifier shard (writable)

    // Add all recipient accounts
    for recipient in recipients {
        accounts.push(AccountMeta::new(*recipient, false));
    }

    // Add system program at the end
    accounts.push(AccountMeta::new_readonly(system_program::id(), false));

    // Add SPL token accounts if mint is provided
    if let (Some(_mint), Some(pool_token), Some(recipient_tokens), Some(treasury_token)) = (
        mint,
        pool_token_account,
        recipient_token_accounts,
        treasury_token_account,
    ) {
        accounts.push(AccountMeta::new_readonly(spl_token::id(), false)); // token_program (readonly)
        accounts.push(AccountMeta::new(pool_token, false)); // pool_token_account (writable)
        for token_account in recipient_tokens {
            accounts.push(AccountMeta::new(*token_account, false)); // recipient_token_account (writable)
        }
        accounts.push(AccountMeta::new(treasury_token, false)); // treasury_token_account (writable)
    }

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Build an Instruction for shield-pool::WithdrawSwap (discriminant = 4)
///
/// Accounts layout:
/// 0. pool_pda (writable)
/// 1. treasury (writable)
/// 2. roots_ring_pda (readonly)
/// 3. nullifier_shard_pda (writable)
/// 4. swap_state_pda (writable)
/// 5. system_program (readonly)
/// 6. payer (signer, writable)
pub fn build_withdraw_swap_instruction(
    program_id: Pubkey,
    body: &[u8],
    pool_pda: Pubkey,
    treasury: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    swap_state_pda: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + body.len());
    data.push(ShieldPoolInstruction::WithdrawSwap as u8);
    data.extend_from_slice(body);

    let accounts = vec![
        AccountMeta::new(pool_pda, false),                      // 0
        AccountMeta::new(treasury, false),                      // 1
        AccountMeta::new_readonly(roots_ring_pda, false),       // 2
        AccountMeta::new(nullifier_shard_pda, false),           // 3
        AccountMeta::new(swap_state_pda, false),                // 4
        AccountMeta::new_readonly(system_program::id(), false), // 5
        AccountMeta::new(payer, true),                          // 6 (signer)
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Build an Instruction for shield-pool::Withdraw with PoW accounts (discriminant = 2)
///
/// 4..4+N. recipients (writable) - 1 to 10 recipient accounts
/// 4+N. system_program (readonly)
/// 4+N+1. scramble_registry_program (readonly)
/// 4+N+2. claim_pda (writable)
/// 4+N+3. miner_pda (writable)
/// 4+N+4. registry_pda (writable)
/// 4+N+5. clock_sysvar (readonly)
/// 4+N+6. miner_authority (writable)
/// 4+N+7. shield_pool_program (readonly)
/// 4+N+8. token_program (readonly) [present when mint provided]
/// 4+N+9. pool_token_account (writable)
/// 4+N+10..4+N+9+M. recipient_token_accounts (writable, one per recipient)
/// next. treasury_token_account (writable)
/// final (optional). miner_token_account (writable, when PoW + SPL)
pub fn build_withdraw_instruction_with_pow(
    program_id: Pubkey,
    body: &[u8],
    pool_pda: Pubkey,
    treasury: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    recipients: &[Pubkey],
    scramble_registry_program: Pubkey,
    claim_pda: Pubkey,
    miner_pda: Pubkey,
    registry_pda: Pubkey,
    miner_authority: Pubkey,
    mint: Option<Pubkey>,
    pool_token_account: Option<Pubkey>,
    recipient_token_accounts: Option<&[Pubkey]>,
    treasury_token_account: Option<Pubkey>,
    miner_token_account: Option<Pubkey>,
) -> Instruction {
    use solana_sdk::sysvar;

    let mut data = Vec::with_capacity(1 + body.len());
    data.push(ShieldPoolInstruction::Withdraw as u8);
    data.extend_from_slice(body);

    let mut accounts = Vec::with_capacity(12 + recipients.len());
    // Standard withdraw accounts
    accounts.push(AccountMeta::new(pool_pda, false));
    accounts.push(AccountMeta::new(treasury, false));
    accounts.push(AccountMeta::new_readonly(roots_ring_pda, false));
    accounts.push(AccountMeta::new(nullifier_shard_pda, false));

    // Add all recipient accounts
    for recipient in recipients {
        accounts.push(AccountMeta::new(*recipient, false));
    }

    // System program and PoW accounts
    accounts.push(AccountMeta::new_readonly(system_program::id(), false));
    accounts.push(AccountMeta::new_readonly(scramble_registry_program, false));
    accounts.push(AccountMeta::new(claim_pda, false));
    accounts.push(AccountMeta::new(miner_pda, false));
    accounts.push(AccountMeta::new(registry_pda, false));
    accounts.push(AccountMeta::new_readonly(sysvar::clock::id(), false));
    accounts.push(AccountMeta::new(miner_authority, false)); // Receives scrambler fee share
    accounts.push(AccountMeta::new_readonly(program_id, false)); // Shield-pool program ID (for CPI signer)

    // Add SPL token accounts if mint is provided
    if let (
        Some(_mint),
        Some(pool_token),
        Some(recipient_tokens),
        Some(treasury_token),
        Some(miner_token),
    ) = (
        mint,
        pool_token_account,
        recipient_token_accounts,
        treasury_token_account,
        miner_token_account,
    ) {
        accounts.push(AccountMeta::new_readonly(spl_token::id(), false)); // token_program (readonly)
        accounts.push(AccountMeta::new(pool_token, false)); // pool_token_account (writable)
        for token_account in recipient_tokens {
            accounts.push(AccountMeta::new(*token_account, false)); // recipient_token_account (writable)
        }
        accounts.push(AccountMeta::new(treasury_token, false)); // treasury_token_account (writable)
        accounts.push(AccountMeta::new(miner_token, false)); // miner_token_account (writable)
    }

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Derive Shield Pool PDAs according to the on-chain program seeds.
/// Now includes mint for multi-token support.
pub(crate) fn derive_shield_pool_pdas(
    program_id: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], program_id);
    let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury", mint.as_ref()], program_id);
    let (roots_ring_pda, _) =
        Pubkey::find_program_address(&[b"roots_ring", mint.as_ref()], program_id);
    let (nullifier_shard_pda, _) =
        Pubkey::find_program_address(&[b"nullifier_shard", mint.as_ref()], program_id);
    (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda)
}

/// Derive the swap state PDA for a given nullifier
pub(crate) fn derive_swap_state_pda(program_id: &Pubkey, nullifier: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"swap_state", nullifier.as_ref()], program_id)
}

/// Derive commitments PDA for a specific mint.
pub(crate) fn derive_commitments_pda(program_id: &Pubkey, mint: &Pubkey) -> Pubkey {
    let (commitments_pda, _) =
        Pubkey::find_program_address(&[b"commitments", mint.as_ref()], program_id);
    commitments_pda
}

/// Derive Scramble Registry PDAs according to the on-chain program seeds.
///
/// # Arguments
/// * `registry_program_id` - The scramble-registry program ID
/// * `miner_authority` - The miner's authority pubkey
/// * `batch_hash` - The batch commitment hash (32 bytes)
/// * `slot` - The slot when the claim was mined
///
/// # Returns
/// * `registry_pda` - Singleton registry state
/// * `miner_pda` - Miner account for the authority
/// * `claim_pda` - Claim account for this specific batch
pub fn derive_scramble_registry_pdas(
    registry_program_id: &Pubkey,
    miner_authority: &Pubkey,
    batch_hash: &[u8; 32],
    slot: u64,
) -> (Pubkey, Pubkey, Pubkey) {
    // Registry PDA: ["registry"]
    let (registry_pda, _) = Pubkey::find_program_address(&[b"registry"], registry_program_id);

    // Miner PDA: ["miner", authority]
    let (miner_pda, _) =
        Pubkey::find_program_address(&[b"miner", miner_authority.as_ref()], registry_program_id);

    // Claim PDA: ["claim", miner_authority, batch_hash, slot_le]
    let (claim_pda, _) = Pubkey::find_program_address(
        &[
            b"claim",
            miner_authority.as_ref(),
            batch_hash,
            &slot.to_le_bytes(),
        ],
        registry_program_id,
    );

    (registry_pda, miner_pda, claim_pda)
}

/// Build a full legacy Transaction including compute budget and priority fee (no PoW).
pub fn build_withdraw_transaction(
    proof_bytes: Vec<u8>,
    public_104: [u8; PUBLIC_INPUTS_LEN],
    outputs: &[Output],
    program_id: Pubkey,
    pool_pda: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    treasury: Pubkey,
    recipients: &[Pubkey],
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    priority_micro_lamports: u64,
    mint: Option<Pubkey>,
    pool_token_account: Option<Pubkey>,
    recipient_token_accounts: Option<&[Pubkey]>,
    treasury_token_account: Option<Pubkey>,
) -> Result<Transaction, Error> {
    let body = build_withdraw_ix_body(proof_bytes.as_slice(), &public_104, outputs)?;
    let withdraw_ix = build_withdraw_instruction(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        recipients,
        mint,
        pool_token_account,
        recipient_token_accounts,
        treasury_token_account,
    );

    // Optimize compute units for transaction size
    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);

    let mut msg = Message::new(&[cu_ix, pri_ix, withdraw_ix], Some(&fee_payer));
    msg.recent_blockhash = recent_blockhash;
    let tx = Transaction::new_unsigned(msg);
    Ok(tx)
}

/// Build a full Transaction for WithdrawSwap including compute budget
pub fn build_withdraw_swap_transaction(
    proof_bytes: Vec<u8>,
    public_104: [u8; PUBLIC_INPUTS_LEN],
    output_mint: Pubkey,
    recipient_ata: Pubkey,
    min_output_amount: u64,
    program_id: Pubkey,
    pool_pda: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    treasury: Pubkey,
    swap_state_pda: Pubkey,
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    priority_micro_lamports: u64,
) -> Result<Transaction, Error> {
    let body = build_withdraw_swap_ix_body(
        proof_bytes.as_slice(),
        &public_104,
        &output_mint,
        &recipient_ata,
        min_output_amount,
    )?;

    let withdraw_swap_ix = build_withdraw_swap_instruction(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        swap_state_pda,
        fee_payer,
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(600_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);
    let mut msg = Message::new(&[cu_ix, pri_ix, withdraw_swap_ix], Some(&fee_payer));
    msg.recent_blockhash = recent_blockhash;
    Ok(Transaction::new_unsigned(msg))
}

/// Build an Instruction for shield-pool::ExecuteSwap (discriminant = 5)
/// Data: [nullifier (32)]
/// Accounts:
/// 0. swap_state_pda (writable)
/// 1. recipient_ata (readonly)
/// 2. payer (writable)
/// 3. token_program (readonly)
pub fn build_execute_swap_instruction(
    program_id: Pubkey,
    nullifier: [u8; 32],
    swap_state_pda: Pubkey,
    recipient_ata: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + 32);
    data.push(ShieldPoolInstruction::ExecuteSwap as u8);
    data.extend_from_slice(&nullifier);

    let accounts = vec![
        AccountMeta::new(swap_state_pda, false),
        AccountMeta::new_readonly(recipient_ata, false),
        AccountMeta::new(payer, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn build_release_swap_funds_instruction(
    program_id: Pubkey,
    swap_state_pda: Pubkey,
    relay: Pubkey,
) -> Instruction {
    let data = vec![ShieldPoolInstruction::ReleaseSwapFunds as u8];

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(swap_state_pda, false),
            AccountMeta::new(relay, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    }
}

/// Build a full legacy Transaction with PoW support.
///
/// This variant includes the PoW scrambler accounts and batch_hash in instruction data.
#[allow(clippy::too_many_arguments)]
pub fn build_withdraw_transaction_with_pow(
    proof_bytes: Vec<u8>,
    public_104: [u8; PUBLIC_INPUTS_LEN],
    outputs: &[Output],
    batch_hash: [u8; POW_BATCH_HASH_LEN],
    program_id: Pubkey,
    pool_pda: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    treasury: Pubkey,
    recipients: &[Pubkey],
    scramble_registry_program: Pubkey,
    claim_pda: Pubkey,
    miner_pda: Pubkey,
    registry_pda: Pubkey,
    miner_authority: Pubkey,
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    priority_micro_lamports: u64,
    mint: Option<Pubkey>,
    pool_token_account: Option<Pubkey>,
    recipient_token_accounts: Option<&[Pubkey]>,
    treasury_token_account: Option<Pubkey>,
    miner_token_account: Option<Pubkey>,
) -> Result<Transaction, Error> {
    let body =
        build_withdraw_ix_body_with_pow(proof_bytes.as_slice(), &public_104, outputs, &batch_hash)?;
    let withdraw_ix = build_withdraw_instruction_with_pow(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        recipients,
        scramble_registry_program,
        claim_pda,
        miner_pda,
        registry_pda,
        miner_authority,
        mint,
        pool_token_account,
        recipient_token_accounts,
        treasury_token_account,
        miner_token_account,
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);

    let mut msg = Message::new(&[cu_ix, pri_ix, withdraw_ix], Some(&fee_payer));
    msg.recent_blockhash = recent_blockhash;
    let tx = Transaction::new_unsigned(msg);
    Ok(tx)
}

/// Build a VersionedTransaction (for Jito bundles) when feature `jito` is enabled.
#[cfg(feature = "jito")]
pub fn build_withdraw_versioned(
    proof_bytes: Vec<u8>,
    public_104: [u8; PUBLIC_INPUTS_LEN],
    recipient_addr_32: [u8; RECIPIENT_ADDR_LEN],
    recipient_amount: u64,
    program_id: Pubkey,
    pool_pda: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    treasury: Pubkey,
    recipient: Pubkey,
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    priority_micro_lamports: u64,
) -> Result<VersionedTransaction, Error> {
    let body = build_withdraw_ix_body(
        proof_bytes.as_slice(),
        &public_104,
        &recipient_addr_32,
        recipient_amount,
    )?;
    let withdraw_ix = build_withdraw_instruction(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        recipient,
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);

    let mut legacy = Message::new(&[cu_ix, pri_ix, withdraw_ix], Some(&fee_payer));
    legacy.recent_blockhash = recent_blockhash;
    let vmsg = VersionedMessage::Legacy(legacy);
    let vtx = VersionedTransaction {
        message: vmsg,
        signatures: vec![],
    };
    Ok(vtx)
}

/// Build a VersionedTransaction with a Jito tip instruction (no PoW).
/// The tip is added as the final instruction in the transaction.
#[cfg(feature = "jito")]
pub fn build_withdraw_versioned_with_tip(
    proof_bytes: Vec<u8>,
    public_104: [u8; PUBLIC_INPUTS_LEN],
    recipient_addr_32: [u8; RECIPIENT_ADDR_LEN],
    recipient_amount: u64,
    program_id: Pubkey,
    pool_pda: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    treasury: Pubkey,
    recipient: Pubkey,
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    priority_micro_lamports: u64,
    jito_tip_account: Pubkey,
    jito_tip_lamports: u64,
) -> Result<VersionedTransaction, Error> {
    use solana_sdk::system_instruction;

    let body = build_withdraw_ix_body(
        proof_bytes.as_slice(),
        &public_104,
        &recipient_addr_32,
        recipient_amount,
    )?;
    let withdraw_ix = build_withdraw_instruction(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        recipient,
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);

    // Add tip instruction as the last instruction in the bundle
    let tip_ix = system_instruction::transfer(&fee_payer, &jito_tip_account, jito_tip_lamports);

    let mut legacy = Message::new(&[cu_ix, pri_ix, withdraw_ix, tip_ix], Some(&fee_payer));
    legacy.recent_blockhash = recent_blockhash;
    let vmsg = VersionedMessage::Legacy(legacy);
    let vtx = VersionedTransaction {
        message: vmsg,
        signatures: vec![],
    };
    Ok(vtx)
}

/// Build a VersionedTransaction with PoW support and Jito tip.
#[cfg(feature = "jito")]
#[allow(clippy::too_many_arguments)]
pub fn build_withdraw_versioned_with_tip_and_pow(
    proof_bytes: Vec<u8>,
    public_104: [u8; PUBLIC_INPUTS_LEN],
    recipient_addr_32: [u8; RECIPIENT_ADDR_LEN],
    recipient_amount: u64,
    batch_hash: [u8; 32],
    program_id: Pubkey,
    pool_pda: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    treasury: Pubkey,
    recipient: Pubkey,
    scramble_registry_program: Pubkey,
    claim_pda: Pubkey,
    miner_pda: Pubkey,
    registry_pda: Pubkey,
    miner_authority: Pubkey,
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    priority_micro_lamports: u64,
    jito_tip_account: Pubkey,
    jito_tip_lamports: u64,
) -> Result<VersionedTransaction, Error> {
    use solana_sdk::system_instruction;

    let body = build_withdraw_ix_body_with_pow(
        proof_bytes.as_slice(),
        &public_104,
        &recipient_addr_32,
        recipient_amount,
        &batch_hash,
    )?;
    let withdraw_ix = build_withdraw_instruction_with_pow(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        recipient,
        scramble_registry_program,
        claim_pda,
        miner_pda,
        registry_pda,
        miner_authority,
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);

    // Add tip instruction as the last instruction in the bundle
    let tip_ix = system_instruction::transfer(&fee_payer, &jito_tip_account, jito_tip_lamports);

    let mut legacy = Message::new(&[cu_ix, pri_ix, withdraw_ix, tip_ix], Some(&fee_payer));
    legacy.recent_blockhash = recent_blockhash;
    let vmsg = VersionedMessage::Legacy(legacy);
    let vtx = VersionedTransaction {
        message: vmsg,
        signatures: vec![],
    };
    Ok(vtx)
}

/// Simulate a transaction and return consumed compute units.
pub async fn simulate(
    client: &solana_client::nonblocking::rpc_client::RpcClient,
    tx: &Transaction,
) -> Result<u64, Error> {
    let res = client
        .simulate_transaction(tx)
        .await
        .map_err(|e| Error::InternalServerError(format!("simulate failed: {}", e)))?;
    let cu = res.value.units_consumed.unwrap_or(0);
    tracing::info!("simulation units_consumed = {}", cu);
    Ok(cu)
}

// Back-compat wrapper used by SolanaService; extracts fragments and builds a basic transaction.
pub fn build_withdraw_instruction_legacy(
    program_id: &Pubkey,
    mint: &Pubkey,
    proof_bytes: &[u8],
    public_inputs_104: &[u8],
    outputs: &[Output],
    recent_blockhash: Hash,
) -> Result<Transaction, Error> {
    if public_inputs_104.len() != PUBLIC_INPUTS_LEN {
        return Err(Error::ValidationError(
            "public inputs must be 104 bytes".into(),
        ));
    }
    if outputs.is_empty() || outputs.len() > 10 {
        return Err(Error::ValidationError(
            "number of outputs must be between 1 and 10".into(),
        ));
    }

    if proof_bytes.is_empty() {
        return Err(Error::ValidationError(
            "proof bytes must be non-empty".into(),
        ));
    }

    // Derive PDAs using canonical seeds with mint
    let (pool_pda, treasury, roots_ring_pda, nullifier_shard_pda) =
        derive_shield_pool_pdas(program_id, mint);

    // Convert outputs to recipient pubkeys
    let recipients: Vec<Pubkey> = outputs
        .iter()
        .map(|o| Pubkey::new_from_array(o.address))
        .collect();

    // Use first recipient as fee payer by default (unsigned; caller can replace/sign appropriately)
    let fee_payer = recipients[0];

    let mut public_104_arr = [0u8; PUBLIC_INPUTS_LEN];
    public_104_arr.copy_from_slice(public_inputs_104);

    // Collect SPL token accounts when mint is provided
    let mut recipient_token_accounts_vec = Vec::new();
    let mut pool_token_account = None;
    let mut treasury_token_account = None;

    if *mint != Pubkey::default() {
        pool_token_account = Some(get_associated_token_address(&pool_pda, mint));
        treasury_token_account = Some(get_associated_token_address(&treasury, mint));
        for recipient in &recipients {
            recipient_token_accounts_vec.push(get_associated_token_address(recipient, mint));
        }
    }

    let recipient_token_accounts_slice = if recipient_token_accounts_vec.is_empty() {
        None
    } else {
        Some(recipient_token_accounts_vec.as_slice())
    };

    build_withdraw_transaction(
        proof_bytes.to_vec(),
        public_104_arr,
        outputs,
        *program_id,
        pool_pda,
        roots_ring_pda,
        nullifier_shard_pda,
        treasury,
        &recipients,
        fee_payer,
        recent_blockhash,
        1_000, // default priority fee (micro-lamports per CU)
        Some(*mint),
        pool_token_account,
        recipient_token_accounts_slice,
        treasury_token_account,
    )
}

// ============================================================================
// FUND WSOL ATA INSTRUCTION
// ============================================================================

/// Build FundWsolAta instruction to transfer SOL from SwapState PDA to wSOL ATA
///
/// This must be called BEFORE OrcaSwap to fund the wSOL token account.
/// Separated because Solana doesn't allow manual lamport manipulation + CPI in same instruction.
///
/// Instruction data: [discriminator:1][amount:8]
pub fn build_fund_wsol_instruction(
    program_id: Pubkey,
    nullifier: [u8; 32],
    amount: u64,
) -> Instruction {
    let (swap_state_pda, _) = derive_swap_state_pda(&program_id, &nullifier);
    let wsol_mint = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
    let wsol_ata = get_associated_token_address(&swap_state_pda, &wsol_mint);

    let mut data = Vec::with_capacity(9);
    data.push(7u8); // FundWsolAta discriminator
    data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
        AccountMeta::new(swap_state_pda, false),
        AccountMeta::new(wsol_ata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

// REMOVED: build_orca_swap_instruction - replaced by build_execute_swap_via_orca_instruction

// Commented out unused function - kept for reference
#[allow(dead_code)]
fn _build_orca_swap_instruction_old(
    program_id: Pubkey,
    nullifier: [u8; 32],
    amount: u64,
    min_output_amount: u64,
    output_mint: Pubkey,
    recipient_ata: Pubkey,
) -> Result<Instruction, Error> {
    // Orca Whirlpool Program ID (same for mainnet & devnet)
    let whirlpool_program_id = solana_sdk::pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

    // Derive SwapState PDA
    let (swap_state_pda, _) = derive_swap_state_pda(&program_id, &nullifier);

    // Native SOL mint (wrapped SOL)
    let wsol_mint = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");

    // Derive wSOL ATA for SwapState PDA
    let wsol_ata = get_associated_token_address(&swap_state_pda, &wsol_mint);

    // For devnet SOL/USDC pool with tick spacing 64
    // Devnet whirlpool config
    let devnet_config = solana_sdk::pubkey!("FcrweFY1G9HJAHG5inkGB6pKg1HZ6x9UC2WioAfWrGkR");

    // Derive Whirlpool PDA for SOL/USDC pool
    let (whirlpool_pda, _) = Pubkey::find_program_address(
        &[
            b"whirlpool",
            devnet_config.as_ref(),
            wsol_mint.as_ref(),
            output_mint.as_ref(),
            &64u16.to_le_bytes(), // tick spacing
        ],
        &whirlpool_program_id,
    );

    // For simplified implementation, we'll derive tick arrays with default indices
    // In production, these should be queried from the pool's current price
    let (tick_array_0, _) = Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool_pda.as_ref(),
            &0i32.to_le_bytes(), // start tick index
        ],
        &whirlpool_program_id,
    );

    let (tick_array_1, _) = Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool_pda.as_ref(),
            &(64i32).to_le_bytes(), // start tick index + tick spacing
        ],
        &whirlpool_program_id,
    );

    let (tick_array_2, _) = Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool_pda.as_ref(),
            &(128i32).to_le_bytes(), // start tick index + 2 * tick spacing
        ],
        &whirlpool_program_id,
    );

    // Note: In production, vault addresses should be queried from the whirlpool account
    // For now, we use the standard PDA derivation
    let (vault_a, _) =
        Pubkey::find_program_address(&[b"vault", whirlpool_pda.as_ref()], &whirlpool_program_id);

    let (vault_b, _) =
        Pubkey::find_program_address(&[b"vault", whirlpool_pda.as_ref()], &whirlpool_program_id);

    let (oracle, _) =
        Pubkey::find_program_address(&[b"oracle", whirlpool_pda.as_ref()], &whirlpool_program_id);

    // Build instruction data
    let mut data = Vec::with_capacity(33);
    data.push(6u8); // OrcaSwap discriminator
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&min_output_amount.to_le_bytes());
    data.extend_from_slice(&0u128.to_le_bytes()); // sqrt_price_limit = 0 (no limit)

    // Orca Whirlpool Program ID (same for mainnet & devnet)
    let whirlpool_program = solana_sdk::pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

    // Build accounts (old version without payer - function is deprecated)
    let accounts = vec![
        AccountMeta::new(swap_state_pda, false), // 0. swap_state_pda (writable)
        AccountMeta::new(wsol_ata, false),       // 1. swap_wsol_ata (writable)
        AccountMeta::new(recipient_ata, false),  // 2. recipient_ata (writable)
        AccountMeta::new(whirlpool_pda, false),  // 3. whirlpool (writable)
        AccountMeta::new(vault_a, false),        // 4. token_vault_a (writable)
        AccountMeta::new(vault_b, false),        // 5. token_vault_b (writable)
        AccountMeta::new(tick_array_0, false),   // 6. tick_array_0 (writable)
        AccountMeta::new(tick_array_1, false),   // 7. tick_array_1 (writable)
        AccountMeta::new(tick_array_2, false),   // 8. tick_array_2 (writable)
        AccountMeta::new_readonly(oracle, false), // 9. oracle (readonly)
        AccountMeta::new_readonly(spl_token::id(), false), // 10. token_program (readonly)
        AccountMeta::new_readonly(whirlpool_program, false), // 11. whirlpool_program (readonly)
    ];

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

/// Build PrepareSwapSol instruction - Transfer SOL from SwapState to wSOL ATA
///
/// This instruction only transfers lamports (no CPIs). The relay must call
/// SyncNative separately to wrap SOL → wSOL.
///
/// Instruction discriminator: 8 (PrepareSwapSol)
///
/// # Arguments
/// * `program_id` - Shield pool program ID
/// * `nullifier` - Nullifier to derive SwapState PDA
///
/// # Returns
/// * `Instruction` - PrepareSwapSol instruction
pub fn build_prepare_swap_sol_instruction(
    program_id: Pubkey,
    nullifier: [u8; 32],
) -> Result<Instruction, Error> {
    // Derive SwapState PDA
    let (swap_state_pda, _) = derive_swap_state_pda(&program_id, &nullifier);

    // Native SOL mint (wrapped SOL)
    let wsol_mint = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");

    // Derive wSOL ATA for SwapState PDA
    let swap_wsol_ata = get_associated_token_address(&swap_state_pda, &wsol_mint);

    // Build instruction data: just the discriminator (no additional data needed)
    let data = vec![8u8]; // PrepareSwapSol discriminator (ShieldPoolInstruction::PrepareSwapSol = 8)

    // Build accounts (order must match program's expected account order)
    let accounts = vec![
        AccountMeta::new(swap_state_pda, false), // 0. swap_state_pda (writable)
        AccountMeta::new(swap_wsol_ata, false),  // 1. swap_wsol_ata (writable)
    ];

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

/// Build ExecuteSwapViaOrca instruction - Atomic on-chain swap via Orca Whirlpool CPI
///
/// **Prerequisites**: wSOL must already be in swap_wsol_ata (call PrepareSwapSol first).
/// This instruction only performs the Orca swap CPI.
///
/// Instruction discriminator: 7 (ExecuteSwapViaOrca)
///
/// # Arguments
/// * `program_id` - Shield-pool program ID
/// * `nullifier` - Nullifier used to derive SwapState PDA
/// * `recipient_ata` - User's output token ATA (from SwapState)
/// * `amount` - Amount of wSOL to swap (from Orca quote)
/// * `other_amount_threshold` - Minimum output amount (from Orca quote)
/// * `sqrt_price_limit` - Price limit (from Orca quote)
/// * `amount_specified_is_input` - Whether amount is input (from Orca quote)
/// * `a_to_b` - Swap direction (from Orca quote)
/// * `whirlpool` - Orca Whirlpool pool address
/// * `token_vault_a` - Pool's wSOL vault
/// * `token_vault_b` - Pool's output token vault
/// * `tick_array_0` - First tick array
/// * `tick_array_1` - Second tick array
/// * `tick_array_2` - Third tick array
/// * `oracle` - Whirlpool oracle PDA
pub fn build_execute_swap_via_orca_instruction(
    program_id: Pubkey,
    nullifier: [u8; 32],
    recipient_ata: Pubkey,
    // Orca swap quote parameters
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    // Orca pool accounts (queried from on-chain data)
    whirlpool: Pubkey,
    token_vault_a: Pubkey,
    token_vault_b: Pubkey,
    tick_array_0: Pubkey,
    tick_array_1: Pubkey,
    tick_array_2: Pubkey,
    oracle: Pubkey,
    payer: Pubkey, // Payer account (receives rent from closing SwapState)
) -> Result<Instruction, Error> {
    // Derive SwapState PDA
    let (swap_state_pda, _) = derive_swap_state_pda(&program_id, &nullifier);

    // Native SOL mint (wrapped SOL)
    let wsol_mint = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");

    // Derive wSOL ATA for SwapState PDA
    let swap_wsol_ata = get_associated_token_address(&swap_state_pda, &wsol_mint);

    // Orca Whirlpool Program ID
    let whirlpool_program_id = solana_sdk::pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

    // Token program
    let token_program_id = spl_token::id();

    // Build instruction data: [discriminator: 1][amount: 8][other_amount_threshold: 8]
    //                          [sqrt_price_limit: 16][amount_specified_is_input: 1][a_to_b: 1]
    // Total: 35 bytes
    let mut data = Vec::with_capacity(35);
    data.push(7u8); // ExecuteSwapViaOrca discriminator (matches ShieldPoolInstruction::ExecuteSwapViaOrca = 7)
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&other_amount_threshold.to_le_bytes());
    data.extend_from_slice(&sqrt_price_limit.to_le_bytes());
    data.push(if amount_specified_is_input { 1 } else { 0 });
    data.push(if a_to_b { 1 } else { 0 });

    // Build accounts (order must match program's expected account order)
    // Note: payer is needed to receive rent when SwapState is closed after swap
    let accounts = vec![
        AccountMeta::new(swap_state_pda, false), // 0. swap_state_pda (writable, closed after swap)
        AccountMeta::new(swap_wsol_ata, false),  // 1. swap_wsol_ata (writable)
        AccountMeta::new(recipient_ata, false),  // 2. recipient_ata (writable)
        AccountMeta::new(whirlpool, false),      // 3. whirlpool (writable)
        AccountMeta::new(token_vault_a, false),  // 4. token_vault_a (writable)
        AccountMeta::new(token_vault_b, false),  // 5. token_vault_b (writable)
        AccountMeta::new(tick_array_0, false),   // 6. tick_array_0 (writable)
        AccountMeta::new(tick_array_1, false),   // 7. tick_array_1 (writable)
        AccountMeta::new(tick_array_2, false),   // 8. tick_array_2 (writable)
        AccountMeta::new_readonly(oracle, false), // 9. oracle (readonly)
        AccountMeta::new_readonly(token_program_id, false), // 10. token_program (readonly)
        AccountMeta::new_readonly(whirlpool_program_id, false), // 11. whirlpool_program (readonly)
        AccountMeta::new(payer, false), // 12. payer (writable) - receives rent from closing SwapState
    ];

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::{pubkey::Pubkey, system_program};

    use super::*;

    #[test]
    fn test_withdraw_body_layout() {
        const PROOF_LEN: usize = 1506;
        let proof = vec![0xAAu8; PROOF_LEN];
        let mut public = [0u8; PUBLIC_INPUTS_LEN];
        // root 0..32, nf 32..64, outputs_hash 64..96, amount 96..104
        public[0..32].copy_from_slice(&[0x11u8; 32]);
        public[32..64].copy_from_slice(&[0x22u8; 32]);
        public[64..96].copy_from_slice(&[0x33u8; 32]);
        let amt: u64 = 0x0102_0304_0506_0708;
        public[96..104].copy_from_slice(&amt.to_le_bytes());

        let recip = [0x44u8; RECIPIENT_ADDR_LEN];
        let out_amt: u64 = 123_456u64;
        let outputs = vec![Output {
            address: recip,
            amount: out_amt,
        }];
        let body = build_withdraw_ix_body(proof.as_slice(), &public, &outputs).expect("body");
        let expected_len = PROOF_LEN
            + PUBLIC_INPUTS_LEN
            + DUPLICATE_NULLIFIER_LEN
            + NUM_OUTPUTS_LEN
            + RECIPIENT_ADDR_LEN
            + RECIPIENT_AMOUNT_LEN;
        assert_eq!(body.len(), expected_len);

        let public_start = PROOF_LEN;
        let public_end = public_start + PUBLIC_INPUTS_LEN;
        assert_eq!(&body[..PROOF_LEN], proof.as_slice());
        assert_eq!(&body[public_start..public_end], &public);

        let dup_start = public_end;
        let dup_end = dup_start + DUPLICATE_NULLIFIER_LEN;
        assert_eq!(&body[dup_start..dup_end], &public[32..64]);

        let outputs_idx = dup_end;
        assert_eq!(body[outputs_idx], 1u8);

        let recip_start = outputs_idx + NUM_OUTPUTS_LEN;
        let recip_end = recip_start + RECIPIENT_ADDR_LEN;
        assert_eq!(&body[recip_start..recip_end], &recip);

        let amount_start = recip_end;
        let amount_end = amount_start + RECIPIENT_AMOUNT_LEN;
        assert_eq!(&body[amount_start..amount_end], &out_amt.to_le_bytes());
    }

    #[test]
    fn test_legacy_builder_derives_pdas_and_accounts_order() {
        // Program id and PDAs
        let program_id = Pubkey::new_unique();
        let mint = Pubkey::default(); // Native SOL for test

        // Minimal fake SP1 proof bundle
        let bundle = vec![0xABu8; 1506];

        // Public inputs 104 bytes (zeros are acceptable for building)
        let public_inputs = vec![0u8; 104];

        // Single output as required
        let recipient_pubkey = Pubkey::new_unique();
        let outputs = vec![Output {
            address: recipient_pubkey.to_bytes(),
            amount: 1_000_000,
        }];

        let blockhash = solana_sdk::hash::Hash::new_unique();
        let tx = build_withdraw_instruction_legacy(
            &program_id,
            &mint,
            &bundle,
            &public_inputs,
            &outputs,
            blockhash,
        )
        .expect("tx");

        let msg = tx.message();
        assert!(
            msg.instructions.len() >= 3,
            "expect CU, fee, and withdraw ix"
        );
        let ci = &msg.instructions[2];

        // Program id check
        let pid = msg.account_keys[ci.program_id_index as usize];
        assert_eq!(pid, program_id);

        // Resolve accounts by index and verify order
        let (exp_pool, exp_treasury, exp_roots, exp_nullifier) =
            derive_shield_pool_pdas(&program_id, &mint);
        let resolve = |ix: u8| msg.account_keys[ix as usize];
        assert_eq!(resolve(ci.accounts[0]), exp_pool);
        assert_eq!(resolve(ci.accounts[1]), exp_treasury);
        assert_eq!(resolve(ci.accounts[2]), exp_roots);
        assert_eq!(resolve(ci.accounts[3]), exp_nullifier);
        // recipient (we don't assert exact value here) and system program
        assert_eq!(resolve(ci.accounts[5]), system_program::id());

        // Data layout check: first byte is discriminant=2, then dynamic body
        assert_eq!(ci.data[0], 2u8);
        let expected_body_len = bundle.len()
            + PUBLIC_INPUTS_LEN
            + DUPLICATE_NULLIFIER_LEN
            + NUM_OUTPUTS_LEN
            + RECIPIENT_ADDR_LEN
            + RECIPIENT_AMOUNT_LEN;
        assert_eq!(ci.data.len() - 1, expected_body_len);
    }
}
