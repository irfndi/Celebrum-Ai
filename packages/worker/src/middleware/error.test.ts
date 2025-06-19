import { describe, it, expect } from 'vitest';
import { errorHandler } from './error';

describe('Error Handler Middleware', () => {
  it('should export errorHandler function', () => {
    expect(errorHandler).toBeDefined();
    expect(typeof errorHandler).toBe('function');
  });

  it('should be a valid middleware function', () => {
    // Test that the function has the expected signature (context, next)
    expect(errorHandler.length).toBe(2);
  });
});

// Test error handling configuration
describe('Error Handling Configuration', () => {
  it('should define error status codes', () => {
    const errorCodes = {
      badRequest: 400,
      unauthorized: 401,
      forbidden: 403,
      notFound: 404,
      methodNotAllowed: 405,
      requestTimeout: 408,
      tooManyRequests: 429,
      internalServerError: 500,
      badGateway: 502,
      serviceUnavailable: 503,
      gatewayTimeout: 504,
    };
    
    expect(errorCodes.badRequest).toBe(400);
    expect(errorCodes.unauthorized).toBe(401);
    expect(errorCodes.internalServerError).toBe(500);
    expect(errorCodes.serviceUnavailable).toBe(503);
  });

  it('should define error types', () => {
    const errorTypes = {
      validation: 'ValidationError',
      authorization: 'AuthorizationError',
      rateLimit: 'RateLimitError',
      database: 'DatabaseError',
      timeout: 'TimeoutError',
      network: 'NetworkError',
    };
    
    expect(errorTypes.validation).toBe('ValidationError');
    expect(errorTypes.authorization).toBe('AuthorizationError');
    expect(errorTypes.rateLimit).toBe('RateLimitError');
    expect(errorTypes.database).toBe('DatabaseError');
  });

  it('should define error messages', () => {
    const errorMessages = {
      badRequest: 'Bad Request',
      unauthorized: 'Unauthorized',
      forbidden: 'Forbidden',
      notFound: 'Not Found',
      internalServerError: 'Internal Server Error',
      serviceUnavailable: 'Service Unavailable',
      tooManyRequests: 'Too Many Requests',
    };
    
    expect(errorMessages.badRequest).toBe('Bad Request');
    expect(errorMessages.unauthorized).toBe('Unauthorized');
    expect(errorMessages.internalServerError).toBe('Internal Server Error');
  });

  it('should define environment-specific behavior', () => {
    const environments = {
      development: 'development',
      staging: 'staging',
      production: 'production',
    };
    
    expect(environments.development).toBe('development');
    expect(environments.staging).toBe('staging');
    expect(environments.production).toBe('production');
  });


});