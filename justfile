# Cloak - Privacy-Preserving Solana Protocol
# Development and deployment commands

# Default recipe - show help
default: help

# Show available commands with descriptions
help:
    @echo "🔮 Cloak Development Commands"
    @echo "=============================="
    @just --list

# 🏗️  BUILD COMMANDS
# ==================

# Generate VKey hash and write to file
vkey-hash:
    @echo "🔑 Generating VKey hash..."
    @cargo run -p vkey-generator --release
    @echo "✅ VKey hash written to vkey_hash.txt"

# Build the shield pool program (reads VKey from file)
build-program: vkey-hash
    @echo "🔨 Building shield pool program..."
    @cd programs/shield-pool && cargo build-sbf
    @echo "✅ Shield pool program built!"

# Build all Rust components (programs + services + tools)
build-all: vkey-hash
    @echo "🔨 Building all Rust components..."
    @cargo build --release
    @cd programs/shield-pool && cargo build-sbf
    @echo "✅ All components built!"

# Build only the ZK proof system
build-zk:
    @echo "🔨 Building ZK proof system..."
    @cargo build -p zk-guest-sp1-host --release
    @echo "✅ ZK proof system built!"

# Build only the indexer service
build-indexer:
    @echo "🔨 Building indexer service..."
    @cargo build -p indexer --release
    @echo "✅ Indexer service built!"

# 🧪 TEST COMMANDS
# ================

# Run all tests (unit + integration)
test:
    @echo "🧪 Running all tests..."
    @cargo test --release

# Run only shield pool program tests
test-program:
    @echo "🧪 Running shield pool program tests..."
    @cd programs/shield-pool && cargo test --release

# Run only ZK proof system tests
test-zk:
    @echo "🧪 Running ZK proof system tests..."
    @cargo test -p zk-guest-sp1 --release

# Run only indexer tests
test-indexer:
    @echo "🧪 Running indexer tests..."
    @cargo test -p indexer --release

# Run integration tests with real validator
test-integration: build-program build-indexer
    @echo "🧪 Running full integration tests..."
    @echo "⚠️  This will start a local Solana validator and indexer"
    @echo "⚠️  Make sure ports 8899 and 3030 are available"
    @# TODO: Add integration test script when available

# 🔮 ZK PROOF COMMANDS
# ===================

# Generate example ZK proof
prove-example: build-zk
    @echo "🔮 Generating example ZK proof..."
    @cd packages/zk-guest-sp1 && cargo run --release --package zk-guest-sp1-host --bin cloak-zk -- prove \
        --private examples/private.example.json \
        --public examples/public.example.json \
        --outputs examples/outputs.example.json \
        --proof out/proof.bin \
        --pubout out/public.json
    @echo "✅ Example proof generated!"

# Verify example ZK proof
verify-example: build-zk
    @echo "🔮 Verifying example ZK proof..."
    @cd packages/zk-guest-sp1 && cargo run --release --package zk-guest-sp1-host --bin cloak-zk -- verify \
        --proof out/proof.bin \
        --public out/public.json
    @echo "✅ Proof verification complete!"

# Generate test examples for ZK proofs
generate-examples: build-zk
    @echo "🔮 Generating ZK test examples..."
    @cd packages/zk-guest-sp1 && cargo run --package zk-guest-sp1-host --bin generate_examples
    @echo "✅ Test examples generated!"

# 🌐 SERVICE COMMANDS
# ==================

# Start the indexer service (development mode)
start-indexer: build-indexer
    @echo "🌐 Starting indexer service..."
    @cargo run -p indexer -- --rpc-url http://127.0.0.1:8899 --port 3030

# Start a local Solana validator for testing
start-validator:
    @echo "🌐 Starting local Solana validator..."
    @echo "⚠️  This will run on port 8899"
    @solana-test-validator --reset --quiet

# Deploy shield pool program to local validator
deploy-local: build-program
    @echo "🚀 Deploying shield pool program to local validator..."
    @solana program deploy target/deploy/shield_pool.so --url http://127.0.0.1:8899

# Deploy shield pool program to devnet
deploy-devnet: build-program
    @echo "🚀 Deploying shield pool program to devnet..."
    @solana program deploy target/deploy/shield_pool.so --url devnet

# 🔧 UTILITY COMMANDS
# ==================

# Install Node.js dependencies
install-deps:
    @echo "📦 Installing Node.js dependencies..."
    @npm install
    @echo "✅ Dependencies installed!"

# Format all Rust code
fmt:
    @echo "🎨 Formatting Rust code..."
    @cargo fmt --all

# Run Rust linter
lint:
    @echo "🔍 Running Rust linter..."
    @cargo clippy --all-targets --all-features -- -D warnings

# Check Rust code without building
check:
    @echo "🔍 Checking Rust code..."
    @cargo check --all-targets --all-features

# Update Rust dependencies
update-deps:
    @echo "📦 Updating Rust dependencies..."
    @cargo update
    @echo "✅ Dependencies updated!"

# 🧹 CLEANUP COMMANDS
# ==================

# Clean all build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    @cargo clean
    @rm -f vkey_hash.txt
    @rm -f programs/shield-pool/vkey_hash.txt
    @rm -rf node_modules
    @echo "✅ Cleanup complete!"

# Clean only Rust artifacts (keep Node.js)
clean-rust:
    @echo "🧹 Cleaning Rust artifacts..."
    @cargo clean
    @rm -f vkey_hash.txt
    @rm -f programs/shield-pool/vkey_hash.txt
    @echo "✅ Rust cleanup complete!"

# Clean only Node.js artifacts
clean-node:
    @echo "🧹 Cleaning Node.js artifacts..."
    @rm -rf node_modules
    @echo "✅ Node.js cleanup complete!"

# 📊 STATUS COMMANDS
# ==================

# Show project status and health check
status:
    @echo "📊 Cloak Project Status"
    @echo "======================"
    @echo ""
    @echo "🔧 Build Tools:"
    @just --version || echo "❌ just not installed"
    @cargo --version || echo "❌ cargo not installed"
    @solana --version || echo "❌ solana CLI not installed"
    @node --version || echo "❌ node not installed"
    @echo ""
    @echo "📂 Project Structure:"
    @echo "   Programs: $(find programs -name "*.rs" | wc -l | tr -d ' ') Rust files"
    @echo "   Services: $(find services -name "*.rs" | wc -l | tr -d ' ') Rust files"
    @echo "   ZK Tools: $(find packages -name "*.rs" | wc -l | tr -d ' ') Rust files"
    @echo ""
    @echo "🏗️  Build Status:"
    @test -f target/deploy/shield_pool.so && echo "   ✅ Shield pool program built" || echo "   ❌ Shield pool program not built"
    @test -f target/release/indexer && echo "   ✅ Indexer service built" || echo "   ❌ Indexer service not built"
    @test -f target/release/cloak-zk && echo "   ✅ ZK tools built" || echo "   ❌ ZK tools not built"

# Show recent git activity
git-status:
    @echo "📊 Git Status"
    @echo "============="
    @git status --short
    @echo ""
    @echo "Recent commits:"
    @git log --oneline -5

# 🚀 QUICK START COMMANDS
# =======================

# Complete development setup (build everything)
setup: install-deps build-all
    @echo "🚀 Development setup complete!"
    @echo ""
    @echo "Next steps:"
    @echo "  just start-validator    # Start local Solana validator"
    @echo "  just deploy-local       # Deploy program to local validator"
    @echo "  just start-indexer      # Start indexer service"
    @echo "  just test-integration   # Run full integration tests"

# Quick development cycle (format, lint, test)
dev: fmt lint test
    @echo "🚀 Development cycle complete!"

# Full CI/CD simulation (everything that CI would run)
ci: clean check lint test build-all
    @echo "🚀 CI/CD simulation complete!"
