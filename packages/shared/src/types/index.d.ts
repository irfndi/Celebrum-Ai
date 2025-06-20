import { z } from 'zod';
export interface ApiResponse<T = unknown> {
    success: boolean;
    data?: T;
    error?: string;
    message?: string;
    timestamp: string;
}
export declare const UserRole: {
    readonly FREE: "free";
    readonly PRO: "pro";
    readonly ULTRA: "ultra";
    readonly ADMIN: "admin";
    readonly SUPERADMIN: "superadmin";
};
export declare const UserStatus: {
    readonly ACTIVE: "active";
    readonly SUSPENDED: "suspended";
    readonly BANNED: "banned";
};
export type UserRoleType = typeof UserRole[keyof typeof UserRole];
export type UserStatusType = typeof UserStatus[keyof typeof UserStatus];
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
    status: "active" | "suspended" | "banned";
    id: number;
    telegramId: string;
    role: "free" | "pro" | "ultra" | "admin" | "superadmin";
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
    status?: "active" | "suspended" | "banned" | undefined;
    email?: string | undefined;
    firstName?: string | undefined;
    lastName?: string | undefined;
    username?: string | undefined;
    role?: "free" | "pro" | "ultra" | "admin" | "superadmin" | undefined;
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
    status: "open" | "closed" | "partially_filled" | "cancelled";
    id: number;
    type: "long" | "short";
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
    status?: "open" | "closed" | "partially_filled" | "cancelled" | undefined;
    metadata?: {
        fundingRate?: number | undefined;
        correlatedPositions?: string[] | undefined;
        riskScore?: number | undefined;
        autoClose?: boolean | undefined;
    } | undefined;
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
        status: "active" | "suspended" | "banned";
        id: number;
        telegramId: string;
        role: "free" | "pro" | "ultra" | "admin" | "superadmin";
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
        status?: "active" | "suspended" | "banned" | undefined;
        email?: string | undefined;
        firstName?: string | undefined;
        lastName?: string | undefined;
        username?: string | undefined;
        role?: "free" | "pro" | "ultra" | "admin" | "superadmin" | undefined;
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
