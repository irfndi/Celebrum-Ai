import { ERROR_CODES, HTTP_STATUS } from '../config';
// Base Error Class
export class AppError extends Error {
    statusCode;
    code;
    isOperational;
    timestamp;
    details;
    constructor(message, statusCode = HTTP_STATUS.INTERNAL_SERVER_ERROR, code = ERROR_CODES.INTERNAL_ERROR, isOperational = true, details) {
        super(message);
        this.statusCode = statusCode;
        this.code = code;
        this.isOperational = isOperational;
        this.timestamp = new Date().toISOString();
        this.details = details;
        // Maintains proper stack trace for where our error was thrown
        Error.captureStackTrace(this, this.constructor);
    }
    toJSON() {
        return {
            name: this.name,
            message: this.message,
            statusCode: this.statusCode,
            code: this.code,
            timestamp: this.timestamp,
            details: this.details,
            stack: this.stack,
        };
    }
}
// Validation Error
export class ValidationError extends AppError {
    constructor(message, details) {
        super(message, HTTP_STATUS.BAD_REQUEST, ERROR_CODES.INVALID_REQUEST, true, details);
        this.name = 'ValidationError';
    }
}
// Authentication Error
export class AuthenticationError extends AppError {
    constructor(message = 'Authentication failed') {
        super(message, HTTP_STATUS.UNAUTHORIZED, ERROR_CODES.UNAUTHORIZED, true);
        this.name = 'AuthenticationError';
    }
}
// Authorization Error
export class AuthorizationError extends AppError {
    constructor(message = 'Access denied') {
        super(message, HTTP_STATUS.FORBIDDEN, ERROR_CODES.FORBIDDEN, true);
        this.name = 'AuthorizationError';
    }
}
// Not Found Error
export class NotFoundError extends AppError {
    constructor(resource = 'Resource') {
        super(`${resource} not found`, HTTP_STATUS.NOT_FOUND, ERROR_CODES.NOT_FOUND, true);
        this.name = 'NotFoundError';
    }
}
// Rate Limit Error
export class RateLimitError extends AppError {
    constructor(message = 'Rate limit exceeded') {
        super(message, HTTP_STATUS.TOO_MANY_REQUESTS, ERROR_CODES.RATE_LIMITED, true);
        this.name = 'RateLimitError';
    }
}
// Database Error
export class DatabaseError extends AppError {
    constructor(message, details) {
        super(message, HTTP_STATUS.INTERNAL_SERVER_ERROR, ERROR_CODES.DATABASE_ERROR, true, details);
        this.name = 'DatabaseError';
    }
}
// External API Error
export class ExternalAPIError extends AppError {
    constructor(service, message, details) {
        super(`${service} API error: ${message}`, HTTP_STATUS.SERVICE_UNAVAILABLE, ERROR_CODES.EXTERNAL_API_ERROR, true, details);
        this.name = 'ExternalAPIError';
    }
}
// Trading Error
export class TradingError extends AppError {
    constructor(message, statusCode = HTTP_STATUS.UNPROCESSABLE_ENTITY, code = ERROR_CODES.ORDER_FAILED, details) {
        super(message, statusCode, code, true, details);
        this.name = 'TradingError';
    }
}
// Insufficient Balance Error
export class InsufficientBalanceError extends TradingError {
    constructor(required, available) {
        super('Insufficient balance for this operation', HTTP_STATUS.BAD_REQUEST, ERROR_CODES.INSUFFICIENT_BALANCE, {
            required,
            available,
            shortfall: required - available,
        });
        this.name = 'InsufficientBalanceError';
    }
}
// Invalid Symbol Error
export class InvalidSymbolError extends TradingError {
    constructor(symbol) {
        super(`Invalid trading symbol: ${symbol}`, HTTP_STATUS.BAD_REQUEST, ERROR_CODES.INVALID_SYMBOL, { symbol });
        this.name = 'InvalidSymbolError';
    }
}
// Exchange Error
export class ExchangeError extends AppError {
    constructor(exchange, message, details) {
        super(`${exchange} exchange error: ${message}`, HTTP_STATUS.SERVICE_UNAVAILABLE, ERROR_CODES.EXCHANGE_ERROR, true, { exchange, ...details });
        this.name = 'ExchangeError';
    }
}
// Network Error
export class NetworkError extends AppError {
    constructor(message = 'Network connection failed', details) {
        super(message, HTTP_STATUS.SERVICE_UNAVAILABLE, ERROR_CODES.EXTERNAL_API_ERROR, true, details);
        this.name = 'NetworkError';
    }
}
// Timeout Error
export class TimeoutError extends AppError {
    constructor(operation, timeout) {
        super(`Operation '${operation}' timed out after ${timeout}ms`, HTTP_STATUS.SERVICE_UNAVAILABLE, ERROR_CODES.EXTERNAL_API_ERROR, true, { operation, timeout });
        this.name = 'TimeoutError';
    }
}
// Error Factory Functions
export const createValidationError = (field, value, rule) => {
    return new ValidationError(`Validation failed for field '${field}': ${rule}`, { field, value, rule });
};
export const createNotFoundError = (resource, id) => {
    return new NotFoundError(`${resource} with ID '${id}' not found`);
};
export const createDuplicateError = (resource, field, value) => {
    return new AppError(`${resource} with ${field} '${value}' already exists`, HTTP_STATUS.CONFLICT, ERROR_CODES.USER_ALREADY_EXISTS, true, { resource, field, value });
};
// Error Handler Utility
export const handleError = (error) => {
    if (error instanceof AppError) {
        return error;
    }
    if (error instanceof Error) {
        return new AppError(error.message, HTTP_STATUS.INTERNAL_SERVER_ERROR, ERROR_CODES.INTERNAL_ERROR, false);
    }
    return new AppError('An unknown error occurred', HTTP_STATUS.INTERNAL_SERVER_ERROR, ERROR_CODES.INTERNAL_ERROR, false);
};
// Error Response Formatter
export const formatErrorResponse = (error) => {
    return {
        success: false,
        error: {
            message: error.message,
            code: error.code,
            statusCode: error.statusCode,
            timestamp: error.timestamp,
            ...(error.details && { details: error.details }),
        },
    };
};
// Error Logger
export const logError = (error, context) => {
    const logData = {
        ...error.toJSON(),
        context,
    };
    if (error.statusCode >= 500) {
        console.error('Server Error:', logData);
    }
    else if (error.statusCode >= 400) {
        console.warn('Client Error:', logData);
    }
    else {
        console.info('Error:', logData);
    }
};
// Type Guards
export const isAppError = (error) => {
    return error instanceof AppError;
};
export const isOperationalError = (error) => {
    return isAppError(error) && error.isOperational;
};
