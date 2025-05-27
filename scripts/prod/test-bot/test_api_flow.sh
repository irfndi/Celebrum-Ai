#!/bin/bash

# ArbEdge API Flow Test Script
# Tests the complete user journey including RBAC & Subscription logic
# This ensures Telegram bot UX matches our intentions without manual testing

set -e

# Configuration
BASE_URL="${BASE_URL:-https://arb-edge.your-domain.workers.dev}"
TEST_OUTPUT_DIR="./test_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_FILE="$TEST_OUTPUT_DIR/api_test_$TIMESTAMP.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Create test output directory
mkdir -p "$TEST_OUTPUT_DIR"

# Logging function
log() {
    echo -e "$1" | tee -a "$LOG_FILE"
}

# Test function
run_test() {
    local test_name="$1"
    local expected_status="$2"
    local curl_command="$3"
    local validation_function="$4"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    log "${BLUE}[TEST $TOTAL_TESTS] $test_name${NC}"
    
    # Execute curl command and capture response
    local response_file="$TEST_OUTPUT_DIR/response_$TOTAL_TESTS.json"
    local status_code
    
    status_code=$(eval "$curl_command" \
        -w "%{http_code}" \
        -s \
        -o "$response_file" \
        2>/dev/null || echo "000")
    
    # Check status code
    if [ "$status_code" = "$expected_status" ]; then
        log "${GREEN}✅ Status Code: $status_code (Expected: $expected_status)${NC}"
        
        # Run validation function if provided
        if [ -n "$validation_function" ] && command -v "$validation_function" >/dev/null; then
            if $validation_function "$response_file"; then
                log "${GREEN}✅ Response Validation: PASSED${NC}"
                PASSED_TESTS=$((PASSED_TESTS + 1))
            else
                log "${RED}❌ Response Validation: FAILED${NC}"
                FAILED_TESTS=$((FAILED_TESTS + 1))
            fi
        else
            PASSED_TESTS=$((PASSED_TESTS + 1))
        fi
    else
        log "${RED}❌ Status Code: $status_code (Expected: $expected_status)${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    # Show response preview
    if [ -f "$response_file" ]; then
        log "${YELLOW}Response Preview:${NC}"
        head -c 200 "$response_file" | tee -a "$LOG_FILE"
        echo "" | tee -a "$LOG_FILE"
    fi
    
    log "----------------------------------------"
}

# Validation functions
validate_health_response() {
    local response_file="$1"
    jq -e '.success == true and .data.status == "healthy"' "$response_file" >/dev/null
}

validate_error_response() {
    local response_file="$1"
    jq -e '.success == false and .error != null' "$response_file" >/dev/null
}

validate_user_profile() {
    local response_file="$1"
    jq -e '.success == true and .data.user_id != null' "$response_file" >/dev/null
}

validate_opportunities_list() {
    local response_file="$1"
    jq -e '.success == true and (.data | type) == "array"' "$response_file" >/dev/null
}

validate_analytics_data() {
    local response_file="$1"
    jq -e '.success == true and .data.active_users != null' "$response_file" >/dev/null
}

# Test user credentials for different subscription tiers
# Using regular variables instead of associative arrays for compatibility
FREE_USER="user_free_123"
PREMIUM_USER="user_premium_456"
PRO_USER="user_pro_789"
ADMIN_USER="user_admin_000"

# Start testing
log "${BLUE}🚀 Starting ArbEdge API Flow Tests${NC}"
log "${BLUE}Base URL: $BASE_URL${NC}"
log "${BLUE}Timestamp: $TIMESTAMP${NC}"
log "========================================"

# 1. Health Check Tests
log "${YELLOW}📋 HEALTH CHECK TESTS${NC}"

run_test "Basic Health Check" "200" \
    "curl -X GET '$BASE_URL/api/v1/health'" \
    "validate_health_response"

run_test "Detailed Health Check" "200" \
    "curl -X GET '$BASE_URL/api/v1/health/detailed'" \
    "validate_health_response"

# 2. Authentication Tests
log "${YELLOW}🔐 AUTHENTICATION TESTS${NC}"

# Test authentication without credentials
run_test "Unauthorized Access - No Headers" "401" \
    "curl -X GET '$BASE_URL/api/v1/users/profile'" \
    "validate_error_response"

# Test with invalid user ID
run_test "Unauthorized Access - Invalid User" "401" \
    "curl -X GET '$BASE_URL/api/v1/users/profile' -H 'X-User-ID: invalid_user'" \
    "validate_error_response"

# 3. User Profile Tests (RBAC Testing)
log "${YELLOW}👤 USER PROFILE & RBAC TESTS${NC}"

# Test each user tier
for tier_info in "free_user:$FREE_USER" "premium_user:$PREMIUM_USER" "pro_user:$PRO_USER" "admin_user:$ADMIN_USER"; do
    tier=$(echo "$tier_info" | cut -d: -f1)
    user_id=$(echo "$tier_info" | cut -d: -f2)
    
    log "${BLUE}Testing $tier ($user_id)${NC}"
    
    # Get user profile
    run_test "Get Profile - $tier" "200" \
        "curl -X GET '$BASE_URL/api/v1/users/profile' -H 'X-User-ID: $user_id'" \
        "validate_user_profile"
    
    # Get user preferences
    run_test "Get Preferences - $tier" "200" \
        "curl -X GET '$BASE_URL/api/v1/users/preferences' -H 'X-User-ID: $user_id'"
    
    # Update preferences (test write permissions)
    run_test "Update Preferences - $tier" "200" \
        "curl -X PUT '$BASE_URL/api/v1/users/preferences' \
         -H 'X-User-ID: $user_id' \
         -H 'Content-Type: application/json' \
         -d '{\"risk_tolerance\": 0.5, \"preferred_pairs\": [\"BTC/USDT\"]}'"
done

# 4. Opportunity Access Tests (Subscription-based)
log "${YELLOW}💰 OPPORTUNITY ACCESS & SUBSCRIPTION TESTS${NC}"

for tier_info in "free_user:$FREE_USER" "premium_user:$PREMIUM_USER" "pro_user:$PRO_USER" "admin_user:$ADMIN_USER"; do
    tier=$(echo "$tier_info" | cut -d: -f1)
    user_id=$(echo "$tier_info" | cut -d: -f2)
    
    log "${BLUE}Testing opportunity access for $tier ($user_id)${NC}"
    
    # List opportunities (different limits based on subscription)
    run_test "List Opportunities - $tier" "200" \
        "curl -X GET '$BASE_URL/api/v1/opportunities?limit=10' -H 'X-User-ID: $user_id'" \
        "validate_opportunities_list"
    
    # Test premium features access
    if [[ "$tier" == "free_user" ]]; then
        # Free users should have limited access
        run_test "Premium Feature Access - $tier (Should Fail)" "403" \
            "curl -X GET '$BASE_URL/api/v1/opportunities?premium=true' -H 'X-User-ID: $user_id'" \
            "validate_error_response"
    else
        # Premium/Pro users should have access
        run_test "Premium Feature Access - $tier" "200" \
            "curl -X GET '$BASE_URL/api/v1/opportunities?premium=true' -H 'X-User-ID: $user_id'" \
            "validate_opportunities_list"
    fi
done

# 5. Opportunity Execution Tests
log "${YELLOW}⚡ OPPORTUNITY EXECUTION TESTS${NC}"

# Test opportunity execution with different user tiers
for tier_info in "free_user:$FREE_USER" "premium_user:$PREMIUM_USER" "pro_user:$PRO_USER" "admin_user:$ADMIN_USER"; do
    tier=$(echo "$tier_info" | cut -d: -f1)
    user_id=$(echo "$tier_info" | cut -d: -f2)
    
    log "${BLUE}Testing opportunity execution for $tier ($user_id)${NC}"
    
    # Execute opportunity (rate limits apply based on subscription)
    run_test "Execute Opportunity - $tier" "200" \
        "curl -X POST '$BASE_URL/api/v1/opportunities/execute' \
         -H 'X-User-ID: $user_id' \
         -H 'Content-Type: application/json' \
         -d '{\"opportunity_id\": \"test_opp_123\", \"amount\": 100}'"
done

# 6. Analytics Access Tests (Admin/Pro only)
log "${YELLOW}📊 ANALYTICS ACCESS TESTS${NC}"

for tier_info in "free_user:$FREE_USER" "premium_user:$PREMIUM_USER" "pro_user:$PRO_USER" "admin_user:$ADMIN_USER"; do
    tier=$(echo "$tier_info" | cut -d: -f1)
    user_id=$(echo "$tier_info" | cut -d: -f2)
    
    log "${BLUE}Testing analytics access for $tier ($user_id)${NC}"
    
    if [[ "$tier" == "admin_user" || "$tier" == "pro_user" ]]; then
        # Admin and Pro users should have analytics access
        run_test "Dashboard Analytics - $tier" "200" \
            "curl -X GET '$BASE_URL/api/v1/analytics/dashboard' -H 'X-User-ID: $user_id'" \
            "validate_analytics_data"
        
        run_test "User Analytics - $tier" "200" \
            "curl -X GET '$BASE_URL/api/v1/analytics/user' -H 'X-User-ID: $user_id'" \
            "validate_analytics_data"
    else
        # Free and Premium users should be denied
        run_test "Dashboard Analytics - $tier (Should Fail)" "403" \
            "curl -X GET '$BASE_URL/api/v1/analytics/dashboard' -H 'X-User-ID: $user_id'" \
            "validate_error_response"
    fi
done

# 7. Rate Limiting Tests
log "${YELLOW}🚦 RATE LIMITING TESTS${NC}"

# Test rate limiting for free users (should hit limits faster)
user_id="$FREE_USER"
log "${BLUE}Testing rate limits for free user${NC}"

for i in {1..15}; do
    status_code=$(curl -X GET "$BASE_URL/api/v1/opportunities" \
        -H "X-User-ID: $user_id" \
        -w "%{http_code}" \
        -s \
        -o /dev/null)
    
    if [ "$status_code" = "429" ]; then
        log "${GREEN}✅ Rate limit triggered after $i requests${NC}"
        break
    elif [ "$i" = "15" ]; then
        log "${YELLOW}⚠️ Rate limit not triggered after 15 requests${NC}"
    fi
done

# 8. Webhook Simulation Tests (Telegram Bot Flow)
log "${YELLOW}🤖 TELEGRAM BOT FLOW SIMULATION${NC}"

# Simulate Telegram webhook payloads
webhook_payloads=(
    '{"message":{"chat":{"id":123456},"from":{"id":123456,"username":"testuser"},"text":"/start"}}'
    '{"message":{"chat":{"id":123456},"from":{"id":123456,"username":"testuser"},"text":"/opportunities"}}'
    '{"message":{"chat":{"id":123456},"from":{"id":123456,"username":"testuser"},"text":"/profile"}}'
    '{"callback_query":{"id":"test_callback","from":{"id":123456},"data":"execute_opp_123"}}'
)

for i in "${!webhook_payloads[@]}"; do
    payload="${webhook_payloads[$i]}"
    
    run_test "Telegram Webhook Simulation $((i+1))" "200" \
        "curl -X POST '$BASE_URL/webhook/telegram' \
         -H 'Content-Type: application/json' \
         -d '$payload'"
done

# 9. Error Handling Tests
log "${YELLOW}🚨 ERROR HANDLING TESTS${NC}"

# Test various error scenarios
run_test "Invalid JSON Payload" "400" \
    "curl -X POST '$BASE_URL/api/v1/opportunities/execute' \
     -H 'X-User-ID: $FREE_USER' \
     -H 'Content-Type: application/json' \
     -d 'invalid json'" \
    "validate_error_response"

run_test "Missing Required Fields" "400" \
    "curl -X POST '$BASE_URL/api/v1/opportunities/execute' \
     -H 'X-User-ID: $FREE_USER' \
     -H 'Content-Type: application/json' \
     -d '{}'" \
    "validate_error_response"

run_test "Non-existent Endpoint" "404" \
    "curl -X GET '$BASE_URL/api/v1/nonexistent'" \
    "validate_error_response"

# 10. Performance Tests
log "${YELLOW}⚡ PERFORMANCE TESTS${NC}"

# Test concurrent requests
log "${BLUE}Testing concurrent requests${NC}"
start_time=$(date +%s)

for i in {1..10}; do
    curl -X GET "$BASE_URL/api/v1/health" \
        -H "X-User-ID: $PREMIUM_USER" \
        -s -o /dev/null &
done

wait
end_time=$(date +%s)
duration=$((end_time - start_time))

log "${GREEN}✅ 10 concurrent requests completed in ${duration}s${NC}"

# Test Summary
log "========================================"
log "${BLUE}📊 TEST SUMMARY${NC}"
log "${GREEN}✅ Passed: $PASSED_TESTS${NC}"
log "${RED}❌ Failed: $FAILED_TESTS${NC}"
log "${BLUE}📋 Total: $TOTAL_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    log "${GREEN}🎉 ALL TESTS PASSED! API flow validation successful.${NC}"
    exit 0
else
    log "${RED}💥 Some tests failed. Check the logs for details.${NC}"
    exit 1
fi 