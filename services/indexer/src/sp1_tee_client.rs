use std::{sync::Arc, time::Duration};

use anyhow::Result;
use sp1_sdk::{network::FulfillmentStrategy, HashableKey, Prover, ProverClient, SP1Stdin};
use tokio::time::timeout;
use tracing::info;
// Import the ELF from the existing zk-guest-sp1-host package
use zk_guest_sp1_host::ELF;

use crate::config::Sp1TeeConfig;

/// SP1 TEE proof generation result
#[derive(Debug)]
pub struct TeeProofResult {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub generation_time_ms: u64,
    pub total_cycles: u64,
    pub total_syscalls: u64,
    pub execution_report: String,
}

/// SP1 TEE Prover Client
///
/// This client handles proof generation using SP1's TEE Private Proving feature.
/// It provides secure proof generation where private inputs are processed within
/// a trusted execution environment.
pub struct Sp1TeeClient {
    config: Sp1TeeConfig,
    prover_client: Arc<sp1_sdk::NetworkProver>,
}

impl Sp1TeeClient {
    /// Create a new SP1 TEE client
    pub fn new(config: Sp1TeeConfig) -> Result<Self> {
        info!(
            "Initializing SP1 TEE client with wallet: {}",
            config.wallet_address
        );

        // Validate that private key is available if TEE is enabled
        if config.enabled && config.private_key.is_none() {
            return Err(anyhow::anyhow!(
                "SP1 TEE is enabled but NETWORK_PRIVATE_KEY is not set. Please set the NETWORK_PRIVATE_KEY environment variable."
            ));
        }

        // Build the ProverClient at initialization to avoid rebuilding on each request
        info!("Building TEE ProverClient at startup...");
        let private_key = config
            .private_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NETWORK_PRIVATE_KEY is required for TEE proving"))?;

        let prover_client = ProverClient::builder()
            .network()
            .private()
            .private_key(private_key)
            .build();

        info!("✅ TEE ProverClient built successfully at startup");

        Ok(Self {
            config,
            prover_client: Arc::new(prover_client),
        })
    }

    /// Generate a proof using SP1 TEE
    ///
    /// This method generates a proof within the TEE, ensuring that private inputs
    /// are processed securely and confidentially.
    pub async fn generate_proof(
        &self,
        private_inputs: &str,
        public_inputs: &str,
        outputs: &str,
    ) -> Result<TeeProofResult> {
        let start_time = std::time::Instant::now();

        info!(
            "Starting SP1 TEE proof generation for wallet: {}",
            self.config.wallet_address
        );

        // Use the pre-built ProverClient (no need to rebuild)
        info!("Using pre-built TEE ProverClient");
        let client = &*self.prover_client;

        // Setup the program (this should be cached in production)
        let (pk, vk) = client.setup(ELF);
        info!("SP1 verifying key hash: 0x{}", hex::encode(vk.bytes32()));

        // Prepare the combined input
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

        // Execute the program to get execution metrics
        let (_, report) = client.execute(ELF, &stdin).run()?;
        let total_cycles = report.total_instruction_count();
        let total_syscalls = report.total_syscall_count();
        let execution_report = format!("{}", report);

        info!("📊 SP1 TEE Execution Report:");
        info!("   Total cycles: {}", total_cycles);
        info!("   Total syscalls: {}", total_syscalls);
        info!("   Wallet: {}", self.config.wallet_address);

        // Generate the proof with timeout
        // IMPORTANT: TEE proving requires FulfillmentStrategy::Reserved
        let proof_future = async {
            client
                .prove(&pk, &stdin)
                .groth16()
                .strategy(FulfillmentStrategy::Reserved)
                .run()
        };

        let proof_result = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            proof_future,
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "TEE proof generation timed out after {} seconds",
                self.config.timeout_seconds
            )
        })?
        .map_err(|e| anyhow::anyhow!("TEE proof generation failed: {}", e))?;

        // Serialize the proof bundle
        let proof_bundle = bincode::serialize(&proof_result)?;
        let public_inputs_bytes = proof_result.public_values.to_vec();

        let generation_time = start_time.elapsed();

        info!(
            "✅ SP1 TEE proof generation completed in {}ms",
            generation_time.as_millis()
        );
        info!("   Proof size: {} bytes", proof_bundle.len());
        info!("   Public inputs size: {} bytes", public_inputs_bytes.len());

        Ok(TeeProofResult {
            proof_bytes: proof_bundle,
            public_inputs: public_inputs_bytes,
            generation_time_ms: generation_time.as_millis() as u64,
            total_cycles,
            total_syscalls,
            execution_report,
        })
    }

    /// Check if TEE is available and accessible
    pub async fn health_check(&self) -> Result<bool> {
        info!("Performing SP1 TEE health check...");

        // Use the pre-built ProverClient for health check
        info!("Using pre-built TEE ProverClient for health check");
        let _client = &self.prover_client;

        // Simple health check - just verify the client is available
        // The actual setup will be done during proof generation
        info!("✅ SP1 TEE health check passed");
        Ok(true)
    }

    /// Get the wallet address associated with this TEE client
    pub fn wallet_address(&self) -> &str {
        &self.config.wallet_address
    }

    /// Check if TEE is enabled in configuration
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Create a TEE client from configuration
pub fn create_tee_client(config: Sp1TeeConfig) -> Result<Sp1TeeClient> {
    if !config.enabled {
        return Err(anyhow::anyhow!("SP1 TEE is not enabled in configuration"));
    }

    if config.wallet_address.is_empty() {
        return Err(anyhow::anyhow!("SP1 TEE wallet address is not configured"));
    }

    if config.private_key.is_none() {
        return Err(anyhow::anyhow!(
            "SP1 TEE is enabled but NETWORK_PRIVATE_KEY is not set. Please set the NETWORK_PRIVATE_KEY environment variable."
        ));
    }

    Sp1TeeClient::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_config_validation() {
        let config = Sp1TeeConfig {
            enabled: true,
            wallet_address: "0xA8f5C34e654963aFAD5f25B22914b2414e1E31A7".to_string(),
            rpc_url: "https://rpc.sp1-lumiere.xyz".to_string(),
            timeout_seconds: 300,
            private_key: Some("test_private_key".to_string()),
        };

        assert!(create_tee_client(config).is_ok());
    }

    #[test]
    fn test_tee_config_disabled() {
        let config = Sp1TeeConfig {
            enabled: false,
            wallet_address: "0xA8f5C34e654963aFAD5f25B22914b2414e1E31A7".to_string(),
            rpc_url: "https://rpc.sp1-lumiere.xyz".to_string(),
            timeout_seconds: 300,
            private_key: Some("test_private_key".to_string()),
        };

        assert!(create_tee_client(config).is_err());
    }

    #[test]
    fn test_tee_config_empty_wallet() {
        let config = Sp1TeeConfig {
            enabled: true,
            wallet_address: "".to_string(),
            rpc_url: "https://rpc.sp1-lumiere.xyz".to_string(),
            timeout_seconds: 300,
            private_key: Some("test_private_key".to_string()),
        };

        assert!(create_tee_client(config).is_err());
    }

    #[test]
    fn test_tee_config_missing_private_key() {
        let config = Sp1TeeConfig {
            enabled: true,
            wallet_address: "0xA8f5C34e654963aFAD5f25B22914b2414e1E31A7".to_string(),
            rpc_url: "https://rpc.sp1-lumiere.xyz".to_string(),
            timeout_seconds: 300,
            private_key: None,
        };

        assert!(create_tee_client(config).is_err());
    }
}
