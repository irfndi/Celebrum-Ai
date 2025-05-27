#!/bin/bash
# Production Deployment Script for ArbEdge
# Deploys to Cloudflare Workers with all required services

set -e

echo "🚀 Starting ArbEdge Production Deployment..."

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler CLI not found. Installing..."
    npm install -g wrangler@latest
fi

# Authenticate with Cloudflare (if not already authenticated)
echo "🔐 Checking Cloudflare authentication..."
if ! wrangler whoami &> /dev/null; then
    echo "Please authenticate with Cloudflare:"
    wrangler login
fi

# Set required secrets
echo "🔑 Setting up secrets..."
echo "Please set the following secrets:"

# Telegram Bot Token
read -s -p "Enter TELEGRAM_BOT_TOKEN: " TELEGRAM_BOT_TOKEN
echo
wrangler secret put TELEGRAM_BOT_TOKEN --env production <<< "$TELEGRAM_BOT_TOKEN"

# Cloudflare API Token
read -s -p "Enter CLOUDFLARE_API_TOKEN: " CLOUDFLARE_API_TOKEN
echo
wrangler secret put CLOUDFLARE_API_TOKEN --env production <<< "$CLOUDFLARE_API_TOKEN"

# Create KV Namespaces
echo "📦 Creating KV namespaces..."
USER_PROFILES_ID=$(wrangler kv:namespace create "USER_PROFILES" --env production | grep -o 'id = "[^"]*"' | cut -d'"' -f2)
MARKET_CACHE_ID=$(wrangler kv:namespace create "PROD_BOT_MARKET_CACHE" --env production | grep -o 'id = "[^"]*"' | cut -d'"' -f2)
SESSION_STORE_ID=$(wrangler kv:namespace create "PROD_BOT_SESSION_STORE" --env production | grep -o 'id = "[^"]*"' | cut -d'"' -f2)

echo "✅ KV Namespaces created:"
echo "  USER_PROFILES: $USER_PROFILES_ID"
echo "  PROD_BOT_MARKET_CACHE: $MARKET_CACHE_ID"
echo "  PROD_BOT_SESSION_STORE: $SESSION_STORE_ID"

# Create D1 Database
echo "🗄️ Creating D1 database..."
D1_DB_ID=$(wrangler d1 create arbitrage-production --env production | grep -o 'database_id = "[^"]*"' | cut -d'"' -f2)
echo "✅ D1 Database created: $D1_DB_ID"

# Create R2 Buckets
echo "🪣 Creating R2 buckets..."
wrangler r2 bucket create arb-edge-market-data --env production
wrangler r2 bucket create arb-edge-analytics --env production
echo "✅ R2 Buckets created"

# Create Queues
echo "🚥 Creating Cloudflare Queues..."
wrangler queues create opportunity-distribution --env production
wrangler queues create user-notifications --env production
wrangler queues create analytics-events --env production
wrangler queues create dead-letter-queue --env production
echo "✅ Queues created"

# Create Pipelines
echo "🔄 Creating Cloudflare Pipelines..."
wrangler pipelines create market-data-pipeline --r2-bucket arb-edge-market-data --env production
wrangler pipelines create analytics-pipeline --r2-bucket arb-edge-analytics --env production
wrangler pipelines create audit-pipeline --r2-bucket arb-edge-analytics --env production
echo "✅ Pipelines created"

# Update wrangler.toml with actual IDs
echo "📝 Updating wrangler.toml with resource IDs..."
sed -i.bak "s/your-kv-namespace-id-here/$USER_PROFILES_ID/g" wrangler.toml
sed -i.bak "s/your-market-cache-kv-id-here/$MARKET_CACHE_ID/g" wrangler.toml
sed -i.bak "s/your-session-kv-id-here/$SESSION_STORE_ID/g" wrangler.toml
sed -i.bak "s/your-d1-database-id-here/$D1_DB_ID/g" wrangler.toml

# Run D1 migrations
echo "🔄 Running D1 migrations..."
wrangler d1 migrations apply arbitrage-production --env production

# Build and deploy
echo "🔨 Building and deploying Worker..."
cargo install -q worker-build
worker-build --release

# Deploy to production
wrangler deploy --env production

echo "🎉 Deployment completed successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Update your domain DNS to point to the Worker"
echo "2. Test all endpoints"
echo "3. Monitor logs: wrangler tail --env production"
echo "4. Check analytics in Cloudflare dashboard"
echo ""
echo "🔗 Useful commands:"
echo "  View logs: wrangler tail --env production"
echo "  Update secrets: wrangler secret put SECRET_NAME --env production"
echo "  Check KV data: wrangler kv:key list --binding USER_PROFILES --env production"
echo "  Query D1: wrangler d1 execute arbitrage-production --command 'SELECT * FROM users;' --env production" 