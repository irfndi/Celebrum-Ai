#!/bin/bash
# Build script for @celebrum-ai/shared package

set -euo pipefail

echo "📦 Building @celebrum-ai/shared package..."

# Change to package directory
cd "$(dirname "$0")"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    pnpm install
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf dist

# Run TypeScript compilation with tsup
echo "🔨 Building with tsup..."
pnpm run build

# Verify build output
if [ -d "dist" ]; then
    echo "📊 Build output:"
    ls -la dist/
else
    echo "❌ Build failed - no dist directory found"
    exit 1
fi

echo "✅ @celebrum-ai/shared build completed successfully!"