#!/bin/bash
# Development startup script for ArbEdge monorepo
# Usage: ./start-dev.sh [package-name] or ./start-dev.sh all

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}🚀 ArbEdge Development Server${NC}"
    echo -e "${BLUE}==============================${NC}"
}

print_usage() {
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  $0 all                 # Start all development servers"
    echo -e "  $0 worker              # Start worker development server"
    echo -e "  $0 web                 # Start web development server"
    echo -e "  $0 telegram-bot        # Start telegram bot development server"
    echo -e "  $0 db                  # Start database development tools"
    echo -e "\n${YELLOW}Available packages:${NC}"
    echo -e "  - worker: Cloudflare Worker (Hono + TypeScript)"
    echo -e "  - web: Astro web application"
    echo -e "  - telegram-bot: Telegram bot service"
    echo -e "  - db: Database development tools"
}

start_worker() {
    echo -e "${GREEN}⚡ Starting Worker development server...${NC}"
    cd packages/worker
    pnpm run dev
}

start_web() {
    echo -e "${GREEN}🌐 Starting Web development server...${NC}"
    cd packages/web
    pnpm run dev
}

start_telegram_bot() {
    echo -e "${GREEN}🤖 Starting Telegram Bot development server...${NC}"
    cd packages/telegram-bot
    pnpm run dev
}

start_db() {
    echo -e "${GREEN}🗄️ Starting Database development tools...${NC}"
    cd packages/db
    echo -e "${BLUE}Available commands:${NC}"
    echo -e "  - pnpm run db:generate  # Generate migrations"
    echo -e "  - pnpm run db:migrate   # Run migrations"
    echo -e "  - pnpm run db:studio    # Open Drizzle Studio"
    echo -e "\n${YELLOW}Starting Drizzle Studio...${NC}"
    pnpm run db:studio
}

start_all() {
    echo -e "${GREEN}🚀 Starting all development servers...${NC}"
    echo -e "${YELLOW}Note: This will start multiple processes. Use Ctrl+C to stop all.${NC}"
    
    # Use concurrently to run multiple dev servers
    if command -v concurrently &> /dev/null; then
        concurrently \
            --names "worker,web,telegram" \
            --prefix-colors "blue,green,yellow" \
            "cd packages/worker && pnpm run dev" \
            "cd packages/web && pnpm run dev" \
            "cd packages/telegram-bot && pnpm run dev"
    else
        echo -e "${RED}❌ concurrently not found. Install it with: npm install -g concurrently${NC}"
        echo -e "${YELLOW}Starting services sequentially instead...${NC}"
        start_worker &
        start_web &
        start_telegram_bot &
        wait
    fi
}

# Main script
print_header

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "packages" ]; then
    echo -e "${RED}❌ Please run this script from the ArbEdge root directory${NC}"
    exit 1
fi

# Parse arguments
if [ $# -eq 0 ]; then
    print_usage
    exit 1
fi

case "$1" in
    "all")
        start_all
        ;;
    "worker")
        start_worker
        ;;
    "web")
        start_web
        ;;
    "telegram-bot")
        start_telegram_bot
        ;;
    "db")
        start_db
        ;;
    "help" | "-h" | "--help")
        print_usage
        ;;
    *)
        echo -e "${RED}❌ Unknown package: $1${NC}"
        print_usage
        exit 1
        ;;
esac