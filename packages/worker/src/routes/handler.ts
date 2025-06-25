import type { Context } from 'hono';
import type { Env } from '@celebrum-ai/shared';

/**
 * RouteHandler manages unknown routes and provides fallback logic
 */
export class RouteHandler {
  constructor(private env: Env) {}

  /**
   * Handle unknown routes with intelligent fallback
   */
  async handleUnknownRoute(c: Context<{ Bindings: Env }>): Promise<Response> {
    const url = new URL(c.req.url);
    const path = url.pathname;
    const method = c.req.method;

    // Log the unknown route for monitoring
    console.log(`Unknown route: ${method} ${path}`);

    // Try to determine the intended service based on path patterns
    const intendedService = this.detectIntendedService(path);

    // Return appropriate response based on detected service
    switch (intendedService) {
      case 'api':
        return this.handleUnknownApiRoute(c, path);
      case 'web':
        return this.handleUnknownWebRoute(c, path);
      case 'webhook':
        return this.handleUnknownWebhookRoute(c, path);
      case 'admin':
        return this.handleUnknownAdminRoute(c, path);
      default:
        return this.handleGenericUnknownRoute(c, path);
    }
  }

  /**
   * Detect intended service based on path patterns
   */
  private detectIntendedService(path: string): string {
    // API patterns
    if (path.startsWith('/api/') || 
        path.startsWith('/v1/') || 
        path.startsWith('/v2/') ||
        path.includes('/graphql') ||
        path.includes('/rest/')) {
      return 'api';
    }

    // Webhook patterns
    if (path.startsWith('/webhook/') ||
        path.startsWith('/hooks/') ||
        path.includes('/callback') ||
        path.includes('/notify')) {
      return 'webhook';
    }

    // Admin patterns
    if (path.startsWith('/admin/') ||
        path.startsWith('/dashboard/') ||
        path.startsWith('/manage/')) {
      return 'admin';
    }

    // Static asset patterns
    if (path.includes('.') && (
        path.endsWith('.js') ||
        path.endsWith('.css') ||
        path.endsWith('.png') ||
        path.endsWith('.jpg') ||
        path.endsWith('.svg') ||
        path.endsWith('.ico') ||
        path.endsWith('.woff') ||
        path.endsWith('.woff2')
    )) {
      return 'web';
    }

    // Default to web for most other routes
    return 'web';
  }

  /**
   * Handle unknown API routes
   */
  private async handleUnknownApiRoute(c: Context<{ Bindings: Env }>, path: string): Promise<Response> {
    // Store the failed route for analytics
    await this.logFailedRoute('api', path, c.req.method);

    return c.json({
      error: 'API endpoint not found',
      path: path,
      method: c.req.method,
      message: 'The requested API endpoint does not exist',
      suggestions: [
        '/api/health - Check system health',
        '/api/v1/opportunities - Get trading opportunities',
        '/api/v1/user/profile - Get user profile',
        '/api/v1/trading/positions - Get trading positions'
      ],
      timestamp: new Date().toISOString()
    }, 404);
  }

  /**
   * Handle unknown web routes
   */
  private async handleUnknownWebRoute(c: Context<{ Bindings: Env }>, path: string): Promise<Response> {
    // Store the failed route for analytics
    await this.logFailedRoute('web', path, c.req.method);

    // For web routes, return a 404 page
    return c.html(`
      <!DOCTYPE html>
      <html>
      <head>
        <title>Page Not Found - ArbEdge</title>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <style>
          body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
          .container { max-width: 600px; margin: 0 auto; }
          .error-code { font-size: 72px; color: #e74c3c; margin: 20px 0; }
          .error-message { font-size: 24px; margin: 20px 0; }
          .suggestions { text-align: left; margin: 30px 0; }
          .suggestions ul { list-style-type: none; padding: 0; }
          .suggestions li { margin: 10px 0; }
          .suggestions a { color: #3498db; text-decoration: none; }
          .suggestions a:hover { text-decoration: underline; }
        </style>
      </head>
      <body>
        <div class="container">
          <div class="error-code">404</div>
          <div class="error-message">Page Not Found</div>
          <p>The page you're looking for doesn't exist.</p>
          <div class="suggestions">
            <h3>Try these instead:</h3>
            <ul>
              <li><a href="/">Home Page</a></li>
              <li><a href="/api/health">API Health Check</a></li>
              <li><a href="/admin">Admin Panel</a></li>
            </ul>
          </div>
          <p><small>Path: ${path}</small></p>
        </div>
      </body>
      </html>
    `, 404);
  }

  /**
   * Handle unknown webhook routes
   */
  private async handleUnknownWebhookRoute(c: Context<{ Bindings: Env }>, path: string): Promise<Response> {
    // Store the failed route for analytics
    await this.logFailedRoute('webhook', path, c.req.method);

    return c.json({
      error: 'Webhook endpoint not found',
      path: path,
      method: c.req.method,
      message: 'The requested webhook endpoint does not exist',
      availableWebhooks: [
        '/webhook/telegram/* - Telegram bot webhooks',
        '/webhook/discord/* - Discord bot webhooks'
      ],
      timestamp: new Date().toISOString()
    }, 404);
  }

  /**
   * Handle unknown admin routes
   */
  private async handleUnknownAdminRoute(c: Context<{ Bindings: Env }>, path: string): Promise<Response> {
    // Store the failed route for analytics
    await this.logFailedRoute('admin', path, c.req.method);

    // Check for admin authentication
    const authHeader = c.req.header('Authorization');
    if (!authHeader) {
      return c.json({ error: 'Authentication required for admin routes' }, 401);
    }

    return c.json({
      error: 'Admin endpoint not found',
      path: path,
      method: c.req.method,
      message: 'The requested admin endpoint does not exist',
      availableEndpoints: [
        '/admin/users - User management',
        '/admin/trading - Trading settings',
        '/admin/analytics - System analytics'
      ],
      timestamp: new Date().toISOString()
    }, 404);
  }

  /**
   * Handle generic unknown routes
   */
  private async handleGenericUnknownRoute(c: Context<{ Bindings: Env }>, path: string): Promise<Response> {
    // Store the failed route for analytics
    await this.logFailedRoute('unknown', path, c.req.method);

    // Determine response format based on Accept header
    const acceptHeader = c.req.header('Accept') || '';
    
    if (acceptHeader.includes('application/json')) {
      return c.json({
        error: 'Route not found',
        path: path,
        method: c.req.method,
        message: 'The requested route does not exist',
        timestamp: new Date().toISOString()
      }, 404);
    }

    // Default to HTML response
    return c.html(`
      <!DOCTYPE html>
      <html>
      <head>
        <title>Not Found - ArbEdge</title>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
      </head>
      <body>
        <h1>404 - Not Found</h1>
        <p>The requested resource could not be found.</p>
        <p><a href="/">Return to Home</a></p>
      </body>
      </html>
    `, 404);
  }

  /**
   * Log failed routes for analytics and monitoring
   */
  private async logFailedRoute(service: string, path: string, method: string): Promise<void> {
    try {
      const logKey = `failed_routes:${Date.now()}:${Math.random().toString(36).substr(2, 9)}`;
      const logData = {
        service,
        path,
        method,
        timestamp: new Date().toISOString(),
        userAgent: '', // Would be populated from request headers
        ip: '', // Would be populated from request
      };

      // Store in KV for analytics (with TTL of 7 days)
      await this.env.ArbEdgeKV.put(logKey, JSON.stringify(logData), {
        expirationTtl: 7 * 24 * 60 * 60 // 7 days
      });
    } catch (error) {
      console.error('Failed to log route:', error);
    }
  }
}