import * as winston from 'winston';
import type { LoggerInterface } from '../types';

const { combine, timestamp, printf, align, json } = winston.format;

// Custom format for JSON logging (e.g., for sending to a log management system or for structured logs in Cloudflare)
const jsonFormat: winston.Logform.Format = combine(
  timestamp(),
  json()
);

// Simplified plain text format for testing or environments where colorization is problematic
const plainConsoleFormat: winston.Logform.Format = combine(
  timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
  align(),
  printf((info) => `[${info.timestamp}] ${info.level}: ${info.message}`)
);

export function createLogger(logLevel = 'info'): LoggerInterface {
  const transports: winston.transport[] = [];
  let formatToUse: winston.Logform.Format;

  if (process.env.VITEST === 'true' || process.env.NODE_ENV === 'test') {
    formatToUse = plainConsoleFormat;
  } else {
    // Only define and use colorize when not in test/Vitest
    const { colorize } = winston.format; // Access it here, only if needed
    const consoleFormatWithColors: winston.Logform.Format = combine( // Renamed to avoid conflict if logger was global
      colorize({ all: true }), // Use it here
      timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
      align(),
      printf((info) => `[${info.timestamp}] ${info.level}: ${info.message}`)
    );
    formatToUse = consoleFormatWithColors;
  }

  // Determine if running in a Cloudflare Worker-like environment or a test environment
  const isWorkerEnvironment = 
    (typeof self !== 'undefined' && typeof (self as { ServiceWorkerGlobalScope?: unknown }).ServiceWorkerGlobalScope !== 'undefined') ||
    process.env.VITEST === 'true' || 
    process.env.NODE_ENV === 'test';

  if (isWorkerEnvironment) {
    // For Cloudflare Workers and test environments, use a simple console transport.
    // The format will be applied at the logger level.
    transports.push(new winston.transports.Console());
  } else {
    // For local development or other non-worker Node.js environments,
    // also use console transport, but explicitly pass the format if it includes colors.
    // If formatToUse is plainConsoleFormat, it's fine too.
    transports.push(
      new winston.transports.Console({
        format: formatToUse, // Apply the determined format directly to the transport for non-workers
      })
    );
  }
  
  const logger: winston.Logger = winston.createLogger({
    level: logLevel,
    format: formatToUse, // Apply the chosen format at the logger level
    transports,
    exitOnError: false,
  });

  const addContext = (context: Record<string, any>): void => {
    // This is a placeholder. True context addition might involve child loggers
    // or modifying the logger's format.
  };

  const addError = (error: Error, context?: Record<string, any>): void => {
    logger.error(error.message, { error, ...context });
  };
  
  const getLogLevel = (): string => {
    return logger.level;
  };

  const setLogLevel = (newLevel: string): void => {
    logger.level = newLevel;
  };

  return {
    error: logger.error.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    warn: logger.warn.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    info: logger.info.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    http: logger.http.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    verbose: logger.verbose.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    debug: logger.debug.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    silly: logger.silly.bind(logger) as (message: string, ...meta: any[]) => winston.Logger,
    addContext,
    addError,
    child: (context: Record<string, any>): LoggerInterface => {
      const childLoggerInstance: winston.Logger = winston.createLogger({
        level: logger.level,
        format: formatToUse,
        transports: logger.transports,
      });
      // Simple context injection for child - might need more robust solution
      const wrap = (fn: (message: string, ...meta: any[]) => winston.Logger) => (message: string, ...meta: any[]): winston.Logger => {
        const mergedMeta = meta.length > 0 && typeof meta[0] === 'object' && meta[0] !== null
          ? { ...context, ...meta[0] }
          : context;
        
        if (meta.length > 0 && typeof meta[0] === 'object' && meta[0] !== null) {
            return fn(message, mergedMeta, ...meta.slice(1));
        } else {
            return fn(message, mergedMeta, ...meta);
        }
      };
      return {
        error: wrap(childLoggerInstance.error.bind(childLoggerInstance)),
        warn: wrap(childLoggerInstance.warn.bind(childLoggerInstance)),
        info: wrap(childLoggerInstance.info.bind(childLoggerInstance)),
        http: wrap(childLoggerInstance.http.bind(childLoggerInstance)),
        verbose: wrap(childLoggerInstance.verbose.bind(childLoggerInstance)),
        debug: wrap(childLoggerInstance.debug.bind(childLoggerInstance)),
        silly: wrap(childLoggerInstance.silly.bind(childLoggerInstance)),
        addContext: (_newContext: Record<string, any>): void => {},
        addError: (err: Error, errContext?: Record<string, any>): void => {
          childLoggerInstance.error(err.message, { error: err, ...context, ...errContext });
        },
        child: (_newContext: Record<string, any>): LoggerInterface => { throw new Error("Recursive child loggers not supported in this simplified version"); },
        getLogLevel: (): string => childLoggerInstance.level,
        setLogLevel: (lvl: string): void => { childLoggerInstance.level = lvl; },
      };
    },
    getLogLevel,
    setLogLevel,
  };
}
