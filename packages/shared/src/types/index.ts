// @arb-edge/shared - Shared Types
import { z } from 'zod';

// Common API Response Types
export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

// User Role and Status Enums
export const UserRole = {
  FREE: 'free',
  PRO: 'pro', 
  ULTRA: 'ultra',
  ADMIN: 'admin',
  SUPERADMIN: 'superadmin'
} as const;

export const UserStatus = {
  ACTIVE: 'active',
  SUSPENDED: 'suspended',
  BANNED: 'banned'
} as const;

export type UserRoleType = typeof UserRole[keyof typeof UserRole];
export type UserStatusType = typeof UserStatus[keyof typeof UserStatus];

// User Types
export const UserSchema = z.object({
  id: z.number(),
  telegramId: z.string(),
  firstName: z.string().optional(),
  lastName: z.string().optional(),
  username: z.string().optional(),
  email: z.string().optional(),
  role: z.enum(['free', 'pro', 'ultra', 'admin', 'superadmin']).default('free'),
  status: z.enum(['active', 'suspended', 'banned']).default('active'),
  createdAt: z.date(),
  updatedAt: z.date(),
  lastActiveAt: z.date().optional(),
  settings: z.object({
    notifications: z.boolean().optional(),
    theme: z.enum(['light', 'dark']).optional(),
    language: z.string().optional(),
    timezone: z.string().optional(),
  }).optional(),
  apiLimits: z.object({
    exchangeApis: z.number().optional(),
    aiApis: z.number().optional(),
    maxDailyRequests: z.number().optional(),
  }).optional(),
  accountBalance: z.string().default('0.00'),
  betaExpiresAt: z.date().optional(),
  tradingPreferences: z.object({
    percentagePerTrade: z.number().optional(),
    maxConcurrentTrades: z.number().optional(),
    maxLeverage: z.number().optional(),
    stopLoss: z.number().optional(),
    takeProfit: z.number().optional(),
    riskTolerance: z.enum(['low', 'medium', 'high']).optional(),
    autoTrade: z.boolean().optional(),
  }).optional(),
});

export type User = z.infer<typeof UserSchema>;
export type NewUser = Omit<User, 'id' | 'createdAt' | 'updatedAt'>;

// Exchange Types
export const ExchangeId = {
  BINANCE: 'binance',
  BYBIT: 'bybit',
  OKX: 'okx',
  BITGET: 'bitget',
  KUCOIN: 'kucoin',
  GATE: 'gate',
  MEXC: 'mexc',
  HUOBI: 'huobi',
  KRAKEN: 'kraken',
  COINBASE: 'coinbase'
} as const;

export type ExchangeIdType = typeof ExchangeId[keyof typeof ExchangeId];

// Trading Types
export const PositionType = {
  LONG: 'long',
  SHORT: 'short'
} as const;

export const PositionStatus = {
  OPEN: 'open',
  CLOSED: 'closed',
  PARTIALLY_FILLED: 'partially_filled',
  CANCELLED: 'cancelled'
} as const;

export const TradingStrategy = {
  ARBITRAGE: 'arbitrage',
  TECHNICAL: 'technical',
  MANUAL: 'manual'
} as const;

export const OpportunityType = {
  ARBITRAGE: 'arbitrage',
  TECHNICAL: 'technical'
} as const;

export type PositionTypeType = typeof PositionType[keyof typeof PositionType];
export type PositionStatusType = typeof PositionStatus[keyof typeof PositionStatus];
export type TradingStrategyType = typeof TradingStrategy[keyof typeof TradingStrategy];
export type OpportunityTypeType = typeof OpportunityType[keyof typeof OpportunityType];

// Position Schema
export const PositionSchema = z.object({
  id: z.number(),
  userId: z.number(),
  exchangeId: z.string(),
  symbol: z.string(),
  type: z.enum(['long', 'short']),
  strategy: z.enum(['arbitrage', 'technical', 'manual']),
  entryPrice: z.number(),
  exitPrice: z.number().optional(),
  quantity: z.number(),
  leverage: z.number().default(1),
  stopLoss: z.number().optional(),
  takeProfit: z.number().optional(),
  status: z.enum(['open', 'closed', 'partially_filled', 'cancelled']).default('open'),
  pnl: z.number().default(0),
  fees: z.number().default(0),
  metadata: z.object({
    fundingRate: z.number().optional(),
    correlatedPositions: z.array(z.string()).optional(),
    riskScore: z.number().optional(),
    autoClose: z.boolean().optional(),
  }).optional(),
  createdAt: z.date(),
  updatedAt: z.date(),
  closedAt: z.date().optional(),
});

export type Position = z.infer<typeof PositionSchema>;
export type NewPosition = Omit<Position, 'id' | 'createdAt' | 'updatedAt'>;

// Opportunity Schema
export const OpportunitySchema = z.object({
  id: z.number(),
  type: z.enum(['arbitrage', 'technical']),
  symbol: z.string(),
  exchange1: z.string(),
  exchange2: z.string(),
  price1: z.number(),
  price2: z.number(),
  profitPercentage: z.number(),
  confidence: z.number(),
  expiresAt: z.date(),
  isActive: z.boolean().default(true),
  createdAt: z.date(),
});

export type Opportunity = z.infer<typeof OpportunitySchema>;
export type NewOpportunity = Omit<Opportunity, 'id' | 'createdAt'>;

// Legacy ArbitrageOpportunity for backward compatibility
export const ArbitrageOpportunitySchema = z.object({
  id: z.string(),
  symbol: z.string(),
  exchange_a: z.string(),
  exchange_b: z.string(),
  price_a: z.number(),
  price_b: z.number(),
  profit_percentage: z.number(),
  confidence_score: z.number(),
  generated_at: z.string(),
  expires_at: z.string(),
});

export type ArbitrageOpportunity = z.infer<typeof ArbitrageOpportunitySchema>;

// Telegram Bot Types
export interface TelegramUser {
  id: number;
  is_bot: boolean;
  first_name: string;
  last_name?: string;
  username?: string;
  language_code?: string;
}

export interface TelegramChat {
  id: number;
  type: 'private' | 'group' | 'supergroup' | 'channel';
  title?: string;
  username?: string;
  first_name?: string;
  last_name?: string;
}

export interface TelegramMessageEntity {
  type: string;
  offset: number;
  length: number;
  url?: string;
  user?: TelegramUser;
}

export interface TelegramMessage {
  message_id: number;
  from?: TelegramUser;
  chat: TelegramChat;
  date: number;
  text?: string;
  entities?: TelegramMessageEntity[];
}

export interface TelegramCallbackQuery {
  id: string;
  from: TelegramUser;
  message?: TelegramMessage;
  data?: string;
}

export interface TelegramUpdate {
  update_id: number;
  message?: TelegramMessage;
  edited_message?: TelegramMessage;
  channel_post?: TelegramMessage;
  edited_channel_post?: TelegramMessage;
  callback_query?: TelegramCallbackQuery;
}

export interface TelegramBotResponse {
  chat_id: number;
  text: string;
  parse_mode?: 'HTML' | 'Markdown' | 'MarkdownV2';
  reply_markup?: TelegramInlineKeyboard;
}

export interface TelegramInlineKeyboard {
  inline_keyboard: TelegramInlineKeyboardButton[][];
}

export interface TelegramInlineKeyboardButton {
  text: string;
  callback_data?: string;
  url?: string;
}

export interface TelegramHandler {
  command: string;
  description: string;
  handler: (update: TelegramUpdate, context: TelegramWebhookContext) => Promise<TelegramBotResponse | null>;
}

export interface TelegramWebhookContext {
  env: Record<string, string>;
  kv: KVNamespace;
}

export interface TelegramIntegrationConfig {
  botToken: string;
  webhookUrl: string;
  allowedUpdates?: string[];
  maxConnections?: number;
}

export interface TelegramServiceInterface {
  sendMessage(chatId: number, text: string, options?: Partial<TelegramBotResponse>): Promise<boolean>;
  setWebhook(url: string): Promise<boolean>;
  deleteWebhook(): Promise<boolean>;
  getMe(): Promise<TelegramUser | null>;
}

// Environment Configuration
export interface Environment {
  NODE_ENV: 'development' | 'production' | 'test';
  TELEGRAM_BOT_TOKEN?: string;
  DATABASE_URL?: string;
  KV_NAMESPACE?: string;
}

// Service Status Types
export enum ServiceStatus {
  HEALTHY = 'healthy',
  DEGRADED = 'degraded',
  UNHEALTHY = 'unhealthy',
  UNKNOWN = 'unknown',
}

export interface ServiceHealth {
  service: string;
  status: ServiceStatus;
  timestamp: string;
  details?: Record<string, unknown>;
}

// Cloudflare Worker Types
export interface CloudflareEnv {
  // KV Namespaces
  ArbEdgeKV: KVNamespace;
  PROD_BOT_MARKET_CACHE: KVNamespace;
  PROD_BOT_SESSION_STORE: KVNamespace;
  
  // D1 Database
  ArbEdgeD1: D1Database;
  
  // Environment Variables
  ENVIRONMENT: string;
  LOG_LEVEL: string;
  RATE_LIMIT_REQUESTS_PER_MINUTE: string;
  CACHE_TTL_SECONDS: string;
  SUPER_ADMIN_USER_ID: string;
  EXCHANGES: string;
  ARBITRAGE_THRESHOLD: string;
  TELEGRAM_CHAT_ID: string;
  TELEGRAM_TEST_MODE: string;
  
  // Service URLs
  WEB_SERVICE_URL?: string;
  API_SERVICE_URL?: string;
  DISCORD_BOT_SERVICE_URL?: string;
  TELEGRAM_BOT_SERVICE_URL?: string;
}

// Rate Limiting Types
export interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
  keyGenerator?: (request: any) => string;
  skipSuccessfulRequests?: boolean;
}

export interface RateLimitInfo {
  limit: number;
  remaining: number;
  reset: number;
  retryAfter?: number;
}

// Error Response Types
export interface ErrorResponse {
  error: string;
  message: string;
  errorId: string;
  timestamp: string;
  details?: Record<string, any>;
}

// Health Check Types
export interface HealthCheckResult {
  status: 'healthy' | 'degraded' | 'unhealthy' | 'not_configured';
  message: string;
  latency?: string | null;
  url?: string | null;
}

export interface HealthCheckResponse {
  status: 'healthy' | 'degraded' | 'unhealthy' | 'error';
  timestamp: string;
  responseTime: string;
  version: string;
  environment: string;
  checks: Record<string, any>;
  uptime: string;
  worker?: {
    region: string;
    colo: string;
  };
  error?: string;
}

// Error Types
export class ArbEdgeError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly status: number = 500,
    public readonly details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'ArbEdgeError';
  }
}

// Export all schemas for validation
export const Schemas = {
  User: UserSchema,
  ArbitrageOpportunity: ArbitrageOpportunitySchema,
} as const;