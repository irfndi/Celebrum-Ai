#!/bin/bash
# Build script for @arb-edge/worker package

set -euo pipefail

echo "⚡ Building @arb-edge/worker package..."

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

# Build TypeScript with tsup
echo "🔨 Building TypeScript with tsup..."
pnpm run build

# Build Rust components if present
if [ -f "Cargo.toml" ]; then
    echo "🦀 Building Rust components for WASM..."
    cargo build --target wasm32-unknown-unknown --release
fi

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

echo "✅ @arb-edge/worker build completed successfully!"
echo "💡 Use 'pnpm run deploy' to deploy to Cloudflare Workers"