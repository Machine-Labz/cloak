//! A WASM wrapper around the `sp1_verifier` crate for withdrawal proof verification.

use sp1_verifier::{Groth16Verifier, PlonkVerifier, GROTH16_VK_BYTES, PLONK_VK_BYTES};
use wasm_bindgen::prelude::*;

/// Withdrawal proof data structure for JSON serialization
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WithdrawalProofData {
    pub proof: String,           // Hex-encoded proof bytes
    pub public_inputs: String,   // Hex-encoded public inputs
    pub vkey_hash: String,       // Verification key hash
    pub mode: String,            // "groth16" or "plonk"
    pub user_address: String,    // Hex-encoded user address
    pub pool_id: u64,
    pub user_balance: u64,
    pub withdrawal_amount: u64,
    pub pool_liquidity: u64,
    pub timestamp: u64,
    pub is_valid: bool,
}

/// Wrapper around [`sp1_verifier::Groth16Verifier::verify`] for withdrawal proofs.
///
/// We hardcode the Groth16 VK bytes to only verify SP1 proofs.
#[wasm_bindgen]
pub fn verify_withdrawal_groth16(proof: &[u8], public_inputs: &[u8], sp1_vk_hash: &str) -> bool {
    Groth16Verifier::verify(proof, public_inputs, sp1_vk_hash, *GROTH16_VK_BYTES).is_ok()
}

/// Wrapper around [`sp1_verifier::PlonkVerifier::verify`] for withdrawal proofs.
///
/// We hardcode the Plonk VK bytes to only verify SP1 proofs.
#[wasm_bindgen]
pub fn verify_withdrawal_plonk(proof: &[u8], public_inputs: &[u8], sp1_vk_hash: &str) -> bool {
    PlonkVerifier::verify(proof, public_inputs, sp1_vk_hash, *PLONK_VK_BYTES).is_ok()
}

/// Verify a withdrawal proof from JSON data
#[wasm_bindgen]
pub fn verify_withdrawal_proof_json(proof_data: &str) -> Result<bool, JsValue> {
    let proof_data: WithdrawalProofData = serde_json::from_str(proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse proof data: {}", e)))?;

    let proof_bytes = hex::decode(&proof_data.proof)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode proof: {}", e)))?;
    
    let public_inputs_bytes = hex::decode(&proof_data.public_inputs)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode public inputs: {}", e)))?;

    let result = match proof_data.mode.as_str() {
        "groth16" => verify_withdrawal_groth16(&proof_bytes, &public_inputs_bytes, &proof_data.vkey_hash),
        "plonk" => verify_withdrawal_plonk(&proof_bytes, &public_inputs_bytes, &proof_data.vkey_hash),
        _ => return Err(JsValue::from_str("Unsupported proof mode")),
    };

    Ok(result)
}

/// Parse withdrawal proof data from JSON and return structured data
#[wasm_bindgen]
pub fn parse_withdrawal_proof(proof_data: &str) -> Result<JsValue, JsValue> {
    let proof_data: WithdrawalProofData = serde_json::from_str(proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse proof data: {}", e)))?;

    serde_wasm_bindgen::to_value(&proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof data: {}", e)))
}

/// Convert hex string to Uint8Array (helper function for JavaScript)
#[wasm_bindgen]
pub fn hex_to_bytes(hex_string: &str) -> Result<Vec<u8>, JsValue> {
    hex::decode(hex_string)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode hex string: {}", e)))
}

/// Convert bytes to hex string (helper function for JavaScript)
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

