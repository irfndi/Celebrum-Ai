#!/usr/bin/env bash

# Setup script for Telegram Bot Webhook
set -euo pipefail

# Show usage if help requested
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    echo "Usage: $0 [WORKER_URL]"
    echo ""
    echo "Setup Telegram Bot Webhook for ArbEdge"
    echo ""
    echo "Arguments:"
    echo "  WORKER_URL    Worker URL (default: https://arb-edge.irfandimarsya.workers.dev)"
    echo "                Can also be set via WORKER_URL environment variable"
    echo ""
    echo "Examples:"
    echo "  $0                                              # Use default URL"
    echo "  $0 https://my-worker.example.workers.dev       # Use custom URL"
    echo "  WORKER_URL=https://staging.workers.dev $0      # Use environment variable"
    exit 0
fi

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

# Check if bot token exists in Cloudflare Secrets (without storing in variable)
echo "🔑 Verifying bot token in Cloudflare Secrets..."
if ! wrangler secret get TELEGRAM_BOT_TOKEN >/dev/null 2>&1; then
    echo "❌ Error: TELEGRAM_BOT_TOKEN not found in Cloudflare Secrets" >&2
    echo "💡 Please set it with: wrangler secret put TELEGRAM_BOT_TOKEN" >&2
    exit 1
fi

# Worker URL - accept as environment variable or script argument with default
WORKER_URL="${1:-${WORKER_URL:-https://arb-edge.irfandimarsya.workers.dev}}"
WEBHOOK_URL="$WORKER_URL/webhook"

echo "📡 Setting webhook URL: $WEBHOOK_URL"

# Set webhook (using token directly from wrangler)
if ! RESPONSE=$(curl -s --max-time 30 --connect-timeout 10 -X POST "https://api.telegram.org/bot$(wrangler secret get TELEGRAM_BOT_TOKEN)/setWebhook" \
    -H "Content-Type: application/json" \
    -d "{\"url\": \"$WEBHOOK_URL\"}"); then
    echo "❌ Failed to connect to Telegram API (timeout or network error)" >&2
    exit 1
fi

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
if ! WEBHOOK_INFO=$(curl -s --max-time 30 --connect-timeout 10 "https://api.telegram.org/bot$(wrangler secret get TELEGRAM_BOT_TOKEN)/getWebhookInfo"); then
    echo "⚠️ Warning: Failed to verify webhook setup (timeout or network error)" >&2
    echo "✅ Webhook was set, but verification failed"
else
    echo "📋 Webhook Info: $WEBHOOK_INFO"
fi

echo "✅ Telegram webhook setup completed!"
echo "💡 Test the bot by sending /start to @your_bot_username" 