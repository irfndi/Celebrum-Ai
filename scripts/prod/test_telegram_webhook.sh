#!/bin/bash

# Comprehensive Test Script for PRODUCTION Telegram Bot
# Tests all implemented commands with SuperAdmin user (1082762347)
# Covers: basic commands, opportunities, settings, trade, AI, admin commands

echo "🧪 Testing PRODUCTION Telegram Bot - Comprehensive Command Validation"
echo "=============================================================================="

# Production webhook URL
PROD_URL="${TELEGRAM_PROD_WEBHOOK_URL}"

if [ -z "$PROD_URL" ]; then
    echo "🛑 Error: TELEGRAM_PROD_WEBHOOK_URL environment variable is not set."
    echo "Please set it to your production webhook URL before running this script."
    echo "Example: export TELEGRAM_PROD_WEBHOOK_URL='https://arb-edge.irfandimarsya.workers.dev/telegram/webhook'"
    exit 1
fi

echo "🎯 Target URL: $PROD_URL"
echo "👤 Testing with SuperAdmin user: 1082762347"

# SuperAdmin user credentials for testing
SUPERADMIN_USER_ID=1082762347
USERNAME="SuperAdmin"
FIRST_NAME="SuperAdmin"

# Function to send command and show response
send_command() {
    local command="$1"
    local test_name="$2"
    local update_id="$3"
    
    echo ""
    echo "🧪 Test $update_id: $test_name"
    echo "Command: $command"
    echo "----------------------------------------"

    PAYLOAD="{
        \"update_id\": $update_id,
        \"message\": {
            \"message_id\": $update_id,
            \"from\": {
                \"id\": $SUPERADMIN_USER_ID,
                \"is_bot\": false,
                \"first_name\": \"$FIRST_NAME\",
                \"username\": \"$USERNAME\"
            },
            \"chat\": {
                \"id\": $SUPERADMIN_USER_ID,
                \"first_name\": \"$FIRST_NAME\",
                \"username\": \"$USERNAME\",
                \"type\": \"private\"
            },
            \"date\": $(date +%s),
            \"text\": \"$command\"
        }
    }"

    RESPONSE=$(curl -X POST "$PROD_URL" \
        -H "Content-Type: application/json" \
        -d "$PAYLOAD" \
        --silent --show-error --max-time 30)

    echo "Response: $RESPONSE"
    
    # Brief pause between commands
    sleep 1
}

# Test 1: Basic Commands
echo ""
echo "📚 SECTION 1: BASIC COMMANDS"
echo "============================================"

send_command "/start" "Start command (user registration/welcome)" 1001
send_command "/help" "Help command (should show all available commands)" 1002

# Test 2: Profile Commands
echo ""
echo "👤 SECTION 2: PROFILE COMMANDS"
echo "============================================"

send_command "/profile" "Profile overview" 1003
send_command "/profile view" "Profile view sub-command" 1004
send_command "/profile api" "Profile API management" 1005
send_command "/profile settings" "Profile settings" 1006
send_command "/subscription" "Subscription management" 1007

# Test 3: Opportunities Commands
echo ""
echo "💰 SECTION 3: OPPORTUNITIES COMMANDS"
echo "============================================"

send_command "/opportunities" "Opportunities overview (should show real data)" 1008
send_command "/opportunities list" "Opportunities list sub-command" 1009
send_command "/opportunities_list" "Opportunities list (clickable)" 1010
send_command "/opportunities manual" "Manual opportunities scan" 1011
send_command "/opportunities_manual" "Manual opportunities scan (clickable)" 1012
send_command "/opportunities auto" "Auto opportunities settings" 1013
send_command "/opportunities_auto" "Auto opportunities settings (clickable)" 1014

# Test 4: Settings Commands
echo ""
echo "⚙️ SECTION 4: SETTINGS COMMANDS"
echo "============================================"

send_command "/settings" "Settings overview" 1015
send_command "/settings notifications" "Notification settings" 1016
send_command "/settings_notifications" "Notification settings (clickable)" 1017
send_command "/settings trading" "Trading settings" 1018
send_command "/settings_trading" "Trading settings (clickable)" 1019
send_command "/settings alerts" "Alert settings" 1020
send_command "/settings_alerts" "Alert settings (clickable)" 1021
send_command "/settings privacy" "Privacy settings" 1022
send_command "/settings_privacy" "Privacy settings (clickable)" 1023
send_command "/settings api" "API settings" 1024
send_command "/settings_api" "API settings (clickable)" 1025

# Test 5: Trade Commands
echo ""
echo "📈 SECTION 5: TRADE COMMANDS"
echo "============================================"

send_command "/trade" "Trade overview" 1026
send_command "/trade manual" "Manual trade execution" 1027
send_command "/trade_manual" "Manual trade execution (clickable)" 1028
send_command "/trade manual binance" "Manual trade with exchange filter" 1029
send_command "/trade manual binance BTC/USDT" "Manual trade with exchange and pair" 1030
send_command "/trade auto" "Auto trading settings" 1031
send_command "/trade_auto" "Auto trading settings (clickable)" 1032
send_command "/trade status" "Trade status" 1033
send_command "/trade_status" "Trade status (clickable)" 1034

# Test 6: AI Commands
echo ""
echo "🤖 SECTION 6: AI COMMANDS (BYOK)"
echo "============================================"

send_command "/ai" "AI overview" 1035
send_command "/ai analyze" "AI market analysis" 1036
send_command "/ai_analyze" "AI market analysis (clickable)" 1037
send_command "/ai analyze BTC/USDT" "AI analysis for specific pair" 1038
send_command "/ai predict" "AI price prediction" 1039
send_command "/ai_predict" "AI price prediction (clickable)" 1040
send_command "/ai predict BTC/USDT 1h" "AI prediction with timeframe" 1041
send_command "/ai sentiment" "AI sentiment analysis" 1042
send_command "/ai_sentiment" "AI sentiment analysis (clickable)" 1043
send_command "/ai sentiment BTC/USDT" "AI sentiment for specific pair" 1044
send_command "/ai usage" "AI usage statistics" 1045
send_command "/ai_usage" "AI usage statistics (clickable)" 1046

# Test 7: Beta Commands (for SuperAdmin)
echo ""
echo "🧪 SECTION 7: BETA COMMANDS"
echo "============================================"

send_command "/beta" "Beta overview" 1055
send_command "/beta opportunities" "Beta opportunities" 1056
send_command "/beta ai" "Beta AI features" 1057
send_command "/beta analytics" "Beta analytics" 1058

# Test 8: Admin Commands (SuperAdmin only)
echo ""
echo "🔧 SECTION 8: ADMIN COMMANDS (SuperAdmin)"
echo "============================================"

send_command "/admin" "Admin overview" 1059
send_command "/admin users" "Admin user management" 1060
send_command "/admin system" "Admin system management" 1061
send_command "/admin config" "Admin configuration" 1062
send_command "/admin stats" "Admin statistics" 1063

# Test 9: Error Handling
echo ""
echo "❌ SECTION 9: ERROR HANDLING"
echo "============================================"

send_command "/nonexistent" "Unknown command (should show error message)" 1064
send_command "/opportunities invalidsubcommand" "Invalid sub-command" 1065
send_command "/settings invalidparam" "Invalid settings parameter" 1066

echo ""
echo "✅ COMPREHENSIVE TESTING COMPLETED!"
echo "===================================="
echo ""
echo "📋 Commands Tested:"
echo "   • 7 Basic & Profile commands (including clickable variants)"
echo "   • 6 Opportunities commands (space-separated + clickable)"
echo "   • 12 Settings commands (space-separated + clickable)" 
echo "   • 8 Trade commands (space-separated + clickable)"
echo "   • 12 AI commands (space-separated + clickable)"
echo "   • 4 Beta commands"
echo "   • 5 Admin commands"
echo "   • 3 Error handling tests"
echo "   Total: 57 comprehensive test cases"
echo ""
echo "🔍 Expected Results:"
echo "   ✓ All commands should return proper responses"
echo "   ✓ No service errors or 500 responses"
echo "   ✓ RBAC should show appropriate features for SuperAdmin"
echo "   ✓ Sub-commands should route correctly"
echo "   ✓ Clickable commands (with underscores) should work identically to space-separated ones"
echo "   ✓ Real opportunities data should be displayed (not placeholders)"
echo "   ✓ AI commands should show BYOK setup prompts"
echo "   ✓ Admin commands should be accessible"
echo "   ✓ Error messages should be helpful and user-friendly"
echo ""
echo "⚠️ Important Notes:"
echo "   • SuperAdmin user (1082762347) should have access to ALL features"
echo "   • Clickable commands (e.g., /opportunities_manual) are now available for better UX"
echo "   • Both space-separated (/opportunities manual) and underscore (/opportunities_manual) formats work"
echo "   • Opportunities should show real data from distribution service"
echo "   • AI features require BYOK setup"
echo "   • Trade commands should show proper UX and guidance"
echo "   • Settings should provide comprehensive configuration options"
echo ""
echo "🚀 Next Steps:"
echo "   1. Verify all responses are appropriate and error-free"
echo "   2. Check that clickable commands work as expected"
echo "   3. Ensure opportunities actually find/send real opportunities"
echo "   4. Ensure RBAC is working correctly for all user roles"
echo "   5. Validate sub-command routing is working properly"
echo "   6. Test group/channel access patterns if applicable" 