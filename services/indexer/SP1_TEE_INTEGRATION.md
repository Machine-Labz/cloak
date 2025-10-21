# SP1 TEE Private Proving Integration

This document describes the integration of SP1's TEE Private Proving into the Cloak Indexer's `/prove` route.

## Overview

The integration provides secure proof generation using SP1's Trusted Execution Environment (TEE), ensuring that private inputs are processed confidentially within a trusted environment. The system automatically falls back to local proof generation if TEE is unavailable.

## Configuration

### Environment Variables

Add the following environment variables to your `.env` file:

```bash
# SP1 TEE Configuration
SP1_TEE_ENABLED=true
SP1_TEE_WALLET_ADDRESS=0xA8f5C34e654963aFAD5f25B22914b2414e1E31A7
SP1_TEE_RPC_URL=https://rpc.sp1-lumiere.xyz
SP1_TEE_TIMEOUT_SECONDS=300
# Required for SP1 TEE authentication - set this to your private key (without 0x prefix)
NETWORK_PRIVATE_KEY=your_private_key_here
```

### Configuration Options

- `SP1_TEE_ENABLED`: Enable/disable TEE proving (default: false)
- `SP1_TEE_WALLET_ADDRESS`: Your wallet address with TEE access
- `SP1_TEE_RPC_URL`: SP1 TEE RPC endpoint (default: https://rpc.sp1-lumiere.xyz)
- `SP1_TEE_TIMEOUT_SECONDS`: Timeout for TEE operations (default: 300)
- `NETWORK_PRIVATE_KEY`: **Required** - Your private key for TEE authentication

## API Usage

### POST /api/v1/prove

The `/prove` endpoint now supports both TEE and local proof generation:

**Request:**
```json
{
  "private_inputs": "{\"secret\": \"value\"}",
  "public_inputs": "{\"public\": \"data\"}",
  "outputs": "{\"result\": \"output\"}"
}
```

**Response (TEE):**
```json
{
  "success": true,
  "proof": "hex_encoded_proof_bytes",
  "public_inputs": "hex_encoded_public_inputs",
  "generation_time_ms": 1500,
  "total_cycles": 1000000,
  "total_syscalls": 500,
  "execution_report": "detailed_execution_report",
  "proof_method": "tee",
  "wallet_address": "0xA8f5C34e654963aFAD5f25B22914b2414e1E31A7",
  "error": null
}
```

**Response (Local Fallback):**
```json
{
  "success": true,
  "proof": "hex_encoded_proof_bytes",
  "public_inputs": "hex_encoded_public_inputs",
  "generation_time_ms": 2000,
  "total_cycles": 1000000,
  "total_syscalls": 500,
  "execution_report": "detailed_execution_report",
  "proof_method": "local",
  "wallet_address": null,
  "error": null
}
```

## Implementation Details

### TEE Client Module

The `Sp1TeeClient` handles communication with SP1's TEE:

```rust
use crate::sp1_tee_client::{Sp1TeeClient, create_tee_client};

// Create TEE client
let tee_client = create_tee_client(config.sp1_tee.clone())?;

// Generate proof
let result = tee_client.generate_proof(
    &private_inputs,
    &public_inputs,
    &outputs,
).await?;
```

### Key Implementation Detail: Private Key Configuration

**CRITICAL**: The SP1 SDK requires the private key to be passed via the `.private_key()` method when building the TEE client:

```rust
let client = ProverClient::builder()
    .network()
    .private()
    .private_key(private_key)  // ⚠️ THIS IS REQUIRED!
    .build();
```

**Without** the `.private_key()` call, the SDK will:
1. Look for `NETWORK_PRIVATE_KEY` environment variable
2. If not found, **silently fall back to local proving**

This was the root cause of the original issue where TEE proving was not working despite being enabled and configured correctly.

### Fallback Mechanism

The system automatically falls back to local proof generation if:
- TEE is disabled in configuration
- TEE client creation fails
- TEE proof generation fails
- TEE timeout is exceeded

### Error Handling

The integration includes comprehensive error handling:
- Configuration validation
- Network connectivity issues
- TEE service unavailability
- Timeout handling
- Graceful fallback to local proving

## Hardware Requirements

For production deployment, ensure your environment meets SP1 TEE requirements:

### Supported Cloud Providers

**Google Cloud Platform (GCP):**
- Instance Type: c3-standard-* family
- OS: containerOS, RHEL0, SLES-15-sp5, Ubuntu 22.04
- Zones: asia-southeast-1-{a,b,c}, europe-west4-{a,b}, us-central1-{a,b,c}

**Microsoft Azure:**
- Instance Types: DCesv5-series, DCedsv5-series, ECesv5-series, ECedsv5-series
- OS: Ubuntu 24.04/22.04 Server (Confidential VM)
- Regions: West Europe, Central US, East US 2, North Europe

### System Requirements

- Linux Kernel: Version 6.7 or higher
- Hypervisor: KVM
- Access to `/sys/kernel/config/tsm/report`
- Sudo privileges for temporary directory creation

## Testing

Run the TEE integration tests:

```bash
cargo test sp1_tee_client --lib
```

## Security Considerations

1. **Private Input Protection**: TEE ensures private inputs are processed within a trusted environment
2. **Wallet Authentication**: Only authorized wallets can access TEE proving
3. **Network Security**: All communication with TEE uses secure TLS
4. **Fallback Security**: Local proving maintains existing security guarantees

## Monitoring and Logging

The integration includes comprehensive logging:

- TEE client creation and configuration
- Proof generation attempts and results
- Fallback triggers and reasons
- Performance metrics (cycles, syscalls, timing)
- Error conditions and recovery

## Troubleshooting

### Common Issues

1. **NETWORK_PRIVATE_KEY Not Set Error**
   ```
   NETWORK_PRIVATE_KEY environment variable is not set. Please set it to your private key or use the .private_key() method.
   ```
   - **Solution**: Set the `NETWORK_PRIVATE_KEY` environment variable to your private key
   - **Important**: The private key should be in hex format WITHOUT the `0x` prefix
   - **Example**: `NETWORK_PRIVATE_KEY=your_private_key_here`
   - **Note**: If your private key has `0x` prefix, the system will automatically strip it

2. **TEE Client Creation Fails**
   - Verify wallet address is correct
   - Check network connectivity to TEE endpoint
   - Ensure TEE service is available

3. **Proof Generation Timeout**
   - Increase `SP1_TEE_TIMEOUT_SECONDS`
   - Check TEE service performance
   - Verify input complexity

4. **Fallback to Local Proving**
   - Check TEE configuration
   - Verify wallet permissions
   - Review error logs for specific issues

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=cloak_indexer=debug
```

## Production Deployment

1. **Environment Setup**: Configure TEE environment variables
2. **Hardware Verification**: Ensure TEE-compatible infrastructure
3. **Wallet Configuration**: Set up authorized wallet address
4. **Monitoring**: Implement health checks and alerting
5. **Testing**: Perform end-to-end testing with real workloads

## References

- [SP1 TEE Documentation](https://docs.succinct.xyz/docs/sp1/prover-network/private-proving)
- [SP1 TEE Prover GitHub](https://github.com/automata-network/sp1-tee-prover)
- [Intel TDX Documentation](https://www.intel.com/content/www/us/en/developer/tools/trust-domain-extensions/overview.html)
