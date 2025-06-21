#!/bin/bash
# Build script for @arb-edge/db package

set -euo pipefail

echo "🗄️ Building @arb-edge/db package..."

# Change to package directory
cd "$(dirname "$0")"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    pnpm install
fi

# Run TypeScript compilation
echo "🔨 Compiling TypeScript..."
pnpm run build

# Generate database schema if needed
if [ -f "drizzle.config.ts" ]; then
    echo "🗄️ Generating database schema..."
    pnpm run db:generate
fi

echo "✅ @arb-edge/db build completed successfully!"