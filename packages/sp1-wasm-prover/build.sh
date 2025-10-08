#!/bin/bash

# Build script for SP1 WASM Prover
# This script builds the WASM module for use in the frontend

set -e

echo "🔨 Building SP1 WASM Prover..."

# Navigate to the package directory
cd packages/sp1-wasm-prover

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module
echo "🔨 Building WASM module..."
wasm-pack build --target web --dev --out-dir pkg

echo "✅ WASM build completed!"
echo "📁 Output files:"
echo "   - pkg/sp1_wasm_prover.js"
echo "   - pkg/sp1_wasm_prover_bg.wasm"
echo "   - pkg/sp1_wasm_prover.d.ts"

echo ""
echo "💡 Next steps:"
echo "   1. Copy the pkg/ files to your frontend project"
echo "   2. Import and initialize the WASM module"
echo "   3. Use the cryptographic functions for proof validation"
