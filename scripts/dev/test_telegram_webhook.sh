#!/bin/bash

# Test script for DEVELOPMENT Telegram webhook functionality
# Tests the service injection and command routing

echo "🧪 Testing DEVELOPMENT Telegram Bot"
echo "===================================="

# Development server URL - can be overridden via environment variable
DEV_URL="${TELEGRAM_DEV_WEBHOOK_URL:-http://localhost:8787/telegram/webhook}"

echo "🎯 Target URL: $DEV_URL"

# Validate that the URL is accessible (for localhost, check if server is running)
if [[ "$DEV_URL" == *"localhost"* ]] || [[ "$DEV_URL" == *"127.0.0.1"* ]]; then
    echo "🔍 Checking if development server is accessible..."
    if ! curl -s "$DEV_URL" -o /dev/null --connect-timeout 2; then
        echo "⚠️  Warning: Development server might not be running at $DEV_URL"
        echo "   Please ensure 'wrangler dev' is running first."
        echo ""
    fi
fi

# Test 1: /help command
echo ""
echo "📚 Test 1: Testing /help command (should show command list)"
echo "----------------------------------------------------------"

HELP_PAYLOAD='{
  "update_id": 123456789,
  "message": {
    "message_id": 1,
    "from": {
      "id": 123456789,
      "is_bot": false,
      "first_name": "DevTest",
      "username": "devtestuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "DevTest",
      "username": "devtestuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/help"
  }
}'

echo "Sending /help command..."
RESPONSE=$(curl -X POST "$DEV_URL" \
  -H "Content-Type: application/json" \
  -d "$HELP_PAYLOAD" \
  --silent --show-error --connect-timeout 10)

echo "Response: $RESPONSE"

echo ""
echo "💰 Test 2: Testing /opportunities command (should show trading opportunities)"
echo "--------------------------------------------------------------------------"

OPPORTUNITIES_PAYLOAD='{
  "update_id": 123456790,
  "message": {
    "message_id": 2,
    "from": {
      "id": 123456789,
      "is_bot": false,
      "first_name": "DevTest",
      "username": "devtestuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "DevTest",
      "username": "devtestuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/opportunities"
  }
}'

echo "Sending /opportunities command..."
RESPONSE=$(curl -X POST "$DEV_URL" \
  -H "Content-Type: application/json" \
  -d "$OPPORTUNITIES_PAYLOAD" \
  --silent --show-error --connect-timeout 10)

echo "Response: $RESPONSE"

echo ""
echo "👤 Test 3: Testing /profile command (should show user profile)"
echo "------------------------------------------------------------"

PROFILE_PAYLOAD='{
  "update_id": 123456791,
  "message": {
    "message_id": 3,
    "from": {
      "id": 123456789,
      "is_bot": false,
      "first_name": "DevTest",
      "username": "devtestuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "DevTest",
      "username": "devtestuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/profile"
  }
}'

echo "Sending /profile command..."
RESPONSE=$(curl -X POST "$DEV_URL" \
  -H "Content-Type: application/json" \
  -d "$PROFILE_PAYLOAD" \
  --silent --show-error --connect-timeout 10)

echo "Response: $RESPONSE"

echo ""
echo "✅ Testing completed!"
echo "===================="
echo ""
echo "Expected Results:"
echo "- /help should show a list of available commands"
echo "- /opportunities should show real trading opportunities or appropriate messages"
echo "- /profile should show user profile information or setup prompts"
echo ""
echo "Environment Variables:"
echo "- TELEGRAM_DEV_WEBHOOK_URL: Override the default localhost URL (optional)"
echo ""
echo "Note: Ensure development server is running with 'wrangler dev'"
echo "If commands don't work, the webhook handler may need to use the CommandRouter properly." 