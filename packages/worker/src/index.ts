import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { logger } from 'hono/logger';
import { prettyJSON } from 'hono/pretty-json';
import { secureHeaders } from 'hono/secure-headers';
import { timing } from 'hono/timing';
import { RouteHandler } from './routes/handler';
import { ServiceRouter } from './services/router';
import { HealthCheck } from './middleware/health';
import { RateLimiter } from './middleware/rate-limit';
import { ErrorHandler } from './middleware/error';
import { AppError } from '@arb-edge/shared/errors';
import { APP_NAME, APP_VERSION } from '@arb-edge/shared/constants';
import { ApiResponse } from '@arb-edge/shared/types';

// Cloudflare Worker Environment
export interface Env {
  // KV Namespaces
  ArbEdgeKV: KVNamespace;
  PROD_BOT_MARKET_CACHE: KVNamespace;
  PROD_BOT_SESSION_STORE: KVNamespace;
  
  // D1 Database
  ArbEdgeD1: D1Database;
  
  // Environment Variables
  ENVIRONMENT: string;
  LOG_LEVEL: string;
  RATE_LIMIT_REQUESTS_PER_MINUTE: string;
  CACHE_TTL_SECONDS: string;
  SUPER_ADMIN_USER_ID: string;
  EXCHANGES: string;
  ARBITRAGE_THRESHOLD: string;
  TELEGRAM_CHAT_ID: string;
  TELEGRAM_TEST_MODE: string;
  
  // Service URLs (for routing)
  WEB_SERVICE_URL?: string;
  API_SERVICE_URL?: string;
  DISCORD_BOT_SERVICE_URL?: string;
  TELEGRAM_BOT_SERVICE_URL?: string;
}

// Initialize Hono app
const app = new Hono<{ Bindings: Env }>();

// Global middleware
app.use('*', logger());
app.use('*', timing());
app.use('*', prettyJSON());
app.use('*', secureHeaders());
app.use('*', cors({
  origin: ['https://arb-edge.com', 'https://*.arb-edge.com'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization', 'X-API-Key'],
  credentials: true,
}));

// Rate limiting middleware
app.use('*', async (c, next) => {
  const rateLimiter = new RateLimiter(c.env.ArbEdgeKV);
  return rateLimiter.handle(c, next);
});

// Health check endpoint
app.get('/health', async (c) => {
  const healthCheck = new HealthCheck(c.env);
  return healthCheck.check(c);
});

// Service routing based on path patterns
const serviceRouter = new ServiceRouter();

// Web interface routes (static assets and pages)
app.route('/web/*', serviceRouter.webHandler());
app.route('/', serviceRouter.webHandler()); // Root redirects to web

// API routes
app.route('/api/*', serviceRouter.apiHandler());
app.route('/v1/*', serviceRouter.apiHandler()); // Legacy API support

// Bot webhook routes
app.route('/webhook/telegram/*', serviceRouter.telegramBotHandler());
app.route('/webhook/discord/*', serviceRouter.discordBotHandler());

// Admin routes
app.route('/admin/*', serviceRouter.adminHandler());

// Catch-all route handler
app.all('*', async (c) => {
  const routeHandler = new RouteHandler(c.env);
  return routeHandler.handleUnknownRoute(c);
});

// Global error handler
app.onError((err, c) => {
  const errorHandler = new ErrorHandler();
  return errorHandler.handle(err, c);
});

// Export the worker
const worker = {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    return app.fetch(request, env, ctx);
  },
};

// Export app and worker
export { app };
export default worker;