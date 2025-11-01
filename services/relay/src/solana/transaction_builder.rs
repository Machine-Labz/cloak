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

use crate::error::Error;
use crate::planner::Output;

const PUBLIC_INPUTS_LEN: usize = 104;
const DUPLICATE_NULLIFIER_LEN: usize = 32;
const NUM_OUTPUTS_LEN: usize = 1;
const RECIPIENT_ADDR_LEN: usize = 32;
const RECIPIENT_AMOUNT_LEN: usize = 8;
const POW_BATCH_HASH_LEN: usize = 32;

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
        return Err(Error::ValidationError("Number of outputs must be between 1 and 10".into()));
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
        return Err(Error::ValidationError("Number of outputs must be between 1 and 10".into()));
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

/// Build an Instruction for shield-pool::Withdraw with discriminant = 2 (legacy, no PoW)
pub fn build_withdraw_instruction(
    program_id: Pubkey,
    body: &[u8],
    pool_pda: Pubkey,
    treasury: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    recipients: &[Pubkey], // 1-N recipient accounts
) -> Instruction {
    let mut data = Vec::with_capacity(1 + body.len());
    data.push(2u8); // ShieldPoolInstruction::Withdraw
    data.extend_from_slice(body);

    let mut accounts = Vec::with_capacity(5 + recipients.len());
    accounts.push(AccountMeta::new(pool_pda, false));                      // pool (writable)
    accounts.push(AccountMeta::new(treasury, false));                      // treasury (writable)
    accounts.push(AccountMeta::new_readonly(roots_ring_pda, false));       // roots ring (readonly)
    accounts.push(AccountMeta::new(nullifier_shard_pda, false));           // nullifier shard (writable)
    
    // Add all recipient accounts
    for recipient in recipients {
        accounts.push(AccountMeta::new(*recipient, false));
    }
    
    // Add system program at the end
    accounts.push(AccountMeta::new_readonly(system_program::id(), false));

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Build an Instruction for shield-pool::Withdraw with PoW accounts (discriminant = 2)
///
/// Accounts layout:
/// 0. pool_pda (writable)
/// 1. treasury (writable)
/// 2. roots_ring_pda (readonly)
/// 3. nullifier_shard_pda (writable)
/// 4..4+N. recipients (writable) - 1 to 10 recipient accounts
/// 4+N. system_program (readonly)
/// 4+N+1. scramble_registry_program (readonly) - NEW
/// 4+N+2. claim_pda (writable) - NEW
/// 4+N+3. miner_pda (writable) - NEW
/// 4+N+4. registry_pda (writable) - NEW
/// 4+N+5. clock_sysvar (readonly) - NEW
/// 4+N+6. miner_authority (writable) - NEW (receives fee share)
/// 4+N+7. shield_pool_program (readonly) - NEW (for CPI signer)
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
) -> Instruction {
    use solana_sdk::sysvar;

    let mut data = Vec::with_capacity(1 + body.len());
    data.push(2u8); // ShieldPoolInstruction::Withdraw
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

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Derive Shield Pool PDAs according to the on-chain program seeds.
pub(crate) fn derive_shield_pool_pdas(program_id: &Pubkey) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], program_id);
    let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], program_id);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], program_id);
    let (nullifier_shard_pda, _) = Pubkey::find_program_address(&[b"nullifier_shard"], program_id);
    (pool_pda, treasury_pda, roots_ring_pda, nullifier_shard_pda)
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
    let (registry_pda, _) =
        Pubkey::find_program_address(&[b"registry"], registry_program_id);

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
) -> Result<Transaction, Error> {
    let body = build_withdraw_ix_body(
        proof_bytes.as_slice(),
        &public_104,
        outputs,
    )?;
    let withdraw_ix = build_withdraw_instruction(
        program_id,
        &body,
        pool_pda,
        treasury,
        roots_ring_pda,
        nullifier_shard_pda,
        recipients,
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
    let pri_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_micro_lamports);

    let mut msg = Message::new(&[cu_ix, pri_ix, withdraw_ix], Some(&fee_payer));
    msg.recent_blockhash = recent_blockhash;
    let tx = Transaction::new_unsigned(msg);
    Ok(tx)
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
) -> Result<Transaction, Error> {
    let body = build_withdraw_ix_body_with_pow(
        proof_bytes.as_slice(),
        &public_104,
        outputs,
        &batch_hash,
    )?;
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
    );

    let cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
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

    // Derive PDAs using canonical seeds
    let (pool_pda, treasury, roots_ring_pda, nullifier_shard_pda) =
        derive_shield_pool_pdas(program_id);

    // Convert outputs to recipient pubkeys
    let recipients: Vec<Pubkey> = outputs
        .iter()
        .map(|o| Pubkey::new_from_array(o.address))
        .collect();

    // Use first recipient as fee payer by default (unsigned; caller can replace/sign appropriately)
    let fee_payer = recipients[0];

    let mut public_104_arr = [0u8; PUBLIC_INPUTS_LEN];
    public_104_arr.copy_from_slice(public_inputs_104);

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
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{pubkey::Pubkey, system_program};

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
        let body =
            build_withdraw_ix_body(proof.as_slice(), &public, &outputs).expect("body");
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
            derive_shield_pool_pdas(&program_id);
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
