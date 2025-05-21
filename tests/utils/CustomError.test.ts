import {
  CustomError,
  NetworkError,
  APIError,
  ValidationError,
  NotFoundError,
  AuthenticationError,
  AuthorizationError,
  type CustomErrorDetails,
} from '../../src/utils/CustomError';

describe('CustomError', () => {
  it('should create an instance of CustomError with all properties', () => {
    const message = 'Test error message';
    const details: CustomErrorDetails = { info: 'additional info' };
    const originalError = new Error('Original error');
    const method = 'testMethod';
    const status = 500;
    const errorCode = 'TEST_CODE';

    const error = new CustomError(message, details, originalError, method, status, errorCode);

    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(CustomError);
    expect(error.name).toBe('CustomError');
    expect(error.message).toBe(message);
    expect(error.details).toEqual(details);
    expect(error.originalError).toBe(originalError);
    expect(error.method).toBe(method);
    expect(error.status).toBe(status);
    expect(error.errorCode).toBe(errorCode);
    expect(error.stack).toBeDefined();
  });

  it('should create an instance of CustomError with only a message', () => {
    const message = 'Simple error';
    const error = new CustomError(message);

    expect(error).toBeInstanceOf(CustomError);
    expect(error.name).toBe('CustomError');
    expect(error.message).toBe(message);
    expect(error.details).toBeUndefined();
    expect(error.originalError).toBeUndefined();
    expect(error.method).toBeUndefined();
    expect(error.status).toBeUndefined();
    expect(error.errorCode).toBeUndefined();
  });

  it('should capture stack trace if Error.captureStackTrace is available', () => {
    const error = new CustomError('Test stack trace');
    // V8 specific, so might not be testable in all environments directly
    // but we can check if the stack is a string
    expect(typeof error.stack).toBe('string');
  });

  it('should allow CustomError without captureStackTrace (e.g. Firefox)', () => {
    const originalCaptureStackTrace = Error.captureStackTrace;
    // @ts-expect-error
    Error.captureStackTrace = undefined; // Simulate environment without it
    const error = new CustomError('No captureStackTrace');
    expect(error.stack).toBeDefined(); // .stack is still usually present
    Error.captureStackTrace = originalCaptureStackTrace; // Restore
  });
});

describe('Derived Error Classes', () => {
  describe('NetworkError', () => {
    it('should create an instance with default values', () => {
      const error = new NetworkError();
      expect(error).toBeInstanceOf(NetworkError);
      expect(error).toBeInstanceOf(CustomError);
      expect(error.name).toBe('NetworkError');
      expect(error.message).toBe('A network error occurred');
      expect(error.status).toBe(503);
      expect(error.errorCode).toBe('NETWORK_ERROR');
    });

    it('should create an instance with custom message and other params', () => {
      const message = 'Custom network error';
      const details: CustomErrorDetails = { host: 'example.com' };
      const originalError = new Error('Socket timeout');
      const method = 'fetchData';
      const error = new NetworkError(message, details, originalError, method);

      expect(error.message).toBe(message);
      expect(error.details).toEqual(details);
      expect(error.originalError).toBe(originalError);
      expect(error.method).toBe(method);
      expect(error.status).toBe(503);
      expect(error.errorCode).toBe('NETWORK_ERROR');
    });
  });

  describe('APIError', () => {
    it('should create an instance with default values', () => {
      const error = new APIError();
      expect(error).toBeInstanceOf(APIError);
      expect(error.name).toBe('APIError');
      expect(error.message).toBe('An API error occurred');
      expect(error.status).toBe(500); // Default status
      expect(error.errorCode).toBe('API_ERROR');
    });

    it('should create an instance with a custom status', () => {
      const error = new APIError('Specific API Error', undefined, undefined, undefined, 404);
      expect(error.message).toBe('Specific API Error');
      expect(error.status).toBe(404);
      expect(error.errorCode).toBe('API_ERROR');
    });

    it('should create an instance with all custom parameters', () => {
      const message = 'Custom API error full';
      const details: CustomErrorDetails = { endpoint: '/users' };
      const originalError = new Error('Upstream service failed');
      const method = 'POST /users';
      const status = 502;
      const error = new APIError(message, details, originalError, method, status);

      expect(error).toBeInstanceOf(APIError);
      expect(error.name).toBe('APIError');
      expect(error.message).toBe(message);
      expect(error.details).toEqual(details);
      expect(error.originalError).toBe(originalError);
      expect(error.method).toBe(method);
      expect(error.status).toBe(status);
      expect(error.errorCode).toBe('API_ERROR');
    });
  });

  describe('ValidationError', () => {
    it('should create an instance with default values', () => {
      const error = new ValidationError();
      expect(error).toBeInstanceOf(ValidationError);
      expect(error.name).toBe('ValidationError');
      expect(error.message).toBe('Validation failed');
      expect(error.status).toBe(400);
      expect(error.errorCode).toBe('VALIDATION_ERROR');
    });

    it('should create an instance with custom message and other params', () => {
      const message = 'Custom validation error';
      const details: CustomErrorDetails = { field: 'email' };
      const originalError = new Error('Invalid format');
      const method = 'validateInput';
      const error = new ValidationError(message, details, originalError, method);

      expect(error).toBeInstanceOf(ValidationError);
      expect(error.name).toBe('ValidationError');
      expect(error.message).toBe(message);
      expect(error.details).toEqual(details);
      expect(error.originalError).toBe(originalError);
      expect(error.method).toBe(method);
      expect(error.status).toBe(400);
      expect(error.errorCode).toBe('VALIDATION_ERROR');
    });
  });

  describe('NotFoundError', () => {
    it('should create an instance with default values', () => {
      const error = new NotFoundError();
      expect(error).toBeInstanceOf(NotFoundError);
      expect(error.name).toBe('NotFoundError');
      expect(error.message).toBe('Resource not found');
      expect(error.status).toBe(404);
      expect(error.errorCode).toBe('NOT_FOUND');
    });

    it('should create an instance with custom message and other params', () => {
      const message = 'Custom resource not found';
      const details: CustomErrorDetails = { resourceId: '123' };
      const originalError = new Error('DB query returned null');
      const method = 'getResource';
      const error = new NotFoundError(message, details, originalError, method);

      expect(error).toBeInstanceOf(NotFoundError);
      expect(error.name).toBe('NotFoundError');
      expect(error.message).toBe(message);
      expect(error.details).toEqual(details);
      expect(error.originalError).toBe(originalError);
      expect(error.method).toBe(method);
      expect(error.status).toBe(404);
      expect(error.errorCode).toBe('NOT_FOUND');
    });
  });

  describe('AuthenticationError', () => {
    it('should create an instance with default values', () => {
      const error = new AuthenticationError();
      expect(error).toBeInstanceOf(AuthenticationError);
      expect(error.name).toBe('AuthenticationError');
      expect(error.message).toBe('Authentication failed');
      expect(error.status).toBe(401);
      expect(error.errorCode).toBe('AUTH_ERROR');
    });

    it('should create an instance with custom message and other params', () => {
      const message = 'Custom authentication failed';
      const details: CustomErrorDetails = { reason: 'invalid token' };
      const originalError = new Error('Token expired');
      const method = 'verifyToken';
      const error = new AuthenticationError(message, details, originalError, method);

      expect(error).toBeInstanceOf(AuthenticationError);
      expect(error.name).toBe('AuthenticationError');
      expect(error.message).toBe(message);
      expect(error.details).toEqual(details);
      expect(error.originalError).toBe(originalError);
      expect(error.method).toBe(method);
      expect(error.status).toBe(401);
      expect(error.errorCode).toBe('AUTH_ERROR');
    });
  });

  describe('AuthorizationError', () => {
    it('should create an instance with default values', () => {
      const error = new AuthorizationError();
      expect(error).toBeInstanceOf(AuthorizationError);
      expect(error.name).toBe('AuthorizationError');
      expect(error.message).toBe('Authorization failed');
      expect(error.status).toBe(403);
      expect(error.errorCode).toBe('AUTH_Z_ERROR');
    });

    it('should create an instance with custom message and other params', () => {
      const message = 'Custom authorization failed';
      const details: CustomErrorDetails = { permission: 'delete_user' };
      const originalError = new Error('User lacks role');
      const method = 'checkPermission';
      const error = new AuthorizationError(message, details, originalError, method);

      expect(error).toBeInstanceOf(AuthorizationError);
      expect(error.name).toBe('AuthorizationError');
      expect(error.message).toBe(message);
      expect(error.details).toEqual(details);
      expect(error.originalError).toBe(originalError);
      expect(error.method).toBe(method);
      expect(error.status).toBe(403);
      expect(error.errorCode).toBe('AUTH_Z_ERROR');
    });
  });
}); 