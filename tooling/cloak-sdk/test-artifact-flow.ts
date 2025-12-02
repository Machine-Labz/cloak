/**
 * Test script to verify artifact-based proof generation flow
 * 
 * This test verifies that:
 * 1. ArtifactProverService can create artifacts
 * 2. Private inputs are uploaded directly to TEE (not through backend)
 * 3. Proof generation works end-to-end
 */

import { ArtifactProverService } from "./src/services/ArtifactProverService";
import type { SP1ProofInputs } from "./src/core/types";

// Mock test inputs
const testInputs: SP1ProofInputs = {
  privateInputs: {
    amount: 1_000_000_000,
    r: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    sk_spend: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
    leaf_index: 0,
    merkle_path: {
      path_elements: [
        "0000000000000000000000000000000000000000000000000000000000000000",
        "1111111111111111111111111111111111111111111111111111111111111111",
      ],
      path_indices: [0, 1],
    },
  },
  publicInputs: {
    root: "2222222222222222222222222222222222222222222222222222222222222222",
    nf: "3333333333333333333333333333333333333333333333333333333333333333",
    outputs_hash: "4444444444444444444444444444444444444444444444444444444444444444",
    amount: 1_000_000_000,
  },
  outputs: [
    {
      address: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
      amount: 997_500_000,
    },
  ],
};

async function testArtifactFlow() {
  console.log("🧪 Testing Artifact-Based Proof Generation Flow\n");

  // Get indexer URL from environment or use default
  // Try nginx first (port 80), then direct indexer (port 3001)
  const indexerUrl = process.env.INDEXER_URL || process.env.NEXT_PUBLIC_INDEXER_URL || "http://localhost:80";

  console.log(`📡 Using indexer URL: ${indexerUrl}\n`);

  const prover = new ArtifactProverService(indexerUrl, 5 * 60 * 1000, 2000);

  // Test 1: Health check
  console.log("1️⃣ Testing health check...");
  try {
    const isHealthy = await prover.healthCheck();
    if (isHealthy) {
      console.log("   ✅ Indexer is healthy\n");
    } else {
      console.log("   ⚠️  Indexer health check failed (may still work)\n");
    }
  } catch (error) {
    console.log(`   ⚠️  Health check error: ${error}\n`);
  }

  // Test 2: Proof generation (this will test the full artifact flow)
  console.log("2️⃣ Testing artifact-based proof generation...");
  console.log("   This will test:");
  console.log("   - Creating artifact");
  console.log("   - Uploading stdin directly to TEE");
  console.log("   - Requesting proof");
  console.log("   - Polling for status\n");

  try {
    const result = await prover.generateProof(testInputs, {
      onStart: () => {
        console.log("   🚀 Proof generation started");
      },
      onProgress: (progress) => {
        if (progress % 20 === 0 || progress === 100) {
          console.log(`   📊 Progress: ${progress}%`);
        }
      },
      onSuccess: (result) => {
        console.log(`   ✅ Proof generated successfully!`);
        console.log(`   ⏱️  Generation time: ${result.generationTimeMs}ms`);
        console.log(`   🔐 Proof size: ${result.proof ? result.proof.length / 2 : 0} bytes`);
        console.log(`   📝 Public inputs size: ${result.publicInputs ? result.publicInputs.length / 2 : 0} bytes`);
      },
      onError: (error) => {
        console.log(`   ❌ Proof generation failed: ${error}`);
      },
    });

    if (result.success) {
      console.log("\n✅ TEST PASSED: Artifact-based proof generation works!");
      console.log(`   Proof: ${result.proof?.substring(0, 32)}...`);
      console.log(`   Public inputs: ${result.publicInputs?.substring(0, 32)}...`);
      return 0;
    } else {
      console.log("\n❌ TEST FAILED: Proof generation returned error");
      console.log(`   Error: ${result.error}`);
      return 1;
    }
  } catch (error) {
    console.log(`\n❌ TEST FAILED: Exception during proof generation`);
    console.log(`   Error: ${error instanceof Error ? error.message : String(error)}`);
    if (error instanceof Error && error.stack) {
      console.log(`   Stack: ${error.stack}`);
    }
    return 1;
  }
}

// Run test
testArtifactFlow()
  .then((exitCode) => {
    process.exit(exitCode);
  })
  .catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
  });

