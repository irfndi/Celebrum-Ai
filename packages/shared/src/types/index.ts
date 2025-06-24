// @celebrum-ai/shared - Shared Types
import { z } from 'zod';
import type { D1Database, KVNamespace } from '@cloudflare/workers-types';

export interface Env {
  DB: D1Database;
  SESSIONS: KVNamespace;
}

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
  FREE: 'free' as const,
  PRO: 'pro' as const,
  ULTRA: 'ultra' as const,
  ADMIN: 'admin' as const,
  SUPERADMIN: 'superadmin' as const
};

export const UserStatus = {
  ACTIVE: 'active' as const,
  SUSPENDED: 'suspended' as const,
  BANNED: 'banned' as const
};

export const SubscriptionTier = {
  FREE: 'free' as const,
  PRO: 'pro' as const,
  ULTRA: 'ultra' as const,
  ENTERPRISE: 'enterprise' as const
};

export type UserRoleType = typeof UserRole[keyof typeof UserRole];
export type UserStatusType = typeof UserStatus[keyof typeof UserStatus];
export type SubscriptionTierType = typeof SubscriptionTier[keyof typeof SubscriptionTier];

// RBAC Permission Types
export const Permission = {
  // Basic permissions
  READ_PROFILE: 'read:profile' as const,
  UPDATE_PROFILE: 'update:profile' as const,
  
  // Trading permissions
  TRADE_MANUAL: 'trade:manual' as const,
  TRADE_AUTO: 'trade:auto' as const,
  TRADE_VIEW_POSITIONS: 'trade:view_positions' as const,
  TRADE_MANAGE_CONFIG: 'trade:manage_config' as const,
  
  // API permissions
  API_EXCHANGE_ACCESS: 'api:exchange_access' as const,
  API_AI_ACCESS: 'api:ai_access' as const,
  API_MANAGE_KEYS: 'api:manage_keys' as const,
  
  // Opportunity permissions
  OPPORTUNITY_VIEW: 'opportunity:view' as const,
  OPPORTUNITY_EXECUTE: 'opportunity:execute' as const,
  OPPORTUNITY_CREATE_ALERTS: 'opportunity:create_alerts' as const,
  
  // Strategy permissions
  STRATEGY_VIEW: 'strategy:view' as const,
  STRATEGY_CREATE: 'strategy:create' as const,
  STRATEGY_EXECUTE: 'strategy:execute' as const,
  STRATEGY_BACKTEST: 'strategy:backtest' as const,
  
  // Admin permissions
  ADMIN_USER_MANAGEMENT: 'admin:user_management' as const,
  ADMIN_SYSTEM_CONFIG: 'admin:system_config' as const,
  ADMIN_VIEW_ANALYTICS: 'admin:view_analytics' as const,
  ADMIN_MANAGE_FEATURES: 'admin:manage_features' as const,
  
  // Super admin permissions
  SUPERADMIN_FULL_ACCESS: 'superadmin:full_access' as const
};

export type PermissionType = typeof Permission[keyof typeof Permission];

// Risk Management Types
export const RiskLevel = {
  LOW: 'low' as const,
  MEDIUM: 'medium' as const,
  HIGH: 'high' as const
};

export const PositionSizingMethod = {
  FIXED_AMOUNT: 'fixed_amount' as const,
  PERCENTAGE_OF_PORTFOLIO: 'percentage_of_portfolio' as const,
  KELLY_FORMULA: 'kelly_formula' as const,
  VOLATILITY_BASED: 'volatility_based' as const,
  RISK_PARITY: 'risk_parity' as const
};

export type RiskLevelType = typeof RiskLevel[keyof typeof RiskLevel];
export type PositionSizingMethodType = typeof PositionSizingMethod[keyof typeof PositionSizingMethod];

// User Types
export const UserSchema = z.object({
  id: z.number(),
  telegramId: z.string(),
  firstName: z.string().optional(),
  lastName: z.string().optional(),
  username: z.string().optional(),
  languageCode: z.string().optional(),
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

// RBAC and Trading Configuration Types
export const RiskManagementConfigSchema = z.object({
  maxDailyLossPercent: z.number().min(0).max(100),
  maxDrawdownPercent: z.number().min(0).max(100),
  positionSizingMethod: z.enum(['fixed_amount', 'percentage_of_portfolio', 'kelly_formula', 'volatility_based', 'risk_parity']),
  stopLossRequired: z.boolean(),
  takeProfitRecommended: z.boolean(),
  trailingStopEnabled: z.boolean(),
  riskRewardRatioMin: z.number().min(0),
});

export const TradingConfigSchema = z.object({
  userId: z.string(),
  role: z.enum(['free', 'pro', 'ultra', 'admin', 'superadmin']),
  percentagePerTrade: z.number().min(0).max(100),
  maxConcurrentTrades: z.number().min(1).max(50),
  maxLeverage: z.number().min(1).max(100),
  stopLoss: z.number().optional(),
  takeProfit: z.number().optional(),
  riskTolerance: z.enum(['low', 'medium', 'high']),
  autoTradingEnabled: z.boolean(),
  manualTradingEnabled: z.boolean(),
  riskManagement: RiskManagementConfigSchema,
  lastUpdated: z.number(),
});

export const ApiAccessSchema = z.object({
  userId: z.string(),
  role: z.enum(['free', 'pro', 'ultra', 'admin', 'superadmin']),
  exchangeApis: z.array(z.object({
    exchangeId: z.string(),
    apiKey: z.string(),
    secretKey: z.string(),
    passphrase: z.string().optional(),
    sandbox: z.boolean().default(false),
    permissions: z.array(z.string()),
    isActive: z.boolean().default(true),
    lastUsed: z.number().optional(),
  })),
  aiApis: z.array(z.object({
    provider: z.string(),
    apiKey: z.string(),
    model: z.string().optional(),
    maxTokens: z.number().optional(),
    isActive: z.boolean().default(true),
    lastUsed: z.number().optional(),
  })),
  limits: z.object({
    maxExchangeApis: z.number(),
    maxAiApis: z.number(),
    dailyRequestLimit: z.number(),
    hourlyRequestLimit: z.number(),
  }),
  usage: z.object({
    dailyRequests: z.number().default(0),
    hourlyRequests: z.number().default(0),
    totalRequests: z.number().default(0),
    lastReset: z.number(),
  }),
  lastUpdated: z.number(),
});

export const OpportunityLimitsSchema = z.object({
  dailyLimit: z.number(),
  dailyUsed: z.number(),
  hourlyLimit: z.number(),
  hourlyUsed: z.number(),
  totalAccessed: z.number(),
  successRate: z.number().min(0).max(1),
});

export const StrategyLimitsSchema = z.object({
  maxStrategies: z.number(),
  createdStrategies: z.number(),
  maxActiveStrategies: z.number(),
  activeStrategies: z.number(),
  maxConcurrentBacktests: z.number(),
  concurrentBacktests: z.number(),
});

export const UserAccessSummarySchema = z.object({
  userId: z.string(),
  role: z.enum(['free', 'pro', 'ultra', 'admin', 'superadmin']),
  subscriptionTier: z.enum(['free', 'pro', 'ultra', 'enterprise']),
  permissions: z.array(z.string()),
  apiAccess: ApiAccessSchema,
  tradingConfig: TradingConfigSchema.optional(),
  opportunityLimits: OpportunityLimitsSchema,
  strategyLimits: StrategyLimitsSchema,
  featureFlags: z.record(z.boolean()),
  lastUpdated: z.number(),
});

export const TechnicalStrategySchema = z.object({
  id: z.string(),
  userId: z.string(),
  name: z.string(),
  description: z.string().optional(),
  version: z.string(),
  yamlConfig: z.string(), // YAML strategy configuration
  isActive: z.boolean().default(false),
  indicators: z.array(z.object({
    name: z.string(),
    parameters: z.record(z.any()),
    timeframe: z.string(),
  })),
  conditions: z.array(z.object({
    type: z.enum(['entry', 'exit', 'stop_loss', 'take_profit']),
    logic: z.string(),
    parameters: z.record(z.any()),
  })),
  riskManagement: RiskManagementConfigSchema,
  backtestResults: z.array(z.object({
    id: z.string(),
    startDate: z.string(),
    endDate: z.string(),
    totalReturn: z.number(),
    sharpeRatio: z.number(),
    maxDrawdown: z.number(),
    winRate: z.number(),
    totalTrades: z.number(),
    createdAt: z.number(),
  })).optional(),
  createdAt: z.number(),
  updatedAt: z.number(),
});

export const RBACOperationResultSchema = z.object({
  success: z.boolean(),
  message: z.string(),
  data: z.any().optional(),
  timestamp: z.number(),
  errors: z.array(z.string()).optional(),
});

// Export inferred types
export type RiskManagementConfig = z.infer<typeof RiskManagementConfigSchema>;
export type TradingConfig = z.infer<typeof TradingConfigSchema>;
export type ApiAccess = z.infer<typeof ApiAccessSchema>;
export type OpportunityLimits = z.infer<typeof OpportunityLimitsSchema>;
export type StrategyLimits = z.infer<typeof StrategyLimitsSchema>;
export type UserAccessSummary = z.infer<typeof UserAccessSummarySchema>;
export type TechnicalStrategy = z.infer<typeof TechnicalStrategySchema>;
export type RBACOperationResult = z.infer<typeof RBACOperationResultSchema>;

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