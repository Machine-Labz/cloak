# Cloak Makefile
.PHONY: vkey-hash build test clean

# Generate VKey hash and write to file
vkey-hash:
	@echo "🔑 Generating VKey hash..."
	@cargo run -p vkey-generator --release
	@echo "✅ VKey hash written to vkey_hash.txt"

# Build the shield pool program (reads VKey from file)
build: vkey-hash
	@echo "🔨 Building shield pool program..."
	@cd programs/shield-pool && cargo build-sbf
	@echo "✅ Build complete!"

# Run all tests
test:
	@echo "🧪 Running all tests..."
	@cargo test --release

# Clean build artifacts
clean:
	@cargo clean
	@rm -f vkey_hash.txt
	@rm -f target/vkey_hash.txt
	@rm -f packages/vkey-generator/target/vkey_hash.txt
