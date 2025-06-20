import { ERROR_CODES, HTTP_STATUS } from '../config';
export declare class AppError extends Error {
    readonly statusCode: number;
    readonly code: string;
    readonly isOperational: boolean;
    readonly timestamp: string;
    readonly details?: Record<string, unknown>;
    constructor(message: string, statusCode?: number, code?: string, isOperational?: boolean, details?: Record<string, unknown>);
    toJSON(): {
        name: string;
        message: string;
        statusCode: number;
        code: string;
        timestamp: string;
        details: Record<string, unknown> | undefined;
        stack: string | undefined;
    };
}
export declare class ValidationError extends AppError {
    constructor(message: string, details?: Record<string, unknown>);
}
export declare class AuthenticationError extends AppError {
    constructor(message?: string);
}
export declare class AuthorizationError extends AppError {
    constructor(message?: string);
}
export declare class NotFoundError extends AppError {
    constructor(resource?: string);
}
export declare class RateLimitError extends AppError {
    constructor(message?: string);
}
export declare class DatabaseError extends AppError {
    constructor(message: string, details?: Record<string, unknown>);
}
export declare class ExternalAPIError extends AppError {
    constructor(service: string, message: string, details?: Record<string, unknown>);
}
export declare class TradingError extends AppError {
    constructor(message: string, statusCode?: number, code?: string, details?: Record<string, unknown>);
}
export declare class InsufficientBalanceError extends TradingError {
    constructor(required: number, available: number);
}
export declare class InvalidSymbolError extends TradingError {
    constructor(symbol: string);
}
export declare class ExchangeError extends AppError {
    constructor(exchange: string, message: string, details?: Record<string, unknown>);
}
export declare class NetworkError extends AppError {
    constructor(message?: string, details?: Record<string, unknown>);
}
export declare class TimeoutError extends AppError {
    constructor(operation: string, timeout: number);
}
export declare const createValidationError: (field: string, value: unknown, rule: string) => ValidationError;
export declare const createNotFoundError: (resource: string, id: string | number) => NotFoundError;
export declare const createDuplicateError: (resource: string, field: string, value: unknown) => AppError;
export declare const handleError: (error: unknown) => AppError;
export declare const formatErrorResponse: (error: AppError) => {
    success: boolean;
    error: {
        details?: Record<string, unknown>;
        message: string;
        code: string;
        statusCode: number;
        timestamp: string;
    };
};
export declare const logError: (error: AppError, context?: Record<string, unknown>) => void;
export declare const isAppError: (error: unknown) => error is AppError;
export declare const isOperationalError: (error: unknown) => boolean;
export type ErrorCode = typeof ERROR_CODES[keyof typeof ERROR_CODES];
export type HttpStatusCode = typeof HTTP_STATUS[keyof typeof HTTP_STATUS];
