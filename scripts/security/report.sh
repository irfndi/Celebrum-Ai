#!/bin/bash

# Security Report Generator for ArbEdge Monorepo
# Generates comprehensive security reports from audit and scan results

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
OUTPUT_FORMAT="html"
OPEN_REPORT=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --format)
            OUTPUT_FORMAT="$2"
            shift 2
            ;;
        --open)
            OPEN_REPORT=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--format html|json|text] [--open] [--help]"
            echo "  --format   Output format (html, json, text) - default: html"
            echo "  --open     Open the report in default browser (HTML only)"
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

# Function to get the latest report files
get_latest_reports() {
    local report_type="$1"
    find "$REPORT_DIR" -name "*${report_type}*" -type f | sort -r | head -10
}

# Function to parse JSON audit reports
parse_audit_reports() {
    local reports=()
    mapfile -t reports < <(get_latest_reports "npm-audit")
    
    local total_vulnerabilities=0
    local critical_count=0
    local high_count=0
    local moderate_count=0
    local low_count=0
    local packages_with_issues=()
    
    for report in "${reports[@]}"; do
        if [[ -f "$report" && -s "$report" ]]; then
            local package_name
            package_name=$(basename "$report" | sed 's/npm-audit-\(.*\)-[0-9]*.json/\1/')
            
            # Parse JSON if it contains actual vulnerabilities
            if jq -e '.vulnerabilities' "$report" >/dev/null 2>&1; then
                local vuln_count
                vuln_count=$(jq '.metadata.vulnerabilities.total // 0' "$report" 2>/dev/null || echo "0")
                
                if [[ "$vuln_count" -gt 0 ]]; then
                    packages_with_issues+=("$package_name")
                    total_vulnerabilities=$((total_vulnerabilities + vuln_count))
                    
                    # Count by severity
                    critical_count=$((critical_count + $(jq '.metadata.vulnerabilities.critical // 0' "$report" 2>/dev/null || echo "0")))
                    high_count=$((high_count + $(jq '.metadata.vulnerabilities.high // 0' "$report" 2>/dev/null || echo "0")))
                    moderate_count=$((moderate_count + $(jq '.metadata.vulnerabilities.moderate // 0' "$report" 2>/dev/null || echo "0")))
                    low_count=$((low_count + $(jq '.metadata.vulnerabilities.low // 0' "$report" 2>/dev/null || echo "0")))
                fi
            fi
        fi
    done
    
    # Output results
    echo "AUDIT_TOTAL:$total_vulnerabilities"
    echo "AUDIT_CRITICAL:$critical_count"
    echo "AUDIT_HIGH:$high_count"
    echo "AUDIT_MODERATE:$moderate_count"
    echo "AUDIT_LOW:$low_count"
    echo "AUDIT_PACKAGES:${packages_with_issues[*]}"
}

# Function to parse security scan reports
parse_scan_reports() {
    local reports=()
    mapfile -t reports < <(get_latest_reports "security-scan")
    
    local total_issues=0
    local error_count=0
    local warning_count=0
    local packages_with_issues=()
    
    for report in "${reports[@]}"; do
        if [[ -f "$report" && -s "$report" ]]; then
            local package_name
            package_name=$(basename "$report" | sed 's/security-scan-\(.*\)-[0-9]*.json/\1/')
            
            # Parse JSON ESLint output
            if jq -e '.[0]' "$report" >/dev/null 2>&1; then
                local issue_count
                issue_count=$(jq '[.[].messages | length] | add // 0' "$report" 2>/dev/null || echo "0")
                
                if [[ "$issue_count" -gt 0 ]]; then
                    packages_with_issues+=("$package_name")
                    total_issues=$((total_issues + issue_count))
                    
                    # Count by severity
                    error_count=$((error_count + $(jq '[.[].messages[] | select(.severity == 2)] | length' "$report" 2>/dev/null || echo "0")))
                    warning_count=$((warning_count + $(jq '[.[].messages[] | select(.severity == 1)] | length' "$report" 2>/dev/null || echo "0")))
                fi
            fi
        fi
    done
    
    # Output results
    echo "SCAN_TOTAL:$total_issues"
    echo "SCAN_ERRORS:$error_count"
    echo "SCAN_WARNINGS:$warning_count"
    echo "SCAN_PACKAGES:${packages_with_issues[*]}"
}

# Function to generate HTML report
generate_html_report() {
    local output_file="$REPORT_DIR/security-report-${TIMESTAMP}.html"
    
    log_info "Generating HTML security report..."
    
    # Parse audit and scan results
    local audit_results
    local scan_results
    audit_results=$(parse_audit_reports)
    scan_results=$(parse_scan_reports)
    
    # Extract values
    local audit_total audit_critical audit_high audit_moderate audit_low audit_packages
    local scan_total scan_errors scan_warnings scan_packages
    
    eval "$(echo "$audit_results" | sed 's/^/local /' | tr ':' '=')"
    eval "$(echo "$scan_results" | sed 's/^/local /' | tr ':' '=')"
    
    # Generate HTML
    cat > "$output_file" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ArbEdge Security Report - $(date)</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }
        .header h1 {
            margin: 0;
            font-size: 2.5em;
        }
        .header p {
            margin: 10px 0 0 0;
            opacity: 0.9;
        }
        .content {
            padding: 30px;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .card {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            border-left: 4px solid #007bff;
        }
        .card.critical { border-left-color: #dc3545; }
        .card.warning { border-left-color: #ffc107; }
        .card.success { border-left-color: #28a745; }
        .card h3 {
            margin: 0 0 10px 0;
            color: #333;
        }
        .card .number {
            font-size: 2em;
            font-weight: bold;
            color: #007bff;
        }
        .card.critical .number { color: #dc3545; }
        .card.warning .number { color: #ffc107; }
        .card.success .number { color: #28a745; }
        .section {
            margin-bottom: 30px;
        }
        .section h2 {
            color: #333;
            border-bottom: 2px solid #eee;
            padding-bottom: 10px;
        }
        .vulnerability-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 15px;
        }
        .vulnerability-table th,
        .vulnerability-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        .vulnerability-table th {
            background-color: #f8f9fa;
            font-weight: 600;
        }
        .severity {
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.85em;
            font-weight: bold;
        }
        .severity.critical {
            background-color: #dc3545;
            color: white;
        }
        .severity.high {
            background-color: #fd7e14;
            color: white;
        }
        .severity.moderate {
            background-color: #ffc107;
            color: #212529;
        }
        .severity.low {
            background-color: #6c757d;
            color: white;
        }
        .footer {
            background-color: #f8f9fa;
            padding: 20px;
            text-align: center;
            color: #6c757d;
            border-top: 1px solid #eee;
        }
        .status-indicator {
            display: inline-block;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            margin-right: 8px;
        }
        .status-indicator.safe { background-color: #28a745; }
        .status-indicator.warning { background-color: #ffc107; }
        .status-indicator.danger { background-color: #dc3545; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🛡️ ArbEdge Security Report</h1>
            <p>Generated on $(date '+%B %d, %Y at %I:%M %p')</p>
        </div>
        
        <div class="content">
            <div class="summary">
                <div class="card $([ "${AUDIT_TOTAL:-0}" -eq 0 ] && echo "success" || echo "critical")">
                    <h3>Dependency Vulnerabilities</h3>
                    <div class="number">${AUDIT_TOTAL:-0}</div>
                    <p>Total vulnerabilities found in dependencies</p>
                </div>
                
                <div class="card $([ "${SCAN_TOTAL:-0}" -eq 0 ] && echo "success" || echo "warning")">
                    <h3>Code Security Issues</h3>
                    <div class="number">${SCAN_TOTAL:-0}</div>
                    <p>Security issues found in code analysis</p>
                </div>
                
                <div class="card $([ "${AUDIT_CRITICAL:-0}" -eq 0 ] && echo "success" || echo "critical")">
                    <h3>Critical Issues</h3>
                    <div class="number">${AUDIT_CRITICAL:-0}</div>
                    <p>Require immediate attention</p>
                </div>
                
                <div class="card">
                    <h3>Overall Status</h3>
                    <div class="number">
                        $(if [ "${AUDIT_TOTAL:-0}" -eq 0 ] && [ "${SCAN_TOTAL:-0}" -eq 0 ]; then
                            echo '<span class="status-indicator safe"></span>SECURE'
                        elif [ "${AUDIT_CRITICAL:-0}" -gt 0 ]; then
                            echo '<span class="status-indicator danger"></span>CRITICAL'
                        else
                            echo '<span class="status-indicator warning"></span>REVIEW'
                        fi)
                    </div>
                </div>
            </div>
            
            <div class="section">
                <h2>📊 Vulnerability Breakdown</h2>
                <table class="vulnerability-table">
                    <thead>
                        <tr>
                            <th>Severity</th>
                            <th>Count</th>
                            <th>Description</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><span class="severity critical">CRITICAL</span></td>
                            <td>${AUDIT_CRITICAL:-0}</td>
                            <td>Vulnerabilities that require immediate attention</td>
                        </tr>
                        <tr>
                            <td><span class="severity high">HIGH</span></td>
                            <td>${AUDIT_HIGH:-0}</td>
                            <td>Serious vulnerabilities that should be addressed soon</td>
                        </tr>
                        <tr>
                            <td><span class="severity moderate">MODERATE</span></td>
                            <td>${AUDIT_MODERATE:-0}</td>
                            <td>Moderate risk vulnerabilities</td>
                        </tr>
                        <tr>
                            <td><span class="severity low">LOW</span></td>
                            <td>${AUDIT_LOW:-0}</td>
                            <td>Low risk vulnerabilities</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            
            <div class="section">
                <h2>🔍 Code Analysis Results</h2>
                <table class="vulnerability-table">
                    <thead>
                        <tr>
                            <th>Type</th>
                            <th>Count</th>
                            <th>Description</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><span class="severity critical">ERRORS</span></td>
                            <td>${SCAN_ERRORS:-0}</td>
                            <td>Security errors that must be fixed</td>
                        </tr>
                        <tr>
                            <td><span class="severity moderate">WARNINGS</span></td>
                            <td>${SCAN_WARNINGS:-0}</td>
                            <td>Security warnings that should be reviewed</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            
            <div class="section">
                <h2>📦 Affected Packages</h2>
                <p><strong>Packages with dependency vulnerabilities:</strong> ${AUDIT_PACKAGES:-"None"}</p>
                <p><strong>Packages with code security issues:</strong> ${SCAN_PACKAGES:-"None"}</p>
            </div>
            
            <div class="section">
                <h2>🔧 Recommendations</h2>
                <ul>
                    $(if [ "${AUDIT_TOTAL:-0}" -gt 0 ]; then
                        echo "<li>Run <code>pnpm run security:audit:fix</code> to automatically fix dependency vulnerabilities</li>"
                    fi)
                    $(if [ "${SCAN_TOTAL:-0}" -gt 0 ]; then
                        echo "<li>Review and fix code security issues identified in the scan reports</li>"
                    fi)
                    <li>Regularly update dependencies to their latest secure versions</li>
                    <li>Enable automated security scanning in your CI/CD pipeline</li>
                    <li>Consider implementing additional security tools like Snyk or GitHub Security Advisories</li>
                </ul>
            </div>
        </div>
        
        <div class="footer">
            <p>Report generated by ArbEdge Security Audit System</p>
            <p>For detailed information, check individual report files in <code>.security-reports/</code></p>
        </div>
    </div>
</body>
</html>
EOF
    
    log_success "HTML report generated: $output_file"
    
    if [[ "$OPEN_REPORT" == "true" ]]; then
        if command -v open &> /dev/null; then
            open "$output_file"
        elif command -v xdg-open &> /dev/null; then
            xdg-open "$output_file"
        else
            log_info "Cannot open report automatically. Please open: $output_file"
        fi
    fi
    
    echo "$output_file"
}

# Function to generate JSON report
generate_json_report() {
    local output_file="$REPORT_DIR/security-report-${TIMESTAMP}.json"
    
    log_info "Generating JSON security report..."
    
    # Parse audit and scan results
    local audit_results
    local scan_results
    audit_results=$(parse_audit_reports)
    scan_results=$(parse_scan_reports)
    
    # Create JSON structure
    cat > "$output_file" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "report_version": "1.0",
  "project": "ArbEdge",
  "summary": {
EOF
    
    # Add audit results
    echo "$audit_results" | while IFS=':' read -r key value; do
        case $key in
            AUDIT_TOTAL) echo "    \"total_vulnerabilities\": $value," >> "$output_file" ;;
            AUDIT_CRITICAL) echo "    \"critical_vulnerabilities\": $value," >> "$output_file" ;;
            AUDIT_HIGH) echo "    \"high_vulnerabilities\": $value," >> "$output_file" ;;
            AUDIT_MODERATE) echo "    \"moderate_vulnerabilities\": $value," >> "$output_file" ;;
            AUDIT_LOW) echo "    \"low_vulnerabilities\": $value," >> "$output_file" ;;
        esac
    done
    
    # Add scan results
    echo "$scan_results" | while IFS=':' read -r key value; do
        case $key in
            SCAN_TOTAL) echo "    \"total_code_issues\": $value," >> "$output_file" ;;
            SCAN_ERRORS) echo "    \"code_errors\": $value," >> "$output_file" ;;
            SCAN_WARNINGS) echo "    \"code_warnings\": $value" >> "$output_file" ;;
        esac
    done
    
    cat >> "$output_file" << EOF
  },
  "generated_at": "$(date)",
  "report_location": "$REPORT_DIR"
}
EOF
    
    log_success "JSON report generated: $output_file"
    echo "$output_file"
}

# Function to generate text report
generate_text_report() {
    local output_file="$REPORT_DIR/security-report-${TIMESTAMP}.txt"
    
    log_info "Generating text security report..."
    
    # Parse audit and scan results
    local audit_results
    local scan_results
    audit_results=$(parse_audit_reports)
    scan_results=$(parse_scan_reports)
    
    cat > "$output_file" << EOF
================================================================================
                        ARBEDGE SECURITY REPORT
================================================================================

Generated: $(date)
Report Directory: $REPORT_DIR

================================================================================
                              SUMMARY
================================================================================

DEPENDENCY VULNERABILITIES:
EOF
    
    echo "$audit_results" | while IFS=':' read -r key value; do
        case $key in
            AUDIT_TOTAL) echo "  Total: $value" >> "$output_file" ;;
            AUDIT_CRITICAL) echo "  Critical: $value" >> "$output_file" ;;
            AUDIT_HIGH) echo "  High: $value" >> "$output_file" ;;
            AUDIT_MODERATE) echo "  Moderate: $value" >> "$output_file" ;;
            AUDIT_LOW) echo "  Low: $value" >> "$output_file" ;;
            AUDIT_PACKAGES) echo "  Affected Packages: $value" >> "$output_file" ;;
        esac
    done
    
    cat >> "$output_file" << EOF

CODE SECURITY ISSUES:
EOF
    
    echo "$scan_results" | while IFS=':' read -r key value; do
        case $key in
            SCAN_TOTAL) echo "  Total: $value" >> "$output_file" ;;
            SCAN_ERRORS) echo "  Errors: $value" >> "$output_file" ;;
            SCAN_WARNINGS) echo "  Warnings: $value" >> "$output_file" ;;
            SCAN_PACKAGES) echo "  Affected Packages: $value" >> "$output_file" ;;
        esac
    done
    
    cat >> "$output_file" << EOF

================================================================================
                            RECOMMENDATIONS
================================================================================

1. Run 'pnpm run security:audit:fix' to automatically fix dependency vulnerabilities
2. Review and fix code security issues identified in the scan reports
3. Regularly update dependencies to their latest secure versions
4. Enable automated security scanning in your CI/CD pipeline
5. Consider implementing additional security tools like Snyk or GitHub Security Advisories

================================================================================
                              REPORT FILES
================================================================================

Detailed reports can be found in: $REPORT_DIR

$(ls -la "$REPORT_DIR" | tail -10)

================================================================================
EOF
    
    log_success "Text report generated: $output_file"
    echo "$output_file"
}

# Main execution
main() {
    log_info "Generating security report in $OUTPUT_FORMAT format..."
    
    # Ensure we're in the root directory
    cd "$ROOT_DIR"
    
    # Check if jq is available for JSON parsing
    if ! command -v jq &> /dev/null; then
        log_warning "jq is not installed. Some features may not work properly."
        log_info "Install jq with: brew install jq (macOS) or apt-get install jq (Ubuntu)"
    fi
    
    # Check if report directory exists and has files
    if [[ ! -d "$REPORT_DIR" ]] || [[ -z "$(ls -A "$REPORT_DIR" 2>/dev/null)" ]]; then
        log_warning "No security reports found. Run 'pnpm run security:check' first."
        exit 1
    fi
    
    local report_file
    
    case $OUTPUT_FORMAT in
        html)
            report_file=$(generate_html_report)
            ;;
        json)
            report_file=$(generate_json_report)
            ;;
        text)
            report_file=$(generate_text_report)
            ;;
        *)
            log_error "Unsupported format: $OUTPUT_FORMAT"
            log_info "Supported formats: html, json, text"
            exit 1
            ;;
    esac
    
    log_success "Security report generated successfully!"
    log_info "Report file: $report_file"
}

# Run main function
main "$@"