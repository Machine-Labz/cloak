#!/bin/bash
# Script to generate SQLx offline metadata for the relay service
# This must be run with a live database connection

set -e

echo "🔧 Preparing SQLx metadata for relay service..."

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL environment variable is not set"
    echo "Please set it to your PostgreSQL connection string, e.g.:"
    echo "export DATABASE_URL=\"postgresql://cloak:password@localhost:5434/cloak_relay\""
    exit 1
fi

echo "📍 Working directory: $(pwd)"
echo "🔗 DATABASE_URL: $DATABASE_URL"

# Navigate to workspace root
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ../..)"

echo "📦 Running cargo check to compile queries..."
cd services/relay
cargo check --bin relay

echo "✅ SQLx metadata prepared!"
echo "📁 Metadata saved to: services/relay/.sqlx/"

# Verify the metadata was created
if [ -d ".sqlx" ] && [ "$(ls -A .sqlx 2>/dev/null)" ]; then
    echo "✅ .sqlx directory created with files"
    ls -lh .sqlx/
else
    echo "⚠️  Warning: .sqlx directory is empty or not created"
    echo "The queries may not have been found during compilation"
fi

