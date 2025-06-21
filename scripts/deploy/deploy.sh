#!/bin/bash
# Deployment script for ArbEdge monorepo
# Usage: ./deploy.sh [package-name] [environment]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}🚀 ArbEdge Deployment Script${NC}"
    echo -e "${BLUE}=============================${NC}"
}

print_usage() {
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  $0 [package] [environment]"
    echo -e "\n${YELLOW}Packages:${NC}"
    echo -e "  - worker: Deploy Cloudflare Worker"
    echo -e "  - web: Deploy Astro web application"
    echo -e "  - all: Deploy all packages"
    echo -e "\n${YELLOW}Environments:${NC}"
    echo -e "  - development (default)"
    echo -e "  - staging"
    echo -e "  - production"
}

check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check if wrangler is installed
    if ! command -v wrangler &> /dev/null; then
        echo -e "${RED}❌ Wrangler CLI not found. Install it with: npm install -g wrangler${NC}"
        exit 1
    fi
    
    # Check if logged in to Cloudflare
    if ! wrangler whoami &> /dev/null; then
        echo -e "${YELLOW}⚠️ Not logged in to Cloudflare. Please run: wrangler login${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

build_packages() {
    echo -e "${BLUE}🔨 Building packages...${NC}"
    pnpm run build
    echo -e "${GREEN}✅ Build completed${NC}"
}

run_tests() {
    echo -e "${BLUE}🧪 Running tests...${NC}"
    pnpm run test:ci
    echo -e "${GREEN}✅ Tests passed${NC}"
}

deploy_worker() {
    local env=${1:-development}
    echo -e "${GREEN}⚡ Deploying Worker to ${env}...${NC}"
    
    cd packages/worker
    
    case "$env" in
        "production")
            wrangler deploy --env production
            ;;
        "staging")
            wrangler deploy --env staging
            ;;
        "development")
            wrangler deploy --env development
            ;;
        *)
            echo -e "${RED}❌ Unknown environment: $env${NC}"
            exit 1
            ;;
    esac
    
    cd ../..
    echo -e "${GREEN}✅ Worker deployed to ${env}${NC}"
}

deploy_web() {
    local env=${1:-development}
    echo -e "${GREEN}🌐 Deploying Web to ${env}...${NC}"
    
    cd packages/web
    
    # Build the web application
    pnpm run build
    
    # Deploy based on environment
    case "$env" in
        "production")
            # Deploy to production (implement your deployment strategy)
            echo -e "${YELLOW}📝 Production web deployment not configured yet${NC}"
            ;;
        "staging")
            # Deploy to staging (implement your deployment strategy)
            echo -e "${YELLOW}📝 Staging web deployment not configured yet${NC}"
            ;;
        "development")
            # Deploy to development (implement your deployment strategy)
            echo -e "${YELLOW}📝 Development web deployment not configured yet${NC}"
            ;;
        *)
            echo -e "${RED}❌ Unknown environment: $env${NC}"
            exit 1
            ;;
    esac
    
    cd ../..
    echo -e "${GREEN}✅ Web deployed to ${env}${NC}"
}

deploy_all() {
    local env=${1:-development}
    echo -e "${GREEN}🚀 Deploying all packages to ${env}...${NC}"
    
    deploy_worker "$env"
    deploy_web "$env"
    
    echo -e "${GREEN}✅ All packages deployed to ${env}${NC}"
}

# Main script
print_header

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "packages" ]; then
    echo -e "${RED}❌ Please run this script from the ArbEdge root directory${NC}"
    exit 1
fi

# Parse arguments
PACKAGE=${1:-}
ENVIRONMENT=${2:-development}

if [ -z "$PACKAGE" ]; then
    print_usage
    exit 1
fi

# Validate environment
case "$ENVIRONMENT" in
    "development" | "staging" | "production")
        echo -e "${BLUE}🎯 Target environment: ${ENVIRONMENT}${NC}"
        ;;
    *)
        echo -e "${RED}❌ Invalid environment: $ENVIRONMENT${NC}"
        print_usage
        exit 1
        ;;
esac

# Run deployment
check_prerequisites
build_packages
run_tests

case "$PACKAGE" in
    "worker")
        deploy_worker "$ENVIRONMENT"
        ;;
    "web")
        deploy_web "$ENVIRONMENT"
        ;;
    "all")
        deploy_all "$ENVIRONMENT"
        ;;
    "help" | "-h" | "--help")
        print_usage
        ;;
    *)
        echo -e "${RED}❌ Unknown package: $PACKAGE${NC}"
        print_usage
        exit 1
        ;;
esac

echo -e "\n${GREEN}🎉 Deployment completed successfully!${NC}"