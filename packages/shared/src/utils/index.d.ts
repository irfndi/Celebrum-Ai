export declare function formatTimestamp(date: Date | string): string;
export declare function isExpired(expiresAt: Date | string): boolean;
export declare function getTimeUntilExpiry(expiresAt: Date | string): number;
export declare function addMinutes(date: Date, minutes: number): Date;
export declare function addHours(date: Date, hours: number): Date;
export declare function addDays(date: Date, days: number): Date;
export declare function isValidTelegramId(id: number | string): boolean;
export declare function isValidEmail(email: string): boolean;
export declare function isValidSymbol(symbol: string): boolean;
export declare function isValidExchange(exchange: string): boolean;
export declare function isValidPrice(price: number): boolean;
export declare function isValidQuantity(quantity: number): boolean;
export declare function formatCurrency(amount: number, currency?: string, decimals?: number): string;
export declare function formatPercentage(value: number, decimals?: number): string;
export declare function formatNumber(value: number, decimals?: number): string;
export declare function formatLargeNumber(value: number): string;
export declare function calculateProfitPercentage(entryPrice: number, exitPrice: number, type: 'long' | 'short'): number;
export declare function calculatePnL(entryPrice: number, exitPrice: number, quantity: number, type: 'long' | 'short'): number;
export declare const generateErrorId: () => string;
export declare const extractClientId: (request: any) => string;
export declare const extractUserIdFromAuth: (authHeader: string) => string | null;
export declare const getRateLimitForRoute: (path: string, method: string, env: any) => number;
export declare const createRateLimitResponse: (limit: number, windowStart: number, windowSize?: number) => Response;
export declare const logErrorToKV: (errorId: string, error: Error, context: any, kv: KVNamespace) => Promise<void>;
export declare function calculateLeverage(position: number, collateral: number): number;
export declare function calculateLiquidationPrice(entryPrice: number, leverage: number, type: 'long' | 'short', maintenanceMargin?: number): number;
export declare function calculateArbitrageProfit(price1: number, price2: number, quantity: number, fee1?: number, fee2?: number): {
    profit: number;
    profitPercentage: number;
};
export declare function calculatePositionSize(accountBalance: number, riskPercentage: number, entryPrice: number, stopLoss: number): number;
export declare function calculateRiskReward(entryPrice: number, stopLoss: number, takeProfit: number, type: 'long' | 'short'): number;
export declare function createApiResponse<T>(success: boolean, data?: T, error?: string, message?: string): {
    success: boolean;
    data: T | undefined;
    error: string | undefined;
    message: string | undefined;
    timestamp: string;
};
export declare function createSuccessResponse<T>(data: T, message?: string): {
    success: boolean;
    data: T | undefined;
    error: string | undefined;
    message: string | undefined;
    timestamp: string;
};
export declare function createErrorResponse(error: string, message?: string): {
    success: boolean;
    data: undefined;
    error: string | undefined;
    message: string | undefined;
    timestamp: string;
};
export declare const createDetailedErrorResponse: (error: Error, errorId: string, timestamp: string) => {
    status: number;
    body: {
        details?: {
            name: string;
            message: string;
            stack: string | undefined;
        };
        error: string;
        message: string;
        errorId: string;
        timestamp: string;
    };
};
export declare function slugify(text: string): string;
export declare function truncate(text: string, length: number): string;
export declare function chunk<T>(array: T[], size: number): T[][];
export declare function unique<T>(array: T[]): T[];
export declare function omit<T extends Record<string, any>, K extends keyof T>(obj: T, keys: K[]): Omit<T, K>;
export declare function pick<T extends Record<string, any>, K extends keyof T>(obj: T, keys: K[]): Pick<T, K>;
export declare function delay(ms: number): Promise<void>;
export declare function retry<T>(fn: () => Promise<T>, attempts?: number, delayMs?: number): Promise<T>;
