pub mod encoding;

use anyhow::Result;
use sp1_sdk::{include_elf, Prover, ProverClient, SP1Stdin};

pub const ELF: &[u8] = include_elf!("zk-guest-sp1-guest");

/// SP1 proof generation result
#[derive(Debug)]
pub struct ProofResult {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub generation_time_ms: u64,
    pub total_cycles: u64,
    pub total_syscalls: u64,
    pub execution_report: String, // Full execution report as formatted string
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

    let job =
        move || -> Result<(sp1_sdk::SP1ProofWithPublicValues, u64, u64, String), anyhow::Error> {
            let client = ProverClient::builder().cpu().build();
            let (pk, _vk) = client.setup(ELF);

            let combined_input = format!(
                r#"{{
                "private": {},
                "public": {},
                "outputs": {}
            }}"#,
                private_inputs, public_inputs, outputs
            );

            let mut stdin = SP1Stdin::new();
            stdin.write(&combined_input);

            // First, execute to get full execution report
            let (_, report) = client.execute(ELF, &stdin).run()?;
            let total_cycles = report.total_instruction_count();
            let total_syscalls = report.total_syscall_count();
            let execution_report = format!("{}", report); // Full formatted report

            // Then generate the proof
            let proof = client.prove(&pk, &stdin).groth16().run()?;

            Ok((proof, total_cycles, total_syscalls, execution_report))
        };

    let (proof_result, total_cycles, total_syscalls, execution_report) = run_prover_job(job)?;

    // Serialize the full SP1ProofWithPublicValues bundle (needed by relay to extract proof)
    // The relay will use cloak_proof_extract::extract_groth16_260 to get the 260-byte proof
    let proof_bundle = bincode::serialize(&proof_result)?;
    let public_inputs_bytes = proof_result.public_values.to_vec();

    let generation_time = start_time.elapsed();

    Ok(ProofResult {
        proof_bytes: proof_bundle,
        public_inputs: public_inputs_bytes,
        generation_time_ms: generation_time.as_millis() as u64,
        total_cycles,
        total_syscalls,
        execution_report,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn run_prover_job<F>(
    job: F,
) -> Result<(sp1_sdk::SP1ProofWithPublicValues, u64, u64, String), anyhow::Error>
where
    F: FnOnce() -> Result<(sp1_sdk::SP1ProofWithPublicValues, u64, u64, String), anyhow::Error>
        + Send
        + 'static,
{
    match std::thread::spawn(job).join() {
        Ok(Ok(artifacts)) => Ok(artifacts),
        Ok(Err(err)) => Err(err.into()),
        Err(_panic_info) => Err(anyhow::anyhow!(
            "SP1 proof generation panicked - this usually means invalid input data or circuit constraint failure"
        )),
    }
}

#[cfg(target_arch = "wasm32")]
fn run_prover_job<F>(
    job: F,
) -> Result<(sp1_sdk::SP1ProofWithPublicValues, u64, u64, String), anyhow::Error>
where
    F: FnOnce() -> Result<(sp1_sdk::SP1ProofWithPublicValues, u64, u64, String), anyhow::Error>,
{
    job().map_err(Into::into)
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
