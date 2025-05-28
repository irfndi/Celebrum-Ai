#!/bin/bash

# Test script for Telegram webhook functionality
# Tests the service injection fix

echo "🧪 Testing Telegram Bot Service Injection Fix"
echo "=============================================="

# Local development server URL
LOCAL_URL="http://localhost:8787/telegram/webhook"

# Test 1: /status command
echo ""
echo "📊 Test 1: Testing /status command (should show services as online)"
echo "-------------------------------------------------------------------"

STATUS_PAYLOAD='{
  "update_id": 123456789,
  "message": {
    "message_id": 1,
    "from": {
      "id": 123456789,
      "is_bot": false,
      "first_name": "Test",
      "username": "testuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "Test",
      "username": "testuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/status"
  }
}'

echo "Sending /status command..."
curl -X POST "$LOCAL_URL" \
  -H "Content-Type: application/json" \
  -d "$STATUS_PAYLOAD" \
  --silent --show-error | jq '.' || echo "Response received (not JSON)"

echo ""
echo "🔍 Test 2: Testing /opportunities command (should show real data)"
echo "----------------------------------------------------------------"

OPPORTUNITIES_PAYLOAD='{
  "update_id": 123456790,
  "message": {
    "message_id": 2,
    "from": {
      "id": 123456789,
      "is_bot": false,
      "first_name": "Test",
      "username": "testuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "Test",
      "username": "testuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/opportunities"
  }
}'

echo "Sending /opportunities command..."
curl -X POST "$LOCAL_URL" \
  -H "Content-Type: application/json" \
  -d "$OPPORTUNITIES_PAYLOAD" \
  --silent --show-error | jq '.' || echo "Response received (not JSON)"

echo ""
echo "🤖 Test 3: Testing /ai_insights command (should show real AI analysis)"
echo "---------------------------------------------------------------------"

AI_INSIGHTS_PAYLOAD='{
  "update_id": 123456791,
  "message": {
    "message_id": 3,
    "from": {
      "id": 123456789,
      "is_bot": false,
      "first_name": "Test",
      "username": "testuser"
    },
    "chat": {
      "id": 123456789,
      "first_name": "Test",
      "username": "testuser",
      "type": "private"
    },
    "date": 1640995200,
    "text": "/ai_insights"
  }
}'

echo "Sending /ai_insights command..."
curl -X POST "$LOCAL_URL" \
  -H "Content-Type: application/json" \
  -d "$AI_INSIGHTS_PAYLOAD" \
  --silent --show-error | jq '.' || echo "Response received (not JSON)"

echo ""
echo "✅ Testing completed!"
echo "===================="
echo ""
echo "Expected Results:"
echo "- /status should show services as '🟢 Online' instead of '🔴 Offline'"
echo "- /opportunities should show real opportunity data instead of mock examples"
echo "- /ai_insights should show real AI analysis instead of fallback messages"
echo ""
echo "If services still show as offline, check environment variables and service initialization." 