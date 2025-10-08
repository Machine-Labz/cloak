// Example: Using the proof service with WASM validation
//
// This demonstrates the production-grade architecture where:
// 1. WASM validates inputs and computes cryptographic primitives (BLAKE3)
// 2. Backend service generates the actual SP1 proofs

import init, {
    SP1WasmProver,
    compute_nullifier,
    compute_outputs_hash,
    compute_commitment,
    compute_pk_spend
} from './pkg/sp1_wasm_prover.js';

// Configuration
const PROOF_SERVICE_URL = 'http://localhost:3003';
const INDEXER_URL = 'http://localhost:3001';

async function generateProofWithValidation(inputs) {
    console.log('🔐 Starting production proof generation...');

    // Step 1: Initialize WASM module
    console.log('📦 Loading WASM module...');
    await init();

    // Step 2: Create prover instance
    const prover = new SP1WasmProver();

    // Step 3: Validate inputs locally using WASM
    console.log('✅ Validating inputs locally...');
    try {
        prover.validate_proof_inputs(JSON.stringify(inputs));
        console.log('✅ Input validation passed');
    } catch (error) {
        console.error('❌ Input validation failed:', error);
        throw error;
    }

    // Step 4: Call proof service to generate real proof
    console.log('🔄 Calling proof service at:', PROOF_SERVICE_URL);

    const response = await fetch(`${PROOF_SERVICE_URL}/api/v1/proof/generate`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(inputs)
    });

    if (!response.ok) {
        const error = await response.json();
        throw new Error(`Proof generation failed: ${error.error}`);
    }

    const result = await response.json();

    if (!result.success) {
        throw new Error(`Proof generation failed: ${result.error}`);
    }

    console.log('✅ Proof generated successfully!');
    console.log('   Proof size:', result.proof_bytes.length, 'bytes');
    console.log('   Public inputs size:', result.public_inputs.length, 'bytes');

    return result;
}

async function generateProofWithIndexer(amount, sk_spend_hex, r_hex) {
    console.log('🔗 Generating proof with indexer integration...');

    // Step 1: Initialize WASM
    await init();

    // Step 2: Fetch Merkle tree data from indexer
    console.log('📊 Fetching Merkle tree state from indexer...');
    const merkleResponse = await fetch(`${INDEXER_URL}/api/v1/merkle/root`);
    const merkleData = await merkleResponse.json();

    console.log('   Root:', merkleData.root);
    console.log('   Next index:', merkleData.next_index);

    if (merkleData.next_index === 0) {
        throw new Error('No leaves in merkle tree. Please make a deposit first.');
    }

    // Use the last deposited leaf
    const leafIndex = merkleData.next_index - 1;

    // Fetch Merkle proof for this leaf
    console.log('🔍 Fetching Merkle proof for index:', leafIndex);
    const proofResponse = await fetch(`${INDEXER_URL}/api/v1/merkle/proof/${leafIndex}`);
    const merkleProof = await proofResponse.json();

    console.log('   Path elements:', merkleProof.path_elements.length);
    console.log('   Path indices:', merkleProof.path_indices.length);

    // Step 3: Compute cryptographic values using WASM
    console.log('🔐 Computing cryptographic values...');

    const nullifier = compute_nullifier(sk_spend_hex, leafIndex);
    console.log('   Nullifier:', nullifier);

    // Define outputs (withdraw less than amount due to fees)
    const fee = calculateFee(amount);
    const withdrawAmount = amount - fee;

    const outputs = [
        {
            address: "3333333333333333333333333333333333333333333333333333333333333333",
            amount: withdrawAmount
        }
    ];

    const outputs_hash = compute_outputs_hash(JSON.stringify(outputs));
    console.log('   Outputs hash:', outputs_hash);

    // Step 4: Build complete inputs
    const inputs = {
        private: {
            amount: amount,
            r: r_hex,
            sk_spend: sk_spend_hex,
            leaf_index: leafIndex,
            merkle_path: {
                path_elements: merkleProof.path_elements,
                path_indices: merkleProof.path_indices
            }
        },
        public: {
            root: merkleData.root,
            nf: nullifier,
            outputs_hash: outputs_hash,
            amount: amount
        },
        outputs: outputs
    };

    // Step 5: Generate proof
    return await generateProofWithValidation(inputs);
}

function calculateFee(amount) {
    const FIXED_FEE = 2_500_000; // 0.0025 SOL
    const VARIABLE_FEE_NUMERATOR = 5;
    const VARIABLE_FEE_DENOMINATOR = 1_000; // 0.5%

    return FIXED_FEE + Math.floor((amount * VARIABLE_FEE_NUMERATOR) / VARIABLE_FEE_DENOMINATOR);
}

// Example usage
async function main() {
    try {
        // Example: Generate proof with indexer integration
        const proof = await generateProofWithIndexer(
            100_000_000, // 0.1 SOL
            "3eeb66404b23fbd9fc9e4bcf800b45c1b955de569db2ed6c938cffbd6bd3c628",
            "c5a222dd127daa6498572f2a166d81a294e2ee676c79e54d85c1e6baabc19915"
        );

        console.log('\n✅ Final proof:', proof);

        // Now this proof can be submitted to the Solana program
        console.log('\n💡 Next step: Submit proof to shield-pool withdraw instruction');

    } catch (error) {
        console.error('\n❌ Error:', error.message);
        console.error(error.stack);
    }
}

// Run if loaded as script
if (typeof window !== 'undefined') {
    window.generateProofWithValidation = generateProofWithValidation;
    window.generateProofWithIndexer = generateProofWithIndexer;
    window.calculateFee = calculateFee;
}

export {
    generateProofWithValidation,
    generateProofWithIndexer,
    calculateFee
};
