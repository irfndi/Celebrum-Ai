#!/bin/bash
# Build script for @arb-edge/web package

set -euo pipefail

echo "🌐 Building @arb-edge/web package..."

# Change to package directory
cd "$(dirname "$0")"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    pnpm install
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf dist build .astro

# Run Astro build
echo "🚀 Building with Astro..."
pnpm run build

# Verify build output
if [ -d "dist" ]; then
    echo "📊 Build output:"
    ls -la dist/
    echo "📈 Build size:"
    du -sh dist/
else
    echo "❌ Build failed - no dist directory found"
    exit 1
fi

echo "✅ @arb-edge/web build completed successfully!"