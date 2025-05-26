#!/bin/bash
# Comprehensive validation script with coverage and extensive checks
# Runs all possible validations for thorough code quality assurance

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

print_info() {
    echo -e "${PURPLE}ℹ️  $1${NC}"
}

print_header() {
    echo -e "\n${BLUE}=============================================${NC}"
    echo -e "${BLUE}🔍 ArbEdge Comprehensive Code Quality Check${NC}"
    echo -e "${BLUE}=============================================${NC}"
}

# Ensure we're using correct Rust toolchain
export PATH="$HOME/.cargo/bin:$PATH"

print_header

# Step 1: Environment info
print_step "Environment Information"
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Git branch: $(git branch --show-current 2>/dev/null || echo 'detached')"
echo "Git status: $(git status --porcelain | wc -l | tr -d ' ') uncommitted changes"

# Step 2: Clean build
print_step "Cleaning previous builds"
cargo clean
print_success "Build artifacts cleaned"

# Step 3: Check dependencies
print_step "Checking dependencies and security"
if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues"
    fi
else
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

# Step 4: Format check
print_step "Checking code formatting"
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found"
    echo "Run 'cargo fmt --all' to fix formatting"
    exit 1
fi

# Step 5: Comprehensive clippy check
print_step "Running comprehensive clippy analysis"
if cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic; then
    print_success "Clippy checks passed (including pedantic)"
else
    print_error "Clippy found issues"
    exit 1
fi

# Step 6: Run tests with output
print_step "Running all tests with detailed output"
if cargo test --all-features --all-targets --verbose; then
    print_success "All tests passed"
else
    print_error "Tests failed"
    exit 1
fi

# Step 7: Coverage analysis
print_step "Generating test coverage report"
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    if cargo tarpaulin --out Html --output-dir coverage --timeout 300; then
        print_success "Coverage report generated in coverage/"
        # Show coverage summary if available
        if [ -f "coverage/tarpaulin-report.html" ]; then
            print_info "Coverage report available at: coverage/tarpaulin-report.html"
        fi
    else
        print_warning "Coverage generation failed"
    fi
else
    print_warning "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
    print_info "Alternative: Use 'cargo test' with coverage tools"
fi

# Step 8: Build checks
print_step "Building for all targets"

# Native build
if cargo build --release; then
    print_success "Native release build successful"
else
    print_error "Native build failed"
    exit 1
fi

# WASM build
if cargo build --target wasm32-unknown-unknown --release; then
    print_success "WASM release build successful"
else
    print_error "WASM build failed"
    exit 1
fi

# Step 9: Documentation check
print_step "Checking documentation"
if cargo doc --no-deps --document-private-items; then
    print_success "Documentation builds successfully"
else
    print_warning "Documentation has issues"
fi

# Step 10: Wrangler validation
print_step "Validating Cloudflare deployment"
if command -v wrangler >/dev/null 2>&1; then
    if wrangler deploy --dry-run; then
        print_success "Wrangler deployment validation passed"
    else
        print_error "Wrangler deployment validation failed"
        exit 1
    fi
else
    print_warning "Wrangler not installed. Install with: npm install -g wrangler@latest"
fi

# Step 11: Code quality metrics
print_step "Analyzing code quality metrics"

# Count lines of code
if command -v tokei >/dev/null 2>&1; then
    print_info "Code statistics:"
    tokei
else
    print_info "Lines of Rust code: $(find src -name '*.rs' -exec wc -l {} + | tail -1 | awk '{print $1}')"
fi

# Check for common patterns
print_info "Code quality checks:"
UNWRAP_COUNT=$(find src -name '*.rs' -exec grep -l "unwrap()" {} \; 2>/dev/null | wc -l | tr -d ' ')
TODO_COUNT=$(find src -name '*.rs' -exec grep -l "TODO\|FIXME" {} \; 2>/dev/null | wc -l | tr -d ' ')
echo "  - Files with unwrap(): $UNWRAP_COUNT"
echo "  - Files with TODO/FIXME: $TODO_COUNT"

# Step 12: Final validation
print_step "Final validation checks"

# Check git status
if [ -n "$(git status --porcelain)" ]; then
    print_warning "Uncommitted changes detected"
    echo "Consider committing your changes:"
    git status --short
else
    print_success "Working directory is clean"
fi

# Final summary
echo -e "\n${GREEN}=============================================${NC}"
echo -e "${GREEN}🎉 Comprehensive checks completed!${NC}"
echo -e "${GREEN}=============================================${NC}"
echo -e "Your code has passed all quality checks."
echo -e ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Review any warnings above"
echo -e "  2. Check coverage report if generated"
echo -e "  3. Commit your changes if satisfied"
echo -e "  4. Run './scripts/local-ci.sh' before pushing" 