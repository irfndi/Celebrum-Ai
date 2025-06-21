#!/bin/bash

# Comprehensive Security Check for ArbEdge Monorepo
# Orchestrates all security tools and generates unified reports

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="$ROOT_DIR/.security-reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Default options
RUN_AUDIT=true
RUN_SCAN=true
GENERATE_REPORT=true
AUTO_FIX=false
OPEN_REPORT=false
REPORT_FORMAT="html"
VERBOSE=false
FAIL_ON_ISSUES=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-audit)
            RUN_AUDIT=false
            shift
            ;;
        --skip-scan)
            RUN_SCAN=false
            shift
            ;;
        --skip-report)
            GENERATE_REPORT=false
            shift
            ;;
        --fix)
            AUTO_FIX=true
            shift
            ;;
        --open)
            OPEN_REPORT=true
            shift
            ;;
        --format)
            REPORT_FORMAT="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --fail-on-issues)
            FAIL_ON_ISSUES=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Comprehensive security check for ArbEdge monorepo"
            echo ""
            echo "Options:"
            echo "  --skip-audit      Skip dependency vulnerability audit"
            echo "  --skip-scan       Skip code security scanning"
            echo "  --skip-report     Skip report generation"
            echo "  --fix             Automatically fix issues where possible"
            echo "  --open            Open the generated report"
            echo "  --format FORMAT   Report format (html, json, text) - default: html"
            echo "  --verbose         Enable verbose output"
            echo "  --fail-on-issues  Exit with error code if security issues found"
            echo "  --help            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                          # Run full security check"
            echo "  $0 --fix --open             # Run check, auto-fix, and open report"
            echo "  $0 --skip-audit --verbose   # Skip audit, run scan with verbose output"
            echo "  $0 --format json            # Generate JSON report"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

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
        echo -e "${CYAN}[VERBOSE]${NC} $1"
    fi
}

# Function to check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_tools=()
    
    # Check for required tools
    if ! command -v pnpm &> /dev/null; then
        missing_tools+=("pnpm")
    fi
    
    if ! command -v node &> /dev/null; then
        missing_tools+=("node")
    fi
    
    if ! command -v jq &> /dev/null; then
        log_warning "jq is not installed. Some report features may be limited."
        log_info "Install jq with: brew install jq (macOS) or apt-get install jq (Ubuntu)"
    fi
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Please install the missing tools and try again."
        exit 1
    fi
    
    # Check Node.js version
    local node_version
    node_version=$(node --version | sed 's/v//')
    local required_version="22.0.0"
    
    if ! printf '%s\n%s\n' "$required_version" "$node_version" | sort -V -C; then
        log_warning "Node.js version $node_version is below recommended $required_version"
    fi
    
    # Check if we're in the right directory
    if [[ ! -f "$ROOT_DIR/package.json" ]]; then
        log_error "Not in ArbEdge root directory. Please run from project root."
        exit 1
    fi
    
    log_verbose "Prerequisites check completed"
}

# Function to install dependencies if needed
ensure_dependencies() {
    log_info "Ensuring security dependencies are installed..."
    
    # Check if security dependencies are installed
    if [[ ! -d "$ROOT_DIR/node_modules/eslint" ]] || [[ ! -d "$ROOT_DIR/node_modules/eslint-plugin-security" ]]; then
        log_info "Installing security dependencies..."
        cd "$ROOT_DIR"
        pnpm install --dev
    fi
    
    log_verbose "Dependencies check completed"
}

# Function to run dependency audit
run_audit() {
    if [[ "$RUN_AUDIT" != "true" ]]; then
        log_verbose "Skipping dependency audit"
        return 0
    fi
    
    log_info "Running dependency vulnerability audit..."
    
    local audit_args=()
    if [[ "$AUTO_FIX" == "true" ]]; then
        audit_args+=("--fix")
    fi
    
    if [[ "$VERBOSE" == "true" ]]; then
        "$SCRIPT_DIR/audit.sh" "${audit_args[@]}"
    else
        "$SCRIPT_DIR/audit.sh" "${audit_args[@]}" > /dev/null 2>&1 || {
            log_warning "Dependency audit found issues. Check detailed reports in $REPORT_DIR"
            return 1
        }
    fi
    
    log_success "Dependency audit completed"
    return 0
}

# Function to run security scan
run_scan() {
    if [[ "$RUN_SCAN" != "true" ]]; then
        log_verbose "Skipping code security scan"
        return 0
    fi
    
    log_info "Running code security scan..."
    
    local scan_args=()
    if [[ "$AUTO_FIX" == "true" ]]; then
        scan_args+=("--fix")
    fi
    
    if [[ "$VERBOSE" == "true" ]]; then
        "$SCRIPT_DIR/scan.sh" "${scan_args[@]}"
    else
        "$SCRIPT_DIR/scan.sh" "${scan_args[@]}" > /dev/null 2>&1 || {
            log_warning "Code security scan found issues. Check detailed reports in $REPORT_DIR"
            return 1
        }
    fi
    
    log_success "Code security scan completed"
    return 0
}

# Function to generate security report
generate_report() {
    if [[ "$GENERATE_REPORT" != "true" ]]; then
        log_verbose "Skipping report generation"
        return 0
    fi
    
    log_info "Generating security report..."
    
    local report_args=("--format" "$REPORT_FORMAT")
    if [[ "$OPEN_REPORT" == "true" ]]; then
        report_args+=("--open")
    fi
    
    local report_file
    if [[ "$VERBOSE" == "true" ]]; then
        report_file=$("$SCRIPT_DIR/report.sh" "${report_args[@]}")
    else
        report_file=$("$SCRIPT_DIR/report.sh" "${report_args[@]}" 2>/dev/null)
    fi
    
    if [[ -n "$report_file" ]]; then
        log_success "Security report generated: $report_file"
    else
        log_warning "Report generation completed but file path not returned"
    fi
    
    return 0
}

# Function to analyze results and determine exit code
analyze_results() {
    if [[ "$FAIL_ON_ISSUES" != "true" ]]; then
        return 0
    fi
    
    log_info "Analyzing security results..."
    
    local has_critical_issues=false
    
    # Check for critical vulnerabilities in audit reports
    if [[ -d "$REPORT_DIR" ]]; then
        local audit_reports
        mapfile -t audit_reports < <(find "$REPORT_DIR" -name "*npm-audit*.json" -type f | head -5)
        
        for report in "${audit_reports[@]}"; do
            if [[ -f "$report" && -s "$report" ]]; then
                if command -v jq &> /dev/null; then
                    local critical_count
                    critical_count=$(jq '.metadata.vulnerabilities.critical // 0' "$report" 2>/dev/null || echo "0")
                    
                    if [[ "$critical_count" -gt 0 ]]; then
                        has_critical_issues=true
                        log_warning "Found $critical_count critical vulnerabilities in $(basename "$report")"
                    fi
                fi
            fi
        done
        
        # Check for security errors in scan reports
        local scan_reports
        mapfile -t scan_reports < <(find "$REPORT_DIR" -name "*security-scan*.json" -type f | head -5)
        
        for report in "${scan_reports[@]}"; do
            if [[ -f "$report" && -s "$report" ]]; then
                if command -v jq &> /dev/null; then
                    local error_count
                    error_count=$(jq '[.[].messages[] | select(.severity == 2)] | length' "$report" 2>/dev/null || echo "0")
                    
                    if [[ "$error_count" -gt 0 ]]; then
                        has_critical_issues=true
                        log_warning "Found $error_count security errors in $(basename "$report")"
                    fi
                fi
            fi
        done
    fi
    
    if [[ "$has_critical_issues" == "true" ]]; then
        log_error "Critical security issues found. Failing build."
        return 1
    fi
    
    log_success "No critical security issues found"
    return 0
}

# Function to cleanup temporary files
cleanup() {
    log_verbose "Cleaning up temporary files..."
    
    # Remove temporary ESLint config if it exists
    if [[ -f "$ROOT_DIR/.eslintrc.temp.js" ]]; then
        rm -f "$ROOT_DIR/.eslintrc.temp.js"
    fi
    
    # Clean up old report files (keep last 10)
    if [[ -d "$REPORT_DIR" ]]; then
        find "$REPORT_DIR" -name "*.json" -type f | sort -r | tail -n +11 | xargs rm -f 2>/dev/null || true
        find "$REPORT_DIR" -name "*.txt" -type f | sort -r | tail -n +11 | xargs rm -f 2>/dev/null || true
        find "$REPORT_DIR" -name "*.html" -type f | sort -r | tail -n +6 | xargs rm -f 2>/dev/null || true
    fi
}

# Function to display summary
display_summary() {
    echo ""
    echo -e "${PURPLE}=================================================================================${NC}"
    echo -e "${PURPLE}                        ARBEDGE SECURITY CHECK SUMMARY${NC}"
    echo -e "${PURPLE}=================================================================================${NC}"
    echo ""
    
    if [[ "$RUN_AUDIT" == "true" ]]; then
        echo -e "${CYAN}✓${NC} Dependency vulnerability audit completed"
    else
        echo -e "${YELLOW}⚠${NC} Dependency vulnerability audit skipped"
    fi
    
    if [[ "$RUN_SCAN" == "true" ]]; then
        echo -e "${CYAN}✓${NC} Code security scan completed"
    else
        echo -e "${YELLOW}⚠${NC} Code security scan skipped"
    fi
    
    if [[ "$GENERATE_REPORT" == "true" ]]; then
        echo -e "${CYAN}✓${NC} Security report generated ($REPORT_FORMAT format)"
    else
        echo -e "${YELLOW}⚠${NC} Security report generation skipped"
    fi
    
    if [[ "$AUTO_FIX" == "true" ]]; then
        echo -e "${GREEN}✓${NC} Auto-fix enabled"
    fi
    
    echo ""
    echo -e "${BLUE}Report Directory:${NC} $REPORT_DIR"
    echo -e "${BLUE}Timestamp:${NC} $TIMESTAMP"
    
    if [[ -d "$REPORT_DIR" ]]; then
        local report_count
        report_count=$(find "$REPORT_DIR" -name "*${TIMESTAMP}*" -type f | wc -l)
        echo -e "${BLUE}Generated Files:${NC} $report_count"
    fi
    
    echo ""
    echo -e "${BLUE}Next Steps:${NC}"
    echo "  • Review detailed reports in $REPORT_DIR"
    echo "  • Run 'pnpm run security:audit:fix' to auto-fix dependency issues"
    echo "  • Run 'pnpm run security:scan --fix' to auto-fix code issues"
    echo "  • Integrate security checks into your CI/CD pipeline"
    echo ""
    echo -e "${PURPLE}=================================================================================${NC}"
}

# Main execution
main() {
    local start_time
    start_time=$(date +%s)
    
    echo -e "${PURPLE}🛡️  ArbEdge Security Check${NC}"
    echo -e "${BLUE}Starting comprehensive security analysis...${NC}"
    echo ""
    
    # Ensure we're in the root directory
    cd "$ROOT_DIR"
    
    # Create reports directory
    mkdir -p "$REPORT_DIR"
    
    # Set up cleanup trap
    trap cleanup EXIT
    
    local exit_code=0
    
    # Run security checks
    check_prerequisites
    ensure_dependencies
    
    # Run audit (don't fail immediately)
    if ! run_audit; then
        exit_code=1
    fi
    
    # Run scan (don't fail immediately)
    if ! run_scan; then
        exit_code=1
    fi
    
    # Generate report
    generate_report
    
    # Analyze results and potentially fail
    if ! analyze_results; then
        exit_code=1
    fi
    
    # Display summary
    display_summary
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo -e "${BLUE}Security check completed in ${duration}s${NC}"
    
    if [[ $exit_code -eq 0 ]]; then
        log_success "Security check passed!"
    else
        log_warning "Security check completed with issues. Review the reports for details."
    fi
    
    exit $exit_code
}

# Run main function
main "$@"