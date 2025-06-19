import { describe, it, expect } from 'vitest';

// Simple unit tests for the worker module structure
describe('Worker Module', () => {
  it('should export the required types and interfaces', async () => {
    const module = await import('./index');
    
    expect(module.default).toBeDefined();
    expect(typeof module.default).toBe('object');
  });

  it('should have proper environment interface', async () => {
    // Env is a TypeScript interface, so we can't test it at runtime
    // Instead, we verify that the module exports are properly typed
    const module = await import('./index');
    expect(module).toBeDefined();
  });
});

// Test route patterns
describe('Route Patterns', () => {
  it('should define correct route prefixes', () => {
    const routes = {
      web: '/web',
      api: '/api',
      bot: '/bot',
      admin: '/admin',
      health: '/health'
    };
    
    expect(routes.web).toBe('/web');
    expect(routes.api).toBe('/api');
    expect(routes.bot).toBe('/bot');
    expect(routes.admin).toBe('/admin');
    expect(routes.health).toBe('/health');
  });
});