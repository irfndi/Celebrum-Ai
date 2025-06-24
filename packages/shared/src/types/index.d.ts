import { z } from 'zod';
import type { D1Database, KVNamespace } from '@cloudflare/workers-types';
export interface Env {
    DB: D1Database;
    SESSIONS: KVNamespace;
}
export interface ApiResponse<T = unknown> {
    success: boolean;
    data?: T;
    error?: string;
    message?: string;
    timestamp: string;
}
export declare const UserRole: {
    FREE: "free";
    PRO: "pro";
    ULTRA: "ultra";
    ADMIN: "admin";
    SUPERADMIN: "superadmin";
};
export declare const UserStatus: {
    ACTIVE: "active";
    SUSPENDED: "suspended";
    BANNED: "banned";
};
export declare const SubscriptionTier: {
    FREE: "free";
    PRO: "pro";
    ULTRA: "ultra";
    ENTERPRISE: "enterprise";
};
export type UserRoleType = typeof UserRole[keyof typeof UserRole];
export type UserStatusType = typeof UserStatus[keyof typeof UserStatus];
export type SubscriptionTierType = typeof SubscriptionTier[keyof typeof SubscriptionTier];
export declare const Permission: {
    READ_PROFILE: "read:profile";
    UPDATE_PROFILE: "update:profile";
    TRADE_MANUAL: "trade:manual";
    TRADE_AUTO: "trade:auto";
    TRADE_VIEW_POSITIONS: "trade:view_positions";
    TRADE_MANAGE_CONFIG: "trade:manage_config";
    API_EXCHANGE_ACCESS: "api:exchange_access";
    API_AI_ACCESS: "api:ai_access";
    API_MANAGE_KEYS: "api:manage_keys";
    OPPORTUNITY_VIEW: "opportunity:view";
    OPPORTUNITY_EXECUTE: "opportunity:execute";
    OPPORTUNITY_CREATE_ALERTS: "opportunity:create_alerts";
    STRATEGY_VIEW: "strategy:view";
    STRATEGY_CREATE: "strategy:create";
    STRATEGY_EXECUTE: "strategy:execute";
    STRATEGY_BACKTEST: "strategy:backtest";
    ADMIN_USER_MANAGEMENT: "admin:user_management";
    ADMIN_SYSTEM_CONFIG: "admin:system_config";
    ADMIN_VIEW_ANALYTICS: "admin:view_analytics";
    ADMIN_MANAGE_FEATURES: "admin:manage_features";
    SUPERADMIN_FULL_ACCESS: "superadmin:full_access";
};
export type PermissionType = typeof Permission[keyof typeof Permission];
export declare const RiskLevel: {
    LOW: "low";
    MEDIUM: "medium";
    HIGH: "high";
};
export declare const PositionSizingMethod: {
    FIXED_AMOUNT: "fixed_amount";
    PERCENTAGE_OF_PORTFOLIO: "percentage_of_portfolio";
    KELLY_FORMULA: "kelly_formula";
    VOLATILITY_BASED: "volatility_based";
    RISK_PARITY: "risk_parity";
};
export type RiskLevelType = typeof RiskLevel[keyof typeof RiskLevel];
export type PositionSizingMethodType = typeof PositionSizingMethod[keyof typeof PositionSizingMethod];
export declare const UserSchema: z.ZodObject<{
    id: z.ZodNumber;
    telegramId: z.ZodString;
    firstName: z.ZodOptional<z.ZodString>;
    lastName: z.ZodOptional<z.ZodString>;
    username: z.ZodOptional<z.ZodString>;
    email: z.ZodOptional<z.ZodString>;
    role: z.ZodDefault<z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>>;
    status: z.ZodDefault<z.ZodEnum<["active", "suspended", "banned"]>>;
    createdAt: z.ZodDate;
    updatedAt: z.ZodDate;
    lastActiveAt: z.ZodOptional<z.ZodDate>;
    settings: z.ZodOptional<z.ZodObject<{
        notifications: z.ZodOptional<z.ZodBoolean>;
        theme: z.ZodOptional<z.ZodEnum<["light", "dark"]>>;
        language: z.ZodOptional<z.ZodString>;
        timezone: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        notifications?: boolean | undefined;
        theme?: "light" | "dark" | undefined;
        language?: string | undefined;
        timezone?: string | undefined;
    }, {
        notifications?: boolean | undefined;
        theme?: "light" | "dark" | undefined;
        language?: string | undefined;
        timezone?: string | undefined;
    }>>;
    apiLimits: z.ZodOptional<z.ZodObject<{
        exchangeApis: z.ZodOptional<z.ZodNumber>;
        aiApis: z.ZodOptional<z.ZodNumber>;
        maxDailyRequests: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        exchangeApis?: number | undefined;
        aiApis?: number | undefined;
        maxDailyRequests?: number | undefined;
    }, {
        exchangeApis?: number | undefined;
        aiApis?: number | undefined;
        maxDailyRequests?: number | undefined;
    }>>;
    accountBalance: z.ZodDefault<z.ZodString>;
    betaExpiresAt: z.ZodOptional<z.ZodDate>;
    tradingPreferences: z.ZodOptional<z.ZodObject<{
        percentagePerTrade: z.ZodOptional<z.ZodNumber>;
        maxConcurrentTrades: z.ZodOptional<z.ZodNumber>;
        maxLeverage: z.ZodOptional<z.ZodNumber>;
        stopLoss: z.ZodOptional<z.ZodNumber>;
        takeProfit: z.ZodOptional<z.ZodNumber>;
        riskTolerance: z.ZodOptional<z.ZodEnum<["low", "medium", "high"]>>;
        autoTrade: z.ZodOptional<z.ZodBoolean>;
    }, "strip", z.ZodTypeAny, {
        percentagePerTrade?: number | undefined;
        maxConcurrentTrades?: number | undefined;
        maxLeverage?: number | undefined;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
        riskTolerance?: "low" | "high" | "medium" | undefined;
        autoTrade?: boolean | undefined;
    }, {
        percentagePerTrade?: number | undefined;
        maxConcurrentTrades?: number | undefined;
        maxLeverage?: number | undefined;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
        riskTolerance?: "low" | "high" | "medium" | undefined;
        autoTrade?: boolean | undefined;
    }>>;
}, "strip", z.ZodTypeAny, {
    id: number;
    telegramId: string;
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    status: "active" | "suspended" | "banned";
    createdAt: Date;
    updatedAt: Date;
    accountBalance: string;
    email?: string | undefined;
    firstName?: string | undefined;
    lastName?: string | undefined;
    username?: string | undefined;
    lastActiveAt?: Date | undefined;
    settings?: {
        notifications?: boolean | undefined;
        theme?: "light" | "dark" | undefined;
        language?: string | undefined;
        timezone?: string | undefined;
    } | undefined;
    apiLimits?: {
        exchangeApis?: number | undefined;
        aiApis?: number | undefined;
        maxDailyRequests?: number | undefined;
    } | undefined;
    betaExpiresAt?: Date | undefined;
    tradingPreferences?: {
        percentagePerTrade?: number | undefined;
        maxConcurrentTrades?: number | undefined;
        maxLeverage?: number | undefined;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
        riskTolerance?: "low" | "high" | "medium" | undefined;
        autoTrade?: boolean | undefined;
    } | undefined;
}, {
    id: number;
    telegramId: string;
    createdAt: Date;
    updatedAt: Date;
    email?: string | undefined;
    firstName?: string | undefined;
    lastName?: string | undefined;
    username?: string | undefined;
    role?: "free" | "pro" | "ultra" | "admin" | "superadmin" | undefined;
    status?: "active" | "suspended" | "banned" | undefined;
    lastActiveAt?: Date | undefined;
    settings?: {
        notifications?: boolean | undefined;
        theme?: "light" | "dark" | undefined;
        language?: string | undefined;
        timezone?: string | undefined;
    } | undefined;
    apiLimits?: {
        exchangeApis?: number | undefined;
        aiApis?: number | undefined;
        maxDailyRequests?: number | undefined;
    } | undefined;
    accountBalance?: string | undefined;
    betaExpiresAt?: Date | undefined;
    tradingPreferences?: {
        percentagePerTrade?: number | undefined;
        maxConcurrentTrades?: number | undefined;
        maxLeverage?: number | undefined;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
        riskTolerance?: "low" | "high" | "medium" | undefined;
        autoTrade?: boolean | undefined;
    } | undefined;
}>;
export type User = z.infer<typeof UserSchema>;
export type NewUser = Omit<User, 'id' | 'createdAt' | 'updatedAt'>;
export declare const ExchangeId: {
    readonly BINANCE: "binance";
    readonly BYBIT: "bybit";
    readonly OKX: "okx";
    readonly BITGET: "bitget";
    readonly KUCOIN: "kucoin";
    readonly GATE: "gate";
    readonly MEXC: "mexc";
    readonly HUOBI: "huobi";
    readonly KRAKEN: "kraken";
    readonly COINBASE: "coinbase";
};
export type ExchangeIdType = typeof ExchangeId[keyof typeof ExchangeId];
export declare const PositionType: {
    readonly LONG: "long";
    readonly SHORT: "short";
};
export declare const PositionStatus: {
    readonly OPEN: "open";
    readonly CLOSED: "closed";
    readonly PARTIALLY_FILLED: "partially_filled";
    readonly CANCELLED: "cancelled";
};
export declare const TradingStrategy: {
    readonly ARBITRAGE: "arbitrage";
    readonly TECHNICAL: "technical";
    readonly MANUAL: "manual";
};
export declare const OpportunityType: {
    readonly ARBITRAGE: "arbitrage";
    readonly TECHNICAL: "technical";
};
export type PositionTypeType = typeof PositionType[keyof typeof PositionType];
export type PositionStatusType = typeof PositionStatus[keyof typeof PositionStatus];
export type TradingStrategyType = typeof TradingStrategy[keyof typeof TradingStrategy];
export type OpportunityTypeType = typeof OpportunityType[keyof typeof OpportunityType];
export declare const PositionSchema: z.ZodObject<{
    id: z.ZodNumber;
    userId: z.ZodNumber;
    exchangeId: z.ZodString;
    symbol: z.ZodString;
    type: z.ZodEnum<["long", "short"]>;
    strategy: z.ZodEnum<["arbitrage", "technical", "manual"]>;
    entryPrice: z.ZodNumber;
    exitPrice: z.ZodOptional<z.ZodNumber>;
    quantity: z.ZodNumber;
    leverage: z.ZodDefault<z.ZodNumber>;
    stopLoss: z.ZodOptional<z.ZodNumber>;
    takeProfit: z.ZodOptional<z.ZodNumber>;
    status: z.ZodDefault<z.ZodEnum<["open", "closed", "partially_filled", "cancelled"]>>;
    pnl: z.ZodDefault<z.ZodNumber>;
    fees: z.ZodDefault<z.ZodNumber>;
    metadata: z.ZodOptional<z.ZodObject<{
        fundingRate: z.ZodOptional<z.ZodNumber>;
        correlatedPositions: z.ZodOptional<z.ZodArray<z.ZodString, "many">>;
        riskScore: z.ZodOptional<z.ZodNumber>;
        autoClose: z.ZodOptional<z.ZodBoolean>;
    }, "strip", z.ZodTypeAny, {
        fundingRate?: number | undefined;
        correlatedPositions?: string[] | undefined;
        riskScore?: number | undefined;
        autoClose?: boolean | undefined;
    }, {
        fundingRate?: number | undefined;
        correlatedPositions?: string[] | undefined;
        riskScore?: number | undefined;
        autoClose?: boolean | undefined;
    }>>;
    createdAt: z.ZodDate;
    updatedAt: z.ZodDate;
    closedAt: z.ZodOptional<z.ZodDate>;
}, "strip", z.ZodTypeAny, {
    symbol: string;
    id: number;
    type: "long" | "short";
    status: "open" | "closed" | "partially_filled" | "cancelled";
    createdAt: Date;
    updatedAt: Date;
    userId: number;
    exchangeId: string;
    strategy: "manual" | "arbitrage" | "technical";
    entryPrice: number;
    quantity: number;
    leverage: number;
    pnl: number;
    fees: number;
    metadata?: {
        fundingRate?: number | undefined;
        correlatedPositions?: string[] | undefined;
        riskScore?: number | undefined;
        autoClose?: boolean | undefined;
    } | undefined;
    stopLoss?: number | undefined;
    takeProfit?: number | undefined;
    exitPrice?: number | undefined;
    closedAt?: Date | undefined;
}, {
    symbol: string;
    id: number;
    type: "long" | "short";
    createdAt: Date;
    updatedAt: Date;
    userId: number;
    exchangeId: string;
    strategy: "manual" | "arbitrage" | "technical";
    entryPrice: number;
    quantity: number;
    metadata?: {
        fundingRate?: number | undefined;
        correlatedPositions?: string[] | undefined;
        riskScore?: number | undefined;
        autoClose?: boolean | undefined;
    } | undefined;
    status?: "open" | "closed" | "partially_filled" | "cancelled" | undefined;
    stopLoss?: number | undefined;
    takeProfit?: number | undefined;
    exitPrice?: number | undefined;
    leverage?: number | undefined;
    pnl?: number | undefined;
    fees?: number | undefined;
    closedAt?: Date | undefined;
}>;
export type Position = z.infer<typeof PositionSchema>;
export type NewPosition = Omit<Position, 'id' | 'createdAt' | 'updatedAt'>;
export declare const OpportunitySchema: z.ZodObject<{
    id: z.ZodNumber;
    type: z.ZodEnum<["arbitrage", "technical"]>;
    symbol: z.ZodString;
    exchange1: z.ZodString;
    exchange2: z.ZodString;
    price1: z.ZodNumber;
    price2: z.ZodNumber;
    profitPercentage: z.ZodNumber;
    confidence: z.ZodNumber;
    expiresAt: z.ZodDate;
    isActive: z.ZodDefault<z.ZodBoolean>;
    createdAt: z.ZodDate;
}, "strip", z.ZodTypeAny, {
    symbol: string;
    id: number;
    type: "arbitrage" | "technical";
    createdAt: Date;
    exchange1: string;
    exchange2: string;
    price1: number;
    price2: number;
    profitPercentage: number;
    confidence: number;
    expiresAt: Date;
    isActive: boolean;
}, {
    symbol: string;
    id: number;
    type: "arbitrage" | "technical";
    createdAt: Date;
    exchange1: string;
    exchange2: string;
    price1: number;
    price2: number;
    profitPercentage: number;
    confidence: number;
    expiresAt: Date;
    isActive?: boolean | undefined;
}>;
export type Opportunity = z.infer<typeof OpportunitySchema>;
export type NewOpportunity = Omit<Opportunity, 'id' | 'createdAt'>;
export declare const ArbitrageOpportunitySchema: z.ZodObject<{
    id: z.ZodString;
    symbol: z.ZodString;
    exchange_a: z.ZodString;
    exchange_b: z.ZodString;
    price_a: z.ZodNumber;
    price_b: z.ZodNumber;
    profit_percentage: z.ZodNumber;
    confidence_score: z.ZodNumber;
    generated_at: z.ZodString;
    expires_at: z.ZodString;
}, "strip", z.ZodTypeAny, {
    symbol: string;
    id: string;
    exchange_a: string;
    exchange_b: string;
    price_a: number;
    price_b: number;
    profit_percentage: number;
    confidence_score: number;
    generated_at: string;
    expires_at: string;
}, {
    symbol: string;
    id: string;
    exchange_a: string;
    exchange_b: string;
    price_a: number;
    price_b: number;
    profit_percentage: number;
    confidence_score: number;
    generated_at: string;
    expires_at: string;
}>;
export type ArbitrageOpportunity = z.infer<typeof ArbitrageOpportunitySchema>;
export declare const RiskManagementConfigSchema: z.ZodObject<{
    maxDailyLossPercent: z.ZodNumber;
    maxDrawdownPercent: z.ZodNumber;
    positionSizingMethod: z.ZodEnum<["fixed_amount", "percentage_of_portfolio", "kelly_formula", "volatility_based", "risk_parity"]>;
    stopLossRequired: z.ZodBoolean;
    takeProfitRecommended: z.ZodBoolean;
    trailingStopEnabled: z.ZodBoolean;
    riskRewardRatioMin: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    maxDailyLossPercent: number;
    maxDrawdownPercent: number;
    positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
    stopLossRequired: boolean;
    takeProfitRecommended: boolean;
    trailingStopEnabled: boolean;
    riskRewardRatioMin: number;
}, {
    maxDailyLossPercent: number;
    maxDrawdownPercent: number;
    positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
    stopLossRequired: boolean;
    takeProfitRecommended: boolean;
    trailingStopEnabled: boolean;
    riskRewardRatioMin: number;
}>;
export declare const TradingConfigSchema: z.ZodObject<{
    userId: z.ZodString;
    role: z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>;
    percentagePerTrade: z.ZodNumber;
    maxConcurrentTrades: z.ZodNumber;
    maxLeverage: z.ZodNumber;
    stopLoss: z.ZodOptional<z.ZodNumber>;
    takeProfit: z.ZodOptional<z.ZodNumber>;
    riskTolerance: z.ZodEnum<["low", "medium", "high"]>;
    autoTradingEnabled: z.ZodBoolean;
    manualTradingEnabled: z.ZodBoolean;
    riskManagement: z.ZodObject<{
        maxDailyLossPercent: z.ZodNumber;
        maxDrawdownPercent: z.ZodNumber;
        positionSizingMethod: z.ZodEnum<["fixed_amount", "percentage_of_portfolio", "kelly_formula", "volatility_based", "risk_parity"]>;
        stopLossRequired: z.ZodBoolean;
        takeProfitRecommended: z.ZodBoolean;
        trailingStopEnabled: z.ZodBoolean;
        riskRewardRatioMin: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    }, {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    }>;
    lastUpdated: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    percentagePerTrade: number;
    maxConcurrentTrades: number;
    maxLeverage: number;
    riskTolerance: "low" | "high" | "medium";
    userId: string;
    autoTradingEnabled: boolean;
    manualTradingEnabled: boolean;
    riskManagement: {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    };
    lastUpdated: number;
    stopLoss?: number | undefined;
    takeProfit?: number | undefined;
}, {
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    percentagePerTrade: number;
    maxConcurrentTrades: number;
    maxLeverage: number;
    riskTolerance: "low" | "high" | "medium";
    userId: string;
    autoTradingEnabled: boolean;
    manualTradingEnabled: boolean;
    riskManagement: {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    };
    lastUpdated: number;
    stopLoss?: number | undefined;
    takeProfit?: number | undefined;
}>;
export declare const ApiAccessSchema: z.ZodObject<{
    userId: z.ZodString;
    role: z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>;
    exchangeApis: z.ZodArray<z.ZodObject<{
        exchangeId: z.ZodString;
        apiKey: z.ZodString;
        secretKey: z.ZodString;
        passphrase: z.ZodOptional<z.ZodString>;
        sandbox: z.ZodDefault<z.ZodBoolean>;
        permissions: z.ZodArray<z.ZodString, "many">;
        isActive: z.ZodDefault<z.ZodBoolean>;
        lastUsed: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        exchangeId: string;
        isActive: boolean;
        apiKey: string;
        secretKey: string;
        sandbox: boolean;
        permissions: string[];
        passphrase?: string | undefined;
        lastUsed?: number | undefined;
    }, {
        exchangeId: string;
        apiKey: string;
        secretKey: string;
        permissions: string[];
        isActive?: boolean | undefined;
        passphrase?: string | undefined;
        sandbox?: boolean | undefined;
        lastUsed?: number | undefined;
    }>, "many">;
    aiApis: z.ZodArray<z.ZodObject<{
        provider: z.ZodString;
        apiKey: z.ZodString;
        model: z.ZodOptional<z.ZodString>;
        maxTokens: z.ZodOptional<z.ZodNumber>;
        isActive: z.ZodDefault<z.ZodBoolean>;
        lastUsed: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        isActive: boolean;
        apiKey: string;
        provider: string;
        lastUsed?: number | undefined;
        model?: string | undefined;
        maxTokens?: number | undefined;
    }, {
        apiKey: string;
        provider: string;
        isActive?: boolean | undefined;
        lastUsed?: number | undefined;
        model?: string | undefined;
        maxTokens?: number | undefined;
    }>, "many">;
    limits: z.ZodObject<{
        maxExchangeApis: z.ZodNumber;
        maxAiApis: z.ZodNumber;
        dailyRequestLimit: z.ZodNumber;
        hourlyRequestLimit: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        maxExchangeApis: number;
        maxAiApis: number;
        dailyRequestLimit: number;
        hourlyRequestLimit: number;
    }, {
        maxExchangeApis: number;
        maxAiApis: number;
        dailyRequestLimit: number;
        hourlyRequestLimit: number;
    }>;
    usage: z.ZodObject<{
        dailyRequests: z.ZodDefault<z.ZodNumber>;
        hourlyRequests: z.ZodDefault<z.ZodNumber>;
        totalRequests: z.ZodDefault<z.ZodNumber>;
        lastReset: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        dailyRequests: number;
        hourlyRequests: number;
        totalRequests: number;
        lastReset: number;
    }, {
        lastReset: number;
        dailyRequests?: number | undefined;
        hourlyRequests?: number | undefined;
        totalRequests?: number | undefined;
    }>;
    lastUpdated: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    exchangeApis: {
        exchangeId: string;
        isActive: boolean;
        apiKey: string;
        secretKey: string;
        sandbox: boolean;
        permissions: string[];
        passphrase?: string | undefined;
        lastUsed?: number | undefined;
    }[];
    aiApis: {
        isActive: boolean;
        apiKey: string;
        provider: string;
        lastUsed?: number | undefined;
        model?: string | undefined;
        maxTokens?: number | undefined;
    }[];
    userId: string;
    lastUpdated: number;
    limits: {
        maxExchangeApis: number;
        maxAiApis: number;
        dailyRequestLimit: number;
        hourlyRequestLimit: number;
    };
    usage: {
        dailyRequests: number;
        hourlyRequests: number;
        totalRequests: number;
        lastReset: number;
    };
}, {
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    exchangeApis: {
        exchangeId: string;
        apiKey: string;
        secretKey: string;
        permissions: string[];
        isActive?: boolean | undefined;
        passphrase?: string | undefined;
        sandbox?: boolean | undefined;
        lastUsed?: number | undefined;
    }[];
    aiApis: {
        apiKey: string;
        provider: string;
        isActive?: boolean | undefined;
        lastUsed?: number | undefined;
        model?: string | undefined;
        maxTokens?: number | undefined;
    }[];
    userId: string;
    lastUpdated: number;
    limits: {
        maxExchangeApis: number;
        maxAiApis: number;
        dailyRequestLimit: number;
        hourlyRequestLimit: number;
    };
    usage: {
        lastReset: number;
        dailyRequests?: number | undefined;
        hourlyRequests?: number | undefined;
        totalRequests?: number | undefined;
    };
}>;
export declare const OpportunityLimitsSchema: z.ZodObject<{
    dailyLimit: z.ZodNumber;
    dailyUsed: z.ZodNumber;
    hourlyLimit: z.ZodNumber;
    hourlyUsed: z.ZodNumber;
    totalAccessed: z.ZodNumber;
    successRate: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    dailyLimit: number;
    dailyUsed: number;
    hourlyLimit: number;
    hourlyUsed: number;
    totalAccessed: number;
    successRate: number;
}, {
    dailyLimit: number;
    dailyUsed: number;
    hourlyLimit: number;
    hourlyUsed: number;
    totalAccessed: number;
    successRate: number;
}>;
export declare const StrategyLimitsSchema: z.ZodObject<{
    maxStrategies: z.ZodNumber;
    createdStrategies: z.ZodNumber;
    maxActiveStrategies: z.ZodNumber;
    activeStrategies: z.ZodNumber;
    maxConcurrentBacktests: z.ZodNumber;
    concurrentBacktests: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    maxStrategies: number;
    createdStrategies: number;
    maxActiveStrategies: number;
    activeStrategies: number;
    maxConcurrentBacktests: number;
    concurrentBacktests: number;
}, {
    maxStrategies: number;
    createdStrategies: number;
    maxActiveStrategies: number;
    activeStrategies: number;
    maxConcurrentBacktests: number;
    concurrentBacktests: number;
}>;
export declare const UserAccessSummarySchema: z.ZodObject<{
    userId: z.ZodString;
    role: z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>;
    subscriptionTier: z.ZodEnum<["free", "pro", "ultra", "enterprise"]>;
    permissions: z.ZodArray<z.ZodString, "many">;
    apiAccess: z.ZodObject<{
        userId: z.ZodString;
        role: z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>;
        exchangeApis: z.ZodArray<z.ZodObject<{
            exchangeId: z.ZodString;
            apiKey: z.ZodString;
            secretKey: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
            sandbox: z.ZodDefault<z.ZodBoolean>;
            permissions: z.ZodArray<z.ZodString, "many">;
            isActive: z.ZodDefault<z.ZodBoolean>;
            lastUsed: z.ZodOptional<z.ZodNumber>;
        }, "strip", z.ZodTypeAny, {
            exchangeId: string;
            isActive: boolean;
            apiKey: string;
            secretKey: string;
            sandbox: boolean;
            permissions: string[];
            passphrase?: string | undefined;
            lastUsed?: number | undefined;
        }, {
            exchangeId: string;
            apiKey: string;
            secretKey: string;
            permissions: string[];
            isActive?: boolean | undefined;
            passphrase?: string | undefined;
            sandbox?: boolean | undefined;
            lastUsed?: number | undefined;
        }>, "many">;
        aiApis: z.ZodArray<z.ZodObject<{
            provider: z.ZodString;
            apiKey: z.ZodString;
            model: z.ZodOptional<z.ZodString>;
            maxTokens: z.ZodOptional<z.ZodNumber>;
            isActive: z.ZodDefault<z.ZodBoolean>;
            lastUsed: z.ZodOptional<z.ZodNumber>;
        }, "strip", z.ZodTypeAny, {
            isActive: boolean;
            apiKey: string;
            provider: string;
            lastUsed?: number | undefined;
            model?: string | undefined;
            maxTokens?: number | undefined;
        }, {
            apiKey: string;
            provider: string;
            isActive?: boolean | undefined;
            lastUsed?: number | undefined;
            model?: string | undefined;
            maxTokens?: number | undefined;
        }>, "many">;
        limits: z.ZodObject<{
            maxExchangeApis: z.ZodNumber;
            maxAiApis: z.ZodNumber;
            dailyRequestLimit: z.ZodNumber;
            hourlyRequestLimit: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            maxExchangeApis: number;
            maxAiApis: number;
            dailyRequestLimit: number;
            hourlyRequestLimit: number;
        }, {
            maxExchangeApis: number;
            maxAiApis: number;
            dailyRequestLimit: number;
            hourlyRequestLimit: number;
        }>;
        usage: z.ZodObject<{
            dailyRequests: z.ZodDefault<z.ZodNumber>;
            hourlyRequests: z.ZodDefault<z.ZodNumber>;
            totalRequests: z.ZodDefault<z.ZodNumber>;
            lastReset: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            dailyRequests: number;
            hourlyRequests: number;
            totalRequests: number;
            lastReset: number;
        }, {
            lastReset: number;
            dailyRequests?: number | undefined;
            hourlyRequests?: number | undefined;
            totalRequests?: number | undefined;
        }>;
        lastUpdated: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        exchangeApis: {
            exchangeId: string;
            isActive: boolean;
            apiKey: string;
            secretKey: string;
            sandbox: boolean;
            permissions: string[];
            passphrase?: string | undefined;
            lastUsed?: number | undefined;
        }[];
        aiApis: {
            isActive: boolean;
            apiKey: string;
            provider: string;
            lastUsed?: number | undefined;
            model?: string | undefined;
            maxTokens?: number | undefined;
        }[];
        userId: string;
        lastUpdated: number;
        limits: {
            maxExchangeApis: number;
            maxAiApis: number;
            dailyRequestLimit: number;
            hourlyRequestLimit: number;
        };
        usage: {
            dailyRequests: number;
            hourlyRequests: number;
            totalRequests: number;
            lastReset: number;
        };
    }, {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        exchangeApis: {
            exchangeId: string;
            apiKey: string;
            secretKey: string;
            permissions: string[];
            isActive?: boolean | undefined;
            passphrase?: string | undefined;
            sandbox?: boolean | undefined;
            lastUsed?: number | undefined;
        }[];
        aiApis: {
            apiKey: string;
            provider: string;
            isActive?: boolean | undefined;
            lastUsed?: number | undefined;
            model?: string | undefined;
            maxTokens?: number | undefined;
        }[];
        userId: string;
        lastUpdated: number;
        limits: {
            maxExchangeApis: number;
            maxAiApis: number;
            dailyRequestLimit: number;
            hourlyRequestLimit: number;
        };
        usage: {
            lastReset: number;
            dailyRequests?: number | undefined;
            hourlyRequests?: number | undefined;
            totalRequests?: number | undefined;
        };
    }>;
    tradingConfig: z.ZodOptional<z.ZodObject<{
        userId: z.ZodString;
        role: z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>;
        percentagePerTrade: z.ZodNumber;
        maxConcurrentTrades: z.ZodNumber;
        maxLeverage: z.ZodNumber;
        stopLoss: z.ZodOptional<z.ZodNumber>;
        takeProfit: z.ZodOptional<z.ZodNumber>;
        riskTolerance: z.ZodEnum<["low", "medium", "high"]>;
        autoTradingEnabled: z.ZodBoolean;
        manualTradingEnabled: z.ZodBoolean;
        riskManagement: z.ZodObject<{
            maxDailyLossPercent: z.ZodNumber;
            maxDrawdownPercent: z.ZodNumber;
            positionSizingMethod: z.ZodEnum<["fixed_amount", "percentage_of_portfolio", "kelly_formula", "volatility_based", "risk_parity"]>;
            stopLossRequired: z.ZodBoolean;
            takeProfitRecommended: z.ZodBoolean;
            trailingStopEnabled: z.ZodBoolean;
            riskRewardRatioMin: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            maxDailyLossPercent: number;
            maxDrawdownPercent: number;
            positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
            stopLossRequired: boolean;
            takeProfitRecommended: boolean;
            trailingStopEnabled: boolean;
            riskRewardRatioMin: number;
        }, {
            maxDailyLossPercent: number;
            maxDrawdownPercent: number;
            positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
            stopLossRequired: boolean;
            takeProfitRecommended: boolean;
            trailingStopEnabled: boolean;
            riskRewardRatioMin: number;
        }>;
        lastUpdated: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        percentagePerTrade: number;
        maxConcurrentTrades: number;
        maxLeverage: number;
        riskTolerance: "low" | "high" | "medium";
        userId: string;
        autoTradingEnabled: boolean;
        manualTradingEnabled: boolean;
        riskManagement: {
            maxDailyLossPercent: number;
            maxDrawdownPercent: number;
            positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
            stopLossRequired: boolean;
            takeProfitRecommended: boolean;
            trailingStopEnabled: boolean;
            riskRewardRatioMin: number;
        };
        lastUpdated: number;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
    }, {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        percentagePerTrade: number;
        maxConcurrentTrades: number;
        maxLeverage: number;
        riskTolerance: "low" | "high" | "medium";
        userId: string;
        autoTradingEnabled: boolean;
        manualTradingEnabled: boolean;
        riskManagement: {
            maxDailyLossPercent: number;
            maxDrawdownPercent: number;
            positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
            stopLossRequired: boolean;
            takeProfitRecommended: boolean;
            trailingStopEnabled: boolean;
            riskRewardRatioMin: number;
        };
        lastUpdated: number;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
    }>>;
    opportunityLimits: z.ZodObject<{
        dailyLimit: z.ZodNumber;
        dailyUsed: z.ZodNumber;
        hourlyLimit: z.ZodNumber;
        hourlyUsed: z.ZodNumber;
        totalAccessed: z.ZodNumber;
        successRate: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        dailyLimit: number;
        dailyUsed: number;
        hourlyLimit: number;
        hourlyUsed: number;
        totalAccessed: number;
        successRate: number;
    }, {
        dailyLimit: number;
        dailyUsed: number;
        hourlyLimit: number;
        hourlyUsed: number;
        totalAccessed: number;
        successRate: number;
    }>;
    strategyLimits: z.ZodObject<{
        maxStrategies: z.ZodNumber;
        createdStrategies: z.ZodNumber;
        maxActiveStrategies: z.ZodNumber;
        activeStrategies: z.ZodNumber;
        maxConcurrentBacktests: z.ZodNumber;
        concurrentBacktests: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        maxStrategies: number;
        createdStrategies: number;
        maxActiveStrategies: number;
        activeStrategies: number;
        maxConcurrentBacktests: number;
        concurrentBacktests: number;
    }, {
        maxStrategies: number;
        createdStrategies: number;
        maxActiveStrategies: number;
        activeStrategies: number;
        maxConcurrentBacktests: number;
        concurrentBacktests: number;
    }>;
    featureFlags: z.ZodRecord<z.ZodString, z.ZodBoolean>;
    lastUpdated: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    userId: string;
    lastUpdated: number;
    permissions: string[];
    subscriptionTier: "free" | "pro" | "ultra" | "enterprise";
    apiAccess: {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        exchangeApis: {
            exchangeId: string;
            isActive: boolean;
            apiKey: string;
            secretKey: string;
            sandbox: boolean;
            permissions: string[];
            passphrase?: string | undefined;
            lastUsed?: number | undefined;
        }[];
        aiApis: {
            isActive: boolean;
            apiKey: string;
            provider: string;
            lastUsed?: number | undefined;
            model?: string | undefined;
            maxTokens?: number | undefined;
        }[];
        userId: string;
        lastUpdated: number;
        limits: {
            maxExchangeApis: number;
            maxAiApis: number;
            dailyRequestLimit: number;
            hourlyRequestLimit: number;
        };
        usage: {
            dailyRequests: number;
            hourlyRequests: number;
            totalRequests: number;
            lastReset: number;
        };
    };
    opportunityLimits: {
        dailyLimit: number;
        dailyUsed: number;
        hourlyLimit: number;
        hourlyUsed: number;
        totalAccessed: number;
        successRate: number;
    };
    strategyLimits: {
        maxStrategies: number;
        createdStrategies: number;
        maxActiveStrategies: number;
        activeStrategies: number;
        maxConcurrentBacktests: number;
        concurrentBacktests: number;
    };
    featureFlags: Record<string, boolean>;
    tradingConfig?: {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        percentagePerTrade: number;
        maxConcurrentTrades: number;
        maxLeverage: number;
        riskTolerance: "low" | "high" | "medium";
        userId: string;
        autoTradingEnabled: boolean;
        manualTradingEnabled: boolean;
        riskManagement: {
            maxDailyLossPercent: number;
            maxDrawdownPercent: number;
            positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
            stopLossRequired: boolean;
            takeProfitRecommended: boolean;
            trailingStopEnabled: boolean;
            riskRewardRatioMin: number;
        };
        lastUpdated: number;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
    } | undefined;
}, {
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
    userId: string;
    lastUpdated: number;
    permissions: string[];
    subscriptionTier: "free" | "pro" | "ultra" | "enterprise";
    apiAccess: {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        exchangeApis: {
            exchangeId: string;
            apiKey: string;
            secretKey: string;
            permissions: string[];
            isActive?: boolean | undefined;
            passphrase?: string | undefined;
            sandbox?: boolean | undefined;
            lastUsed?: number | undefined;
        }[];
        aiApis: {
            apiKey: string;
            provider: string;
            isActive?: boolean | undefined;
            lastUsed?: number | undefined;
            model?: string | undefined;
            maxTokens?: number | undefined;
        }[];
        userId: string;
        lastUpdated: number;
        limits: {
            maxExchangeApis: number;
            maxAiApis: number;
            dailyRequestLimit: number;
            hourlyRequestLimit: number;
        };
        usage: {
            lastReset: number;
            dailyRequests?: number | undefined;
            hourlyRequests?: number | undefined;
            totalRequests?: number | undefined;
        };
    };
    opportunityLimits: {
        dailyLimit: number;
        dailyUsed: number;
        hourlyLimit: number;
        hourlyUsed: number;
        totalAccessed: number;
        successRate: number;
    };
    strategyLimits: {
        maxStrategies: number;
        createdStrategies: number;
        maxActiveStrategies: number;
        activeStrategies: number;
        maxConcurrentBacktests: number;
        concurrentBacktests: number;
    };
    featureFlags: Record<string, boolean>;
    tradingConfig?: {
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        percentagePerTrade: number;
        maxConcurrentTrades: number;
        maxLeverage: number;
        riskTolerance: "low" | "high" | "medium";
        userId: string;
        autoTradingEnabled: boolean;
        manualTradingEnabled: boolean;
        riskManagement: {
            maxDailyLossPercent: number;
            maxDrawdownPercent: number;
            positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
            stopLossRequired: boolean;
            takeProfitRecommended: boolean;
            trailingStopEnabled: boolean;
            riskRewardRatioMin: number;
        };
        lastUpdated: number;
        stopLoss?: number | undefined;
        takeProfit?: number | undefined;
    } | undefined;
}>;
export declare const TechnicalStrategySchema: z.ZodObject<{
    id: z.ZodString;
    userId: z.ZodString;
    name: z.ZodString;
    description: z.ZodOptional<z.ZodString>;
    version: z.ZodString;
    yamlConfig: z.ZodString;
    isActive: z.ZodDefault<z.ZodBoolean>;
    indicators: z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        parameters: z.ZodRecord<z.ZodString, z.ZodAny>;
        timeframe: z.ZodString;
    }, "strip", z.ZodTypeAny, {
        name: string;
        parameters: Record<string, any>;
        timeframe: string;
    }, {
        name: string;
        parameters: Record<string, any>;
        timeframe: string;
    }>, "many">;
    conditions: z.ZodArray<z.ZodObject<{
        type: z.ZodEnum<["entry", "exit", "stop_loss", "take_profit"]>;
        logic: z.ZodString;
        parameters: z.ZodRecord<z.ZodString, z.ZodAny>;
    }, "strip", z.ZodTypeAny, {
        type: "entry" | "exit" | "stop_loss" | "take_profit";
        parameters: Record<string, any>;
        logic: string;
    }, {
        type: "entry" | "exit" | "stop_loss" | "take_profit";
        parameters: Record<string, any>;
        logic: string;
    }>, "many">;
    riskManagement: z.ZodObject<{
        maxDailyLossPercent: z.ZodNumber;
        maxDrawdownPercent: z.ZodNumber;
        positionSizingMethod: z.ZodEnum<["fixed_amount", "percentage_of_portfolio", "kelly_formula", "volatility_based", "risk_parity"]>;
        stopLossRequired: z.ZodBoolean;
        takeProfitRecommended: z.ZodBoolean;
        trailingStopEnabled: z.ZodBoolean;
        riskRewardRatioMin: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    }, {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    }>;
    backtestResults: z.ZodOptional<z.ZodArray<z.ZodObject<{
        id: z.ZodString;
        startDate: z.ZodString;
        endDate: z.ZodString;
        totalReturn: z.ZodNumber;
        sharpeRatio: z.ZodNumber;
        maxDrawdown: z.ZodNumber;
        winRate: z.ZodNumber;
        totalTrades: z.ZodNumber;
        createdAt: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        id: string;
        createdAt: number;
        startDate: string;
        endDate: string;
        totalReturn: number;
        sharpeRatio: number;
        maxDrawdown: number;
        winRate: number;
        totalTrades: number;
    }, {
        id: string;
        createdAt: number;
        startDate: string;
        endDate: string;
        totalReturn: number;
        sharpeRatio: number;
        maxDrawdown: number;
        winRate: number;
        totalTrades: number;
    }>, "many">>;
    createdAt: z.ZodNumber;
    updatedAt: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    id: string;
    createdAt: number;
    updatedAt: number;
    userId: string;
    isActive: boolean;
    riskManagement: {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    };
    name: string;
    version: string;
    yamlConfig: string;
    indicators: {
        name: string;
        parameters: Record<string, any>;
        timeframe: string;
    }[];
    conditions: {
        type: "entry" | "exit" | "stop_loss" | "take_profit";
        parameters: Record<string, any>;
        logic: string;
    }[];
    description?: string | undefined;
    backtestResults?: {
        id: string;
        createdAt: number;
        startDate: string;
        endDate: string;
        totalReturn: number;
        sharpeRatio: number;
        maxDrawdown: number;
        winRate: number;
        totalTrades: number;
    }[] | undefined;
}, {
    id: string;
    createdAt: number;
    updatedAt: number;
    userId: string;
    riskManagement: {
        maxDailyLossPercent: number;
        maxDrawdownPercent: number;
        positionSizingMethod: "fixed_amount" | "percentage_of_portfolio" | "kelly_formula" | "volatility_based" | "risk_parity";
        stopLossRequired: boolean;
        takeProfitRecommended: boolean;
        trailingStopEnabled: boolean;
        riskRewardRatioMin: number;
    };
    name: string;
    version: string;
    yamlConfig: string;
    indicators: {
        name: string;
        parameters: Record<string, any>;
        timeframe: string;
    }[];
    conditions: {
        type: "entry" | "exit" | "stop_loss" | "take_profit";
        parameters: Record<string, any>;
        logic: string;
    }[];
    isActive?: boolean | undefined;
    description?: string | undefined;
    backtestResults?: {
        id: string;
        createdAt: number;
        startDate: string;
        endDate: string;
        totalReturn: number;
        sharpeRatio: number;
        maxDrawdown: number;
        winRate: number;
        totalTrades: number;
    }[] | undefined;
}>;
export declare const RBACOperationResultSchema: z.ZodObject<{
    success: z.ZodBoolean;
    message: z.ZodString;
    data: z.ZodOptional<z.ZodAny>;
    timestamp: z.ZodNumber;
    errors: z.ZodOptional<z.ZodArray<z.ZodString, "many">>;
}, "strip", z.ZodTypeAny, {
    message: string;
    success: boolean;
    timestamp: number;
    data?: any;
    errors?: string[] | undefined;
}, {
    message: string;
    success: boolean;
    timestamp: number;
    data?: any;
    errors?: string[] | undefined;
}>;
export type RiskManagementConfig = z.infer<typeof RiskManagementConfigSchema>;
export type TradingConfig = z.infer<typeof TradingConfigSchema>;
export type ApiAccess = z.infer<typeof ApiAccessSchema>;
export type OpportunityLimits = z.infer<typeof OpportunityLimitsSchema>;
export type StrategyLimits = z.infer<typeof StrategyLimitsSchema>;
export type UserAccessSummary = z.infer<typeof UserAccessSummarySchema>;
export type TechnicalStrategy = z.infer<typeof TechnicalStrategySchema>;
export type RBACOperationResult = z.infer<typeof RBACOperationResultSchema>;
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
export interface Environment {
    NODE_ENV: 'development' | 'production' | 'test';
    TELEGRAM_BOT_TOKEN?: string;
    DATABASE_URL?: string;
    KV_NAMESPACE?: string;
}
export declare enum ServiceStatus {
    HEALTHY = "healthy",
    DEGRADED = "degraded",
    UNHEALTHY = "unhealthy",
    UNKNOWN = "unknown"
}
export interface ServiceHealth {
    service: string;
    status: ServiceStatus;
    timestamp: string;
    details?: Record<string, unknown>;
}
export interface CloudflareEnv {
    ArbEdgeKV: KVNamespace;
    PROD_BOT_MARKET_CACHE: KVNamespace;
    PROD_BOT_SESSION_STORE: KVNamespace;
    ArbEdgeD1: D1Database;
    ENVIRONMENT: string;
    LOG_LEVEL: string;
    RATE_LIMIT_REQUESTS_PER_MINUTE: string;
    CACHE_TTL_SECONDS: string;
    SUPER_ADMIN_USER_ID: string;
    EXCHANGES: string;
    ARBITRAGE_THRESHOLD: string;
    TELEGRAM_CHAT_ID: string;
    TELEGRAM_TEST_MODE: string;
    WEB_SERVICE_URL?: string;
    API_SERVICE_URL?: string;
    DISCORD_BOT_SERVICE_URL?: string;
    TELEGRAM_BOT_SERVICE_URL?: string;
}
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
export interface ErrorResponse {
    error: string;
    message: string;
    errorId: string;
    timestamp: string;
    details?: Record<string, any>;
}
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
export declare class ArbEdgeError extends Error {
    readonly code: string;
    readonly status: number;
    readonly details?: Record<string, unknown> | undefined;
    constructor(message: string, code: string, status?: number, details?: Record<string, unknown> | undefined);
}
export declare const Schemas: {
    readonly User: z.ZodObject<{
        id: z.ZodNumber;
        telegramId: z.ZodString;
        firstName: z.ZodOptional<z.ZodString>;
        lastName: z.ZodOptional<z.ZodString>;
        username: z.ZodOptional<z.ZodString>;
        email: z.ZodOptional<z.ZodString>;
        role: z.ZodDefault<z.ZodEnum<["free", "pro", "ultra", "admin", "superadmin"]>>;
        status: z.ZodDefault<z.ZodEnum<["active", "suspended", "banned"]>>;
        createdAt: z.ZodDate;
        updatedAt: z.ZodDate;
        lastActiveAt: z.ZodOptional<z.ZodDate>;
        settings: z.ZodOptional<z.ZodObject<{
            notifications: z.ZodOptional<z.ZodBoolean>;
            theme: z.ZodOptional<z.ZodEnum<["light", "dark"]>>;
            language: z.ZodOptional<z.ZodString>;
            timezone: z.ZodOptional<z.ZodString>;
        }, "strip", z.ZodTypeAny, {
            notifications?: boolean | undefined;
            theme?: "light" | "dark" | undefined;
            language?: string | undefined;
            timezone?: string | undefined;
        }, {
            notifications?: boolean | undefined;
            theme?: "light" | "dark" | undefined;
            language?: string | undefined;
            timezone?: string | undefined;
        }>>;
        apiLimits: z.ZodOptional<z.ZodObject<{
            exchangeApis: z.ZodOptional<z.ZodNumber>;
            aiApis: z.ZodOptional<z.ZodNumber>;
            maxDailyRequests: z.ZodOptional<z.ZodNumber>;
        }, "strip", z.ZodTypeAny, {
            exchangeApis?: number | undefined;
            aiApis?: number | undefined;
            maxDailyRequests?: number | undefined;
        }, {
            exchangeApis?: number | undefined;
            aiApis?: number | undefined;
            maxDailyRequests?: number | undefined;
        }>>;
        accountBalance: z.ZodDefault<z.ZodString>;
        betaExpiresAt: z.ZodOptional<z.ZodDate>;
        tradingPreferences: z.ZodOptional<z.ZodObject<{
            percentagePerTrade: z.ZodOptional<z.ZodNumber>;
            maxConcurrentTrades: z.ZodOptional<z.ZodNumber>;
            maxLeverage: z.ZodOptional<z.ZodNumber>;
            stopLoss: z.ZodOptional<z.ZodNumber>;
            takeProfit: z.ZodOptional<z.ZodNumber>;
            riskTolerance: z.ZodOptional<z.ZodEnum<["low", "medium", "high"]>>;
            autoTrade: z.ZodOptional<z.ZodBoolean>;
        }, "strip", z.ZodTypeAny, {
            percentagePerTrade?: number | undefined;
            maxConcurrentTrades?: number | undefined;
            maxLeverage?: number | undefined;
            stopLoss?: number | undefined;
            takeProfit?: number | undefined;
            riskTolerance?: "low" | "high" | "medium" | undefined;
            autoTrade?: boolean | undefined;
        }, {
            percentagePerTrade?: number | undefined;
            maxConcurrentTrades?: number | undefined;
            maxLeverage?: number | undefined;
            stopLoss?: number | undefined;
            takeProfit?: number | undefined;
            riskTolerance?: "low" | "high" | "medium" | undefined;
            autoTrade?: boolean | undefined;
        }>>;
    }, "strip", z.ZodTypeAny, {
        id: number;
        telegramId: string;
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
        status: "active" | "suspended" | "banned";
        createdAt: Date;
        updatedAt: Date;
        accountBalance: string;
        email?: string | undefined;
        firstName?: string | undefined;
        lastName?: string | undefined;
        username?: string | undefined;
        lastActiveAt?: Date | undefined;
        settings?: {
            notifications?: boolean | undefined;
            theme?: "light" | "dark" | undefined;
            language?: string | undefined;
            timezone?: string | undefined;
        } | undefined;
        apiLimits?: {
            exchangeApis?: number | undefined;
            aiApis?: number | undefined;
            maxDailyRequests?: number | undefined;
        } | undefined;
        betaExpiresAt?: Date | undefined;
        tradingPreferences?: {
            percentagePerTrade?: number | undefined;
            maxConcurrentTrades?: number | undefined;
            maxLeverage?: number | undefined;
            stopLoss?: number | undefined;
            takeProfit?: number | undefined;
            riskTolerance?: "low" | "high" | "medium" | undefined;
            autoTrade?: boolean | undefined;
        } | undefined;
    }, {
        id: number;
        telegramId: string;
        createdAt: Date;
        updatedAt: Date;
        email?: string | undefined;
        firstName?: string | undefined;
        lastName?: string | undefined;
        username?: string | undefined;
        role?: "free" | "pro" | "ultra" | "admin" | "superadmin" | undefined;
        status?: "active" | "suspended" | "banned" | undefined;
        lastActiveAt?: Date | undefined;
        settings?: {
            notifications?: boolean | undefined;
            theme?: "light" | "dark" | undefined;
            language?: string | undefined;
            timezone?: string | undefined;
        } | undefined;
        apiLimits?: {
            exchangeApis?: number | undefined;
            aiApis?: number | undefined;
            maxDailyRequests?: number | undefined;
        } | undefined;
        accountBalance?: string | undefined;
        betaExpiresAt?: Date | undefined;
        tradingPreferences?: {
            percentagePerTrade?: number | undefined;
            maxConcurrentTrades?: number | undefined;
            maxLeverage?: number | undefined;
            stopLoss?: number | undefined;
            takeProfit?: number | undefined;
            riskTolerance?: "low" | "high" | "medium" | undefined;
            autoTrade?: boolean | undefined;
        } | undefined;
    }>;
    readonly ArbitrageOpportunity: z.ZodObject<{
        id: z.ZodString;
        symbol: z.ZodString;
        exchange_a: z.ZodString;
        exchange_b: z.ZodString;
        price_a: z.ZodNumber;
        price_b: z.ZodNumber;
        profit_percentage: z.ZodNumber;
        confidence_score: z.ZodNumber;
        generated_at: z.ZodString;
        expires_at: z.ZodString;
    }, "strip", z.ZodTypeAny, {
        symbol: string;
        id: string;
        exchange_a: string;
        exchange_b: string;
        price_a: number;
        price_b: number;
        profit_percentage: number;
        confidence_score: number;
        generated_at: string;
        expires_at: string;
    }, {
        symbol: string;
        id: string;
        exchange_a: string;
        exchange_b: string;
        price_a: number;
        price_b: number;
        profit_percentage: number;
        confidence_score: number;
        generated_at: string;
        expires_at: string;
    }>;
};
