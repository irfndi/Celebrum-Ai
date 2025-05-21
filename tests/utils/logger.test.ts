import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as winston from 'winston';
import type { Logform } from 'winston'; // Import Logform for TransformableInfo
import { createLogger } from '../../src/utils/logger';
import type { LoggerInterface } from '../../src/types';

// Mock winston transports to spy on them
const mockConsoleTransport = {
  log: vi.fn(),
  on: vi.fn(),
  once: vi.fn(),
  emit: vi.fn(),
  // Add other methods if Winston's Console transport class needs them for construction
  writable: true,
  destroy: vi.fn(),
  close: vi.fn(),
};

vi.mock('winston', async (importOriginal) => {
  const actualWinston = await importOriginal<typeof winston>();
  
  // Define simple mocks for formatters directly inside or ensure they are accessible
  const internalMockColorize = vi.fn((opts?: Record<string, unknown>) => (info: Logform.TransformableInfo) => ({ ...info, level: `colorized-${info.level}` }));

  return {
    ...actualWinston,
    format: {
      ...actualWinston.format,
      combine: vi.fn((...args) => actualWinston.format.combine(...args)), // Keep actual combine or mock if needed
      colorize: vi.fn(() => internalMockColorize), // Use the internally defined mock
      timestamp: vi.fn(() => vi.fn()), // Return a new vi.fn()
      align: vi.fn(() => vi.fn()),     // Return a new vi.fn()
      printf: vi.fn(() => vi.fn()),    // Return a new vi.fn()
      json: vi.fn(() => vi.fn()),       // Return a new vi.fn()
    },
    transports: {
      ...actualWinston.transports,
      Console: vi.fn(() => mockConsoleTransport),
    },
    createLogger: vi.fn((options) => {
      const actualLogger = actualWinston.createLogger(options); // Call actual to get a base logger structure
      return {
        error: vi.fn(), warn: vi.fn(), info: vi.fn(), http: vi.fn(), verbose: vi.fn(), debug: vi.fn(), silly: vi.fn(),
        level: options?.level || 'info',
        transports: options?.transports || [],
        format: options?.format, // Pass format through for inspection
        // Mock other methods if necessary or keep from actualLogger if it doesn't interfere
        on: actualLogger.on?.bind(actualLogger) || vi.fn(), // Ensure essential methods exist
        configure: actualLogger.configure?.bind(actualLogger) || vi.fn(),
        // Add any other methods that might be called internally by winston if not using full actualLogger methods
      };
    }),
  };
});

describe('Logger Utilities', () => {
  let logger: LoggerInterface;

  beforeEach(() => {
    vi.clearAllMocks();
    process.env.VITEST_OLD = process.env.VITEST;
    process.env.NODE_ENV_OLD = process.env.NODE_ENV;
    // @ts-ignore
    global.self_OLD = global.self;
    delete process.env.VITEST;
    delete process.env.NODE_ENV;
    // @ts-ignore
    delete global.self; // To simulate non-worker environment initially
  });

  afterEach(() => {
    process.env.VITEST = process.env.VITEST_OLD;
    process.env.NODE_ENV = process.env.NODE_ENV_OLD;
    // @ts-ignore
    global.self = global.self_OLD;
    delete process.env.VITEST_OLD;
    delete process.env.NODE_ENV_OLD;
    // @ts-ignore
    delete global.self_OLD;
  });

  describe('createLogger', () => {
    it('should create a logger with default level "info"', () => {
      logger = createLogger();
      expect(winston.createLogger).toHaveBeenCalled();
      // @ts-ignore - Accessing mocked logger internals for verification
      const mockCreatedLogger = (winston.createLogger as vi.Mock).mock.results[0].value;
      expect(mockCreatedLogger.level).toBe('info');
    });

    it('should create a logger with a specified log level', () => {
      logger = createLogger('debug');
      expect(winston.createLogger).toHaveBeenCalled();
      // @ts-ignore
      const mockCreatedLogger = (winston.createLogger as vi.Mock).mock.results[0].value;
      expect(mockCreatedLogger.level).toBe('debug');
    });

    it('should use plainConsoleFormat when VITEST is true', () => {
      process.env.VITEST = 'true';
      logger = createLogger();
      expect(winston.createLogger).toHaveBeenCalled();
      // @ts-ignore
      const createLoggerOpts = (winston.createLogger as vi.Mock).mock.calls[0][0];
      // Check if the format name contains 'plainConsoleFormat' related parts
      // This is a bit indirect; ideally, we'd check the format object itself.
      expect(createLoggerOpts.format.constructor.name).not.toContain('Colorize'); // Assuming colorize adds a specific format object
    });

    it('should use consoleFormatWithColors when not in VITEST/test env', () => {
      // Ensure VITEST and NODE_ENV are not 'test'
      logger = createLogger();
      expect(winston.createLogger).toHaveBeenCalled();
      // @ts-ignore
      const createLoggerOpts = (winston.createLogger as vi.Mock).mock.calls[0][0];
       // This check is fragile. A better way would be to export and check format names or specific properties.
      // For now, we assume colorize is part of the default format when not in test.
      // We can check that the mocked winston.format.colorize was (or wasn't) called based on the logic in createLogger
      // However, winston.format.colorize is not directly mockable in this setup without more complex module mocking.
      // A simple check might be that the format IS NOT plainConsoleFormat
      expect(createLoggerOpts.format).toBeDefined(); 
      // Add more specific check if possible, e.g. by inspecting the format properties if they are exposed or predictable.
    });

    it('should attempt to use colorize format when not in VITEST/test env and not worker', () => {
      // Ensure VITEST and NODE_ENV are not 'test'
      // global.self is deleted in beforeEach, simulating non-worker
      logger = createLogger();
      expect(winston.format.colorize).toHaveBeenCalled();
    });

    it('should configure Console transport with specific format for non-worker env', () => {
      // global.self is deleted in beforeEach, simulating non-worker
      // process.env.VITEST and NODE_ENV are undefined here
      logger = createLogger();
      expect(winston.transports.Console).toHaveBeenCalled();
      const consoleOptions = (winston.transports.Console as vi.Mock).mock.calls[0][0];
      expect(consoleOptions).toBeDefined();
      expect(consoleOptions.format).toBeDefined(); // Check that format is passed to transport
      // We expect this format to be the one including colorize
      // The mock for createLogger passes the options.format through.
      const loggerFormat = (winston.createLogger as vi.Mock).mock.calls[0][0].format;
      expect(consoleOptions.format).toBe(loggerFormat); 
    });

    it('should use plain console format for worker-like env (self defined)', () => {
      // @ts-ignore Simulate worker environment
      global.self = { ServiceWorkerGlobalScope: {} }; 
      // VITEST and NODE_ENV are undefined here by default from beforeEach
      logger = createLogger();
      // const createLoggerOpts = (winston.createLogger as vi.Mock).mock.calls[0][0]; // Not strictly needed for this assertion
      const consoleOptions = (winston.transports.Console as vi.Mock).mock.calls[0][0];
      expect(consoleOptions).toBeUndefined(); // For worker, Console is called without options, this is correct.
      
      // If not VITEST/NODE_ENV=test, colorize WILL be called to set up formatToUse,
      // even if that format isn't directly passed to the Console transport for workers.
      // The logger itself will still use that format.
      expect(winston.format.colorize).toHaveBeenCalled(); // Corrected assertion
    });

    // Basic logging method tests
    it('should call underlying info method', () => {
      logger = createLogger();
      logger.info('Test info message');
      // @ts-ignore
      const mockInternalLogger = (winston.createLogger as vi.Mock).mock.results[0].value;
      expect(mockInternalLogger.info).toHaveBeenCalledWith('Test info message');
    });

    it('should call underlying error method', () => {
      logger = createLogger();
      logger.error('Test error message');
      // @ts-ignore
      const mockInternalLogger = (winston.createLogger as vi.Mock).mock.results[0].value;
      expect(mockInternalLogger.error).toHaveBeenCalledWith('Test error message');
    });

    it('should call underlying warn method', () => {
      logger = createLogger();
      logger.warn('Test warn message');
      // @ts-ignore
      const mockInternalLogger = (winston.createLogger as vi.Mock).mock.results[0].value;
      expect(mockInternalLogger.warn).toHaveBeenCalledWith('Test warn message');
    });

    // Test for addError
    it('addError should call underlying error method with error object', () => {
      logger = createLogger();
      const testError = new Error('Something went wrong');
      logger.addError(testError, { customContext: 'value' });
      // @ts-ignore
      const mockInternalLogger = (winston.createLogger as vi.Mock).mock.results[0].value;
      expect(mockInternalLogger.error).toHaveBeenCalledWith(testError.message, {
        error: testError,
        customContext: 'value',
      });
    });

    // Tests for getLogLevel and setLogLevel
    it('getLogLevel should return the current log level of the winston logger', () => {
      logger = createLogger('warn');
      // @ts-ignore
      (winston.createLogger as vi.Mock).mock.results[0].value.level = 'warn'; // Simulate level being set
      expect(logger.getLogLevel()).toBe('warn');
    });

    it('setLogLevel should update the log level of the winston logger', () => {
      logger = createLogger('info');
      logger.setLogLevel('debug');
      // @ts-ignore
      expect((winston.createLogger as vi.Mock).mock.results[0].value.level).toBe('debug');
    });

    describe('child logger', () => {
      let parentLogger: LoggerInterface;
      let childLogger: LoggerInterface;
      const parentContext = { parentKey: 'parentValue' };
      const childContext = { childKey: 'childValue' };

      beforeEach(() => {
        delete process.env.VITEST;
        delete process.env.NODE_ENV;
        parentLogger = createLogger('debug');
        // @ts-ignore
        vi.spyOn((winston.createLogger as vi.Mock).mock.results[0].value, 'debug');
        childLogger = parentLogger.child(parentContext);
      });

      it('should create a child logger that logs with parent context', () => {
        childLogger.debug('Child message');
        // The mock for winston.createLogger returns a new logger instance for the child
        // We need to find that instance to check its mocked log methods.
        // This assumes the child logger uses the *second* logger created by winston.createLogger mock calls.
        // The first is the parent, the second (if child() internally calls createLogger) would be the child.
        // The current logger.ts child() *does* call winston.createLogger.
        expect((winston.createLogger as vi.Mock).mock.calls.length).toBeGreaterThanOrEqual(2);
        // @ts-ignore
        const mockInternalChildLogger = (winston.createLogger as vi.Mock).mock.results[1].value;
        expect(mockInternalChildLogger.debug).toHaveBeenCalledWith('Child message', parentContext);
      });

      it('should allow child logger to log with additional context', () => {
        childLogger.debug('Child message with more context', { extra: 'data' });
        // @ts-ignore
        const mockInternalChildLogger = (winston.createLogger as vi.Mock).mock.results[1].value;
        expect(mockInternalChildLogger.debug).toHaveBeenCalledWith(
          'Child message with more context',
          { ...parentContext, extra: 'data' }
        );
      });

      it('addError on child should include parent and error context', () => {
        const error = new Error('Child error');
        const errorCtx = { errorKey: 'errorValue' };
        childLogger.addError(error, errorCtx);
        // @ts-ignore
        const mockInternalChildLogger = (winston.createLogger as vi.Mock).mock.results[1].value;
        expect(mockInternalChildLogger.error).toHaveBeenCalledWith(error.message, {
          error,
          ...parentContext,
          ...errorCtx,
        });
      });

      it('getLogLevel on child should return its level (inherited from parent)', () => {
        // @ts-ignore
        (winston.createLogger as vi.Mock).mock.results[1].value.level = 'debug';
        expect(childLogger.getLogLevel()).toBe('debug');
      });

      it('setLogLevel on child should update its level', () => {
        childLogger.setLogLevel('silly');
        // @ts-ignore
        expect((winston.createLogger as vi.Mock).mock.results[1].value.level).toBe('silly');
      });

      it('addContext on child logger should be a no-op', () => {
        // The addContext on child is a no-op, so no error and no change expected.
        expect(() => childLogger.addContext({ newChildCtx: 'test' })).not.toThrow();
        // No easy way to check for no-op without more complex spying on its empty function body.
      });
      
      it('calling child() on a child logger should throw an error', () => {
        expect(() => childLogger.child(childContext)).toThrow("Recursive child loggers not supported");
      });
    });
  });
}); 