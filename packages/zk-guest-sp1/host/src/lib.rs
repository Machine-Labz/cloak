pub mod encoding;

use anyhow::Result;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

const ELF: &[u8] = include_elf!("zk-guest-sp1-guest");

/// SP1 proof generation result
#[derive(Debug)]
pub struct ProofResult {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub generation_time_ms: u64,
}

/// Generate an SP1 proof directly from input data
/// 
/// This function replaces the need to call the cloak-zk binary externally.
/// It performs the same operations as the binary but as a library function.
pub fn generate_proof(
    private_inputs: &str,
    public_inputs: &str,
    outputs: &str,
) -> Result<ProofResult> {
    let start_time = std::time::Instant::now();

    // Convert to owned strings for thread safety
    let private_inputs = private_inputs.to_string();
    let public_inputs = public_inputs.to_string();
    let outputs = outputs.to_string();

    // Run SP1 proof generation in a separate thread to isolate panics
    let result = std::thread::spawn(move || {
        // Setup SP1 prover client
        let client = ProverClient::from_env();
        let (pk, _vk) = client.setup(ELF);

        // Create combined input (same format as the binary)
        let combined_input = format!(
            r#"{{
                "private": {},
                "public": {},
                "outputs": {}
            }}"#,
            private_inputs, public_inputs, outputs
        );

        // Generate proof
        let mut stdin = SP1Stdin::new();
        stdin.write(&combined_input);

        client.prove(&pk, &stdin).groth16().run()
    }).join();

    let proof_result = match result {
        Ok(Ok(proof)) => proof,
        Ok(Err(e)) => return Err(e),
        Err(_panic_info) => {
            return Err(anyhow::anyhow!("SP1 proof generation panicked - this usually means invalid input data or circuit constraint failure"));
        }
    };

    // Extract proof bytes and public inputs
    let proof_bytes = proof_result.bytes();
    let public_inputs_bytes = proof_result.public_values.to_vec();

    let generation_time = start_time.elapsed();

    Ok(ProofResult {
        proof_bytes: proof_bytes.to_vec(),
        public_inputs: public_inputs_bytes,
        generation_time_ms: generation_time.as_millis() as u64,
    })
}

/// Generate proof from structured input data
pub fn generate_proof_from_data(
    private: &serde_json::Value,
    public: &serde_json::Value,
    outputs: &serde_json::Value,
) -> Result<ProofResult> {
    let private_str = serde_json::to_string(private)?;
    let public_str = serde_json::to_string(public)?;
    let outputs_str = serde_json::to_string(outputs)?;

    generate_proof(&private_str, &public_str, &outputs_str)
}
