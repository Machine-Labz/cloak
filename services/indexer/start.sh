#!/bin/bash

# Cloak Indexer Startup Script
# Checks prerequisites and starts the indexer with proper configuration

set -e

echo "🚀 Starting Cloak Indexer..."
echo ""

# Check if PostgreSQL is running
echo "📊 Checking PostgreSQL..."
if docker ps | grep -q cloak-postgres; then
    echo "✅ PostgreSQL is running (cloak-postgres)"
else
    echo "❌ PostgreSQL is not running!"
    echo ""
    echo "To start PostgreSQL:"
    echo "  cd docker"
    echo "  docker-compose up -d postgres"
    echo ""
    exit 1
fi

# Check database connection
echo "🔗 Testing database connection..."
if PGPASSWORD=development_password_change_in_production psql -h localhost -p 5434 -U cloak -d cloak_indexer -c "SELECT 1" > /dev/null 2>&1; then
    echo "✅ Database connection successful"
else
    echo "⚠️  Could not connect to database"
    echo "   This might be okay if the database hasn't been created yet"
fi

echo ""
echo "🏗️  Starting indexer..."
echo ""

# Export default environment variables if not already set
export DB_HOST="${DB_HOST:-localhost}"
export DB_PORT="${DB_PORT:-5434}"
export DB_NAME="${DB_NAME:-cloak_indexer}"
export DB_USER="${DB_USER:-cloak}"
export DB_PASSWORD="${DB_PASSWORD:-development_password_change_in_production}"
export PORT="${PORT:-3001}"
export NODE_ENV="${NODE_ENV:-development}"
export LOG_LEVEL="${LOG_LEVEL:-info}"
export RUST_LOG="${RUST_LOG:-info}"

# Run the indexer
cargo run --release

