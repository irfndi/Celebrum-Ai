#!/bin/bash

# Security Code Scanning Script for ArbEdge Monorepo
# Performs static code analysis for security vulnerabilities

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="$ROOT_DIR/.security-reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
FIX_MODE=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --fix)
            FIX_MODE=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--fix] [--verbose] [--help]"
            echo "  --fix      Automatically fix security issues where possible"
            echo "  --verbose  Show detailed output"
            echo "  --help     Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Create reports directory
mkdir -p "$REPORT_DIR"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Function to create ESLint security config
create_eslint_config() {
    local config_file="$ROOT_DIR/.eslintrc.security.js"
    
    log_info "Creating ESLint security configuration..."
    
    cat > "$config_file" << 'EOF'
module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: 'module',
    project: './tsconfig.json'
  },
  plugins: [
    '@typescript-eslint',
    'security'
  ],
  extends: [
    'eslint:recommended',
    '@typescript-eslint/recommended',
    'plugin:security/recommended'
  ],
  rules: {
    // Security-focused rules
    'security/detect-buffer-noassert': 'error',
    'security/detect-child-process': 'error',
    'security/detect-disable-mustache-escape': 'error',
    'security/detect-eval-with-expression': 'error',
    'security/detect-new-buffer': 'error',
    'security/detect-no-csrf-before-method-override': 'error',
    'security/detect-non-literal-fs-filename': 'warn',
    'security/detect-non-literal-regexp': 'warn',
    'security/detect-non-literal-require': 'error',
    'security/detect-object-injection': 'warn',
    'security/detect-possible-timing-attacks': 'warn',
    'security/detect-pseudoRandomBytes': 'error',
    'security/detect-unsafe-regex': 'error',
    
    // TypeScript security rules
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-unsafe-assignment': 'warn',
    '@typescript-eslint/no-unsafe-call': 'warn',
    '@typescript-eslint/no-unsafe-member-access': 'warn',
    '@typescript-eslint/no-unsafe-return': 'warn',
    
    // General security best practices
    'no-eval': 'error',
    'no-implied-eval': 'error',
    'no-new-func': 'error',
    'no-script-url': 'error',
    'no-console': 'warn'
  },
  env: {
    node: true,
    es2022: true
  },
  ignorePatterns: [
    'dist/',
    'build/',
    'node_modules/',
    '*.d.ts',
    'target/',
    '.astro/'
  ]
};
EOF
    
    log_success "ESLint security configuration created"
}

# Function to run security scan on a package
run_security_scan() {
    local package_dir="$1"
    local package_name="$2"
    
    log_info "Running security scan for $package_name..."
    
    cd "$package_dir"
    
    # Check if there are TypeScript/JavaScript files to scan
    if ! find . -name "*.ts" -o -name "*.js" -o -name "*.tsx" -o -name "*.jsx" | grep -q .; then
        log_warning "No TypeScript/JavaScript files found in $package_name, skipping..."
        cd "$ROOT_DIR"
        return 0
    fi
    
    local scan_report="$REPORT_DIR/security-scan-${package_name}-${TIMESTAMP}.json"
    local scan_summary="$REPORT_DIR/security-scan-${package_name}-${TIMESTAMP}.txt"
    
    # Run ESLint with security rules
    log_verbose "Running ESLint security scan..."
    
    local eslint_cmd="npx eslint"
    if command -v pnpm &> /dev/null; then
        eslint_cmd="pnpm exec eslint"
    fi
    
    local eslint_args=(
        "--config" "$ROOT_DIR/.eslintrc.security.js"
        "--ext" ".ts,.js,.tsx,.jsx"
        "--format" "json"
        "--output-file" "$scan_report"
    )
    
    if [[ "$FIX_MODE" == "true" ]]; then
        eslint_args+=("--fix")
    fi
    
    # Add source directories
    if [[ -d "src" ]]; then
        eslint_args+=("src")
    fi
    if [[ -d "lib" ]]; then
        eslint_args+=("lib")
    fi
    if [[ -d "tests" && "$package_name" != "root" ]]; then
        eslint_args+=("tests")
    fi
    
    # Run the scan
    if $eslint_cmd "${eslint_args[@]}" 2>/dev/null; then
        log_success "No security issues found in $package_name"
        echo "No security issues found" > "$scan_summary"
    else
        local exit_code=$?
        if [[ $exit_code -eq 1 ]]; then
            log_warning "Security issues found in $package_name"
            
            # Generate human-readable summary
            $eslint_cmd "${eslint_args[@]%--output-file*}" "${eslint_args[@]##*--output-file}" --format stylish > "$scan_summary" 2>/dev/null || true
            
            # Show summary if verbose
            if [[ "$VERBOSE" == "true" ]]; then
                echo "Security scan summary for $package_name:"
                head -30 "$scan_summary" || true
            fi
            
            echo "Full security scan report saved to: $scan_report"
            
            if [[ "$FIX_MODE" == "true" ]]; then
                log_info "Auto-fix was applied where possible"
            fi
        else
            log_error "Security scan failed for $package_name with exit code $exit_code"
            cd "$ROOT_DIR"
            return 1
        fi
    fi
    
    cd "$ROOT_DIR"
}

# Function to scan for hardcoded secrets
scan_secrets() {
    local package_dir="$1"
    local package_name="$2"
    
    log_info "Scanning for potential secrets in $package_name..."
    
    cd "$package_dir"
    
    local secrets_report="$REPORT_DIR/secrets-scan-${package_name}-${TIMESTAMP}.txt"
    
    # Patterns to look for
    local patterns=(
        "password\s*=\s*['\"][^'\"]+['\"]"  # password assignments
        "api[_-]?key\s*[=:]\s*['\"][^'\"]+['\"]"  # API keys
        "secret\s*[=:]\s*['\"][^'\"]+['\"]"  # secrets
        "token\s*[=:]\s*['\"][^'\"]+['\"]"  # tokens
        "['\"][A-Za-z0-9]{32,}['\"]"  # long strings that might be keys
        "-----BEGIN [A-Z ]+-----"  # PEM keys
    )
    
    local found_issues=false
    
    for pattern in "${patterns[@]}"; do
        if grep -r -i -E "$pattern" . --include="*.ts" --include="*.js" --include="*.json" --exclude-dir=node_modules --exclude-dir=dist --exclude-dir=build 2>/dev/null >> "$secrets_report"; then
            found_issues=true
        fi
    done
    
    if [[ "$found_issues" == "true" ]]; then
        log_warning "Potential secrets found in $package_name - review $secrets_report"
    else
        log_success "No obvious secrets found in $package_name"
        echo "No secrets detected" > "$secrets_report"
    fi
    
    cd "$ROOT_DIR"
}

# Function to check file permissions
check_file_permissions() {
    local package_dir="$1"
    local package_name="$2"
    
    log_info "Checking file permissions for $package_name..."
    
    cd "$package_dir"
    
    local perms_report="$REPORT_DIR/permissions-${package_name}-${TIMESTAMP}.txt"
    
    # Check for overly permissive files
    find . -type f \( -perm -002 -o -perm -020 \) -not -path "./node_modules/*" -not -path "./dist/*" -not -path "./build/*" > "$perms_report" 2>/dev/null || true
    
    if [[ -s "$perms_report" ]]; then
        log_warning "Files with overly permissive permissions found in $package_name"
        if [[ "$VERBOSE" == "true" ]]; then
            cat "$perms_report"
        fi
    else
        log_success "File permissions look good for $package_name"
        echo "No permission issues found" > "$perms_report"
    fi
    
    cd "$ROOT_DIR"
}

# Main execution
main() {
    log_info "Starting security code scanning for ArbEdge monorepo..."
    log_info "Report directory: $REPORT_DIR"
    
    if [[ "$FIX_MODE" == "true" ]]; then
        log_info "Auto-fix mode enabled"
    fi
    
    # Ensure we're in the root directory
    cd "$ROOT_DIR"
    
    # Create ESLint security configuration
    create_eslint_config
    
    # Check if required tools are available
    if ! command -v node &> /dev/null; then
        log_error "Node.js is not installed or not in PATH"
        exit 1
    fi
    
    local packages=(
        "$ROOT_DIR/packages/shared:shared"
        "$ROOT_DIR/packages/worker:worker"
        "$ROOT_DIR/packages/web:web"
        "$ROOT_DIR/packages/db:db"
    )
    
    local overall_status=0
    
    for package_info in "${packages[@]}"; do
        IFS=':' read -r package_dir package_name <<< "$package_info"
        
        if [[ -d "$package_dir" ]]; then
            echo
            log_info "Processing package: $package_name"
            echo "----------------------------------------"
            
            # Run security scan
            if ! run_security_scan "$package_dir" "$package_name"; then
                overall_status=1
            fi
            
            # Scan for secrets
            scan_secrets "$package_dir" "$package_name"
            
            # Check file permissions
            check_file_permissions "$package_dir" "$package_name"
        else
            log_warning "Package directory not found: $package_dir"
        fi
    done
    
    echo
    log_info "Security code scanning completed"
    log_info "Reports saved to: $REPORT_DIR"
    
    if [[ $overall_status -eq 0 ]]; then
        log_success "All security scans passed!"
    else
        log_error "Some security issues were found. Please review the reports."
        exit 1
    fi
}

# Run main function
main "$@"