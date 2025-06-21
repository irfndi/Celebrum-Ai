// Environment Configuration
export const ENV = {
  NODE_ENV: process.env.NODE_ENV || 'development',
  IS_PRODUCTION: process.env.NODE_ENV === 'production',
  IS_DEVELOPMENT: process.env.NODE_ENV === 'development',
  IS_TEST: process.env.NODE_ENV === 'test',
} as const;

// API Configuration
export const API = {
  DEFAULT_TIMEOUT: 30000,
  MAX_RETRIES: 3,
  RETRY_DELAY: 1000,
  BACKOFF_MULTIPLIER: 2,
  MAX_RETRY_DELAY: 10000,
  RATE_LIMIT: {
    WINDOW_MS: 15 * 60 * 1000, // 15 minutes
    MAX_REQUESTS: 100,
    FREE_TIER_LIMIT: 50,
    PRO_TIER_LIMIT: 200,
    ULTRA_TIER_LIMIT: 500,
  },
  CORS: {
    ALLOWED_ORIGINS: ['http://localhost:3000', 'https://arbedge.com'],
    ALLOWED_METHODS: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    ALLOWED_HEADERS: ['Content-Type', 'Authorization', 'X-API-Key'],
  },
} as const;

// Database Configuration
export const DATABASE = {
  CONNECTION_TIMEOUT: 10000,
  QUERY_TIMEOUT: 30000,
  MAX_CONNECTIONS: 10,
  IDLE_TIMEOUT: 300000, // 5 minutes
  RETRY_ATTEMPTS: 3,
  BATCH_SIZE: 1000,
} as const;

// Telegram Bot Configuration
export const TELEGRAM = {
  MAX_MESSAGE_LENGTH: 4096,
  PARSE_MODE: 'HTML' as const,
  DISABLE_WEB_PAGE_PREVIEW: true,
  COMMAND_TIMEOUT: 30000,
  WEBHOOK_TIMEOUT: 10000,
  MAX_CONNECTIONS: 40,
  ALLOWED_UPDATES: ['message', 'callback_query', 'inline_query'],
  RATE_LIMIT: {
    MESSAGES_PER_SECOND: 30,
    MESSAGES_PER_MINUTE: 20,
    MESSAGES_PER_CHAT_PER_SECOND: 1,
  },
  COMMANDS: {
    START: '/start',
    HELP: '/help',
    SETTINGS: '/settings',
    BALANCE: '/balance',
    OPPORTUNITIES: '/opportunities',
    POSITIONS: '/positions',
    STOP: '/stop',
  },
} as const;

// Trading Configuration
export const TRADING = {
  MIN_PROFIT_THRESHOLD: 0.5, // 0.5%
  MAX_POSITION_SIZE: 1000,
  DEFAULT_LEVERAGE: 1,
  MAX_LEVERAGE: 100,
  STOP_LOSS_PERCENTAGE: 2, // 2%
  TAKE_PROFIT_PERCENTAGE: 5, // 5%
  MAX_CONCURRENT_TRADES: 10,
  MIN_TRADE_AMOUNT: 10, // $10
  MAX_TRADE_AMOUNT: 100000, // $100k
  SLIPPAGE_TOLERANCE: 0.1, // 0.1%
  OPPORTUNITY_EXPIRY: 300000, // 5 minutes
  CONFIDENCE_THRESHOLD: 0.7, // 70%
  EXCHANGES: {
    BINANCE: {
      MAKER_FEE: 0.001,
      TAKER_FEE: 0.001,
      WITHDRAWAL_FEE: 0.0005,
    },
    BYBIT: {
      MAKER_FEE: 0.001,
      TAKER_FEE: 0.001,
      WITHDRAWAL_FEE: 0.0005,
    },
    OKX: {
      MAKER_FEE: 0.0008,
      TAKER_FEE: 0.001,
      WITHDRAWAL_FEE: 0.0004,
    },
  },
  RISK_LEVELS: {
    LOW: {
      MAX_POSITION_PERCENTAGE: 5,
      STOP_LOSS_PERCENTAGE: 1,
      MAX_LEVERAGE: 2,
    },
    MEDIUM: {
      MAX_POSITION_PERCENTAGE: 10,
      STOP_LOSS_PERCENTAGE: 2,
      MAX_LEVERAGE: 5,
    },
    HIGH: {
      MAX_POSITION_PERCENTAGE: 20,
      STOP_LOSS_PERCENTAGE: 5,
      MAX_LEVERAGE: 10,
    },
  },
} as const;

// Cache Configuration
export const CACHE = {
  DEFAULT_TTL: 300, // 5 minutes
  SHORT_TTL: 60, // 1 minute
  MEDIUM_TTL: 900, // 15 minutes
  LONG_TTL: 3600, // 1 hour
  VERY_LONG_TTL: 86400, // 24 hours
  KEYS: {
    USER_PROFILE: 'user:profile:',
    USER_SETTINGS: 'user:settings:',
    USER_POSITIONS: 'user:positions:',
    ARBITRAGE_OPPORTUNITIES: 'arbitrage:opportunities',
    EXCHANGE_PRICES: 'exchange:prices:',
    TRADING_PAIRS: 'trading:pairs:',
    MARKET_DATA: 'market:data:',
    EXCHANGE_STATUS: 'exchange:status:',
    RATE_LIMIT: 'rate:limit:',
    SESSION: 'session:',
  },
} as const;

// Security Configuration
export const SECURITY = {
  JWT: {
    EXPIRES_IN: '7d',
    REFRESH_EXPIRES_IN: '30d',
    ALGORITHM: 'HS256' as const,
  },
  API_KEY: {
    LENGTH: 32,
    EXPIRES_IN: '1y',
  },
  PASSWORD: {
    MIN_LENGTH: 8,
    REQUIRE_UPPERCASE: true,
    REQUIRE_LOWERCASE: true,
    REQUIRE_NUMBERS: true,
    REQUIRE_SYMBOLS: false,
  },
  ENCRYPTION: {
    ALGORITHM: 'aes-256-gcm' as const,
    KEY_LENGTH: 32,
    IV_LENGTH: 16,
  },
} as const;

// Monitoring Configuration
export const MONITORING = {
  HEALTH_CHECK_INTERVAL: 30000, // 30 seconds
  METRICS_COLLECTION_INTERVAL: 60000, // 1 minute
  LOG_LEVELS: {
    ERROR: 0,
    WARN: 1,
    INFO: 2,
    DEBUG: 3,
  },
  ALERTS: {
    ERROR_THRESHOLD: 10, // errors per minute
    LATENCY_THRESHOLD: 5000, // 5 seconds
    MEMORY_THRESHOLD: 0.9, // 90%
    CPU_THRESHOLD: 0.8, // 80%
  },
} as const;

// Feature Flags
export const FEATURES = {
  ARBITRAGE_TRADING: true,
  TECHNICAL_ANALYSIS: true,
  SOCIAL_TRADING: false,
  ADVANCED_CHARTS: true,
  MOBILE_NOTIFICATIONS: true,
  EMAIL_NOTIFICATIONS: true,
  TWO_FACTOR_AUTH: true,
  API_ACCESS: true,
  BETA_FEATURES: false,
} as const;

// Subscription Tiers
export const SUBSCRIPTION_TIERS = {
  FREE: {
    MAX_POSITIONS: 3,
    MAX_API_CALLS: 100,
    FEATURES: ['basic_arbitrage', 'telegram_bot'],
    PRICE: 0,
  },
  PRO: {
    MAX_POSITIONS: 10,
    MAX_API_CALLS: 1000,
    FEATURES: ['advanced_arbitrage', 'technical_analysis', 'alerts'],
    PRICE: 29.99,
  },
  ULTRA: {
    MAX_POSITIONS: 50,
    MAX_API_CALLS: 10000,
    FEATURES: ['all_features', 'priority_support', 'custom_strategies'],
    PRICE: 99.99,
  },
} as const;

// Error Messages
export const ERROR_MESSAGES = {
  INVALID_INPUT: 'Invalid input provided',
  UNAUTHORIZED: 'Unauthorized access',
  NOT_FOUND: 'Resource not found',
  INTERNAL_ERROR: 'Internal server error',
  RATE_LIMITED: 'Rate limit exceeded',
  VALIDATION_FAILED: 'Validation failed',
  INSUFFICIENT_BALANCE: 'Insufficient account balance',
  POSITION_LIMIT_EXCEEDED: 'Position limit exceeded',
  INVALID_SYMBOL: 'Invalid trading symbol',
  EXCHANGE_ERROR: 'Exchange API error',
  NETWORK_ERROR: 'Network connection error',
  TIMEOUT_ERROR: 'Request timeout',
} as const;

// Success Messages
export const SUCCESS_MESSAGES = {
  CREATED: 'Resource created successfully',
  UPDATED: 'Resource updated successfully',
  DELETED: 'Resource deleted successfully',
  RETRIEVED: 'Resource retrieved successfully',
  POSITION_OPENED: 'Position opened successfully',
  POSITION_CLOSED: 'Position closed successfully',
  ORDER_PLACED: 'Order placed successfully',
  ORDER_CANCELLED: 'Order cancelled successfully',
} as const;

// Validation Rules
export const VALIDATION_RULES = {
  USERNAME: {
    MIN_LENGTH: 3,
    MAX_LENGTH: 32,
    PATTERN: /^[a-zA-Z0-9_]+$/,
  },
  TELEGRAM_ID: {
    MIN: 1,
    MAX: 9999999999, // Telegram's maximum user ID
  },
  SYMBOL: {
    MIN_LENGTH: 3,
    MAX_LENGTH: 20,
    PATTERN: /^[A-Z]{2,10}[/-]?[A-Z]{2,10}$/,
  },
  AMOUNT: {
    MIN: 0.00000001,
    MAX: 1000000000,
  },
  PERCENTAGE: {
    MIN: 0,
    MAX: 100,
  },
} as const;

// Service URLs and Endpoints
export const ENDPOINTS = {
  TELEGRAM_API: 'https://api.telegram.org/bot',
  BINANCE_API: 'https://api.binance.com/api/v3',
  BYBIT_API: 'https://api.bybit.com/v5',
  OKX_API: 'https://www.okx.com/api/v5',
  BITGET_API: 'https://api.bitget.com/api/v2',
  KUCOIN_API: 'https://api.kucoin.com/api/v1',
  GATE_API: 'https://api.gateio.ws/api/v4',
  MEXC_API: 'https://api.mexc.com/api/v3',
  HUOBI_API: 'https://api.huobi.pro/v1',
  KRAKEN_API: 'https://api.kraken.com/0/public',
  COINBASE_API: 'https://api.exchange.coinbase.com',
} as const;

// Error Codes
export const ERROR_CODES = {
  // General
  INTERNAL_ERROR: 'INTERNAL_ERROR',
  INVALID_REQUEST: 'INVALID_REQUEST',
  UNAUTHORIZED: 'UNAUTHORIZED',
  FORBIDDEN: 'FORBIDDEN',
  NOT_FOUND: 'NOT_FOUND',
  RATE_LIMITED: 'RATE_LIMITED',
  
  // User-related
  USER_NOT_FOUND: 'USER_NOT_FOUND',
  USER_ALREADY_EXISTS: 'USER_ALREADY_EXISTS',
  INVALID_CREDENTIALS: 'INVALID_CREDENTIALS',
  
  // Trading-related
  INSUFFICIENT_BALANCE: 'INSUFFICIENT_BALANCE',
  INVALID_SYMBOL: 'INVALID_SYMBOL',
  MARKET_CLOSED: 'MARKET_CLOSED',
  POSITION_NOT_FOUND: 'POSITION_NOT_FOUND',
  ORDER_FAILED: 'ORDER_FAILED',
  
  // Service-related
  SERVICE_UNAVAILABLE: 'SERVICE_UNAVAILABLE',
  DATABASE_ERROR: 'DATABASE_ERROR',
  CACHE_ERROR: 'CACHE_ERROR',
  EXTERNAL_API_ERROR: 'EXTERNAL_API_ERROR',
  EXCHANGE_ERROR: 'EXCHANGE_ERROR',
} as const;

// HTTP Status Codes
export const HTTP_STATUS = {
  OK: 200,
  CREATED: 201,
  NO_CONTENT: 204,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  CONFLICT: 409,
  UNPROCESSABLE_ENTITY: 422,
  TOO_MANY_REQUESTS: 429,
  INTERNAL_SERVER_ERROR: 500,
  SERVICE_UNAVAILABLE: 503,
} as const;

// Configuration validation
export function validateConfig(): boolean {
  // Add runtime configuration validation here
  return true;
}

// Environment-specific configurations
export function getConfigForEnvironment() {
  return {
    ENV,
    API,
    DATABASE,
    TELEGRAM,
    TRADING,
    CACHE,
    SECURITY,
    MONITORING,
    FEATURES: ENV.IS_PRODUCTION 
      ? { ...FEATURES, BETA_FEATURES: false }
      : FEATURES,
  };
}