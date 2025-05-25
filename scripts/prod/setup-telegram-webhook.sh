#!/usr/bin/env bash

# Setup script for Telegram Bot Webhook
set -euo pipefail

echo "🤖 Setting up Telegram Bot Webhook for ArbEdge..."

# Check if required tools are available
if ! command -v curl &> /dev/null; then
    echo "❌ Error: curl is required to set up webhook" >&2
    exit 1
fi

if ! command -v wrangler &> /dev/null; then
    echo "❌ Error: wrangler is required to get bot token" >&2
    exit 1
fi

# Get bot token from Cloudflare Secrets
echo "🔑 Retrieving bot token from Cloudflare Secrets..."
BOT_TOKEN=$(wrangler secret get TELEGRAM_BOT_TOKEN 2>/dev/null || echo "")

if [[ -z "$BOT_TOKEN" ]]; then
    echo "❌ Error: TELEGRAM_BOT_TOKEN not found in Cloudflare Secrets" >&2
    echo "💡 Please set it with: wrangler secret put TELEGRAM_BOT_TOKEN" >&2
    exit 1
fi

# Worker URL
WORKER_URL="https://arb-edge.irfandimarsya.workers.dev"
WEBHOOK_URL="$WORKER_URL/webhook"

echo "📡 Setting webhook URL: $WEBHOOK_URL"

# Set webhook
RESPONSE=$(curl -s -X POST "https://api.telegram.org/bot$BOT_TOKEN/setWebhook" \
    -H "Content-Type: application/json" \
    -d "{\"url\": \"$WEBHOOK_URL\"}")

# Check response
if echo "$RESPONSE" | grep -q '"ok":true'; then
    echo "✅ Webhook set successfully!"
    echo "📋 Response: $RESPONSE"
else
    echo "❌ Failed to set webhook"
    echo "📋 Response: $RESPONSE"
    exit 1
fi

# Get webhook info to verify
echo "🔍 Verifying webhook setup..."
WEBHOOK_INFO=$(curl -s "https://api.telegram.org/bot$BOT_TOKEN/getWebhookInfo")
echo "📋 Webhook Info: $WEBHOOK_INFO"

echo "✅ Telegram webhook setup completed!"
echo "💡 Test the bot by sending /start to @your_bot_username" 