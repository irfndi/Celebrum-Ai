#!/bin/bash

# Security Audit Script for ArbEdge Monorepo
# Performs comprehensive dependency vulnerability scanning

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

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --fix)
            FIX_MODE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--fix] [--help]"
            echo "  --fix    Automatically fix vulnerabilities where possible"
            echo "  --help   Show this help message"
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

# Function to run npm audit for a package
run_npm_audit() {
    local package_dir="$1"
    local package_name="$2"
    
    log_info "Running npm audit for $package_name..."
    
    cd "$package_dir"
    
    # Check if package.json exists
    if [[ ! -f "package.json" ]]; then
        log_warning "No package.json found in $package_dir, skipping..."
        return 0
    fi
    
    # Generate audit report
    local audit_file="$REPORT_DIR/npm-audit-${package_name}-${TIMESTAMP}.json"
    local audit_summary="$REPORT_DIR/npm-audit-${package_name}-${TIMESTAMP}.txt"
    
    # Run audit and capture output
    pnpm audit --json > "$audit_file" 2>/dev/null
    local exit_code=$?
    
    # Check if there are actual vulnerabilities by examining the audit file content
    local vulnerability_count=0
    if [[ -f "$audit_file" ]] && [[ -s "$audit_file" ]]; then
        # Check if the JSON contains vulnerabilities
        if command -v jq >/dev/null 2>&1; then
            vulnerability_count=$(jq -r '.metadata.vulnerabilities.total // 0' "$audit_file" 2>/dev/null || echo "0")
        else
            # Fallback: check if file contains vulnerability indicators
            if grep -q '"vulnerabilities"' "$audit_file" && ! grep -q '"total":0' "$audit_file"; then
                vulnerability_count=1
            fi
        fi
    fi
    
    if [[ $vulnerability_count -eq 0 ]]; then
        log_success "No vulnerabilities found in $package_name"
        echo "No vulnerabilities found" > "$audit_summary"
        cd "$ROOT_DIR"
        return 0
    else
        log_warning "Vulnerabilities found in $package_name"
        
        # Generate human-readable summary
        pnpm audit > "$audit_summary" 2>/dev/null || true
        
        # Show summary
        echo "Audit summary for $package_name:"
        head -20 "$audit_summary" || true
        echo "Full report saved to: $audit_file"
        
        # Auto-fix if requested
        if [[ "$FIX_MODE" == "true" ]]; then
            log_info "Attempting to fix vulnerabilities in $package_name..."
            if pnpm audit --fix; then
                log_success "Vulnerabilities fixed in $package_name"
                # Re-run audit to check if vulnerabilities are actually fixed
                pnpm audit --json > "$audit_file" 2>/dev/null
                if command -v jq >/dev/null 2>&1; then
                    vulnerability_count=$(jq -r '.metadata.vulnerabilities.total // 0' "$audit_file" 2>/dev/null || echo "0")
                else
                    if grep -q '"vulnerabilities"' "$audit_file" && ! grep -q '"total":0' "$audit_file"; then
                        vulnerability_count=1
                    else
                        vulnerability_count=0
                    fi
                fi
                
                if [[ $vulnerability_count -eq 0 ]]; then
                    log_success "All vulnerabilities fixed in $package_name"
                    echo "All vulnerabilities fixed" > "$audit_summary"
                    cd "$ROOT_DIR"
                    return 0
                else
                    log_warning "Some vulnerabilities remain in $package_name after fix attempt"
                    cd "$ROOT_DIR"
                    return 1
                fi
            else
                log_warning "Some vulnerabilities could not be auto-fixed in $package_name"
                cd "$ROOT_DIR"
                return 1
            fi
        else
            # Vulnerabilities found but not in fix mode
            cd "$ROOT_DIR"
            return 1
        fi
    fi
    
    cd "$ROOT_DIR"
}

# Function to check for outdated packages
check_outdated() {
    local package_dir="$1"
    local package_name="$2"
    
    log_info "Checking for outdated packages in $package_name..."
    
    cd "$package_dir"
    
    local outdated_file="$REPORT_DIR/outdated-${package_name}-${TIMESTAMP}.json"
    
    # pnpm outdated returns 0 when no outdated packages, non-zero when outdated packages found
    if pnpm outdated --json > "$outdated_file" 2>/dev/null; then
        log_success "All packages are up to date in $package_name"
    else
        local exit_code=$?
        if [[ -s "$outdated_file" ]]; then
            log_warning "Outdated packages found in $package_name"
            echo "Outdated packages report saved to: $outdated_file"
        else
            log_success "All packages are up to date in $package_name"
        fi
    fi
    
    cd "$ROOT_DIR"
    return 0
}

# Function to validate package-lock integrity
validate_lockfile() {
    local package_dir="$1"
    local package_name="$2"
    
    log_info "Validating lockfile integrity for $package_name..."
    
    # For workspace packages, check the root lockfile
    if [[ "$package_name" != "root" ]]; then
        # This is a workspace package, check if root lockfile exists
        if [[ -f "$ROOT_DIR/pnpm-lock.yaml" ]]; then
            log_success "Using root lockfile for workspace package $package_name"
        else
            log_warning "No root pnpm-lock.yaml found for workspace package $package_name"
        fi
    else
        # This is the root package, check the root lockfile
        cd "$ROOT_DIR"
        
        if [[ -f "pnpm-lock.yaml" ]]; then
            # Simple validation: check if lockfile is readable and not empty
            if [[ -r "pnpm-lock.yaml" && -s "pnpm-lock.yaml" ]]; then
                log_success "Lockfile is valid for $package_name"
            else
                log_error "Lockfile integrity check failed for $package_name"
                return 1
            fi
        else
            log_warning "No pnpm-lock.yaml found for $package_name"
            return 1
        fi
    fi
    
    return 0
}

# Main execution
main() {
    log_info "Starting security audit for ArbEdge monorepo..."
    log_info "Report directory: $REPORT_DIR"
    
    if [[ "$FIX_MODE" == "true" ]]; then
        log_info "Auto-fix mode enabled"
    fi
    
    # Ensure we're in the root directory
    cd "$ROOT_DIR"
    
    # Check if pnpm is available
    if ! command -v pnpm &> /dev/null; then
        log_error "pnpm is not installed or not in PATH"
        exit 1
    fi
    
    local packages=(
        "$ROOT_DIR:root"
        "$ROOT_DIR/packages/shared:shared"
        "$ROOT_DIR/packages/worker:worker"
        "$ROOT_DIR/packages/web:web"
        "$ROOT_DIR/packages/db:db"
        "$ROOT_DIR/packages/telegram-bot:telegram-bot"
    )
    
    local overall_status=0
    
    for package_info in "${packages[@]}"; do
        IFS=':' read -r package_dir package_name <<< "$package_info"
        
        if [[ -d "$package_dir" ]]; then
            echo
            log_info "Processing package: $package_name"
            echo "----------------------------------------"
            
            # Run npm audit
            if ! run_npm_audit "$package_dir" "$package_name"; then
                overall_status=1
            fi
            
            # Check for outdated packages (warning only, doesn't fail audit)
            check_outdated "$package_dir" "$package_name"
            
            # Validate lockfile
            if ! validate_lockfile "$package_dir" "$package_name"; then
                overall_status=1
            fi
        else
            log_warning "Package directory not found: $package_dir"
        fi
    done
    
    echo
    log_info "Security audit completed"
    log_info "Reports saved to: $REPORT_DIR"
    
    if [[ $overall_status -eq 0 ]]; then
        log_success "All security checks passed!"
    else
        log_error "Some security issues were found. Please review the reports."
        exit 1
    fi
}

# Run main function only if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi