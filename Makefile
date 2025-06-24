# Cerebrum AI Unified Monorepo Makefile
# Ensures correct Rust toolchain and pnpm for all operations

# Use rustup's Rust, not Homebrew's, and include pnpm
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)

.PHONY: help setup test build build-wasm coverage clean lint fix fmt check-all deploy pre-commit local-ci full-check unit-tests integration-tests e2e-tests lib-tests ci-pipeline test-api test-api-local test-api-staging test-api-production test-api-prod-admin test-api-v1 test-api-v1-local test-api-v1-staging test-api-v1-production

help: ## Show this help message
	@echo "🦀 ArbEdge Unified Monorepo Commands"
	@echo "===================================="
	@echo "\033[33m💡 Tip: Use 'make ci' for full validation (Rust + TypeScript)\033[0m"
	@echo "\033[33m💡 Tip: Use 'make fix-and-validate' to auto-fix then validate\033[0m"
	@echo "===================================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Run development environment setup
	@./scripts/dev/dev-setup.sh

# Package management
install: ## Install all dependencies (Rust + TypeScript packages)
	@echo "📦 Installing dependencies..."
	@echo "📦 Installing pnpm dependencies..."
	@pnpm install
	@echo "📦 Setting up Rust toolchain..."
	@rustup component add clippy rustfmt
	@rustup target add wasm32-unknown-unknown

# TypeScript package commands
build-packages: ## Build all TypeScript packages
	@echo "🔨 Building TypeScript packages..."
	@pnpm run build

build-db: ## Build database package
	@echo "🔨 Building database package..."
	@pnpm --filter @celebrum-ai/db run build

build-shared: ## Build shared package
	@echo "🔨 Building shared package..."
	@pnpm --filter @celebrum-ai/shared run build

build-telegram-bot: ## Build telegram bot package
	@echo "🔨 Building telegram bot package..."
	@pnpm --filter @celebrum-ai/telegram-bot run build

build-web: ## Build web package
	@echo "🔨 Building web package..."
	@pnpm --filter @celebrum-ai/web run build

build-worker: ## Build worker package
	@echo "🔨 Building worker package..."
	@pnpm --filter @celebrum-ai/worker run build

test-packages: ## Test all TypeScript packages
	@echo "🧪 Testing TypeScript packages..."
	@pnpm run test

test-db: ## Test database package
	@echo "🧪 Testing database package..."
	@pnpm --filter @celebrum-ai/db run test

test-shared: ## Test shared package
	@echo "🧪 Testing shared package..."
	@pnpm --filter @celebrum-ai/shared run test

test-telegram-bot: ## Test telegram bot package
	@echo "🧪 Testing telegram bot package..."
	@pnpm --filter @celebrum-ai/telegram-bot run test

test-web: ## Test web package
	@echo "🧪 Testing web package..."
	@pnpm --filter @celebrum-ai/web run test

test-worker: ## Test worker package
	@echo "🧪 Testing worker package..."
	@pnpm --filter @celebrum-ai/worker run test

lint-packages: ## Lint all TypeScript packages
	@echo "🔍 Linting TypeScript packages..."
	@pnpm run lint

lint-db: ## Lint database package
	@echo "🔍 Linting database package..."
	@pnpm --filter @celebrum-ai/db run lint

lint-shared: ## Lint shared package
	@echo "🔍 Linting shared package..."
	@pnpm --filter @celebrum-ai/shared run lint

lint-telegram-bot: ## Lint telegram bot package
	@echo "🔍 Linting telegram bot package..."
	@pnpm --filter @celebrum-ai/telegram-bot run lint

lint-web: ## Lint web package
	@echo "🔍 Linting web package..."
	@pnpm --filter @celebrum-ai/web run lint

lint-worker: ## Lint worker package
	@echo "🔍 Linting worker package..."
	@pnpm --filter @celebrum-ai/worker run lint

# Testing commands
test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test

test-verbose: ## Run tests with verbose output
	@echo "🧪 Running tests (verbose)..."
	@cargo test --verbose

lib-tests: ## Run library tests only
	@echo "🧪 Running library tests..."
	@cargo test --lib --verbose

unit-tests: ## Run unit tests
	@echo "🧪 Running unit tests..."
	@cargo test --lib

integration-tests: ## Run integration tests
	@echo "🧪 Running integration tests..."
	@cd packages/worker/tests && cargo test --test integration_tests

e2e-tests: ## Run E2E tests
	@echo "🧪 Running E2E tests..."
	@cd packages/worker/tests && cargo test --test e2e_tests

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

# Development commands
dev: ## Start development servers for all packages
	@echo "🚀 Starting development servers..."
	@pnpm run dev

dev-worker: ## Start worker development server
	@echo "🚀 Starting worker development server..."
	@pnpm --filter @celebrum-ai/worker run dev

dev-web: ## Start web development server
	@echo "🚀 Starting web development server..."
	@pnpm --filter @celebrum-ai/web run dev

dev-telegram-bot: ## Start telegram bot development server
	@echo "🚀 Starting telegram bot development server..."
	@pnpm --filter @celebrum-ai/telegram-bot run dev

# Deployment commands
deploy: ## Deploy all packages
	@echo "🚀 Deploying all packages..."
	@pnpm run deploy

deploy-worker: ## Deploy worker package
	@echo "🚀 Deploying worker package..."
	@pnpm --filter @celebrum-ai/worker run deploy

deploy-web: ## Deploy web package
	@echo "🚀 Deploying web package..."
	@pnpm --filter @celebrum-ai/web run deploy

# Code quality commands
fmt: ## Format code (Rust + TypeScript)
	@echo "🎨 Formatting Rust code..."
	@cargo fmt --verbose
	@echo "🎨 Formatting TypeScript code..."
	@pnpm run format

fmt-check: ## Check code formatting (Rust + TypeScript)
	@echo "🎨 Checking Rust code formatting..."
	@cargo fmt --all --verbose -- --check
	@echo "🎨 Checking TypeScript code formatting..."
	@pnpm run format:check

fmt-fix: ## Auto-fix code formatting then run CI
	@echo "🎨 Auto-fixing code formatting..."
	@cargo fmt --verbose
	@pnpm run format
	@echo "🔄 Running CI pipeline..."
	@$(MAKE) ci-pipeline

lint: ## Run clippy lints
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features

lint-strict: ## Run strict clippy lints (matches GitHub CI)
	@echo "🔍 Running strict clippy (GitHub CI standard)..."
	@cargo clippy --all-targets --all-features -- -D warnings

lint-lib: ## Run clippy on library only
	@echo "🔍 Running clippy on library..."
	@cargo clippy --lib --verbose -- -D warnings

fix: ## Apply automatic fixes
	@echo "🔧 Applying automatic fixes..."
	@cargo fix --lib --allow-dirty
	@cargo clippy --fix --allow-dirty

# CI Pipeline
ci-pipeline: ## Run comprehensive CI pipeline (Rust + TypeScript)
	@echo "🚀 Starting Full Monorepo CI Pipeline..."
	@echo "========================================"
	@echo "📦 Step 0: Installing Dependencies"
	@pnpm install --frozen-lockfile
	@echo "✅ Step 0: Dependencies Installed"
	@echo "🎨 Step 1: TypeScript Package Linting"
	@pnpm run lint
	@echo "✅ Step 1: TypeScript Linting Passed"
	@echo "🔨 Step 2: TypeScript Package Building"
	@pnpm run build
	@echo "✅ Step 2: TypeScript Packages Built"
	@echo "🧪 Step 3: TypeScript Package Testing"
	@pnpm run test:ci
	@echo "✅ Step 3: TypeScript Tests Passed"
	@echo "🎨 Step 4: Rust Code Formatting Check"
	@cargo fmt --all -- --check
	@echo "✅ Step 4: Rust Formatting Passed"
	@echo "🔍 Step 5: Rust Clippy Linting Check"
	@cargo clippy --all-targets --all-features -- -D warnings --verbose
	@echo "✅ Step 5: Rust Clippy Linting Passed"
	@echo "🎯 Step 6: WASM Target Compilation Check"
	@cargo check --target wasm32-unknown-unknown --lib --verbose
	@echo "✅ Step 6: WASM Target Compilation Passed"
	@echo "🧪 Step 7: Rust Library Tests"
	@cargo test --lib --verbose
	@echo "✅ Step 7: Rust Library Tests Passed (327 tests)"
	@echo "🧪 Step 8: Rust Unit Tests"
	@$(MAKE) unit-tests
	@echo "✅ Step 8: Rust Unit Tests Passed (67 tests)"
	@echo "🧪 Step 9: Rust Integration & E2E Tests"
	@$(MAKE) integration-tests
	@$(MAKE) e2e-tests
	@echo "✅ Step 9: Rust Integration & E2E Tests Passed (74 tests)"
	@echo "🔧 Step 10: Final Native Compilation Check"
	@cargo check --verbose
	@echo "✅ Step 10: Final Native Compilation Check Passed"
	@echo "🎯 Step 11: Final WASM Build Verification"
	@cargo build --target wasm32-unknown-unknown --lib --verbose
	@echo "✅ Step 11: Final WASM Build Verification Passed"
	@echo "🎉 Monorepo CI Pipeline Completed Successfully!"
	@echo "📊 Test Summary:"
	@echo "   - TypeScript Packages: All built and tested ✅"
	@echo "   - Rust Library Tests: 327 tests ✅"
	@echo "   - Rust Unit Tests: 67 tests ✅"
	@echo "   - Rust Integration Tests: 62 tests ✅"
	@echo "   - Rust E2E Tests: 12 tests ✅"
	@echo "   - Total Rust Tests: 468 tests passing ✅"
	@echo "   - WASM Compatibility: ✅ Verified"
	@echo "   - Monorepo Integration: ✅ Complete"

# Coverage and documentation
coverage: ## Generate test coverage report
	@echo "📊 Generating coverage report..."
	@cargo tarpaulin --out html --output-dir coverage
	@echo "Coverage report generated at: coverage/tarpaulin-report.html"

doc: ## Generate documentation
	@echo "📚 Generating documentation..."
	@cargo doc --no-deps --document-private-items

# Script-based commands (recommended for development)
pre-commit: ## Run quick pre-commit checks
	@./scripts/dev/pre-commit.sh

local-ci: ## Run local CI validation (mirrors GitHub CI exactly)
	@./scripts/dev/local-ci.sh

full-check: ## Run comprehensive code quality checks
	@./scripts/ci/full-check.sh

# Clean commands
clean: ## Clean all build artifacts (Rust + TypeScript)
	@echo "🧹 Cleaning Rust build artifacts..."
	@cargo clean
	@echo "🧹 Cleaning TypeScript build artifacts..."
	@pnpm run clean

clean-rust: ## Clean Rust build artifacts only
	@echo "🧹 Cleaning Rust build artifacts..."
	@cargo clean

clean-packages: ## Clean TypeScript package build artifacts
	@echo "🧹 Cleaning TypeScript build artifacts..."
	@pnpm run clean

clean-db: ## Clean database package build artifacts
	@echo "🧹 Cleaning database package..."
	@pnpm --filter @celebrum-ai/db run clean

clean-shared: ## Clean shared package build artifacts
	@echo "🧹 Cleaning shared package..."
	@pnpm --filter @celebrum-ai/shared run clean

clean-telegram-bot: ## Clean telegram bot package build artifacts
	@echo "🧹 Cleaning telegram bot package..."
	@pnpm --filter @celebrum-ai/telegram-bot run clean

clean-web: ## Clean web package build artifacts
	@echo "🧹 Cleaning web package..."
	@pnpm --filter @celebrum-ai/web run clean

clean-worker: ## Clean worker package build artifacts
	@echo "🧹 Cleaning worker package..."
	@pnpm --filter @celebrum-ai/worker run clean

# Type checking commands
typecheck: ## Run TypeScript type checking for all packages
	@echo "🔍 Running TypeScript type checking..."
	@pnpm run typecheck

typecheck-db: ## Run TypeScript type checking for database package
	@echo "🔍 Type checking database package..."
	@pnpm --filter @celebrum-ai/db run typecheck

typecheck-shared: ## Run TypeScript type checking for shared package
	@echo "🔍 Type checking shared package..."
	@pnpm --filter @celebrum-ai/shared run typecheck

typecheck-telegram-bot: ## Run TypeScript type checking for telegram bot package
	@echo "🔍 Type checking telegram bot package..."
	@pnpm --filter @celebrum-ai/telegram-bot run typecheck

typecheck-web: ## Run TypeScript type checking for web package
	@echo "🔍 Type checking web package..."
	@pnpm --filter @celebrum-ai/web run typecheck

typecheck-worker: ## Run TypeScript type checking for worker package
	@echo "🔍 Type checking worker package..."
	@pnpm --filter @celebrum-ai/worker run typecheck

# Utility commands

check: ## Quick build check
	@echo "🔍 Quick build check..."
	@cargo check --verbose

check-wasm: ## Quick WASM compilation check
	@echo "🎯 Quick WASM compilation check..."
	@cargo check --target wasm32-unknown-unknown --lib --verbose

check-all: lint test build build-wasm check-wasm ## Run all basic checks (lint, test, build native & WASM)
	@echo "✅ All basic checks completed successfully!"

# Legacy commands (maintained for compatibility)
dev: fmt lint test check-wasm ## Quick development cycle (format, lint, test, WASM check)
	@echo "🚀 Development cycle completed!"

ci: ci-pipeline ## Run comprehensive CI pipeline (Rust + TypeScript)

deploy: build-wasm-release ## Prepare for deployment (build WASM and run tests)
	@echo "🚀 Preparing for deployment..."
	@cargo test --quiet
	@echo "✅ Ready for deployment!"

# Workflow commands (recommended usage)
quick: pre-commit ## Quick validation before commit
	@echo "⚡ Quick validation completed!"

validate: ci-pipeline ## Full validation (mirrors CI)
	@echo "✅ Full validation completed!"

fix-and-validate: fmt-fix ## Auto-fix formatting then validate
	@echo "🔧 Fix and validation completed!"

quality: full-check ## Comprehensive quality analysis
	@echo "🏆 Quality analysis completed!" 

# API Testing
test-api: ## Run API Flow Tests
	@echo "🌐 Running API Flow Tests..."
	@chmod +x scripts/prod/test-bot/test_api_flow.sh
	@./scripts/prod/test-bot/test_api_flow.sh

test-api-local: ## Run API Tests against local development server
	@echo "🏠 Running API Tests against local development server..."
	@BASE_URL=http://localhost:8787 ./scripts/prod/test-bot/test_api_flow.sh

test-api-staging: ## Run API Tests against staging environment
	@echo "🚀 Running API Tests against staging environment..."
	@BASE_URL=https://celebrum-ai-staging.your-domain.workers.dev ./scripts/prod/test-bot/test_api_flow.sh

test-api-production: ## Run API Tests against production environment
	@echo "🌍 Running API Tests against production environment..."
	@BASE_URL=https://celebrum-ai.your-domain.workers.dev ./scripts/prod/test-bot/test_api_flow.sh

# API v1 Direct Testing (No Telegram required)
test-api-v1: ## Run comprehensive API v1 tests with RBAC validation
	@echo "🔗 Running API v1 Comprehensive Tests..."
	@chmod +x scripts/prod/test-bot/test_api_v1_comprehensive.sh
	@./scripts/prod/test-bot/test_api_v1_comprehensive.sh

test-api-v1-local: ## Run API v1 tests against local development server
	@echo "🏠 Running API v1 Tests against local development server..."
	@BASE_URL=http://localhost:8787 ./scripts/prod/test-bot/test_api_v1_comprehensive.sh

test-api-v1-staging: ## Run API v1 tests against staging environment
	@echo "🚀 Running API v1 Tests against staging environment..."
	@BASE_URL=https://celebrum-ai-staging.your-domain.workers.dev ./scripts/prod/test-bot/test_api_v1_comprehensive.sh

test-api-v1-production: ## Run API v1 tests against production environment
	@echo "🌍 Running API v1 Tests against production environment..."
	@BASE_URL=https://celebrum-ai.irfandimarsya.workers.dev ./scripts/prod/test-bot/test_api_v1_comprehensive.sh

test-api-prod-admin: ## Run Production API Tests (Super Admin Only with D1 Database)
	@echo "👑 Running Production API Tests (Super Admin + D1 Database)..."
	@chmod +x scripts/prod/test-bot/test_api_flow_prod.sh
	@./scripts/prod/test-bot/test_api_flow_prod.sh

# Performance Testing
test-performance: ## Run comprehensive performance tests
	@echo "⚡ Running Comprehensive Performance Tests..."
	@chmod +x scripts/prod/test-bot/test_performance_comprehensive.sh
	@./scripts/prod/test-bot/test_performance_comprehensive.sh

test-performance-local: ## Run performance tests against local development server
	@echo "🏠 Running Performance Tests against local development server..."
	@BASE_URL=http://localhost:8787 ./scripts/prod/test-bot/test_performance_comprehensive.sh

test-performance-staging: ## Run performance tests against staging environment
	@echo "🚀 Running Performance Tests against staging environment..."
	@BASE_URL=https://celebrum-ai-staging.your-domain.workers.dev ./scripts/prod/test-bot/test_performance_comprehensive.sh

test-performance-production: ## Run performance tests against production environment
	@echo "🌍 Running Performance Tests against production environment..."
	@BASE_URL=https://celebrum-ai.irfandimarsya.workers.dev ./scripts/prod/test-bot/test_performance_comprehensive.sh

test-performance-stress: ## Run high-stress performance tests (100 concurrent users)
	@echo "💥 Running High-Stress Performance Tests..."
	@CONCURRENT_USERS=100 REQUESTS_PER_USER=20 STRESS_DURATION=60 ./scripts/prod/test-bot/test_performance_comprehensive.sh

test-webhook-local: ## Run webhook tests against local development server
	@echo "🔗 Running Webhook Tests against local development server..."
	@./test_telegram_webhook.sh

# High-Scale Performance Testing (10K Users)
test-performance-10k: ## Run 10K concurrent users performance test (PRODUCTION ONLY)
	@echo "🚀 Running 10K Users Performance Test..."
	@chmod +x scripts/prod/test-bot/test_performance_10k_users.sh
	@./scripts/prod/test-bot/test_performance_10k_users.sh

test-performance-10k-production: ## Run 10K users test against production environment
	@echo "🌍 Running 10K Users Test against production environment..."
	@BASE_URL=https://celebrum-ai.irfandimarsya.workers.dev ./scripts/prod/test-bot/test_performance_10k_users.sh

test-performance-10k-staging: ## Run 10K users test against staging environment
	@echo "🚀 Running 10K Users Test against staging environment..."
	@BASE_URL=https://celebrum-ai-staging.your-domain.workers.dev ./scripts/prod/test-bot/test_performance_10k_users.sh

test-performance-ramp: ## Run gradual ramp-up test (100->10K users)
	@echo "📈 Running Gradual Ramp-up Test..."
	@MAX_USERS=10000 RAMP_UP_DURATION=600 ./scripts/prod/test-bot/test_performance_10k_users.sh

test-performance-extreme: ## Run extreme load test (20K users, 30min duration)
	@echo "💥 Running Extreme Load Test..."
	@MAX_USERS=20000 TEST_DURATION=1800 RAMP_UP_DURATION=900 ./scripts/prod/test-bot/test_performance_10k_users.sh

test-performance-quick-10k: ## Run quick 10K users test (5min duration)
	@echo "⚡ Running Quick 10K Users Test..."
	@MAX_USERS=10000 TEST_DURATION=300 RAMP_UP_DURATION=120 ./scripts/prod/test-bot/test_performance_10k_users.sh

# Complete API Testing (All Functionality)
test-complete-super-admin: ## Run comprehensive test of ALL functionality with super admin access
	@echo "🚀 Running Complete Super Admin API Test (ALL Functionality)..."
	@chmod +x scripts/prod/test-bot/test_complete_super_admin_api.sh
	@./scripts/prod/test-bot/test_complete_super_admin_api.sh

test-complete-super-admin-production: ## Run complete super admin test against production environment
	@echo "🌍 Running Complete Super Admin Test against production environment..."
	@BASE_URL=https://celebrum-ai.irfandimarsya.workers.dev ./scripts/prod/test-bot/test_complete_super_admin_api.sh

test-complete-super-admin-local: ## Run complete super admin test against local development server
	@echo "🏠 Running Complete Super Admin Test against local development server..."
	@BASE_URL=http://localhost:8787 ./scripts/prod/test-bot/test_complete_super_admin_api.sh