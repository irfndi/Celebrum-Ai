#!/bin/bash

# Build script for Cloudflare Workers with Rust support
set -e

echo "🦀 Setting up Rust build environment for Cloudflare Workers..."

# Check if cargo is available, if not, install Rust
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    rustup target add wasm32-unknown-unknown
fi

# Verify cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo still not available after installation"
    exit 1
fi

# Install worker-build if not available
if ! command -v worker-build &> /dev/null; then
    echo "🔧 Installing worker-build..."
    cargo install worker-build --force
fi

# Build the worker
echo "🏗️ Building Rust Worker..."
worker-build --release

echo "✅ Build completed successfully!" 