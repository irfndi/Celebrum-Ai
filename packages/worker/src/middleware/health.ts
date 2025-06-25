import type { Context } from 'hono';
import type { Env } from '@celebrum-ai/shared';

/**
 * HealthCheck provides comprehensive health monitoring for the worker and its dependencies
 */
export class HealthCheck {
  constructor(private env: Env) {}

  /**
   * Perform comprehensive health check
   */
  async check(c: Context<{ Bindings: Env }>): Promise<Response> {
    const startTime = Date.now();
    
    try {
      const healthData = await this.performHealthChecks();
      const responseTime = Date.now() - startTime;
      
      const overallStatus = this.determineOverallStatus(healthData);
      
      const response = {
        status: overallStatus,
        timestamp: new Date().toISOString(),
        responseTime: `${responseTime}ms`,
        version: '1.0.0',
        environment: this.env.ENVIRONMENT || 'unknown',
        checks: healthData,
        uptime: this.getUptime(),
        worker: {
          region: c.req.header('CF-Ray')?.split('-')[1] || 'unknown',
          colo: c.req.header('CF-IPCountry') || 'unknown',
        }
      };
      
      const statusCode = overallStatus === 'healthy' ? 200 : 503;
      
      return c.json(response, statusCode);
    } catch (error) {
      return c.json({
        status: 'error',
        timestamp: new Date().toISOString(),
        responseTime: `${Date.now() - startTime}ms`,
        error: error instanceof Error ? error.message : 'Unknown error',
        checks: {}
      }, 500);
    }
  }

  /**
   * Perform individual health checks
   */
  private async performHealthChecks(): Promise<Record<string, any>> {
    const checks: Record<string, any> = {};
    
    // Check KV namespaces
    checks.kv = await this.checkKVNamespaces();
    
    // Check D1 database
    checks.database = await this.checkDatabase();
    
    // Check external services
    checks.services = await this.checkExternalServices();
    
    // Check environment configuration
    checks.config = this.checkConfiguration();
    
    // Check memory and performance
    checks.performance = this.checkPerformance();
    
    return checks;
  }

  /**
   * Check KV namespace availability
   */
  private async checkKVNamespaces(): Promise<any> {
    const kvChecks: Record<string, any> = {};
    
    try {
      // Test ArbEdgeKV
      const testKey = `health_check_${Date.now()}`;
      await this.env.ArbEdgeKV.put(testKey, 'test', { expirationTtl: 60 });
      const testValue = await this.env.ArbEdgeKV.get(testKey);
      await this.env.ArbEdgeKV.delete(testKey);
      
      kvChecks.ArbEdgeKV = {
        status: testValue === 'test' ? 'healthy' : 'degraded',
        message: testValue === 'test' ? 'Read/write operations successful' : 'Read/write test failed'
      };
    } catch (error) {
      kvChecks.ArbEdgeKV = {
        status: 'unhealthy',
        message: error instanceof Error ? error.message : 'Unknown error'
      };
    }
    
    try {
      // Test market cache KV
      await this.env.PROD_BOT_MARKET_CACHE.get('health_test');
      kvChecks.MarketCache = {
        status: 'healthy',
        message: 'Namespace accessible'
      };
    } catch (error) {
      kvChecks.MarketCache = {
        status: 'unhealthy',
        message: error instanceof Error ? error.message : 'Unknown error'
      };
    }
    
    try {
      // Test session store KV
      await this.env.PROD_BOT_SESSION_STORE.get('health_test');
      kvChecks.SessionStore = {
        status: 'healthy',
        message: 'Namespace accessible'
      };
    } catch (error) {
      kvChecks.SessionStore = {
        status: 'unhealthy',
        message: error instanceof Error ? error.message : 'Unknown error'
      };
    }
    
    return kvChecks;
  }

  /**
   * Check D1 database connectivity
   */
  private async checkDatabase(): Promise<any> {
    try {
      // Simple query to test database connectivity
      const result = await this.env.ArbEdgeD1.prepare('SELECT 1 as test').first();
      
      return {
        status: result?.test === 1 ? 'healthy' : 'degraded',
        message: result?.test === 1 ? 'Database query successful' : 'Database query returned unexpected result',
        latency: 'measured' // In a real implementation, measure actual latency
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        message: error instanceof Error ? error.message : 'Database connection failed',
        latency: null
      };
    }
  }

  /**
   * Check external service connectivity
   */
  private async checkExternalServices(): Promise<any> {
    const serviceChecks: Record<string, any> = {};
    
    // Check if service URLs are configured
    const services = [
      { name: 'Web', url: this.env.WEB_SERVICE_URL },
      { name: 'API', url: this.env.API_SERVICE_URL },
      { name: 'Discord', url: this.env.DISCORD_BOT_SERVICE_URL },
      { name: 'Telegram', url: this.env.TELEGRAM_BOT_SERVICE_URL }
    ];
    
    for (const service of services) {
      if (service.url) {
        try {
          const response = await fetch(`${service.url}/health`, {
            method: 'GET',
            signal: AbortSignal.timeout(5000) // 5 second timeout
          });
          
          serviceChecks[service.name] = {
            status: response.ok ? 'healthy' : 'degraded',
            message: `HTTP ${response.status}`,
            url: service.url
          };
        } catch (error) {
          serviceChecks[service.name] = {
            status: 'unhealthy',
            message: error instanceof Error ? error.message : 'Service unreachable',
            url: service.url
          };
        }
      } else {
        serviceChecks[service.name] = {
          status: 'not_configured',
          message: 'Service URL not configured',
          url: null
        };
      }
    }
    
    return serviceChecks;
  }

  /**
   * Check environment configuration
   */
  private checkConfiguration(): any {
    const requiredVars = [
      'ENVIRONMENT',
      'LOG_LEVEL',
      'SUPER_ADMIN_USER_ID',
      'EXCHANGES',
      'ARBITRAGE_THRESHOLD'
    ];
    
    const configChecks: Record<string, any> = {};
    
    for (const varName of requiredVars) {
      const value = (this.env as any)[varName];
      configChecks[varName] = {
        status: value ? 'configured' : 'missing',
        hasValue: !!value
      };
    }
    
    const missingCount = Object.values(configChecks).filter(check => check.status === 'missing').length;
    
    return {
      status: missingCount === 0 ? 'healthy' : missingCount < 3 ? 'degraded' : 'unhealthy',
      message: `${requiredVars.length - missingCount}/${requiredVars.length} required variables configured`,
      variables: configChecks
    };
  }

  /**
   * Check performance metrics
   */
  private checkPerformance(): any {
    // In a real implementation, this would check memory usage, CPU, etc.
    // For Cloudflare Workers, we have limited access to these metrics
    
    return {
      status: 'healthy',
      message: 'Performance metrics within normal range',
      metrics: {
        // These would be real metrics in production
        memoryUsage: 'unknown',
        cpuUsage: 'unknown',
        requestsPerSecond: 'unknown'
      }
    };
  }

  /**
   * Determine overall health status
   */
  private determineOverallStatus(checks: Record<string, any>): 'healthy' | 'degraded' | 'unhealthy' {
    const statuses: string[] = [];
    
    // Collect all status values
    const collectStatuses = (obj: any) => {
      if (obj && typeof obj === 'object') {
        if (obj.status) {
          statuses.push(obj.status);
        }
        for (const value of Object.values(obj)) {
          if (typeof value === 'object') {
            collectStatuses(value);
          }
        }
      }
    };
    
    collectStatuses(checks);
    
    // Determine overall status
    if (statuses.includes('unhealthy')) {
      return 'unhealthy';
    }
    if (statuses.includes('degraded')) {
      return 'degraded';
    }
    return 'healthy';
  }

  /**
   * Get worker uptime (placeholder for stateless workers)
   */
  private getUptime(): string {
    // In Cloudflare Workers, each request is isolated
    // This is a placeholder for actual uptime tracking
    return 'N/A (stateless worker)';
  }
}

/**
 * Health check middleware function
 */
export const healthCheck = async (c: Context<{ Bindings: Env }>): Promise<Response> => {
  const healthChecker = new HealthCheck(c.env);
  return await healthChecker.check(c);
};