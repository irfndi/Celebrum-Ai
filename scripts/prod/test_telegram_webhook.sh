#!/bin/bash

# Test script for PRODUCTION Telegram webhook functionality
# Tests the service injection and live data responses

echo "🧪 Testing PRODUCTION Telegram Bot"
echo "=============================================="

# IMPORTANT: Set your production webhook URL here
# For example: PROD_URL="https://your-worker.your-domain.workers.dev/telegram/webhook"
PROD_URL="${TELEGRAM_PROD_WEBHOOK_URL}"

if [ -z "$PROD_URL" ]; then
    echo "🛑 Error: TELEGRAM_PROD_WEBHOOK_URL environment variable is not set."
    echo "Please set it to your production webhook URL before running this script."
    echo "Example: export TELEGRAM_PROD_WEBHOOK_URL='https://your-worker.workers.dev/telegram/webhook'"
    exit 1
fi

echo "🎯 Target URL: $PROD_URL"

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
      "first_name": "ProdTest",
      "username": "prodtestuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "ProdTest",
      "username": "prodtestuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/help"
  }
}'

echo "Sending /help command..."
RESPONSE=$(curl -X POST "$PROD_URL" \
  -H "Content-Type: application/json" \
  -d "$HELP_PAYLOAD" \
  --silent --show-error)

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
      "first_name": "ProdTest",
      "username": "prodtestuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "ProdTest",
      "username": "prodtestuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/opportunities"
  }
}'

echo "Sending /opportunities command..."
RESPONSE=$(curl -X POST "$PROD_URL" \
  -H "Content-Type: application/json" \
  -d "$OPPORTUNITIES_PAYLOAD" \
  --silent --show-error)

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
      "first_name": "ProdTest",
      "username": "prodtestuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "ProdTest",
      "username": "prodtestuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/profile"
  }
}'

echo "Sending /profile command..."
RESPONSE=$(curl -X POST "$PROD_URL" \
  -H "Content-Type: application/json" \
  -d "$PROFILE_PAYLOAD" \
  --silent --show-error)

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
echo "Note: Ensure TELEGRAM_PROD_WEBHOOK_URL is set correctly."
echo "If commands don't work, the webhook handler may need to use the CommandRouter properly." 