# ArbEdge Worker

A Cloudflare Worker that provides intelligent route-based service separation and request routing for the ArbEdge monorepo architecture.

## Overview

The ArbEdge Worker acts as a smart gateway that routes incoming requests to appropriate services based on URL patterns, service availability, and request characteristics. It provides:

- **Route-based service separation**: Intelligent routing to web, API, bot, and admin services
- **Rate limiting**: Distributed rate limiting using Cloudflare KV
- **Health monitoring**: Comprehensive health checks for all services and dependencies
- **Error handling**: Centralized error handling with logging and monitoring
- **Security**: CORS, secure headers, and authentication middleware

## Architecture

```
┌─────────────────┐
│   Client        │
└─────────┬───────┘
          │
┌─────────▼───────┐
│  ArbEdge Worker │
│  (Route Gateway)│
└─────────┬───────┘
          │
    ┌─────┴─────┐
    │           │
┌───▼───┐   ┌───▼───┐   ┌─────────┐   ┌──────────┐
│  Web  │   │  API  │   │ Discord │   │ Telegram │
│Service│   │Service│   │   Bot   │   │   Bot    │
└───────┘   └───────┘   └─────────┘   └──────────┘
```

## Features

### 🚦 Intelligent Routing
- **Web Routes** (`/`, `/web/*`): Static assets and web pages
- **API Routes** (`/api/*`, `/v1/*`): Core API endpoints
- **Webhook Routes** (`/webhook/*`): Bot webhook handlers
- **Admin Routes** (`/admin/*`): Administrative interfaces

### 🛡️ Security & Rate Limiting
- Distributed rate limiting with KV storage
- CORS configuration for cross-origin requests
- Secure headers middleware
- Authentication checks for admin routes

### 📊 Monitoring & Health Checks
- Comprehensive health checks for all services
- Error logging and monitoring
- Performance metrics tracking
- Service availability monitoring

### 🔄 Fallback Handling
- Graceful degradation when services are unavailable
- Intelligent route detection for unknown paths
- Proper error responses with helpful suggestions

## Quick Start

### Prerequisites

- Node.js 18+
- Cloudflare account with Workers enabled
- Wrangler CLI installed globally

### Installation

```bash
# Install dependencies
pnpm install

# Build the worker
pnpm run build

# Start development server
pnpm run dev
```

### Development

```bash
# Type checking
pnpm run type-check

# Run tests
pnpm test

# Deploy to staging
wrangler deploy --env staging

# Deploy to production
wrangler deploy --env production
```

## Configuration

### Environment Variables

Configure these variables in `wrangler.toml` or Cloudflare dashboard:

```toml
[vars]
ENVIRONMENT = "production"
LOG_LEVEL = "info"
RATE_LIMIT_REQUESTS_PER_MINUTE = "60"
CACHE_TTL_SECONDS = "300"
SUPER_ADMIN_USER_ID = "admin"
EXCHANGES = "binance,coinbase,kraken"
ARBITRAGE_THRESHOLD = "0.5"
TELEGRAM_CHAT_ID = "your-telegram-chat-id"
TELEGRAM_TEST_MODE = "false"

# Service URLs for routing
WEB_SERVICE_URL = "https://web.arb-edge.com"
API_SERVICE_URL = "https://api.arb-edge.com"
DISCORD_BOT_SERVICE_URL = "https://discord-bot.arb-edge.com"
TELEGRAM_BOT_SERVICE_URL = "https://telegram-bot.arb-edge.com"
```

### KV Namespaces

Required KV namespaces:
- `ArbEdgeKV`: Main application data
- `PROD_BOT_MARKET_CACHE`: Market data caching
- `PROD_BOT_SESSION_STORE`: Session management

### D1 Database

Required D1 database binding:
- `ArbEdgeD1`: Main application database

## Routing Logic

### Web Routes
- `/` → Web service root
- `/web/*` → Web service with path preservation
- `/assets/*` → Static assets with long-term caching

### API Routes
- `/api/*` → API service with path normalization
- `/v1/*` → Legacy API support (mapped to `/api/v1/*`)

### Webhook Routes
- `/webhook/telegram/*` → Telegram bot service
- `/webhook/discord/*` → Discord bot service

### Admin Routes
- `/admin/*` → Admin service (requires authentication)

### Health Check
- `/health` → Comprehensive system health check

## Rate Limiting

The worker implements distributed rate limiting with different limits for different endpoints:

- **Health checks**: 300 requests/minute
- **API endpoints**: 60 requests/minute (configurable)
- **Webhook endpoints**: 120 requests/minute
- **Admin endpoints**: 30 requests/minute (GET), 20 requests/minute (POST)
- **Web assets**: 300 requests/minute

### Rate Limit Headers

All responses include rate limit headers:
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets

## Error Handling

The worker provides comprehensive error handling:

### Error Types
- **400**: Validation errors
- **401**: Authentication required
- **403**: Access forbidden
- **404**: Resource not found
- **408**: Request timeout
- **429**: Rate limit exceeded
- **500**: Internal server error
- **502**: Service unavailable

### Error Logging

Errors are logged to:
- Console (for immediate debugging)
- KV storage (for monitoring and analysis)
- Error index (for daily statistics)

### Error Response Format

```json
{
  "error": "Error Type",
  "message": "Human-readable error message",
  "errorId": "err_abc123_def456",
  "timestamp": "2024-01-01T00:00:00.000Z"
}
```

## Health Monitoring

The `/health` endpoint provides comprehensive system status:

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00.000Z",
  "responseTime": "45ms",
  "version": "1.0.0",
  "environment": "production",
  "checks": {
    "kv": { "ArbEdgeKV": { "status": "healthy" } },
    "database": { "status": "healthy" },
    "services": { "Web": { "status": "healthy" } },
    "config": { "status": "healthy" },
    "performance": { "status": "healthy" }
  }
}
```

## Development

### Local Development

```bash
# Start local development server
npm run dev

# The worker will be available at http://localhost:8787
```

### Testing

```bash
# Run unit tests
npm test

# Run tests in watch mode
npm test -- --watch
```

### Building

```bash
# Build for production
npm run build

# Output will be in dist/ directory
```

## Deployment

### Staging Deployment

```bash
wrangler deploy --env staging
```

### Production Deployment

```bash
wrangler deploy --env production
```

### Environment-specific Configuration

Each environment has its own configuration in `wrangler.toml`:
- **Development**: Local service URLs, debug logging
- **Staging**: Staging service URLs, test mode enabled
- **Production**: Production service URLs, optimized settings

## Monitoring

### Metrics

The worker tracks:
- Request count and response times
- Error rates by type and endpoint
- Rate limit violations
- Service health status
- Cache hit/miss ratios

### Logs

Logs are available in:
- Cloudflare Workers dashboard
- `wrangler tail` for real-time logs
- KV storage for error analysis

### Alerts

Set up alerts for:
- High error rates (>5%)
- Service unavailability
- Rate limit violations
- Database connectivity issues

## Security

### CORS Configuration

```javascript
cors({
  origin: ['https://arb-edge.com', 'https://*.arb-edge.com'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization', 'X-API-Key'],
  credentials: true,
})
```

### Security Headers

Automatically applied:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`

### Authentication

Admin routes require valid Authorization header:
```
Authorization: Bearer <token>
```

## Troubleshooting

### Common Issues

1. **Service Unavailable (502)**
   - Check service URLs in configuration
   - Verify target services are running
   - Check network connectivity

2. **Rate Limit Exceeded (429)**
   - Increase rate limits in configuration
   - Implement request queuing
   - Use authentication for higher limits

3. **KV Errors**
   - Verify KV namespace bindings
   - Check KV namespace IDs in wrangler.toml
   - Ensure proper permissions

4. **Database Errors**
   - Verify D1 database binding
   - Check database ID in wrangler.toml
   - Ensure database is accessible

### Debug Mode

Enable debug logging:
```toml
[vars]
LOG_LEVEL = "debug"
```

### Real-time Logs

```bash
# View real-time logs
wrangler tail

# Filter logs by environment
wrangler tail --env production
```

## Contributing

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Test in staging before production

## License

MIT License - see LICENSE file for details