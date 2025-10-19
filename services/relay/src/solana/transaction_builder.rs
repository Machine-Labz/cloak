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

use super::Output;
use crate::error::Error;

/// Build the 437-byte withdraw instruction body:
/// [proof:260][public:104][nf-dup:32][num_outputs:1][recipient:32][amount:8]
pub fn build_withdraw_ix_body(
    groth16_260: &[u8; 260],
    public_104: &[u8; 104],
    recipient_addr_32: &[u8; 32],
    recipient_amount: u64,
) -> Result<Vec<u8>, Error> {
    let mut data = Vec::with_capacity(437);
    data.extend_from_slice(groth16_260);
    data.extend_from_slice(public_104);
    // duplicate nullifier (bytes 32..64 of public inputs)
    let nf = &public_104[32..64];
    data.extend_from_slice(nf);
    // num_outputs = 1 (MVP)
    data.push(1u8);
    // recipient address
    data.extend_from_slice(recipient_addr_32);
    // recipient amount LE
    data.extend_from_slice(&recipient_amount.to_le_bytes());

    if data.len() != 437 {
        return Err(Error::ValidationError(format!(
            "withdraw body must be 437 bytes, got {}",
            data.len()
        )));
    }
    Ok(data)
}

/// Build an Instruction for shield-pool::Withdraw with discriminant = 2
pub fn build_withdraw_instruction(
    program_id: Pubkey,
    body_437: &[u8],
    pool_pda: Pubkey,
    treasury: Pubkey,
    roots_ring_pda: Pubkey,
    nullifier_shard_pda: Pubkey,
    recipient: Pubkey,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + 437);
    data.push(2u8); // ShieldPoolInstruction::Withdraw
    data.extend_from_slice(body_437);

    let accounts = vec![
        AccountMeta::new(pool_pda, false),                // pool (writable)
        AccountMeta::new(treasury, false),                // treasury (writable)
        AccountMeta::new_readonly(roots_ring_pda, false), // roots ring (readonly)
        AccountMeta::new(nullifier_shard_pda, false),     // nullifier shard (writable)
        AccountMeta::new(recipient, false),               // recipient (writable)
        AccountMeta::new_readonly(system_program::id(), false),
    ];

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

/// Build a full legacy Transaction including compute budget and priority fee.
pub fn build_withdraw_transaction(
    groth16_260: [u8; 260],
    public_104: [u8; 104],
    recipient_addr_32: [u8; 32],
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
) -> Result<Transaction, Error> {
    let body = build_withdraw_ix_body(
        &groth16_260,
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

    let mut msg = Message::new(&[cu_ix, pri_ix, withdraw_ix], Some(&fee_payer));
    msg.recent_blockhash = recent_blockhash;
    let tx = Transaction::new_unsigned(msg);
    Ok(tx)
}

/// Build a VersionedTransaction (for Jito bundles) when feature `jito` is enabled.
#[cfg(feature = "jito")]
pub fn build_withdraw_versioned(
    groth16_260: [u8; 260],
    public_104: [u8; 104],
    recipient_addr_32: [u8; 32],
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
        &groth16_260,
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

/// Build a VersionedTransaction with a Jito tip instruction.
/// The tip is added as the final instruction in the transaction.
#[cfg(feature = "jito")]
pub fn build_withdraw_versioned_with_tip(
    groth16_260: [u8; 260],
    public_104: [u8; 104],
    recipient_addr_32: [u8; 32],
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
        &groth16_260,
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
    proof_bytes_bundle: &[u8],
    public_inputs_104: &[u8],
    outputs: &[Output],
    recent_blockhash: Hash,
) -> Result<Transaction, Error> {
    if public_inputs_104.len() != 104 {
        return Err(Error::ValidationError(
            "public inputs must be 104 bytes".into(),
        ));
    }
    if outputs.len() != 1 {
        return Err(Error::ValidationError(
            "exactly 1 output required in MVP".into(),
        ));
    }
    let out = &outputs[0];
    let recipient = out.to_pubkey()?;

    // Extract 260-byte proof fragment from bundle
    let groth = cloak_proof_extract::extract_groth16_260(proof_bytes_bundle)
        .map_err(|_| Error::ValidationError("failed to extract 260-byte proof".into()))?;

    // Derive PDAs using canonical seeds
    let (pool_pda, treasury, roots_ring_pda, nullifier_shard_pda) =
        derive_shield_pool_pdas(program_id);
    // Use recipient as fee payer by default (unsigned; caller can replace/sign appropriately)
    let fee_payer = recipient;

    let mut public_104_arr = [0u8; 104];
    public_104_arr.copy_from_slice(public_inputs_104);

    let mut recipient_addr_32 = [0u8; 32];
    recipient_addr_32.copy_from_slice(recipient.as_ref());

    build_withdraw_transaction(
        groth,
        public_104_arr,
        recipient_addr_32,
        out.amount,
        *program_id,
        pool_pda,
        roots_ring_pda,
        nullifier_shard_pda,
        treasury,
        recipient,
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
        let proof = [0xAAu8; 260];
        let mut public = [0u8; 104];
        // root 0..32, nf 32..64, outputs_hash 64..96, amount 96..104
        public[0..32].copy_from_slice(&[0x11u8; 32]);
        public[32..64].copy_from_slice(&[0x22u8; 32]);
        public[64..96].copy_from_slice(&[0x33u8; 32]);
        let amt: u64 = 0x0102_0304_0506_0708;
        public[96..104].copy_from_slice(&amt.to_le_bytes());

        let recip = [0x44u8; 32];
        let out_amt: u64 = 123_456u64;
        let body = build_withdraw_ix_body(&proof, &public, &recip, out_amt).expect("body");
        assert_eq!(body.len(), 437);

        // Offsets
        assert_eq!(&body[0..260], &proof);
        assert_eq!(&body[260..364], &public);
        assert_eq!(&body[364..396], &public[32..64]); // nf dup
        assert_eq!(body[396], 1u8); // num outputs
        assert_eq!(&body[397..429], &recip);
        assert_eq!(&body[429..437], &out_amt.to_le_bytes());
    }

    #[test]
    fn test_legacy_builder_derives_pdas_and_accounts_order() {
        // Program id and PDAs
        let program_id = Pubkey::new_unique();

        // Minimal fake SP1 bundle: place a u64 LE length=260 followed by 260 nonzero bytes
        let mut bundle = vec![0u8; 16];
        bundle.extend_from_slice(&260u64.to_le_bytes());
        bundle.extend_from_slice(&[0xABu8; 260]);

        // Public inputs 104 bytes (zeros are acceptable for building)
        let public_inputs = vec![0u8; 104];

        // Single output as required (recipient base58 example)
        let outputs = vec![super::Output {
            recipient: "11111111111111111111111111111112".to_string(),
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

        // Data layout check: first byte is discriminant=2, then 437 bytes body
        assert_eq!(ci.data[0], 2u8);
        assert_eq!(ci.data.len() - 1, 437);
    }
}
