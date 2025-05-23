# ArbEdge Rust Development Makefile
# Ensures correct Rust toolchain for all operations

# Use rustup's Rust, not Homebrew's
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)

.PHONY: help setup test build build-wasm coverage clean lint fix fmt check-all deploy

help: ## Show this help message
	@echo "🦀 ArbEdge Rust Development Commands"
	@echo "===================================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Run development environment setup
	@./scripts/dev-setup.sh

test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test

build: ## Build for native target
	@echo "🔨 Building native..."
	@cargo build

build-wasm: ## Build for WASM target
	@echo "🎯 Building WASM..."
	@cargo build --target wasm32-unknown-unknown

coverage: ## Generate test coverage report
	@echo "📊 Generating coverage report..."
	@cargo tarpaulin --out html --output-dir coverage
	@echo "Coverage report generated at: coverage/tarpaulin-report.html"

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	@cargo clean

lint: ## Run clippy lints
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features

fix: ## Apply automatic fixes
	@echo "🔧 Applying automatic fixes..."
	@cargo fix --lib --allow-dirty
	@cargo clippy --fix --allow-dirty

fmt: ## Format code
	@echo "🎨 Formatting code..."
	@cargo fmt

check-all: lint test build build-wasm ## Run all checks (lint, test, build native & WASM)
	@echo "✅ All checks completed successfully!"

deploy: build-wasm ## Prepare for deployment (build WASM and run tests)
	@echo "🚀 Preparing for deployment..."
	@cargo test --quiet
	@echo "✅ Ready for deployment!"

# Quick development commands
dev: fmt lint test ## Quick development cycle (format, lint, test)
	@echo "🚀 Development cycle completed!"

ci: check-all coverage ## CI pipeline (all checks + coverage)
	@echo "🎯 CI pipeline completed!" 