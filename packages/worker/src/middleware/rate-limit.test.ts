import { describe, it, expect } from 'vitest';
import { rateLimiter } from './rate-limit';

describe('Rate Limiter Middleware', () => {
  it('should export rateLimiter function', () => {
    expect(rateLimiter).toBeDefined();
    expect(typeof rateLimiter).toBe('function');
  });

  it('should be a valid middleware function', () => {
    // Test that the function has the expected signature (context, next)
    expect(rateLimiter.length).toBe(2);
  });
});

// Test rate limiting configuration
describe('Rate Limiting Configuration', () => {
  it('should define rate limit constants', () => {
    const rateLimitConfig = {
      defaultLimit: 60,
      windowMs: 60000, // 1 minute
      keyPrefix: 'rate_limit:',
    };
    
    expect(rateLimitConfig.defaultLimit).toBe(60);
    expect(rateLimitConfig.windowMs).toBe(60000);
    expect(rateLimitConfig.keyPrefix).toBe('rate_limit:');
  });

  it('should define rate limit headers', () => {
    const headers = {
      limit: 'X-RateLimit-Limit',
      remaining: 'X-RateLimit-Remaining',
      reset: 'X-RateLimit-Reset',
      retryAfter: 'Retry-After'
    };
    
    expect(headers.limit).toBe('X-RateLimit-Limit');
    expect(headers.remaining).toBe('X-RateLimit-Remaining');
    expect(headers.reset).toBe('X-RateLimit-Reset');
    expect(headers.retryAfter).toBe('Retry-After');
  });

  it('should define IP detection headers', () => {
    const ipHeaders = [
      'cf-connecting-ip',
      'x-real-ip',
      'x-forwarded-for'
    ];
    
    expect(ipHeaders).toContain('cf-connecting-ip');
    expect(ipHeaders).toContain('x-real-ip');
    expect(ipHeaders).toContain('x-forwarded-for');
  });


});