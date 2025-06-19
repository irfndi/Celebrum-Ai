#!/bin/bash
# Local CI script that mirrors GitHub Actions CI/CD pipeline
# Runs all the same checks as .github/workflows/ci.yml

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Utility functions
print_step() {
    echo -e "\n${BLUE}🔄 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo -e "\n${BLUE}=====================================${NC}"
    echo -e "${BLUE}🦀 ArbEdge Local CI Pipeline${NC}"
    echo -e "${BLUE}=====================================${NC}"
}

# Ensure we're using correct Rust toolchain
export PATH="$HOME/.cargo/bin:$PATH"

print_header

# Step 1: Environment setup (mirrors CI setup steps)
print_step "Setting up environment"
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Node version: $(node --version 2>/dev/null || echo 'Node not found')"
echo "pnpm version: $(pnpm --version 2>/dev/null || echo 'pnpm not found')"

# Step 1.5: Install pnpm dependencies (mirrors CI)
print_step "Installing pnpm dependencies"
if command -v pnpm >/dev/null 2>&1; then
    if pnpm install --frozen-lockfile; then
        print_success "pnpm dependencies installed"
    else
        print_error "pnpm install failed"
        exit 1
    fi
else
    print_error "pnpm not found. Please install pnpm first."
    exit 1
fi

# Step 2: Add WASM target (mirrors CI)
print_step "Adding WASM target"
if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    print_success "WASM target already installed"
else
    rustup target add wasm32-unknown-unknown
    print_success "WASM target added"
fi

# Step 3: Lint TypeScript packages (mirrors CI)
print_step "Linting TypeScript packages"
if pnpm run lint; then
    print_success "TypeScript linting passed"
else
    print_error "TypeScript linting failed"
    exit 1
fi

# Step 4: Type check all packages (mirrors CI)
print_step "Type checking all packages"
if pnpm run typecheck; then
    print_success "TypeScript type checking passed"
else
    print_error "TypeScript type checking failed"
    exit 1
fi

# Step 5: Build TypeScript packages (mirrors CI)
print_step "Building TypeScript packages"
if pnpm run build; then
    print_success "TypeScript packages built successfully"
else
    print_error "TypeScript package build failed"
    exit 1
fi

# Step 6: Test TypeScript packages (mirrors CI)
print_step "Testing TypeScript packages"
if pnpm run test; then
    print_success "TypeScript tests passed"
else
    print_error "TypeScript tests failed"
    exit 1
fi

# Step 6: Check Rust formatting (mirrors CI)
print_step "Checking Rust code formatting"
if cargo fmt --all -- --check; then
    print_success "Rust code formatting is correct"
else
    print_error "Rust code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Step 7: Check Rust compilation (mirrors CI)
print_step "Checking Rust compilation for all targets"
if cargo check --all-targets --all-features; then
    print_success "Rust compilation check passed"
else
    print_error "Rust compilation errors found"
    exit 1
fi

# Step 8: Run Rust linter (mirrors CI)
print_step "Running Rust clippy linter"
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Rust clippy checks passed"
else
    print_error "Rust clippy warnings/errors found"
    exit 1
fi

# Step 9: Run Rust tests (mirrors CI)
print_step "Running Rust tests"
if cargo test --verbose --all-targets --all-features; then
    print_success "All Rust tests passed"
else
    print_error "Rust tests failed"
    exit 1
fi

# Step 10: Build for WASM (mirrors CI)
print_step "Building for WASM target"
if cargo build --target wasm32-unknown-unknown --release; then
    print_success "WASM build successful"
else
    print_error "WASM build failed"
    exit 1
fi

# Step 11: Test wrangler build (mirrors CI dry-run)
print_step "Testing wrangler build (dry-run)"
if command -v wrangler >/dev/null 2>&1; then
    echo "Wrangler version: $(wrangler --version)"
    if wrangler deploy --dry-run; then
        print_success "Wrangler dry-run successful"
    else
        print_error "Wrangler dry-run failed"
        exit 1
    fi
else
    print_warning "Wrangler not installed, skipping dry-run test"
    print_warning "Install with: pnpm add -g wrangler@latest"
fi

# Final summary
echo -e "\n${GREEN}=====================================${NC}"
echo -e "${GREEN}🎉 All CI checks passed locally!${NC}"
echo -e "${GREEN}=====================================${NC}"
echo -e "Your code is ready for commit and push."
echo -e "The GitHub Actions CI should pass without issues."