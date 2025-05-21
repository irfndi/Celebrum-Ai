export interface CustomErrorDetails {
  [key: string]: unknown; // Using unknown for better type safety than any
}

export class CustomError extends Error {
  public originalError?: Error;
  public details?: CustomErrorDetails;
  public status?: number; // HTTP status code
  public errorCode?: string; // Internal error code
  public method?: string; // Method where error originated

  constructor(
    message: string,
    details?: CustomErrorDetails,
    originalError?: Error,
    method?: string,
    status?: number,
    errorCode?: string,
  ) {
    super(message);
    this.name = this.constructor.name;
    this.details = details;
    this.originalError = originalError;
    this.method = method;
    this.status = status;
    this.errorCode = errorCode;

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

// Example of a more specific error type if needed in the future
export class NetworkError extends CustomError {
  constructor(message = "A network error occurred", details?: CustomErrorDetails, originalError?: Error, method?: string) {
    super(message, details, originalError, method, 503, "NETWORK_ERROR");
    this.name = this.constructor.name;
  }
}

export class APIError extends CustomError {
  constructor(message = "An API error occurred", details?: CustomErrorDetails, originalError?: Error, method?: string, status?: number) {
    super(message, details, originalError, method, status || 500, "API_ERROR");
    this.name = this.constructor.name;
  }
}

export class ValidationError extends CustomError {
  constructor(message = "Validation failed", details?: CustomErrorDetails, originalError?: Error, method?: string) {
    super(message, details, originalError, method, 400, "VALIDATION_ERROR");
    this.name = this.constructor.name;
  }
}

export class NotFoundError extends CustomError {
  constructor(message = "Resource not found", details?: CustomErrorDetails, originalError?: Error, method?: string) {
    super(message, details, originalError, method, 404, "NOT_FOUND");
    this.name = this.constructor.name;
  }
}

export class AuthenticationError extends CustomError {
 constructor(message = "Authentication failed", details?: CustomErrorDetails, originalError?: Error, method?: string) {
    super(message, details, originalError, method, 401, "AUTH_ERROR");
    this.name = this.constructor.name;
  }
}

export class AuthorizationError extends CustomError {
  constructor(message = "Authorization failed", details?: CustomErrorDetails, originalError?: Error, method?: string) {
    super(message, details, originalError, method, 403, "AUTH_Z_ERROR");
    this.name = this.constructor.name;
  }
} 