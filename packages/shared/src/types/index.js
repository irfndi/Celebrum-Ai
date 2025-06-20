// @arb-edge/shared - Shared Types
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
