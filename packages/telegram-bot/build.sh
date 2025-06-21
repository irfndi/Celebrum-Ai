#!/bin/bash
# Build script for @arb-edge/telegram-bot package

set -euo pipefail

echo "🤖 Building @arb-edge/telegram-bot package..."

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

# Run TypeScript compilation
echo "🔨 Compiling TypeScript..."
pnpm run build

# Build Rust components if present
if [ -f "Cargo.toml" ]; then
    echo "🦀 Building Rust components..."
    cargo build --release
fi

# Verify build output
if [ -d "dist" ]; then
    echo "📊 Build output:"
    ls -la dist/
else
    echo "❌ Build failed - no dist directory found"
    exit 1
fi

echo "✅ @arb-edge/telegram-bot build completed successfully!"