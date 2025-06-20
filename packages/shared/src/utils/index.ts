// Timestamp utilities
export function formatTimestamp(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toISOString();
}

export function isExpired(expiresAt: Date | string): boolean {
  const expiry = typeof expiresAt === 'string' ? new Date(expiresAt) : expiresAt;
  return expiry.getTime() < Date.now();
}

export function getTimeUntilExpiry(expiresAt: Date | string): number {
  const expiry = typeof expiresAt === 'string' ? new Date(expiresAt) : expiresAt;
  return Math.max(0, expiry.getTime() - Date.now());
}

export function addMinutes(date: Date, minutes: number): Date {
  return new Date(date.getTime() + minutes * 60000);
}

export function addHours(date: Date, hours: number): Date {
  return new Date(date.getTime() + hours * 3600000);
}

export function addDays(date: Date, days: number): Date {
  return new Date(date.getTime() + days * 86400000);
}

// Validation utilities
export function isValidTelegramId(id: number | string): boolean {
  const numId = typeof id === 'string' ? parseInt(id, 10) : id;
  return Number.isInteger(numId) && numId > 0;
}

export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function isValidSymbol(symbol: string): boolean {
  // Basic crypto symbol validation (e.g., BTCUSDT, ETH/USD)
  const symbolRegex = /^[A-Z]{2,10}[/-]?[A-Z]{2,10}$/;
  return symbolRegex.test(symbol.toUpperCase());
}

export function isValidExchange(exchange: string): boolean {
  const validExchanges = ['binance', 'bybit', 'okx', 'bitget', 'kucoin', 'gate', 'mexc', 'huobi', 'kraken', 'coinbase'];
  return validExchanges.includes(exchange.toLowerCase());
}

export function isValidPrice(price: number): boolean {
  return typeof price === 'number' && price > 0 && isFinite(price);
}

export function isValidQuantity(quantity: number): boolean {
  return typeof quantity === 'number' && quantity > 0 && isFinite(quantity);
}

// Formatting utilities
export function formatCurrency(amount: number, currency = 'USD', decimals?: number): string {
  const options: Intl.NumberFormatOptions = {
    style: 'currency',
    currency,
  };
  
  if (decimals !== undefined) {
    options.minimumFractionDigits = decimals;
    options.maximumFractionDigits = decimals;
  }
  
  return new Intl.NumberFormat('en-US', options).format(amount);
}

export function formatPercentage(value: number, decimals = 2): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

export function formatNumber(value: number, decimals = 2): string {
  return value.toFixed(decimals);
}

export function formatLargeNumber(value: number): string {
  if (value >= 1e9) return `${(value / 1e9).toFixed(2)}B`;
  if (value >= 1e6) return `${(value / 1e6).toFixed(2)}M`;
  if (value >= 1e3) return `${(value / 1e3).toFixed(2)}K`;
  return value.toString();
}

// Trading utilities
export function calculateProfitPercentage(entryPrice: number, exitPrice: number, type: 'long' | 'short'): number {
  if (type === 'long') {
    return ((exitPrice - entryPrice) / entryPrice) * 100;
  } else {
    return ((entryPrice - exitPrice) / entryPrice) * 100;
  }
}

export function calculatePnL(entryPrice: number, exitPrice: number, quantity: number, type: 'long' | 'short'): number {
  if (type === 'long') {
    return (exitPrice - entryPrice) * quantity;
  } else {
    return (entryPrice - exitPrice) * quantity;
  }
}

// Cloudflare Worker Utilities

// Generate unique error ID
export const generateErrorId = (): string => {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substr(2, 9);
  return `err_${timestamp}_${random}`;
};

// Extract client ID for rate limiting
export const extractClientId = (request: any): string => {
  // Try to get IP from Cloudflare headers
  const cfConnectingIp = request.header?.('CF-Connecting-IP');
  const xForwardedFor = request.header?.('X-Forwarded-For');
  const xRealIp = request.header?.('X-Real-IP');
  
  const ip = cfConnectingIp || xForwardedFor?.split(',')[0] || xRealIp || 'unknown';
  
  // Try to extract user ID from auth header
  const authHeader = request.header?.('Authorization');
  if (authHeader) {
    try {
      const userId = extractUserIdFromAuth(authHeader);
      if (userId) {
        return `user:${userId}`;
      }
    } catch {
      // Continue with IP-based identification
    }
  }
  
  return `ip:${ip}`;
};

// Extract user ID from authorization header
export const extractUserIdFromAuth = (authHeader: string): string | null => {
  try {
    // Extract token from auth header
    authHeader.replace(/^Bearer\s+/i, '');
    // This would typically decode JWT token
    // For now, return null to use IP-based rate limiting
    return null;
  } catch {
    return null;
  }
};

// Get rate limit for specific route
export const getRateLimitForRoute = (path: string, method: string, env: any): number => {
  const routeLimits: Record<string, number> = {
    'GET:/health': 300,
    'GET:/api/': parseInt(env.RATE_LIMIT_REQUESTS_PER_MINUTE || '60', 10),
    'POST:/api/': parseInt(env.RATE_LIMIT_REQUESTS_PER_MINUTE || '60', 10),
    'POST:/webhook/': 120,
    'GET:/admin/': 30,
    'POST:/admin/': 20,
    'GET:/assets/': 300,
    'GET:/web/': 120,
  };
  
  // Find matching route
  for (const [routePattern, limit] of Object.entries(routeLimits)) {
    const [routeMethod, routePath] = routePattern.split(':');
    if (method === routeMethod && path.startsWith(routePath)) {
      return limit;
    }
  }
  
  return 60; // Default limit
};

// Create rate limit response
export const createRateLimitResponse = (limit: number, windowStart: number, windowSize: number = 60): Response => {
  const resetTime = windowStart + windowSize;
  const retryAfter = resetTime - Math.floor(Date.now() / 1000);
  
  const body = {
    error: 'Rate limit exceeded',
    message: `Too many requests. Limit: ${limit} requests per ${windowSize} seconds`,
    retryAfter,
    resetTime: new Date(resetTime * 1000).toISOString(),
  };
  
  return new Response(JSON.stringify(body), {
    status: 429,
    headers: {
      'Content-Type': 'application/json',
      'X-RateLimit-Limit': limit.toString(),
      'X-RateLimit-Remaining': '0',
      'X-RateLimit-Reset': resetTime.toString(),
      'Retry-After': retryAfter.toString(),
    },
  });
};

// Log error to KV storage
export const logErrorToKV = async (errorId: string, error: Error, context: any, kv: KVNamespace): Promise<void> => {
  try {
    const errorData = {
      id: errorId,
      name: error.name,
      message: error.message,
      stack: error.stack,
      url: context.req?.url,
      method: context.req?.method,
      timestamp: new Date().toISOString(),
      userAgent: context.req?.header?.('User-Agent'),
      ip: context.req?.header?.('CF-Connecting-IP') || context.req?.header?.('X-Forwarded-For'),
      referer: context.req?.header?.('Referer'),
      environment: context.env?.ENVIRONMENT || 'unknown',
    };
    
    // Store individual error
    const errorKey = `error:${errorId}`;
    await kv.put(errorKey, JSON.stringify(errorData), {
      expirationTtl: 7 * 24 * 60 * 60, // 7 days
    });
    
    // Update daily error index
    const dateKey = `error_index:${new Date().toISOString().split('T')[0]}`;
    const existingIndex = await kv.get(dateKey);
    const errorIndex = existingIndex ? JSON.parse(existingIndex) : [];
    
    errorIndex.push({
      id: errorId,
      timestamp: errorData.timestamp,
      name: error.name,
      url: errorData.url,
      method: errorData.method,
    });
    
    // Keep only last 1000 errors per day
    if (errorIndex.length > 1000) {
      errorIndex.splice(0, errorIndex.length - 1000);
    }
    
    await kv.put(dateKey, JSON.stringify(errorIndex), {
      expirationTtl: 30 * 24 * 60 * 60, // 30 days
    });
  } catch (logError) {
    console.error('Failed to store error in KV:', logError);
  }
};

export function calculateLeverage(position: number, collateral: number): number {
  return position / collateral;
}

export function calculateLiquidationPrice(
  entryPrice: number,
  leverage: number,
  type: 'long' | 'short',
  maintenanceMargin = 0.005
): number {
  if (type === 'long') {
    return entryPrice * (1 - (1 / leverage) + maintenanceMargin);
  } else {
    return entryPrice * (1 + (1 / leverage) - maintenanceMargin);
  }
}

export function calculateArbitrageProfit(
  price1: number,
  price2: number,
  quantity: number,
  fee1 = 0.001,
  fee2 = 0.001
): { profit: number; profitPercentage: number } {
  const buyPrice = Math.min(price1, price2);
  const sellPrice = Math.max(price1, price2);
  
  const buyCost = buyPrice * quantity * (1 + fee1);
  const sellRevenue = sellPrice * quantity * (1 - fee2);
  
  const profit = sellRevenue - buyCost;
  const profitPercentage = (profit / buyCost) * 100;
  
  return { profit, profitPercentage };
}

// Risk management utilities
export function calculatePositionSize(
  accountBalance: number,
  riskPercentage: number,
  entryPrice: number,
  stopLoss: number
): number {
  const riskAmount = accountBalance * (riskPercentage / 100);
  const priceRisk = Math.abs(entryPrice - stopLoss);
  return riskAmount / priceRisk;
}

export function calculateRiskReward(
  entryPrice: number,
  stopLoss: number,
  takeProfit: number,
  type: 'long' | 'short'
): number {
  let risk: number;
  let reward: number;
  
  if (type === 'long') {
    risk = entryPrice - stopLoss;
    reward = takeProfit - entryPrice;
  } else {
    risk = stopLoss - entryPrice;
    reward = entryPrice - takeProfit;
  }
  
  return reward / risk;
}

// API utilities
export function createApiResponse<T>(
  success: boolean,
  data?: T,
  error?: string,
  message?: string
) {
  return {
    success,
    data,
    error,
    message,
    timestamp: formatTimestamp(new Date()),
  };
}

export function createSuccessResponse<T>(data: T, message?: string) {
  return createApiResponse(true, data, undefined, message);
}

export function createErrorResponse(error: string, message?: string) {
  return createApiResponse(false, undefined, error, message);
}

// Create error response based on error type
export const createDetailedErrorResponse = (error: Error, errorId: string, timestamp: string) => {
  // Determine error type and appropriate response
  if (error.name === 'ValidationError') {
    return {
      status: 400,
      body: {
        error: 'Validation Error',
        message: error.message,
        errorId,
        timestamp,
      },
    };
  }
  
  if (error.name === 'UnauthorizedError' || error.message.includes('unauthorized')) {
    return {
      status: 401,
      body: {
        error: 'Unauthorized',
        message: 'Authentication required or invalid',
        errorId,
        timestamp,
      },
    };
  }
  
  if (error.name === 'ForbiddenError' || error.message.includes('forbidden')) {
    return {
      status: 403,
      body: {
        error: 'Forbidden',
        message: 'Access denied',
        errorId,
        timestamp,
      },
    };
  }
  
  if (error.name === 'NotFoundError' || error.message.includes('not found')) {
    return {
      status: 404,
      body: {
        error: 'Not Found',
        message: 'The requested resource was not found',
        errorId,
        timestamp,
      },
    };
  }
  
  if (error.name === 'TimeoutError' || error.message.includes('timeout')) {
    return {
      status: 408,
      body: {
        error: 'Request Timeout',
        message: 'The request took too long to process',
        errorId,
        timestamp,
      },
    };
  }
  
  if (error.name === 'RateLimitError' || error.message.includes('rate limit')) {
    return {
      status: 429,
      body: {
        error: 'Rate Limit Exceeded',
        message: 'Too many requests, please try again later',
        errorId,
        timestamp,
      },
    };
  }
  
  if (error.message.includes('fetch') || error.message.includes('network')) {
    return {
      status: 502,
      body: {
        error: 'Service Unavailable',
        message: 'External service is temporarily unavailable',
        errorId,
        timestamp,
      },
    };
  }
  
  // Default internal server error
  return {
    status: 500,
    body: {
      error: 'Internal Server Error',
      message: 'An unexpected error occurred',
      errorId,
      timestamp,
      ...(process.env.NODE_ENV === 'development' && {
        details: {
          name: error.name,
          message: error.message,
          stack: error.stack,
        },
      }),
    },
  };
};

// String utilities
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '')
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

export function truncate(text: string, length: number): string {
  return text.length > length ? text.substring(0, length) + '...' : text;
}

// Array utilities
export function chunk<T>(array: T[], size: number): T[][] {
  const chunks: T[][] = [];
  for (let i = 0; i < array.length; i += size) {
    chunks.push(array.slice(i, i + size));
  }
  return chunks;
}

export function unique<T>(array: T[]): T[] {
  return [...new Set(array)];
}

// Object utilities
export function omit<T extends Record<string, any>, K extends keyof T>(
  obj: T,
  keys: K[]
): Omit<T, K> {
  const result = { ...obj };
  keys.forEach(key => delete result[key]);
  return result;
}

export function pick<T extends Record<string, any>, K extends keyof T>(
  obj: T,
  keys: K[]
): Pick<T, K> {
  const result = {} as Pick<T, K>;
  keys.forEach(key => {
    if (key in obj) {
      result[key] = obj[key];
    }
  });
  return result;
}

// Async utilities
export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export async function retry<T>(
  fn: () => Promise<T>,
  attempts: number = 3,
  delayMs: number = 1000
): Promise<T> {
  let lastError: Error;
  
  for (let i = 0; i < attempts; i++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (i < attempts - 1) {
        await delay(delayMs * Math.pow(2, i)); // Exponential backoff
      }
    }
  }
  
  throw lastError!;
}