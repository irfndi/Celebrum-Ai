#!/usr/bin/env bash

# Build script for Cloudflare Workers with Rust support
set -euo pipefail

echo "🦀 Setting up Rust build environment for Cloudflare Workers..."

# Check if cargo is available, if not, install Rust
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust toolchain..."
    if ! command -v curl &> /dev/null; then
        echo "❌ Error: curl is required to install Rust" >&2
        exit 1
    fi
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    rustup target add wasm32-unknown-unknown
fi

# Verify cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo still not available after installation"
    exit 1
fi

# Install worker-build with pinned version for reproducibility
WORKER_BUILD_VERSION="0.1.2"
if ! command -v worker-build &> /dev/null || [[ "$(worker-build --version 2>/dev/null | grep -o 'v[0-9.]*' | head -1)" != "v${WORKER_BUILD_VERSION}" ]]; then
    echo "🔧 Installing worker-build v${WORKER_BUILD_VERSION}..."
    cargo install worker-build --version "${WORKER_BUILD_VERSION}" --force
fi

# Build the worker
echo "🏗️ Building Rust Worker..."
worker-build --release

echo "✅ Build completed successfully!"

# Note: For faster CI builds, consider caching ~/.cargo/registry and ~/.cargo/git
# in your CI pipeline (e.g., GitHub Actions cache action) 