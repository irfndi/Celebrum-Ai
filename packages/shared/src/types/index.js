// @celebrum-ai/shared - Shared Types
import { z } from 'zod';
// User Role and Status Enums
export const UserRole = {
    FREE: 'free',
    PRO: 'pro',
    ULTRA: 'ultra',
    ADMIN: 'admin',
    SUPERADMIN: 'superadmin'
};
export const UserStatus = {
    ACTIVE: 'active',
    SUSPENDED: 'suspended',
    BANNED: 'banned'
};
export const SubscriptionTier = {
    FREE: 'free',
    PRO: 'pro',
    ULTRA: 'ultra',
    ENTERPRISE: 'enterprise'
};
// RBAC Permission Types
export const Permission = {
    // Basic permissions
    READ_PROFILE: 'read:profile',
    UPDATE_PROFILE: 'update:profile',
    // Trading permissions
    TRADE_MANUAL: 'trade:manual',
    TRADE_AUTO: 'trade:auto',
    TRADE_VIEW_POSITIONS: 'trade:view_positions',
    TRADE_MANAGE_CONFIG: 'trade:manage_config',
    // API permissions
    API_EXCHANGE_ACCESS: 'api:exchange_access',
    API_AI_ACCESS: 'api:ai_access',
    API_MANAGE_KEYS: 'api:manage_keys',
    // Opportunity permissions
    OPPORTUNITY_VIEW: 'opportunity:view',
    OPPORTUNITY_EXECUTE: 'opportunity:execute',
    OPPORTUNITY_CREATE_ALERTS: 'opportunity:create_alerts',
    // Strategy permissions
    STRATEGY_VIEW: 'strategy:view',
    STRATEGY_CREATE: 'strategy:create',
    STRATEGY_EXECUTE: 'strategy:execute',
    STRATEGY_BACKTEST: 'strategy:backtest',
    // Admin permissions
    ADMIN_USER_MANAGEMENT: 'admin:user_management',
    ADMIN_SYSTEM_CONFIG: 'admin:system_config',
    ADMIN_VIEW_ANALYTICS: 'admin:view_analytics',
    ADMIN_MANAGE_FEATURES: 'admin:manage_features',
    // Super admin permissions
    SUPERADMIN_FULL_ACCESS: 'superadmin:full_access'
};
// Risk Management Types
export const RiskLevel = {
    LOW: 'low',
    MEDIUM: 'medium',
    HIGH: 'high'
};
export const PositionSizingMethod = {
    FIXED_AMOUNT: 'fixed_amount',
    PERCENTAGE_OF_PORTFOLIO: 'percentage_of_portfolio',
    KELLY_FORMULA: 'kelly_formula',
    VOLATILITY_BASED: 'volatility_based',
    RISK_PARITY: 'risk_parity'
};
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
};
// Trading Types
export const PositionType = {
    LONG: 'long',
    SHORT: 'short'
};
export const PositionStatus = {
    OPEN: 'open',
    CLOSED: 'closed',
    PARTIALLY_FILLED: 'partially_filled',
    CANCELLED: 'cancelled'
};
export const TradingStrategy = {
    ARBITRAGE: 'arbitrage',
    TECHNICAL: 'technical',
    MANUAL: 'manual'
};
export const OpportunityType = {
    ARBITRAGE: 'arbitrage',
    TECHNICAL: 'technical'
};
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
// Service Status Types
export var ServiceStatus;
(function (ServiceStatus) {
    ServiceStatus["HEALTHY"] = "healthy";
    ServiceStatus["DEGRADED"] = "degraded";
    ServiceStatus["UNHEALTHY"] = "unhealthy";
    ServiceStatus["UNKNOWN"] = "unknown";
})(ServiceStatus || (ServiceStatus = {}));
// Error Types
export class ArbEdgeError extends Error {
    code;
    status;
    details;
    constructor(message, code, status = 500, details) {
        super(message);
        this.code = code;
        this.status = status;
        this.details = details;
        this.name = 'ArbEdgeError';
    }
}
// Export all schemas for validation
export const Schemas = {
    User: UserSchema,
    ArbitrageOpportunity: ArbitrageOpportunitySchema,
};
