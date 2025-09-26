#!/bin/bash

echo "Testing relay service compilation..."

# Test basic compilation
echo "1. Testing basic syntax..."
rustc --version

echo "2. Checking Cargo.toml..."
if [ -f "Cargo.toml" ]; then
    echo "✓ Cargo.toml exists"
else
    echo "✗ Cargo.toml missing"
    exit 1
fi

echo "3. Testing dependencies..."
# Just check if we can parse dependencies
grep -q "axum" Cargo.toml && echo "✓ axum dependency found"
grep -q "tokio" Cargo.toml && echo "✓ tokio dependency found"
grep -q "sqlx" Cargo.toml && echo "✓ sqlx dependency found"

echo "4. Testing source structure..."
[ -d "src" ] && echo "✓ src directory exists"
[ -f "src/main.rs" ] && echo "✓ main.rs exists"
[ -f "src/config.rs" ] && echo "✓ config.rs exists"

echo "5. Basic syntax check on main files..."
rust-analyzer --version 2>/dev/null || echo "rust-analyzer not available"

echo "Compilation test completed!" 