import type { Context, Next } from 'hono';
import type { Env } from '../index';

/**
 * ErrorHandler provides centralized error handling and logging
 */
export class ErrorHandler {
  /**
   * Handle errors with appropriate responses and logging
   */
  async handle(error: Error, c: Context<{ Bindings: Env }>): Promise<Response> {
    const errorId = this.generateErrorId();
    const timestamp = new Date().toISOString();
    
    // Log error details
    console.error(`Error ${errorId}:`, {
      message: error.message,
      stack: error.stack,
      url: c.req.url,
      method: c.req.method,
      timestamp,
      userAgent: c.req.header('User-Agent'),
      ip: c.req.header('CF-Connecting-IP') || c.req.header('X-Forwarded-For')
    });
    
    // Store error for monitoring (if KV is available)
    try {
      await this.logError(errorId, error, c);
    } catch (logError) {
      console.error('Failed to log error to KV:', logError);
    }
    
    // Determine error type and response
    const errorResponse = this.createErrorResponse(error, errorId, timestamp);
    
    // Set appropriate headers
    const headers = {
      'Content-Type': 'application/json',
      'X-Error-ID': errorId,
      'X-Timestamp': timestamp
    };
    
    return new Response(JSON.stringify(errorResponse.body), {
      status: errorResponse.status,
      headers
    });
  }

  /**
   * Create appropriate error response based on error type
   */
  private createErrorResponse(error: Error, errorId: string, timestamp: string): {
    status: number;
    body: any;
  } {
    // Handle specific error types
    if (error.name === 'ValidationError') {
      return {
        status: 400,
        body: {
          error: 'Validation Error',
          message: error.message,
          errorId,
          timestamp
        }
      };
    }
    
    if (error.name === 'UnauthorizedError' || error.message.includes('unauthorized')) {
      return {
        status: 401,
        body: {
          error: 'Unauthorized',
          message: 'Authentication required or invalid',
          errorId,
          timestamp
        }
      };
    }
    
    if (error.name === 'ForbiddenError' || error.message.includes('forbidden')) {
      return {
        status: 403,
        body: {
          error: 'Forbidden',
          message: 'Access denied',
          errorId,
          timestamp
        }
      };
    }
    
    if (error.name === 'NotFoundError' || error.message.includes('not found')) {
      return {
        status: 404,
        body: {
          error: 'Not Found',
          message: 'The requested resource was not found',
          errorId,
          timestamp
        }
      };
    }
    
    if (error.name === 'TimeoutError' || error.message.includes('timeout')) {
      return {
        status: 408,
        body: {
          error: 'Request Timeout',
          message: 'The request took too long to process',
          errorId,
          timestamp
        }
      };
    }
    
    if (error.name === 'RateLimitError' || error.message.includes('rate limit')) {
      return {
        status: 429,
        body: {
          error: 'Rate Limit Exceeded',
          message: 'Too many requests, please try again later',
          errorId,
          timestamp
        }
      };
    }
    
    // Handle network/service errors
    if (error.message.includes('fetch') || error.message.includes('network')) {
      return {
        status: 502,
        body: {
          error: 'Service Unavailable',
          message: 'External service is temporarily unavailable',
          errorId,
          timestamp
        }
      };
    }
    
    // Default to internal server error
    return {
      status: 500,
      body: {
        error: 'Internal Server Error',
        message: 'An unexpected error occurred',
        errorId,
        timestamp,
        // Only include error details in development
        ...(process.env.NODE_ENV === 'development' && {
          details: {
            name: error.name,
            message: error.message,
            stack: error.stack
          }
        })
      }
    };
  }

  /**
   * Log error to KV for monitoring and analysis
   */
  private async logError(errorId: string, error: Error, c: Context<{ Bindings: Env }>): Promise<void> {
    try {
      const errorData = {
        id: errorId,
        name: error.name,
        message: error.message,
        stack: error.stack,
        url: c.req.url,
        method: c.req.method,
        timestamp: new Date().toISOString(),
        userAgent: c.req.header('User-Agent'),
        ip: c.req.header('CF-Connecting-IP') || c.req.header('X-Forwarded-For'),
        referer: c.req.header('Referer'),
        environment: c.env.ENVIRONMENT || 'unknown'
      };
      
      const key = `error:${errorId}`;
      
      // Store error with 7 day TTL
      await c.env.ArbEdgeKV.put(key, JSON.stringify(errorData), {
        expirationTtl: 7 * 24 * 60 * 60 // 7 days
      });
      
      // Also store in error index for monitoring
      const indexKey = `error_index:${new Date().toISOString().split('T')[0]}`; // Daily index
      const existingIndex = await c.env.ArbEdgeKV.get(indexKey);
      const errorIndex = existingIndex ? JSON.parse(existingIndex) : [];
      
      errorIndex.push({
        id: errorId,
        timestamp: errorData.timestamp,
        name: error.name,
        url: errorData.url,
        method: errorData.method
      });
      
      // Keep only last 1000 errors per day
      if (errorIndex.length > 1000) {
        errorIndex.splice(0, errorIndex.length - 1000);
      }
      
      await c.env.ArbEdgeKV.put(indexKey, JSON.stringify(errorIndex), {
        expirationTtl: 30 * 24 * 60 * 60 // 30 days
      });
    } catch (kvError) {
      console.error('Failed to store error in KV:', kvError);
    }
  }

  /**
   * Generate unique error ID
   */
  private generateErrorId(): string {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substr(2, 9);
    return `err_${timestamp}_${random}`;
  }
}

/**
 * Custom error classes for specific error types
 */
export class ValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ValidationError';
  }
}

export class UnauthorizedError extends Error {
  constructor(message: string = 'Unauthorized') {
    super(message);
    this.name = 'UnauthorizedError';
  }
}

export class ForbiddenError extends Error {
  constructor(message: string = 'Forbidden') {
    super(message);
    this.name = 'ForbiddenError';
  }
}

export class NotFoundError extends Error {
  constructor(message: string = 'Not Found') {
    super(message);
    this.name = 'NotFoundError';
  }
}

export class TimeoutError extends Error {
  constructor(message: string = 'Request Timeout') {
    super(message);
    this.name = 'TimeoutError';
  }
}

export class RateLimitError extends Error {
  constructor(message: string = 'Rate Limit Exceeded') {
    super(message);
    this.name = 'RateLimitError';
  }
}

/**
 * Error monitoring utilities
 */
export class ErrorMonitor {
  constructor(private kv: KVNamespace) {}

  /**
   * Get error statistics for monitoring
   */
  async getErrorStats(days: number = 7): Promise<any> {
    const stats: any = {
      totalErrors: 0,
      errorsByDay: {},
      errorsByType: {},
      topErrors: [],
      recentErrors: []
    };
    
    try {
      // Get error data for the specified number of days
      for (let i = 0; i < days; i++) {
        const date = new Date();
        date.setDate(date.getDate() - i);
        const dateKey = date.toISOString().split('T')[0];
        const indexKey = `error_index:${dateKey}`;
        
        const dayErrors = await this.kv.get(indexKey);
        if (dayErrors) {
          const errors = JSON.parse(dayErrors);
          stats.totalErrors += errors.length;
          stats.errorsByDay[dateKey] = errors.length;
          
          // Count errors by type
          for (const error of errors) {
            stats.errorsByType[error.name] = (stats.errorsByType[error.name] || 0) + 1;
          }
          
          // Add to recent errors (if from today)
          if (i === 0) {
            stats.recentErrors = errors.slice(-10); // Last 10 errors
          }
        } else {
          stats.errorsByDay[dateKey] = 0;
        }
      }
      
      // Calculate top errors
      stats.topErrors = Object.entries(stats.errorsByType)
        .sort(([,a], [,b]) => (b as number) - (a as number))
        .slice(0, 10)
        .map(([name, count]) => ({ name, count }));
        
    } catch (error) {
      console.error('Failed to get error stats:', error);
    }
    
    return stats;
  }

  /**
   * Get detailed error information
   */
  async getErrorDetails(errorId: string): Promise<any> {
    try {
      const errorData = await this.kv.get(`error:${errorId}`);
      return errorData ? JSON.parse(errorData) : null;
    } catch (error) {
      console.error('Failed to get error details:', error);
      return null;
    }
  }

  /**
   * Clean up old error data
   */
  async cleanupOldErrors(daysToKeep: number = 30): Promise<void> {
    try {
      // This would require listing all keys, which is expensive in KV
      // In practice, we rely on TTL for cleanup
      console.log(`Error cleanup: relying on TTL for errors older than ${daysToKeep} days`);
    } catch (error) {
      console.error('Failed to cleanup old errors:', error);
    }
  }
}

/**
 * Error handler middleware function
 */
export const errorHandler = async (c: Context<{ Bindings: Env }>, next: Next): Promise<Response | void> => {
  try {
    await next();
  } catch (error) {
    const handler = new ErrorHandler();
    return await handler.handle(error as Error, c);
  }
};