# Cloak - Privacy-Preserving Solana Protocol

# Default recipe - show help
default: help

# Show available commands
help:
    @echo "🔮 Cloak Commands"
    @echo "================="
    @just --list

# Build everything (excluding relay which requires database)
build:
    @echo "🔨 Building Cloak..."
    @cargo run --package zk-guest-sp1-host --bin get_vkey_hash --release > vkey_hash.txt
    @cargo build --release --workspace --exclude relay
    @cd programs/shield-pool && cargo build-sbf
    @echo "✅ Build complete!"

# Build relay service (requires SQLX_OFFLINE=true or DATABASE_URL)
build-relay:
    @echo "🔨 Building relay service..."
    @cd services/relay && SQLX_OFFLINE=true cargo build --release
    @echo "✅ Relay build complete!"

# Run tests
test:
    @echo "🧪 Running tests..."
    @cargo test --release

# Test complete flow on localnet
test-localnet: build
    @echo "🧪 Testing complete flow on localnet..."
    @echo "⚠️  Make sure local Solana validator is running on port 8899"
    @echo "⚠️  Make sure indexer service is running on port 3001"
    @cargo run --package test-complete-flow-rust --bin localnet-test --release

# Test complete flow on testnet
test-testnet: build
    @echo "🧪 Testing complete flow on testnet..."
    @echo "⚠️  Make sure you have testnet SOL for testing"
    @echo "⚠️  Make sure indexer service is running on port 3001"
    @cargo run --package test-complete-flow-rust --bin testnet-test --release

# Start local Solana validator
start-validator:
    @echo "🌐 Starting local Solana validator..."
    @solana-test-validator --quiet

# Deploy program to local validator with the program keypair
deploy-local: build
    @echo "🚀 Deploying to local validator with program ID c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp..."
    @solana program deploy target/deploy/shield_pool.so \
        --program-id c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp.json \
        --url http://127.0.0.1:8899
    @echo "✅ Program deployed with ID: c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"

# Deploy program to local validator (auto-generate keypair)
deploy-local-new: build
    @echo "🚀 Deploying to local validator (new program ID)..."
    @solana program deploy target/deploy/shield_pool.so --url http://127.0.0.1:8899

# Deploy program to devnet
deploy-devnet: build
    @echo "🚀 Deploying to devnet..."
    @solana program deploy target/deploy/shield_pool.so --url devnet

# Clean build artifacts
clean:
    @echo "🧹 Cleaning..."
    @cargo clean
    @rm -f vkey_hash.txt
    @rm -f programs/shield-pool/vkey_hash.txt
    @echo "✅ Clean complete!"
