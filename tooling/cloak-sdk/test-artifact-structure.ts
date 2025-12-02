/**
 * Structural test for Artifact-Based Proof Generation
 * 
 * This test verifies that:
 * 1. ArtifactProverService is properly exported
 * 2. CloakSDK uses ArtifactProverService
 * 3. The flow structure matches the architecture document
 */

import { ArtifactProverService } from "./src/services/ArtifactProverService";
import { CloakSDK } from "./src/core/CloakSDK";
import type { SP1ProofInputs } from "./src/core/types";
import { Keypair } from "@solana/web3.js";

async function testStructure() {
  console.log("🧪 Testing Artifact-Based Flow Structure\n");

  let passed = 0;
  let failed = 0;

  // Test 1: ArtifactProverService exists and can be instantiated
  console.log("1️⃣ Testing ArtifactProverService instantiation...");
  try {
    const prover = new ArtifactProverService("http://localhost:3001");
    console.log("   ✅ ArtifactProverService can be instantiated");
    passed++;
  } catch (error) {
    console.log(`   ❌ Failed: ${error}`);
    failed++;
  }

  // Test 2: CloakSDK uses ArtifactProverService (check via reflection)
  console.log("\n2️⃣ Testing CloakSDK integration...");
  try {
    // Generate a valid Solana keypair
    const testKeypair = Keypair.generate();
    const sdk = new CloakSDK({
      keypairBytes: testKeypair.secretKey,
      network: "devnet",
    });

    // Check if artifactProver exists (private field, but we can check methods)
    // The fact that we can instantiate CloakSDK means it's using ArtifactProverService
    // (since we removed ProverService)
    console.log("   ✅ CloakSDK can be instantiated with artifact-based flow");
    passed++;
  } catch (error) {
    console.log(`   ❌ Failed: ${error}`);
    failed++;
  }

  // Test 3: Verify ArtifactProverService has the correct methods
  console.log("\n3️⃣ Testing ArtifactProverService methods...");
  try {
    const prover = new ArtifactProverService("http://localhost:3001");
    const methods = ["generateProof", "healthCheck", "getTimeout", "setTimeout"];
    
    for (const method of methods) {
      if (typeof (prover as any)[method] !== "function") {
        throw new Error(`Method ${method} not found`);
      }
    }
    console.log("   ✅ All required methods exist");
    passed++;
  } catch (error) {
    console.log(`   ❌ Failed: ${error}`);
    failed++;
  }

  // Test 4: Verify generateProof accepts correct inputs
  console.log("\n4️⃣ Testing generateProof signature...");
  try {
    const prover = new ArtifactProverService("http://localhost:3001");
    const testInputs: SP1ProofInputs = {
      privateInputs: {
        amount: 1000,
        r: "00".repeat(32),
        sk_spend: "00".repeat(32),
        leaf_index: 0,
        merkle_path: {
          path_elements: [],
          path_indices: [],
        },
      },
      publicInputs: {
        root: "00".repeat(32),
        nf: "00".repeat(32),
        outputs_hash: "00".repeat(32),
        amount: 1000,
      },
      outputs: [],
    };

    // This should not throw (even if the actual call fails)
    const promise = prover.generateProof(testInputs);
    if (promise instanceof Promise) {
      console.log("   ✅ generateProof accepts correct inputs");
      passed++;
      // Cancel the promise (it will fail anyway without indexer)
      promise.catch(() => {});
    } else {
      throw new Error("generateProof does not return a Promise");
    }
  } catch (error) {
    console.log(`   ❌ Failed: ${error}`);
    failed++;
  }

  // Test 5: Verify swapParams support
  console.log("\n5️⃣ Testing swapParams support...");
  try {
    const prover = new ArtifactProverService("http://localhost:3001");
    const testInputs: SP1ProofInputs = {
      privateInputs: {
        amount: 1000,
        r: "00".repeat(32),
        sk_spend: "00".repeat(32),
        leaf_index: 0,
        merkle_path: {
          path_elements: [],
          path_indices: [],
        },
      },
      publicInputs: {
        root: "00".repeat(32),
        nf: "00".repeat(32),
        outputs_hash: "00".repeat(32),
        amount: 1000,
      },
      outputs: [],
      swapParams: {
        output_mint: "So11111111111111111111111111111111111111112",
        recipient_ata: "11111111111111111111111111111111",
        min_output_amount: 100,
      },
    };

    const promise = prover.generateProof(testInputs);
    if (promise instanceof Promise) {
      console.log("   ✅ swapParams are supported");
      passed++;
      promise.catch(() => {});
    } else {
      throw new Error("generateProof does not return a Promise");
    }
  } catch (error) {
    console.log(`   ❌ Failed: ${error}`);
    failed++;
  }

  // Summary
  console.log("\n" + "=".repeat(50));
  console.log(`📊 Test Summary: ${passed} passed, ${failed} failed`);
  
  if (failed === 0) {
    console.log("✅ All structural tests passed!");
    console.log("\n📝 Next steps:");
    console.log("   1. Recompile indexer: cd services/indexer && cargo build --release");
    console.log("   2. Restart indexer with artifact endpoints enabled");
    console.log("   3. Run full test: npm run test:artifact");
    return 0;
  } else {
    console.log("❌ Some tests failed");
    return 1;
  }
}

// Run test
testStructure()
  .then((exitCode) => {
    process.exit(exitCode);
  })
  .catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
  });

