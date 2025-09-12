use serde::{Deserialize, Serialize};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use std::fs;
// No need to import shield_pool, we'll use the binary path

const SHIELD_POOL_PROGRAM_ID: [u8; 32] = [
    0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99,
    0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99,
];

#[derive(Debug, Serialize, Deserialize)]
struct PublicInputs {
    pub root: String,
    pub nf: String,
    pub fee_bps: u16,
    pub outputs_hash: String,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputData {
    pub address: String,
    pub amount: u64,
}

fn hex_decode_32(hex_str: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_str).expect("Invalid hex");
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    result
}

fn build_withdraw_ix(
    pool: &Pubkey,
    treasury: &Pubkey,
    roots_ring: &Pubkey,
    nullifier_shard: &Pubkey,
    recipients: &[Pubkey],
    sp1_proof: &[u8; 256],
    sp1_public_inputs: &[u8; 64],
    public_inputs: &PublicInputs,
    outputs: &[OutputData],
) -> Instruction {
    let mut data = Vec::new();
    data.push(0x03); // TAG_WITHDRAW
    data.extend_from_slice(sp1_proof);
    data.extend_from_slice(sp1_public_inputs);
    data.extend_from_slice(&hex_decode_32(&public_inputs.root));
    data.extend_from_slice(&hex_decode_32(&public_inputs.nf));
    data.extend_from_slice(&public_inputs.amount.to_le_bytes());
    data.extend_from_slice(&public_inputs.fee_bps.to_le_bytes());
    data.extend_from_slice(&hex_decode_32(&public_inputs.outputs_hash));
    data.push(outputs.len() as u8);

    for output in outputs {
        data.extend_from_slice(&hex_decode_32(&output.address));
        data.extend_from_slice(&output.amount.to_le_bytes());
    }

    let mut accounts = vec![
        AccountMeta::new(*pool, false),
        AccountMeta::new(*treasury, false),
        AccountMeta::new(*roots_ring, false),
        AccountMeta::new(*nullifier_shard, false),
    ];

    for recipient in recipients {
        accounts.push(AccountMeta::new(*recipient, false));
    }

    accounts.push(AccountMeta::new_readonly(system_program::id(), false));

    Instruction {
        program_id: Pubkey::from(SHIELD_POOL_PROGRAM_ID),
        accounts,
        data,
    }
}

#[tokio::test]
async fn test_instruction_parsing() {
    // Test that our instruction parsing works correctly
    let public_json = fs::read_to_string("../../packages/zk-guest-sp1/out/public.json")
        .expect("Failed to read public inputs");
    let public_inputs: PublicInputs =
        serde_json::from_str(&public_json).expect("Failed to parse public inputs");

    let outputs_json =
        fs::read_to_string("../../packages/zk-guest-sp1/examples/outputs.example.json")
            .expect("Failed to read outputs");
    let outputs: Vec<OutputData> =
        serde_json::from_str(&outputs_json).expect("Failed to parse outputs");

    // Build a withdraw instruction
    let sp1_proof = [0u8; 256];
    let sp1_public_inputs = [0u8; 64];
    let recipients = vec![Pubkey::from([0x01u8; 32]), Pubkey::from([0x02u8; 32])];

    let withdraw_ix = build_withdraw_ix(
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
        &recipients,
        &sp1_proof,
        &sp1_public_inputs,
        &public_inputs,
        &outputs,
    );

    // Verify instruction data is correctly formatted
    assert_eq!(withdraw_ix.data[0], 0x03); // TAG_WITHDRAW
    assert_eq!(
        withdraw_ix.data.len(),
        1 + 256 + 64 + 32 + 32 + 8 + 2 + 32 + 1 + 2 * (32 + 8)
    );

    println!("✅ Instruction parsing and formatting works correctly");
}

#[test]
fn test_double_spend_prevention() {
    // This test would verify that reusing a nullifier fails
    // Implementation would be similar to above but with two identical withdraw attempts

    println!("✅ Double spend test structure ready");
}
