import { z } from 'zod';
import { AppError, ValidationError, AuthenticationError, RateLimitError } from '../errors';
import { formatValidationErrors } from '../validation';
import { API } from '../config';

// Types for middleware
export interface Request {
  headers: Record<string, string | string[] | undefined>;
  body?: unknown;
  query?: Record<string, string | string[] | undefined>;
  params?: Record<string, string>;
  user?: {
    id: string;
    email: string;
    role: string;
    status: string;
  };
  ip?: string;
  method?: string;
  url?: string;
}

export interface Response {
  status: (code: number) => Response;
  json: (data: unknown) => Response;
  send: (data: unknown) => Response;
  setHeader: (name: string, value: string) => Response;
}

export type NextFunction = (error?: Error) => void;
export type MiddlewareFunction = (req: Request, res: Response, next: NextFunction) => void | Promise<void>;

// CORS Middleware
export const corsMiddleware = (options?: {
  origin?: string | string[] | boolean;
  credentials?: boolean;
  methods?: string[];
  allowedHeaders?: string[];
}): MiddlewareFunction => {
  const defaultOptions = {
    origin: API.CORS.ALLOWED_ORIGINS,
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-API-Key'],
  };

  const config = { ...defaultOptions, ...options };

  return (req, res, next) => {
    // Handle preflight requests
    if (req.method === 'OPTIONS') {
      res.setHeader('Access-Control-Allow-Origin', config.origin as string);
      res.setHeader('Access-Control-Allow-Methods', config.methods.join(', '));
      res.setHeader('Access-Control-Allow-Headers', config.allowedHeaders.join(', '));
      
      if (config.credentials) {
        res.setHeader('Access-Control-Allow-Credentials', 'true');
      }
      
      res.status(204).send('');
      return;
    }

    // Set CORS headers for actual requests
    res.setHeader('Access-Control-Allow-Origin', config.origin as string);
    
    if (config.credentials) {
      res.setHeader('Access-Control-Allow-Credentials', 'true');
    }

    next();
  };
};

// Rate Limiting Middleware
interface RateLimitStore {
  [key: string]: {
    count: number;
    resetTime: number;
  };
}

const rateLimitStore: RateLimitStore = {};

export const rateLimitMiddleware = (options?: {
  windowMs?: number;
  max?: number;
  keyGenerator?: (req: Request) => string;
  skipSuccessfulRequests?: boolean;
}): MiddlewareFunction => {
  const defaultOptions = {
    windowMs: API.RATE_LIMIT.WINDOW_MS,
    max: API.RATE_LIMIT.MAX_REQUESTS,
    keyGenerator: (req: Request) => req.ip || 'unknown',
    skipSuccessfulRequests: false,
  };

  const config = { ...defaultOptions, ...options };

  return (req, res, next) => {
    const key = config.keyGenerator(req);
    const now = Date.now();
    const windowStart = now - config.windowMs;

    // Clean up old entries
    Object.keys(rateLimitStore).forEach(storeKey => {
      if (rateLimitStore[storeKey].resetTime < windowStart) {
        delete rateLimitStore[storeKey];
      }
    });

    // Get or create rate limit entry
    if (!rateLimitStore[key] || rateLimitStore[key].resetTime < windowStart) {
      rateLimitStore[key] = {
        count: 0,
        resetTime: now + config.windowMs,
      };
    }

    // Check rate limit
    if (rateLimitStore[key].count >= config.max) {
      const resetTime = Math.ceil((rateLimitStore[key].resetTime - now) / 1000);
      res.setHeader('X-RateLimit-Limit', config.max.toString());
      res.setHeader('X-RateLimit-Remaining', '0');
      res.setHeader('X-RateLimit-Reset', resetTime.toString());
      
      throw new RateLimitError(`Too many requests. Try again in ${resetTime} seconds.`);
    }

    // Increment counter
    rateLimitStore[key].count++;

    // Set rate limit headers
    const remaining = Math.max(0, config.max - rateLimitStore[key].count);
    const resetTime = Math.ceil((rateLimitStore[key].resetTime - now) / 1000);
    
    res.setHeader('X-RateLimit-Limit', config.max.toString());
    res.setHeader('X-RateLimit-Remaining', remaining.toString());
    res.setHeader('X-RateLimit-Reset', resetTime.toString());

    next();
  };
};

// Authentication Middleware
export const authMiddleware = (options?: {
  required?: boolean;
  roles?: string[];
}): MiddlewareFunction => {
  const config = {
    required: true,
    roles: [],
    ...options,
  };

  return (req, _res, next) => {
    const authHeader = req.headers.authorization;
    
    if (!authHeader) {
      if (config.required) {
        throw new AuthenticationError('Authorization header is required');
      }
      return next();
    }

    const token = authHeader.toString().replace('Bearer ', '');
    
    if (!token) {
      if (config.required) {
        throw new AuthenticationError('Invalid authorization format');
      }
      return next();
    }

    try {
      // This would typically verify JWT token
      // For now, we'll simulate user extraction
      const user = extractUserFromToken(token);
      
      if (!user) {
        throw new AuthenticationError('Invalid or expired token');
      }

      // Check role authorization
      if (config.roles.length > 0 && !config.roles.includes(user.role)) {
        throw new AuthenticationError('Insufficient permissions');
      }

      req.user = user;
      next();
    } catch (_error) {
      if (config.required) {
        throw new AuthenticationError('Token verification failed');
      }
      next();
    }
  };
};

// Mock function for token extraction (replace with actual JWT verification)
const extractUserFromToken = (token: string) => {
  // This is a placeholder - implement actual JWT verification
  try {
    // Simulate token parsing
    const payload = JSON.parse(atob(token.split('.')[1]));
    return {
      id: payload.sub,
      email: payload.email,
      role: payload.role,
      status: payload.status,
    };
  } catch {
    return null;
  }
};

// Validation Middleware
export const validateMiddleware = <T>(schema: z.ZodSchema<T>, target: 'body' | 'query' | 'params' = 'body'): MiddlewareFunction => {
  return (req, _res, next) => {
    try {
      const data = req[target];
      const result = schema.safeParse(data);
      
      if (!result.success) {
        const errors = formatValidationErrors(result.error);
        throw new ValidationError('Validation failed', { errors });
      }
      
      // Replace the original data with validated data
      (req as any)[target] = result.data;
      next();
    } catch (error) {
      next(error as Error);
    }
  };
};

// Error Handling Middleware
export const errorMiddleware = (): MiddlewareFunction => {
  return (_req, res, next) => {
    try {
      next();
    } catch (error) {
      if (error instanceof AppError) {
        res.status(error.statusCode).json({
          success: false,
          error: {
            message: error.message,
            code: error.code,
            timestamp: error.timestamp,
            ...(error.details && { details: error.details }),
          },
        });
        return;
      }

      // Handle unknown errors
      console.error('Unhandled error:', error);
      res.status(500).json({
        success: false,
        error: {
          message: 'Internal server error',
          code: 'INTERNAL_ERROR',
          timestamp: new Date().toISOString(),
        },
      });
    }
  };
};

// Request Logging Middleware
export const loggingMiddleware = (options?: {
  includeBody?: boolean;
  includeHeaders?: boolean;
  excludePaths?: string[];
}): MiddlewareFunction => {
  const config = {
    includeBody: false,
    includeHeaders: false,
    excludePaths: ['/health', '/metrics'],
    ...options,
  };

  return (req, res, next) => {
    const start = Date.now();
    
    // Skip logging for excluded paths
    if (config.excludePaths.some(path => req.url?.includes(path))) {
      return next();
    }

    const logData: Record<string, unknown> = {
      method: req.method,
      url: req.url,
      ip: req.ip,
      timestamp: new Date().toISOString(),
    };

    if (config.includeHeaders) {
      logData.headers = req.headers;
    }

    if (config.includeBody && req.body) {
      logData.body = req.body;
    }

    console.log('Request:', logData);

    // Log response time when request completes
    const originalJson = res.json;
    res.json = function(data: unknown) {
      const duration = Date.now() - start;
      console.log('Response:', {
        method: req.method,
        url: req.url,
        duration: `${duration}ms`,
        timestamp: new Date().toISOString(),
      });
      return originalJson.call(this, data);
    };

    next();
  };
};

// Security Headers Middleware
export const securityMiddleware = (): MiddlewareFunction => {
  return (_req, res, next) => {
    // Set security headers
    res.setHeader('X-Content-Type-Options', 'nosniff');
    res.setHeader('X-Frame-Options', 'DENY');
    res.setHeader('X-XSS-Protection', '1; mode=block');
    res.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
    res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
    res.setHeader('Content-Security-Policy', "default-src 'self'");
    
    next();
  };
};

// Health Check Middleware
export const healthCheckMiddleware = (path: string = '/health'): MiddlewareFunction => {
  return (req, res, next) => {
    if (req.url === path && req.method === 'GET') {
      res.status(200).json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        uptime: process.uptime(),
        memory: process.memoryUsage(),
      });
      return;
    }
    next();
  };
};

// Compression Middleware (placeholder)
export const compressionMiddleware = (): MiddlewareFunction => {
  return (_req, _res, next) => {
    // This would typically implement gzip compression
    // For now, just pass through
    next();
  };
};

// Middleware composition utility
export const composeMiddleware = (...middlewares: MiddlewareFunction[]): MiddlewareFunction => {
  return (req, res, next) => {
    let index = 0;
    
    const dispatch = (i: number): void => {
      if (i <= index) {
        throw new Error('next() called multiple times');
      }
      
      index = i;
      
      if (i >= middlewares.length) {
        return next();
      }
      
      const middleware = middlewares[i];
      
      try {
        middleware(req, res, () => dispatch(i + 1));
      } catch (error) {
        next(error instanceof Error ? error : new Error(String(error)));
      }
    };
    
    dispatch(0);
  };
};

// Export all middleware
export const middleware = {
  cors: corsMiddleware,
  rateLimit: rateLimitMiddleware,
  auth: authMiddleware,
  validate: validateMiddleware,
  error: errorMiddleware,
  logging: loggingMiddleware,
  security: securityMiddleware,
  healthCheck: healthCheckMiddleware,
  compression: compressionMiddleware,
  compose: composeMiddleware,
};