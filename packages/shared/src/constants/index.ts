// Application Constants
export const APP_NAME = 'ArbEdge';
export const APP_VERSION = '1.0.0';
export const APP_DESCRIPTION = 'Advanced Arbitrage Trading Platform';

// Time Constants (in milliseconds)
export const TIME = {
  SECOND: 1000,
  MINUTE: 60 * 1000,
  HOUR: 60 * 60 * 1000,
  DAY: 24 * 60 * 60 * 1000,
  WEEK: 7 * 24 * 60 * 60 * 1000,
  MONTH: 30 * 24 * 60 * 60 * 1000,
  YEAR: 365 * 24 * 60 * 60 * 1000,
} as const;

// Supported Cryptocurrencies
export const CRYPTOCURRENCIES = {
  BTC: 'Bitcoin',
  ETH: 'Ethereum',
  BNB: 'Binance Coin',
  ADA: 'Cardano',
  SOL: 'Solana',
  XRP: 'Ripple',
  DOT: 'Polkadot',
  DOGE: 'Dogecoin',
  AVAX: 'Avalanche',
  SHIB: 'Shiba Inu',
  MATIC: 'Polygon',
  LTC: 'Litecoin',
  UNI: 'Uniswap',
  LINK: 'Chainlink',
  ATOM: 'Cosmos',
} as const;

// Supported Fiat Currencies
export const FIAT_CURRENCIES = {
  USD: 'US Dollar',
  EUR: 'Euro',
  GBP: 'British Pound',
  JPY: 'Japanese Yen',
  AUD: 'Australian Dollar',
  CAD: 'Canadian Dollar',
  CHF: 'Swiss Franc',
  CNY: 'Chinese Yuan',
  KRW: 'South Korean Won',
  INR: 'Indian Rupee',
} as const;

// Trading Pairs
export const POPULAR_PAIRS = [
  'BTC/USDT',
  'ETH/USDT',
  'BNB/USDT',
  'ADA/USDT',
  'SOL/USDT',
  'XRP/USDT',
  'DOT/USDT',
  'DOGE/USDT',
  'AVAX/USDT',
  'MATIC/USDT',
] as const;

// Exchange Information
export const EXCHANGE_INFO = {
  binance: {
    name: 'Binance',
    website: 'https://www.binance.com',
    api: 'https://api.binance.com',
    fees: { maker: 0.001, taker: 0.001 },
    countries: ['Global'],
    founded: 2017,
  },
  bybit: {
    name: 'Bybit',
    website: 'https://www.bybit.com',
    api: 'https://api.bybit.com',
    fees: { maker: 0.001, taker: 0.001 },
    countries: ['Global'],
    founded: 2018,
  },
  okx: {
    name: 'OKX',
    website: 'https://www.okx.com',
    api: 'https://www.okx.com/api',
    fees: { maker: 0.0008, taker: 0.001 },
    countries: ['Global'],
    founded: 2017,
  },
  bitget: {
    name: 'Bitget',
    website: 'https://www.bitget.com',
    api: 'https://api.bitget.com',
    fees: { maker: 0.001, taker: 0.001 },
    countries: ['Global'],
    founded: 2018,
  },
  kucoin: {
    name: 'KuCoin',
    website: 'https://www.kucoin.com',
    api: 'https://api.kucoin.com',
    fees: { maker: 0.001, taker: 0.001 },
    countries: ['Global'],
    founded: 2017,
  },
} as const;

// Notification Types
export const NOTIFICATION_TYPES = {
  OPPORTUNITY_FOUND: 'opportunity_found',
  POSITION_OPENED: 'position_opened',
  POSITION_CLOSED: 'position_closed',
  PROFIT_TARGET_HIT: 'profit_target_hit',
  STOP_LOSS_HIT: 'stop_loss_hit',
  BALANCE_LOW: 'balance_low',
  SYSTEM_ALERT: 'system_alert',
  MAINTENANCE: 'maintenance',
} as const;

// WebSocket Events
export const WS_EVENTS = {
  CONNECT: 'connect',
  DISCONNECT: 'disconnect',
  ERROR: 'error',
  PRICE_UPDATE: 'price_update',
  OPPORTUNITY_UPDATE: 'opportunity_update',
  POSITION_UPDATE: 'position_update',
  BALANCE_UPDATE: 'balance_update',
  NOTIFICATION: 'notification',
  HEARTBEAT: 'heartbeat',
} as const;

// API Rate Limits (requests per minute)
export const RATE_LIMITS = {
  FREE_TIER: 100,
  PRO_TIER: 500,
  ULTRA_TIER: 2000,
  ADMIN: 10000,
} as const;

// File Upload Limits
export const UPLOAD_LIMITS = {
  MAX_FILE_SIZE: 10 * 1024 * 1024, // 10MB
  ALLOWED_TYPES: ['image/jpeg', 'image/png', 'image/gif', 'application/pdf'],
  MAX_FILES: 5,
} as const;

// Pagination Defaults
export const PAGINATION = {
  DEFAULT_PAGE: 1,
  DEFAULT_LIMIT: 20,
  MAX_LIMIT: 100,
} as const;

// Regular Expressions
export const REGEX = {
  EMAIL: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
  PHONE: /^\+?[1-9]\d{1,14}$/,
  USERNAME: /^[a-zA-Z0-9_]{3,20}$/,
  PASSWORD: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[a-zA-Z\d@$!%*?&]{8,}$/,
  CRYPTO_ADDRESS: /^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$|^0x[a-fA-F0-9]{40}$/,
  SYMBOL: /^[A-Z]{2,10}[/\-]?[A-Z]{2,10}$/,
  TELEGRAM_USERNAME: /^@[a-zA-Z0-9_]{5,32}$/,
} as const;

// Color Schemes
export const COLORS = {
  SUCCESS: '#10B981',
  ERROR: '#EF4444',
  WARNING: '#F59E0B',
  INFO: '#3B82F6',
  PRIMARY: '#6366F1',
  SECONDARY: '#8B5CF6',
  NEUTRAL: '#6B7280',
  PROFIT: '#10B981',
  LOSS: '#EF4444',
} as const;

// Chart Timeframes
export const TIMEFRAMES = {
  '1m': '1 Minute',
  '5m': '5 Minutes',
  '15m': '15 Minutes',
  '30m': '30 Minutes',
  '1h': '1 Hour',
  '4h': '4 Hours',
  '1d': '1 Day',
  '1w': '1 Week',
  '1M': '1 Month',
} as const;

// Default Settings
export const DEFAULT_SETTINGS = {
  THEME: 'dark',
  LANGUAGE: 'en',
  TIMEZONE: 'UTC',
  CURRENCY: 'USD',
  NOTIFICATIONS: true,
  SOUND_ALERTS: false,
  AUTO_REFRESH: true,
  REFRESH_INTERVAL: 30000, // 30 seconds
} as const;

// Feature Availability by Tier
export const TIER_FEATURES = {
  free: [
    'basic_arbitrage',
    'telegram_bot',
    'price_alerts',
    'basic_charts',
  ],
  pro: [
    'advanced_arbitrage',
    'technical_analysis',
    'custom_alerts',
    'advanced_charts',
    'portfolio_tracking',
    'api_access',
  ],
  ultra: [
    'all_features',
    'priority_support',
    'custom_strategies',
    'white_label',
    'dedicated_server',
    'advanced_analytics',
  ],
} as const;

export type CryptocurrencyType = keyof typeof CRYPTOCURRENCIES;
export type FiatCurrencyType = keyof typeof FIAT_CURRENCIES;
export type ExchangeType = keyof typeof EXCHANGE_INFO;
export type NotificationTypeType = typeof NOTIFICATION_TYPES[keyof typeof NOTIFICATION_TYPES];
export type WSEventType = typeof WS_EVENTS[keyof typeof WS_EVENTS];
export type TimeframeType = keyof typeof TIMEFRAMES;
export type TierType = keyof typeof TIER_FEATURES;