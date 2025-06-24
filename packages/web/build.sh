#!/bin/bash
# Build script for @celebrum-ai/web package

set -euo pipefail

echo "🌐 Building @celebrum-ai/web package..."

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

echo "✅ @celebrum-ai/web build completed successfully!"