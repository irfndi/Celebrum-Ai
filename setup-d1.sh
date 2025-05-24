#!/usr/bin/env bash

# Setup script for Cloudflare D1 Database
set -euo pipefail

echo "🗄️ Setting up Cloudflare D1 Database for ArbEdge..."

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ Error: wrangler is required to create D1 database" >&2
    exit 1
fi

# Use existing D1 database
DB_NAME="prod-arb-edge"
DB_ID="879bf844-93b2-433d-9319-6e6065bbfdfd"

echo "📋 Using existing D1 database:"
echo "   Name: $DB_NAME"
echo "   ID: $DB_ID"

# Verify database exists
echo "🔍 Verifying database exists..."
if wrangler d1 list | grep -q "$DB_NAME"; then
    echo "✅ Database '$DB_NAME' found"
else
    echo "⚠️  Database '$DB_NAME' not found in list, but continuing with configured ID"
fi

# Initialize database schema if available
echo "🏗️ Initializing database schema..."
if [[ -f "sql/schema.sql" ]]; then
    wrangler d1 execute "$DB_NAME" --file=sql/schema.sql
    echo "✅ Database schema initialized"
else
    echo "⚠️  sql/schema.sql not found - skipping schema initialization"
fi

echo "✅ D1 Database setup completed!"
echo "📝 Database ID $DB_ID is configured in wrangler.toml" 