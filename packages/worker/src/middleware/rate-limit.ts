import type { Context, Next } from 'hono';
import type { Env } from '../index';

/**
 * RateLimiter provides request rate limiting functionality
 * using Cloudflare KV for distributed rate limiting
 */
export class RateLimiter {
  private readonly windowSize = 60; // 1 minute window
  private readonly defaultLimit = 60; // requests per minute
  
  constructor(private kv: KVNamespace) {}

  /**
   * Handle rate limiting middleware
   */
  async handle(c: Context<{ Bindings: Env }>, next: Next): Promise<Response | void> {
    try {
      const clientId = this.getClientId(c);
      const limit = this.getLimit(c);
      const windowStart = Math.floor(Date.now() / 1000 / this.windowSize) * this.windowSize;
      const key = `rate_limit:${clientId}:${windowStart}`;
      
      // Get current count
      const currentCountStr = await this.kv.get(key);
      const currentCount = currentCountStr ? parseInt(currentCountStr, 10) : 0;
      
      // Check if limit exceeded
      if (currentCount >= limit) {
        return this.createRateLimitResponse(limit, windowStart);
      }
      
      // Increment counter
      const newCount = currentCount + 1;
      await this.kv.put(key, newCount.toString(), {
        expirationTtl: this.windowSize * 2 // Keep for 2 windows
      });
      
      // Add rate limit headers
      c.header('X-RateLimit-Limit', limit.toString());
      c.header('X-RateLimit-Remaining', Math.max(0, limit - newCount).toString());
      c.header('X-RateLimit-Reset', (windowStart + this.windowSize).toString());
      
      await next();
    } catch (error) {
      console.error('Rate limiting error:', error);
      // Continue without rate limiting if there's an error
      await next();
    }
  }

  /**
   * Get client identifier for rate limiting
   */
  private getClientId(c: Context<{ Bindings: Env }>): string {
    // Try to get client IP from Cloudflare headers
    const cfConnectingIp = c.req.header('CF-Connecting-IP');
    const xForwardedFor = c.req.header('X-Forwarded-For');
    const xRealIp = c.req.header('X-Real-IP');
    
    const ip = cfConnectingIp || xForwardedFor?.split(',')[0] || xRealIp || 'unknown';
    
    // For authenticated requests, use user ID if available
    const authHeader = c.req.header('Authorization');
    if (authHeader) {
      try {
        // In a real implementation, decode the JWT or API key to get user ID
        const userId = this.extractUserIdFromAuth(authHeader);
        if (userId) {
          return `user:${userId}`;
        }
      } catch {
        // Fall back to IP-based limiting
      }
    }
    
    return `ip:${ip}`;
  }

  /**
   * Get rate limit for the request
   */
  private getLimit(c: Context<{ Bindings: Env }>): number {
    const url = new URL(c.req.url);
    const path = url.pathname;
    const method = c.req.method;
    
    // Different limits for different endpoints
    const limits: Record<string, number> = {
      // Health checks - higher limit
      'GET:/health': 300,
      
      // API endpoints - standard limit
      'GET:/api/': parseInt(c.env.RATE_LIMIT_REQUESTS_PER_MINUTE || '60', 10),
      'POST:/api/': parseInt(c.env.RATE_LIMIT_REQUESTS_PER_MINUTE || '60', 10),
      
      // Webhook endpoints - higher limit for bot traffic
      'POST:/webhook/': 120,
      
      // Admin endpoints - lower limit
      'GET:/admin/': 30,
      'POST:/admin/': 20,
      
      // Web assets - higher limit
      'GET:/assets/': 300,
      'GET:/web/': 120,
    };
    
    // Find matching limit
    for (const [pattern, limit] of Object.entries(limits)) {
      const [patternMethod, patternPath] = pattern.split(':');
      if (method === patternMethod && path.startsWith(patternPath)) {
        return limit;
      }
    }
    
    // Default limit
    return this.defaultLimit;
  }

  /**
   * Extract user ID from authorization header
   */
  private extractUserIdFromAuth(authHeader: string): string | null {
    try {
      // Remove 'Bearer ' prefix
      const _token = authHeader.replace(/^Bearer\s+/i, '');
      
      // In a real implementation, this would decode a JWT token
      // For now, return null to fall back to IP-based limiting
      return null;
    } catch {
      return null;
    }
  }

  /**
   * Create rate limit exceeded response
   */
  private createRateLimitResponse(limit: number, windowStart: number): Response {
    const resetTime = windowStart + this.windowSize;
    const retryAfter = resetTime - Math.floor(Date.now() / 1000);
    
    const response = {
      error: 'Rate limit exceeded',
      message: `Too many requests. Limit: ${limit} requests per ${this.windowSize} seconds`,
      retryAfter: retryAfter,
      resetTime: new Date(resetTime * 1000).toISOString()
    };
    
    return new Response(JSON.stringify(response), {
      status: 429,
      headers: {
        'Content-Type': 'application/json',
        'X-RateLimit-Limit': limit.toString(),
        'X-RateLimit-Remaining': '0',
        'X-RateLimit-Reset': resetTime.toString(),
        'Retry-After': retryAfter.toString()
      }
    });
  }
}

/**
 * Advanced rate limiter with multiple strategies
 */
export class AdvancedRateLimiter extends RateLimiter {
  /**
   * Implement sliding window rate limiting
   */
  async slidingWindowLimit(
    clientId: string, 
    limit: number, 
    windowSizeSeconds: number
  ): Promise<{ allowed: boolean; remaining: number; resetTime: number }> {
    const now = Math.floor(Date.now() / 1000);
    const windowStart = now - windowSizeSeconds;
    
    // Get all timestamps in the current window
    const key = `sliding:${clientId}`;
    const timestampsStr = await this.kv.get(key);
    const timestamps: number[] = timestampsStr ? JSON.parse(timestampsStr) : [];
    
    // Remove old timestamps
    const validTimestamps = timestamps.filter(ts => ts > windowStart);
    
    // Check if limit exceeded
    if (validTimestamps.length >= limit) {
      return {
        allowed: false,
        remaining: 0,
        resetTime: validTimestamps[0] + windowSizeSeconds
      };
    }
    
    // Add current timestamp
    validTimestamps.push(now);
    
    // Store updated timestamps
    await this.kv.put(key, JSON.stringify(validTimestamps), {
      expirationTtl: windowSizeSeconds * 2
    });
    
    return {
      allowed: true,
      remaining: limit - validTimestamps.length,
      resetTime: now + windowSizeSeconds
    };
  }

  /**
   * Implement token bucket rate limiting
   */
  async tokenBucketLimit(
    clientId: string,
    capacity: number,
    refillRate: number,
    tokensRequested: number = 1
  ): Promise<{ allowed: boolean; tokensRemaining: number; refillTime: number }> {
    const now = Date.now();
    const key = `bucket:${clientId}`;
    
    // Get current bucket state
    const bucketStr = await this.kv.get(key);
    let bucket = bucketStr ? JSON.parse(bucketStr) : {
      tokens: capacity,
      lastRefill: now
    };
    
    // Calculate tokens to add based on time elapsed
    const timeDiff = now - bucket.lastRefill;
    const tokensToAdd = Math.floor(timeDiff / 1000 * refillRate);
    
    // Refill bucket
    bucket.tokens = Math.min(capacity, bucket.tokens + tokensToAdd);
    bucket.lastRefill = now;
    
    // Check if enough tokens available
    if (bucket.tokens < tokensRequested) {
      // Store current state
      await this.kv.put(key, JSON.stringify(bucket), {
        expirationTtl: 3600 // 1 hour
      });
      
      return {
        allowed: false,
        tokensRemaining: bucket.tokens,
        refillTime: now + ((tokensRequested - bucket.tokens) / refillRate * 1000)
      };
    }
    
    // Consume tokens
    bucket.tokens -= tokensRequested;
    
    // Store updated state
    await this.kv.put(key, JSON.stringify(bucket), {
      expirationTtl: 3600 // 1 hour
    });
    
    return {
      allowed: true,
      tokensRemaining: bucket.tokens,
      refillTime: 0
    };
  }
}

/**
 * Rate limiter middleware function
 */
export const rateLimiter = async (c: Context<{ Bindings: Env }>, next: Next): Promise<Response | void> => {
  const limiter = new RateLimiter(c.env.ArbEdgeKV);
  return await limiter.handle(c, next);
};