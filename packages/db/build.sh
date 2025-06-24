#!/bin/bash
# Build script for @celebrum-ai/db package

set -euo pipefail

echo "🗄️ Building @celebrum-ai/db package..."

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

echo "✅ @celebrum-ai/db build completed successfully!"