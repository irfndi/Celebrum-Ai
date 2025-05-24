# ArbEdge Rust Development Makefile
# Ensures correct Rust toolchain for all operations

# Use rustup's Rust, not Homebrew's
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)

.PHONY: help setup test build build-wasm coverage clean lint fix fmt check-all deploy pre-commit local-ci full-check

help: ## Show this help message
	@echo "🦀 ArbEdge Rust Development Commands"
	@echo "===================================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Run development environment setup
	@./scripts/dev-setup.sh

# Testing commands
test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test

test-verbose: ## Run tests with verbose output
	@echo "🧪 Running tests (verbose)..."
	@cargo test --verbose

# Build commands
build: ## Build for native target
	@echo "🔨 Building native..."
	@cargo build

build-release: ## Build release for native target
	@echo "🔨 Building native (release)..."
	@cargo build --release

build-wasm: ## Build for WASM target
	@echo "🎯 Building WASM..."
	@cargo build --target wasm32-unknown-unknown

build-wasm-release: ## Build release for WASM target
	@echo "🎯 Building WASM (release)..."
	@cargo build --target wasm32-unknown-unknown --release

# Code quality commands
fmt: ## Format code
	@echo "🎨 Formatting code..."
	@cargo fmt

fmt-check: ## Check code formatting
	@echo "🎨 Checking code formatting..."
	@cargo fmt --all -- --check

lint: ## Run clippy lints
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features

lint-strict: ## Run strict clippy lints
	@echo "🔍 Running strict clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

fix: ## Apply automatic fixes
	@echo "🔧 Applying automatic fixes..."
	@cargo fix --lib --allow-dirty
	@cargo clippy --fix --allow-dirty

# Coverage and documentation
coverage: ## Generate test coverage report
	@echo "📊 Generating coverage report..."
	@cargo tarpaulin --out html --output-dir coverage
	@echo "Coverage report generated at: coverage/tarpaulin-report.html"

doc: ## Generate documentation
	@echo "📚 Generating documentation..."
	@cargo doc --no-deps --document-private-items

# Script-based commands (new)
pre-commit: ## Run quick pre-commit checks
	@./scripts/pre-commit.sh

local-ci: ## Run full CI pipeline locally
	@./scripts/local-ci.sh

full-check: ## Run comprehensive code quality checks
	@./scripts/full-check.sh

# Utility commands
clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	@cargo clean

check: ## Quick build check
	@echo "🔍 Quick build check..."
	@cargo check

check-all: lint test build build-wasm ## Run all basic checks (lint, test, build native & WASM)
	@echo "✅ All basic checks completed successfully!"

# Legacy commands (maintained for compatibility)
dev: fmt lint test ## Quick development cycle (format, lint, test)
	@echo "🚀 Development cycle completed!"

ci: local-ci ## Alias for local-ci (legacy)

deploy: build-wasm-release ## Prepare for deployment (build WASM and run tests)
	@echo "🚀 Preparing for deployment..."
	@cargo test --quiet
	@echo "✅ Ready for deployment!"

# Workflow commands (recommended usage)
quick: pre-commit ## Quick validation before commit
	@echo "⚡ Quick validation completed!"

validate: local-ci ## Full validation (mirrors CI)
	@echo "✅ Full validation completed!"

quality: full-check ## Comprehensive quality analysis
	@echo "🏆 Quality analysis completed!" 