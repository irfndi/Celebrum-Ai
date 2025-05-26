#!/bin/bash
# Quick pre-commit validation script
# Runs essential checks before committing code

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

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo -e "\n${BLUE}=====================================${NC}"
    echo -e "${BLUE}🚀 ArbEdge Pre-commit Checks${NC}"
    echo -e "${BLUE}=====================================${NC}"
}

# Configuration
SKIP_TESTS=${SKIP_TESTS:-false}
SKIP_BUILD=${SKIP_BUILD:-false}

# Ensure we're using correct Rust toolchain
export PATH="$HOME/.cargo/bin:$PATH"

print_header

# Step 1: Auto-format code
print_step "Auto-formatting code"
if cargo fmt --all; then
    print_success "Code formatted"
else
    print_error "Formatting failed"
    exit 1
fi

# Step 2: Quick lint check
print_step "Running quick lint check"
if cargo clippy --quiet -- -D warnings; then
    print_success "Lint checks passed"
else
    print_error "Lint issues found"
    echo "Run 'cargo clippy --fix --allow-dirty' to auto-fix some issues"
    exit 1
fi

# Step 3: Run tests (unless skipped)
if [ "$SKIP_TESTS" != "true" ]; then
    print_step "Running tests"
    if cargo test --quiet --all-targets; then
        print_success "Tests passed"
    else
        print_error "Tests failed"
        echo "Use 'SKIP_TESTS=true ./scripts/pre-commit.sh' to skip tests"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  Tests skipped (SKIP_TESTS=true)${NC}"
fi

# Step 4: Quick build check (unless skipped)
if [ "$SKIP_BUILD" != "true" ]; then
    print_step "Quick build check"
    if cargo check --quiet --all-targets; then
        print_success "Build check passed"
    else
        print_error "Build check failed"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  Build check skipped (SKIP_BUILD=true)${NC}"
fi

# Step 5: Check for common issues
print_step "Checking for common issues"

# Check for TODO/FIXME comments in new/modified files
if git diff --cached --name-only | grep -E '\.(rs|toml|md)$' | xargs grep -l "TODO\|FIXME" 2>/dev/null; then
    print_error "Found TODO/FIXME comments in staged files:"
    git diff --cached --name-only | grep -E '\.(rs|toml|md)$' | xargs grep -n "TODO\|FIXME" 2>/dev/null || true
    echo "Consider addressing these before committing, or add them to your commit message"
    # Don't fail, just warn
fi

# Check for unwrap() calls in Rust files
if git diff --cached --name-only | grep '\.rs$' | xargs grep -l "unwrap()" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  Found unwrap() calls in staged Rust files${NC}"
    echo "Consider using proper error handling instead of unwrap()"
    # Don't fail, just warn
fi

print_success "Common issue checks completed"

# Final summary
echo -e "\n${GREEN}=====================================${NC}"
echo -e "${GREEN}🎉 Pre-commit checks passed!${NC}"
echo -e "${GREEN}=====================================${NC}"
echo -e "Your changes are ready to commit."
echo -e "Run './scripts/local-ci.sh' for full CI validation."

# Show what would be committed
echo -e "\n${BLUE}Files to be committed:${NC}"
git diff --cached --name-only | sed 's/^/  /' 