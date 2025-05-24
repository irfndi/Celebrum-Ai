#!/usr/bin/env bash

# Setup script for Cloudflare D1 Database
set -euo pipefail

echo "🗄️ Setting up Cloudflare D1 Database for ArbEdge..."

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ Error: wrangler is required to create D1 database" >&2
    exit 1
fi

# Create D1 database
echo "📦 Creating D1 database 'arb-edge-db'..."
DB_OUTPUT=$(wrangler d1 create arb-edge-db 2>&1 || echo "Database may already exist")

# Extract database ID from output
DB_ID=$(echo "$DB_OUTPUT" | grep -o 'database_id = "[^"]*"' | sed 's/database_id = "\([^"]*\)"/\1/' || echo "")

if [[ -z "$DB_ID" ]]; then
    echo "⚠️  Could not extract database ID. Checking if database already exists..."
    
    # List existing databases to find our database
    EXISTING_DB=$(wrangler d1 list | grep "arb-edge-db" || echo "")
    
    if [[ -n "$EXISTING_DB" ]]; then
        echo "✅ Database 'arb-edge-db' already exists"
        # Extract ID from existing database list
        DB_ID=$(echo "$EXISTING_DB" | awk '{print $1}')
    else
        echo "❌ Failed to create or find D1 database" >&2
        exit 1
    fi
fi

echo "📋 Database ID: $DB_ID"

# Update wrangler.toml with the real database ID
echo "🔧 Updating wrangler.toml with database ID..."
sed -i.bak "s/placeholder-db-id/$DB_ID/g" wrangler.toml

# Initialize database schema
echo "🏗️ Initializing database schema..."
if [[ -f "sql/schema.sql" ]]; then
    wrangler d1 execute arb-edge-db --file=sql/schema.sql
    echo "✅ Database schema initialized"
else
    echo "⚠️  sql/schema.sql not found - skipping schema initialization"
fi

echo "✅ D1 Database setup completed!"
echo "📝 Database ID $DB_ID has been configured in wrangler.toml" 